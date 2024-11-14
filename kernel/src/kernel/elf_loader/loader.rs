use alloc::{alloc::Global, boxed::Box};
use core::{ffi::c_void, marker::PhantomData, mem};
use elf::{abi, endian::LittleEndian};

use crate::{
    kernel::{
        elf_loader::entry::{Lex, PICReg},
        kernel::Kernel,
        stack::Stack,
        thread::Thread,
        userspace, CpuVariant,
    },
    println,
};

use super::{entry::PICRegImpl, section::SectionRef};

const NOINIT_CANARIES_VALUE: u8 = 0xAA;

/// Enumeration of errors that may occur during the ELF loading process.
#[derive(Debug)]
pub enum LoadError {
    /// No memory available.
    NoMemory,
    /// Error when data sections are non-contiguous.
    NonContiguousDataSections,
    /// Error when an unsupported write-allocate section is encountered.
    UnsupportedWASection,
    /// Error related to text alignment.
    TextAlign,
    /// Error when a section alignment is unsupported.
    UnsupportedSectionAlign,
    /// Error when the required `.text` section is missing.
    MissingTextSection,
    /// Missing .got section
    MissingGotSection,
    /// Missing .data section
    MissingDataSection,
    /// Missing .bss section
    MissingBssSection,
    /// Missing .noinit section
    MissingNoinitSection,
    /// Error parsing the ELF headers.
    ParseHeaders,
    /// Parse Error
    ParseElf(elf::ParseError),
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
enum SectionType {
    GOT,
    DATA,
    BSS,
    NOINIT,
}

#[derive(Debug)]
pub struct Loadable<'elf, R: PICRegImpl> {
    entry: *const u8,
    alloc_size: usize,
    elf_addr_base: u64,
    sections: [Option<SectionRef<'elf>>; 4],
    _reg: PhantomData<R>,
}

impl<'elf, R: PICRegImpl> Loadable<'elf, R> {
    pub fn from(elf: elf::ElfBytes<'elf, LittleEndian>) -> Result<Self, LoadError> {
        // Find the .text section
        let text = elf
            .section_header_by_name(".text")
            .map_err(LoadError::ParseElf)?
            .ok_or(LoadError::MissingTextSection)?;
        let text_data = elf.section_data(&text).map_err(LoadError::ParseElf)?.0;

        // Ensure the .text section is aligned
        let alignment = text.sh_addralign as usize;
        if text_data.as_ptr() as usize % alignment != 0 {
            return Err(LoadError::TextAlign);
        }

        // Calculate the entry offset from the start of the .text section
        let rel_offset = (elf.ehdr.e_entry - text.sh_addr) as usize;

        // Calculate the entry function final address
        let entry = unsafe { text_data.as_ptr().add(rel_offset) };

        let mut addr_base = 0;
        let mut addr_cursor = 0;
        let mut first_section = true;
        let mut sections = [const { None }; 4];

        let (headers, strtab) = match elf.section_headers_with_strtab() {
            Ok((Some(headers), Some(strtab))) => (headers, strtab),
            _ => return Err(LoadError::ParseHeaders),
        };

        const WA: u64 = (abi::SHF_ALLOC | abi::SHF_WRITE) as u64;
        for h in headers
            .into_iter()
            .filter(|h| h.sh_flags & WA == WA && h.sh_size > 0)
        {
            // Maximum supported alignment is 4
            if h.sh_addralign > 4 {
                return Err(LoadError::UnsupportedSectionAlign);
            }

            let name = strtab
                .get(h.sh_name as usize)
                .map_err(LoadError::ParseElf)?;

            let section = match name {
                ".got" => SectionType::GOT, // TODO: Ensure .got is first section
                ".data" => SectionType::DATA,
                ".bss" => SectionType::BSS,
                ".noinit" => SectionType::NOINIT,
                _ => return Err(LoadError::UnsupportedWASection),
            };

            // For .bss and .noinit, the data is expected to be empty (i.e. &[])
            // Hence, the length must be retrieved from the section header (i.e. sh_size)
            let section_data = elf.section_data(&h).map_err(LoadError::ParseElf)?.0;

            if first_section {
                addr_base = h.sh_addr;
                addr_cursor = h.sh_addr;
                first_section = false;
            } else if addr_cursor != h.sh_addr {
                return Err(LoadError::NonContiguousDataSections);
            }

            addr_cursor += h.sh_size;

            sections[section as usize] = Some(SectionRef {
                local_data_offset: (h.sh_addr - addr_base) as usize,
                size: h.sh_size as usize,
                copy_from: section_data,
            });
        }

        let alloc_size = (addr_cursor - addr_base) as usize;

        Ok(Loadable {
            entry,
            alloc_size,
            elf_addr_base: addr_base,
            sections,
            _reg: PhantomData,
        })
    }

