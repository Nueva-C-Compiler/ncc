//! People are bound to do stupid things with manual borrowing. This module exists to make that less
//! likely.

use crate::util::number::NumberGenExt;
use derive_where::derive_where;
use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::num::NonZeroU64;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU64;

#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicIsize, Ordering};

pub trait BaseCell: Sized {
    type Value: Sized;

    fn raw(&self) -> &SemiUnsafeCell<Self::Value>;

    fn raw_mut(&mut self) -> &mut SemiUnsafeCell<Self::Value>;

    fn into_raw(self) -> SemiUnsafeCell<Self::Value>;

    fn into_inner(self) -> Self::Value {
        self.into_raw().into_inner()
    }

    fn get_raw(&self) -> *mut Self::Value {
        self.raw().get_raw()
    }

    fn get_mut(&mut self) -> &mut Self::Value {
        self.raw_mut().get_mut()
    }

    unsafe fn borrow_ref_unchecked(&self) -> UnsafeCellRef<'_, Self::Value> {
        self.raw().borrow_ref_unchecked()
    }

    unsafe fn borrow_mut_unchecked(&self) -> UnsafeCellMut<'_, Self::Value> {
        self.raw().borrow_mut_unchecked()
    }
}

// === SemiUnsafeCell === //

#[derive_where(Debug)]
pub struct SemiUnsafeCell<T> {
    value: UnsafeCell<T>,
    #[cfg(debug_assertions)]
    borrows: AtomicIsize,
}

impl<T> SemiUnsafeCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            #[cfg(debug_assertions)]
            borrows: AtomicIsize::new(0),
        }
    }
}

impl<T> BaseCell for SemiUnsafeCell<T> {
    type Value = T;

    fn raw(&self) -> &SemiUnsafeCell<Self::Value> {
        self
    }

    fn raw_mut(&mut self) -> &mut SemiUnsafeCell<Self::Value> {
        self
    }

    fn into_raw(self) -> SemiUnsafeCell<Self::Value> {
        self
    }

    fn into_inner(self) -> Self::Value {
        self.value.into_inner()
    }

    fn get_raw(&self) -> *mut Self::Value {
        self.value.get()
    }

    fn get_mut(&mut self) -> &mut Self::Value {
        self.value.get_mut()
    }

    unsafe fn borrow_ref_unchecked(&self) -> UnsafeCellRef<'_, Self::Value> {
        #[cfg(debug_assertions)]
        {
            let old = self
                .borrows
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                    if value > 0 {
                        None
                    } else {
                        Some(value.checked_sub(1).expect("borrowed too many times"))
                    }
                });

            assert!(
                old.is_ok(),
                "Failed to borrow cell immutably: found outstanding mutable references.\
                 This would be undefined behavior in release builds"
            );
        }

        UnsafeCellRef { cell: self }
    }

    unsafe fn borrow_mut_unchecked(&self) -> UnsafeCellMut<'_, Self::Value> {
        #[cfg(debug_assertions)]
        {
            let old = self
                .borrows
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                    if value == 0 {
                        Some(1)
                    } else {
                        None
                    }
                });

            assert!(
                old.is_ok(),
                "Failed to borrow cell mutably: found outstanding immutable references. \
                 This would cause undefined behavior in release builds."
            );
        }

        UnsafeCellMut { cell: self }
    }
}

impl<T: Default> Default for SemiUnsafeCell<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> From<T> for SemiUnsafeCell<T> {
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

unsafe impl<T> Send for SemiUnsafeCell<T> {}
unsafe impl<T> Sync for SemiUnsafeCell<T> {}

pub struct UnsafeCellRef<'a, T> {
    cell: &'a SemiUnsafeCell<T>,
}

impl<T> Deref for UnsafeCellRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.cell.get_raw() }
    }
}

#[cfg(debug_assertions)]
impl<T> Drop for UnsafeCellRef<'_, T> {
    fn drop(&mut self) {
        self.cell.borrows.fetch_add(1, Ordering::SeqCst);
    }
}

pub struct UnsafeCellMut<'a, T> {
    cell: &'a SemiUnsafeCell<T>,
}

impl<T> Deref for UnsafeCellMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.cell.get_raw() }
    }
}

impl<T> DerefMut for UnsafeCellMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.cell.get_raw() }
    }
}

#[cfg(debug_assertions)]
impl<T> Drop for UnsafeCellMut<'_, T> {
    fn drop(&mut self) {
        self.cell.borrows.fetch_sub(1, Ordering::SeqCst);
    }
}

// === RemoteCell === //

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct RemoteCellOwner {
    id: NonZeroU64,
}

impl Default for RemoteCellOwner {
    fn default() -> Self {
        static ID_GEN: AtomicU64 = AtomicU64::new(1);
        let id = (&ID_GEN)
            .try_generate()
            .expect("created too many `RemoteCellOwner`s");

        Self {
            id: NonZeroU64::new(id).unwrap(),
        }
    }
}

impl RemoteCellOwner {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive_where(Debug)]
pub struct RemoteCell<T> {
    owner: NonZeroU64,
    raw: SemiUnsafeCell<T>,
}

impl<T> RemoteCell<T> {
    pub fn new(owner: &RemoteCellOwner, value: T) -> Self {
        Self {
            owner: owner.id,
            raw: SemiUnsafeCell::new(value),
        }
    }

    pub fn is_owned_by(&self, owner: &RemoteCellOwner) -> bool {
        self.owner == owner.id
    }

    pub fn borrow_ref<'a>(&'a self, owner: &'a RemoteCellOwner) -> UnsafeCellRef<'a, T> {
        assert!(self.is_owned_by(owner));
        unsafe { self.raw.borrow_ref_unchecked() }
    }

    pub fn borrow_mut<'a>(&'a self, owner: &'a mut RemoteCellOwner) -> UnsafeCellMut<'a, T> {
        assert!(self.is_owned_by(owner));
        unsafe { self.raw.borrow_mut_unchecked() }
    }
}

impl<T> BaseCell for RemoteCell<T> {
    type Value = T;

    fn raw(&self) -> &SemiUnsafeCell<Self::Value> {
        &self.raw
    }

    fn raw_mut(&mut self) -> &mut SemiUnsafeCell<Self::Value> {
        &mut self.raw
    }

    fn into_raw(self) -> SemiUnsafeCell<Self::Value> {
        self.raw
    }
}
