use core::marker::PhantomData;

use crate::kernel::Kernel;

trait Sealed {}

pub trait CsDomain: Sealed {}

impl Sealed for Kernel {}
impl CsDomain for Kernel {}

struct CsPrivate;

pub struct Cs<T: CsDomain> {
    domain: PhantomData<T>,
    private: CsPrivate,
}

impl<T: CsDomain> Cs<T> {
    pub unsafe fn new() -> Self {
        Cs {
            domain: PhantomData,
            private: CsPrivate,
        }
    }
}