    extern "C" fn loadable_entry(lex: *mut c_void) -> ! {
        unsafe {
            let lex = &*(lex as *mut Lex);

            // 1. Set arg0 into r0
            // 2. Set .git section address into r9
            // 3. Branch and link to entry
            let r0 = R::call_loadable_entry(lex);

            // don't reference to lex after having call ELF entr,
            // lex might have be overwritten by the thread growing stack

            println!("Loadable ELF returned: {:x}", r0);

            userspace::k_stop();
        }
    }

    fn get_section(&self, section: SectionType) -> &Option<SectionRef<'elf>> {
        &self.sections[section as usize]
    }

    // Make stack size and priority configurable
    pub fn create_thread<'a, CPU: CpuVariant>(
        &self,
        arg0: *mut c_void,
        priority: i8,
    ) -> Result<Thread<'a, CPU>, LoadError>
    where
        // For the moment, the thread executed code is not copied from the ELF file,
        // so the ELF file must remain valid for the lifetime of the thread.
        'a: 'elf,
    {
        // Allocate the stack for the thread
        let stack = Box::try_new(Stack::<8192>::uninit()).map_err(|_| LoadError::NoMemory)?;
        let stack = Box::leak(stack);
        let stack_info = stack.get_info();

        // Allocate the data (.got + .data + .bss + .noinit)
        let data = Box::<[u8]>::new_uninit_slice_in(self.alloc_size, Global);
        let data = Box::leak(data);
        let data_base_ptr = data.as_mut_ptr() as *mut u8;

        // Initialize .got: copy from elf and patch each address in the .got section
        let got_section = self
            .get_section(SectionType::GOT)
            .as_ref()
            .ok_or(LoadError::MissingGotSection)?;
        unsafe {
            got_section.patch_copy_to(data_base_ptr, |addr| {
                addr + data_base_ptr as u32 - self.elf_addr_base as u32
            });
        }

        // Initialize .data
        let data_section = self
            .get_section(SectionType::DATA)
            .as_ref()
            .ok_or(LoadError::MissingDataSection)?;
        unsafe {
            data_section.copy_to(data_base_ptr);
        }

        // Initialize .bss
        let bss_section = self
            .get_section(SectionType::BSS)
            .as_ref()
            .ok_or(LoadError::MissingBssSection)?;
        unsafe {
            bss_section.write_to(data_base_ptr, 0);
        }

        // Initialize .noinit (nothing to do)
        #[cfg(feature = "kernel-noinit-canaries")]
        unsafe {
            self.get_section(SectionType::NOINIT)
                .as_ref()
                .ok_or(LoadError::MissingNoinitSection)?
                .write_to(data_base_ptr, NOINIT_CANARIES_VALUE);
        }

        // Create the context for the thread entry function
        let lex = Lex {
            entry: unsafe { mem::transmute(self.entry) },
            arg0,
            got_addr: data_base_ptr, // .got section is at the beginning of the allocated data section
        };

        // Write the entry context at the start of the allocated stack to
        // not allocate an additionnal buffer
        let ptr = unsafe { stack_info.write_obj_at(0, lex) }.ok_or(LoadError::NoMemory)?;

        // Create thread for loaded program
        let thread = Thread::init(
            &stack_info,
            Self::loadable_entry,
            ptr as *mut c_void,
            priority,
        );

        Ok(thread)
    }
}

impl<'a, CPU: CpuVariant, const K: usize, const F: u32> Kernel<'a, CPU, K, F> {
    /// Load an ELF file of a PIE into memory and create a thread for it.
    ///
    /// # Parameters
    /// - `bytes`: The ELF file bytes.
    ///
    /// # Returns
    /// - A reference to the created thread.
    /// - An error if the ELF file could not be loaded.
    pub fn load_elf(&mut self, bytes: &[u8]) -> Result<&'a Thread<'a, CPU>, LoadError> {
        let elf =
            elf::ElfBytes::<LittleEndian>::minimal_parse(bytes).map_err(LoadError::ParseElf)?;

        let loadable = Loadable::<PICReg>::from(elf)?;

        let thread = loadable.create_thread::<CPU>(core::ptr::null_mut(), 0)?;
        let thread = Box::new(thread);
        let thread = Box::leak(thread);

        self.register_thread(thread);

        Ok(thread)
    }
}
