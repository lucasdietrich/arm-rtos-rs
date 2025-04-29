use core::{arch::asm, ffi::c_void};

use crate::kernel::ThreadEntry;

/// Represents the context for an entry function of a loadable ELF. It is used to
/// pass parameters to the entry function and to set up the Global Offset Table (GOT).
#[derive(Debug)]
#[repr(C, align(4))]
pub struct LoadableEntryContext {
    /// Address of the entry function.
    pub entry: ThreadEntry,
    /// Address of the Global Offset Table.
    pub got_addr: *const u8,
    /// The first argument passed to the entry function.
    pub arg0: *mut c_void,
}

pub type Lex = LoadableEntryContext;

/// A trait defining the protocol for invoking the entry function in a
/// Position-Independent Code (PIC) context.
///
/// This trait is implemented by structures that specify which register
/// to use for the Global Offset Table address (GOT).
pub trait PICRegImpl {
    /// Calls the entry function within the `Lex`, using
    /// a specified register to handle the Global Offset Table address.
    ///
    /// This is documented in <https://gcc.gnu.org/onlinedocs/gcc-6.1.0/gcc/ARM-Options.html>
    /// under the `-msingle-pic-base` and `-mpic-register` options.
    unsafe fn invoke_loadable_entry(lex: &Lex) -> u32;
}
