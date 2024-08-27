use core::marker::PhantomData;

use crate::kernel::kernel::Kernel;

trait Sealed {}

#[allow(private_bounds)]
pub trait CsDomain: Sealed {}

impl Sealed for Kernel {}
impl CsDomain for Kernel {}

pub struct GlobalIrq;

impl Sealed for GlobalIrq {}
impl CsDomain for GlobalIrq {}

// TODO How to make Cs<T> covariant over T ? (by implementing more marker traits ?)
// So that having a Cs<Global> is enough for an atomic section requiring Cs<Kernel>

/* Represent a critical section for a given domain D*/
pub struct Cs<D: CsDomain> {
    /* Keep this field public so that it's impossible to build Cs safely */
    domain: PhantomData<D>,
}

impl<D: CsDomain> Cs<D> {
    #[inline(always)]
    /* This is the only method to obtain a critical session object */
    pub unsafe fn new() -> Self {
        Cs {
            domain: PhantomData,
        }
    }
}
