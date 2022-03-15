use crate::util::accessor::{Accessor, PromiseImmutable, PromiseUnborrowed};
use crate::util::cell::{RemoteCell, RemoteCellOwner, UnsafeCellMut};
use derive_where::derive_where;
use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::iter::repeat_with as iter_repeat_with;
use std::marker::PhantomData;
use std::mem::replace;
use std::num::NonZeroU64;
use std::ptr::NonNull;
use std::rc::Rc;
use thiserror::Error;

// === Archetype marker === //

pub trait ArchMarker: 'static {}

// TODO: Provide a macro to derive these.

// === Worlds === //

#[derive(Debug)]
pub struct World {
    owner: RemoteCellOwner,
    archetypes: HashMap<TypeId, ArchetypeRc>,
}

type ArchetypeRc = Rc<RemoteCell<Archetype>>;

#[derive(Debug, Default)]
struct Archetype {
    // A negative offset from a `u64` entity ID to its index within the
    // `slots` dequeue.
    id_index_offset: u64,

    // A dequeue of slots for each entity in this archetype.
    //
    // For each byte:
    // - A value of `0` means this object is dead.
    // - A value of `1` means that only weak references to this object exist.
    // - Anything greater than `1` is just a regular RC count.
    //
    // There will never be slot with value zero at the beginning or the end of this deque.
    slots: VecDeque<u8>,
}

impl Archetype {
    fn locate_entity(&self, entity: NonZeroU64) -> Option<usize> {
        // Validate bounds
        let index = (entity.get() - 1).checked_sub(self.id_index_offset)?;
        if index >= self.slots.len() as u64 {
            return None;
        }

        let index = index as usize; // cast checked above

        // Validate index
        if self.slots[index] == 0 {
            return None;
        }

        Some(index)
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            owner: RemoteCellOwner::new(),
            archetypes: HashMap::new(),
        }
    }
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    fn fetch_or_make_archetype<'a, A: ArchMarker>(
        archetypes: &'a mut HashMap<TypeId, ArchetypeRc>,
        owner: &RemoteCellOwner,
    ) -> &'a ArchetypeRc {
        archetypes
            .entry(TypeId::of::<A>())
            .or_insert_with(|| Rc::new(RemoteCell::new(owner, Archetype::default())))
    }

    fn fetch_or_make_archetype_mut<A: ArchMarker>(&mut self) -> UnsafeCellMut<Archetype> {
        Self::fetch_or_make_archetype::<A>(&mut self.archetypes, &self.owner)
            .borrow_mut(&mut self.owner)
    }

    pub fn spawn<A: ArchMarker>(&mut self) -> Entity<A> {
        let mut archetype = self.fetch_or_make_archetype_mut::<A>();
        archetype.slots.push_back(1);

        Entity {
            _ty: PhantomData,
            // N.B. This "id" is one more than it should actually be. However, because we encode `0`
            // as `1` in the `NonZeroU64`, this is perfectly acceptable.
            id: NonZeroU64::new(archetype.id_index_offset + archetype.slots.len() as u64).unwrap(),
        }
    }

    pub fn try_despawn<A: ArchMarker>(&mut self, entity: Entity<A>) -> Result<(), DeadEntityError> {
        // Find archetype
        let mut archetype = self.fetch_or_make_archetype_mut::<A>();

        // Validate entity
        let index = archetype.locate_entity(entity.id).ok_or(DeadEntityError)?;

        if archetype.slots[index] == 0 {
            return Err(DeadEntityError);
        }

        // Mark entity as dead.
        archetype.slots[index] = 0;

        // Clean up storage if possible
        if index == 0 {
            let removed = archetype.slots.iter().take_while(|rc| **rc > 0).count();
            archetype.slots.drain(0..removed);
            archetype.id_index_offset += removed as u64;
        } else if index == archetype.slots.len() - 1 {
            let removed = archetype
                .slots
                .iter()
                .rev() // Provided for free by the double ended iterator.
                .take_while(|rc| **rc > 0)
                .count();

            let slots = &mut archetype.slots;
            slots.truncate(slots.len() - removed);
        }

        Ok(())
    }

    pub fn despawn<A: ArchMarker>(&mut self, entity: Entity<A>) {
        self.try_despawn(entity).unwrap();
    }

    pub fn is_alive<A: ArchMarker>(&self, entity: Entity<A>) -> bool {
        // Fetch archetype
        let archetype = match self.archetypes.get(&TypeId::of::<A>()) {
            Some(archetype) => archetype,
            None => return false,
        };

        let archetype = archetype.borrow_ref(&self.owner);

        // Validate entity
        archetype.locate_entity(entity.id).is_some()
    }
}

#[derive(Debug, Copy, Clone, Error)]
#[error("specified entity is dead")]
pub struct DeadEntityError;

#[derive(Debug, Copy, Clone, Error)]
#[error("specified entity is missing")]
pub struct MissingValueError;

#[derive_where(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Entity<A: ArchMarker> {
    _ty: PhantomData<fn(A) -> A>,
    // The ID of the entity in the archetype. ID `0` is encoded as `1`.
    // We use `NonZeroU64` to allow for niche representations such as `Option<Entity>` having the
    // same layout as a single `u64`.
    id: NonZeroU64,
}

// === Storages === //

