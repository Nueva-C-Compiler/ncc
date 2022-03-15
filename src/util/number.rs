use derive_where::derive_where;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive_where(Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
pub struct GenOverflowError<D> {
    _ty: PhantomData<D>,
}

impl<D> GenOverflowError<D> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<D: NumberGenExt> Display for GenOverflowError<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(
            f,
            "generator overflowed (more than {} identifiers generated)",
            D::limit(),
        )
    }
}

pub trait NumberGenExt: Sized {
    type Value: Sized + Display;

    fn limit() -> Self::Value;
    fn try_generate(self) -> Result<Self::Value, GenOverflowError<Self>>;
}

impl<'a> NumberGenExt for &'a mut u64 {
    type Value = u64;

    fn limit() -> Self::Value {
        u64::MAX
    }

    fn try_generate(self) -> Result<Self::Value, GenOverflowError<Self>> {
        self.checked_add(1).ok_or(GenOverflowError::new())
    }
}

impl<'a> NumberGenExt for &'a AtomicU64 {
    type Value = u64;

    fn limit() -> Self::Value {
        u64::MAX - 1000
    }

    fn try_generate(self) -> Result<Self::Value, GenOverflowError<Self>> {
        let id = self.fetch_add(1, Ordering::Relaxed);

        // Look, unless we manage to allocate more than `1000` IDs before this check runs, this check
        // is *perfectly fine*.
        if id > Self::limit() {
            self.store(Self::limit(), Ordering::Relaxed);
            return Err(GenOverflowError::new());
        }

        Ok(id)
    }
}
