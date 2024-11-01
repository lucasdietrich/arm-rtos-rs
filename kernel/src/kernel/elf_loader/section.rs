/// Represents a reference to a section of data in memory, with methods to populate
/// the final memory region (in RAM) on the base of the reguion base pointer.
///
/// The `SectionRef` structure stores an offset and size representing a memory region
/// relative to a given base address, along with a reference to the source data to copy
/// from.&
#[derive(Debug)]
pub struct SectionRef<'a> {
    /// Offset in bytes from the base pointer to the beginning of this section.
    pub local_data_offset: usize,

    /// Size in bytes of this section.
    pub size: usize,

    /// Source data to copy from, it comes from the ELF file.
    /// - It is expected to be empty for `.bss` and `.noinit` sections.
    /// - It is expected to be non-empty for `.got` and `.data` sections with
    ///  a size equal to the section size.
    pub copy_from: &'a [u8],
}

impl<'a> SectionRef<'a> {
    /// Calculates the absolute address of this section in memory relative to a given
    /// `base_ptr`.
    ///
    /// # Parameters
    /// - `base_ptr`: A pointer to the base memory address.
    ///
    /// # Returns
    /// - A pointer to the start of the section within the memory space of `base_ptr`.
    pub unsafe fn get_abs_addr(&self, base_ptr: *mut u8) -> *mut u8 {
        base_ptr.add(self.local_data_offset)
    }

    /// Copies the data from `copy_from` to the memory region specified by this section.
    ///
    /// # Parameters
    /// - `base_ptr`: A pointer to the base memory address.
    pub unsafe fn copy_to(&self, base_ptr: *mut u8) {
        base_ptr
            .add(self.local_data_offset)
            .copy_from(self.copy_from.as_ptr(), self.size)
    }

    /// Copies data from `copy_from` to the memory region, with each value transformed
    /// by a given function `func`. This function is typically called when patching
    /// addresses in the `.got` section.
    ///
    /// # Parameters
    /// - `base_ptr`: A pointer to the base memory address.
    /// - `func`: A transformation function applied to each `u32` value from `copy_from`
    ///   before writing to the destination.
    pub unsafe fn patch_copy_to<F>(&self, base_ptr: *mut u8, func: F)
    where
        F: Fn(u32) -> u32,
    {
        let from_addr = self.copy_from.as_ptr() as *const u32;
        let to_addr = base_ptr.add(self.local_data_offset) as *mut u32;
        for i in 0..self.size / 4 {
            let val = from_addr.add(i).read();
            to_addr.add(i).write(func(val));
        }
    }

    /// Fills the memory region of this section with a specified byte value.
    ///
    /// # Parameters
    /// - `base_ptr`: A pointer to the base memory address.
    /// - `val`: The byte value to fill the section with.
    pub unsafe fn write_to(&self, base_ptr: *mut u8, val: u8) {
        base_ptr
            .add(self.local_data_offset)
            .write_bytes(val, self.size);
    }
}
