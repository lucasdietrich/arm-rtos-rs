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
    unsafe fn call_loadable_entry(lex: &Lex) -> u32;
}

/// Implementation of `PICRegImpl` using register `r9` for the Global Offset Table.
#[derive(Debug)]
pub struct R9;
impl PICRegImpl for R9 {
    unsafe fn call_loadable_entry(lex: &Lex) -> u32 {
        let r0: u32;
        asm!(
            "
            blx {entry}
            ",
            entry = in(reg) lex.entry,
            inout("r0") lex.arg0 => r0,
            in("r9") lex.got_addr,
        );
        r0
    }
}

/// Implementation of `PICRegImpl` using register `r10` for the Global Offset Table.
#[derive(Debug)]
pub struct R10;
impl PICRegImpl for R10 {
    unsafe fn call_loadable_entry(lex: &Lex) -> u32 {
        let r0: u32;
        asm!(
            "
            blx {entry}
            ",
            entry = in(reg) lex.entry,
            inout("r0") lex.arg0 => r0,
            in("r9") lex.got_addr,
        );
        r0
    }
}

#[cfg(all(feature = "loadable-elf-reg-r9", feature = "loadable-elf-reg-r10"))]
compile_error!("Only one PIC register can be used at a time");

/// Type alias that selects the appropriate `PICRegImpl` based on the
/// active feature flag, using either `R9` or `R10`.
#[cfg(feature = "loadable-elf-reg-r9")]
pub type PICReg = R9;
#[cfg(feature = "loadable-elf-reg-r10")]
pub type PICReg = R10;