// TODO: Fix the `UnsafeCell` situation.
#[derive_where(Debug/*, Clone*/; T)]
pub struct ArchStorage<A: ArchMarker, T> {
    _ty: PhantomData<fn(A) -> A>,
    archetype: ArchetypeRc,
    lowest_id: u64,
    values: VecDeque<Option<UnsafeCell<T>>>,
}

impl<A: ArchMarker, T> ArchStorage<A, T> {
    pub fn new(world: &mut World) -> Self {
        let archetype = World::fetch_or_make_archetype::<A>(&mut world.archetypes, &world.owner);

        Self {
            _ty: PhantomData,
            archetype: archetype.clone(),
            lowest_id: archetype.borrow_ref(&world.owner).id_index_offset,
            values: VecDeque::new(),
        }
    }

    pub fn cleanup_capacity(&mut self, world: &World) {
        // Fetch archetype
        let archetype = self.archetype.borrow_ref(&world.owner);

        // Cleanup front
        debug_assert!(self.lowest_id <= archetype.id_index_offset);
        let remove_from_front = archetype.id_index_offset - self.lowest_id;
        self.lowest_id = archetype.id_index_offset;
        self.values.drain(..(remove_from_front as usize));

        // Cleanup back
        self.values.truncate(archetype.slots.len());
    }

    pub fn full_cleanup(&mut self, world: &World) {
        // Cleanup capacity to make filtering out easier.
        self.cleanup_capacity(world);

        // Fetch archetype.
        let archetype = self.archetype.borrow_ref(&world.owner);

        // Filter out values
        for (value, rc) in self.values.iter_mut().zip(archetype.slots.iter().copied()) {
            if rc == 0 {
                *value = None;
            }
        }
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn insert(&mut self, world: &World, target: Entity<A>, value: T) -> Option<T> {
        // Ensure that the two dequeues are aligned.
        self.cleanup_capacity(world);

        // Find entity in archetype.
        let archetype = self.archetype.borrow_ref(&world.owner);
        let index = archetype
            .locate_entity(target.id)
            .expect("cannot insert dead entity into container");

        // Ensure that we have sufficient capacity.
        if index >= self.values.len() {
            let padding = index - self.values.len() + 1;
            let padding = iter_repeat_with(|| None).take(padding);
            self.values.extend(padding);
        }

        // And write it!
        replace(&mut self.values[index], Some(UnsafeCell::new(value))).map(|cell| cell.into_inner())
    }

    pub fn remove(&mut self, world: &World, target: Entity<A>) -> Option<T> {
        // Ensure that the two dequeues are aligned.
        self.cleanup_capacity(world);

        // Find entity in archetype.
        let archetype = self.archetype.borrow_ref(&world.owner);
        let index = archetype
            .locate_entity(target.id)
            .expect("cannot insert dead entity into container");

        if index >= self.values.len() {
            return None;
        }

        self.values[index].take().map(|cell| cell.into_inner())
    }

    pub fn access_raw(&self) -> &PromiseImmutable<Self> {
        unsafe { PromiseImmutable::make_ref(self) }
    }

    pub fn access_raw_mut(&mut self) -> &mut PromiseUnborrowed<Self> {
        unsafe { PromiseUnborrowed::make_mut(self) }
    }
}

impl<A: ArchMarker, T: 'static> Accessor for ArchStorage<A, T> {
    type Index = Entity<A>;
    type Value = T;
    type Ptr = NonNull<T>;
    type OobError = MissingValueError;

    fn try_get_raw(&self, index: Self::Index) -> Result<Self::Ptr, Self::OobError> {
        let offset = (index.id.get() - 1)
            .checked_sub(self.lowest_id)
            .filter(|offset| *offset < self.values.len() as u64)
            .ok_or(MissingValueError)?;

        let offset = offset as usize;
        let cell = self.values[offset].as_ref().ok_or(MissingValueError)?;

        // UnsafeCell<T> is `repr(transparent)` over `T`.
        Ok(NonNull::from(cell).cast::<T>())
    }
}

#[test]
fn basic_use_test() {
    use crate::util::accessor::{RefAccessorExt, MutAccessorExt};

    // Create the world
    let mut world = World::new();

    // Define an entity archetype
    struct EntHirFunction;
    impl ArchMarker for EntHirFunction {}

    // Spawn some entities
    let func_1 = world.spawn::<EntHirFunction>();
    let func_2 = world.spawn::<EntHirFunction>();
    let func_3 = world.spawn::<EntHirFunction>();
    let func_4 = world.spawn::<EntHirFunction>();

    // Attach the entities to a storage
    #[derive(Debug)]
    struct HirFunctionBase {
        name: &'static str,
    }

    let mut function_base = ArchStorage::new(&mut world);
    function_base.insert(&world, func_2, HirFunctionBase { name: "func_2" });
    function_base.insert(&world, func_1, HirFunctionBase { name: "func_1" });
    function_base.insert(&world, func_4, HirFunctionBase { name: "func_4" });

    // Check accessors
    let function_base = function_base.access_raw_mut();
    assert_eq!(function_base.get_ref(func_1).name, "func_1");
    assert_eq!(function_base.get_ref(func_2).name, "func_2");
    assert!(function_base.try_get_ref(func_3).is_err());
    assert_eq!(function_base.get_ref(func_4).name, "func_4");

    function_base.get_mut(func_4).name = "whee";
    assert_eq!(function_base.get_ref(func_4).name, "whee");

    function_base.remove(&world, func_4);
    assert!(function_base.try_get_ref(func_4).is_err());
}
