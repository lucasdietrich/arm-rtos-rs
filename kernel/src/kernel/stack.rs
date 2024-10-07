pub struct Stack {
    pub stack_end: *mut u32,
    pub stack_size: usize,
}

impl Stack {
    pub fn new(stack: &'static mut [u8]) -> Option<Self> {
        let stack_size = stack.len();
        let stack_end = unsafe { stack.as_mut_ptr().add(stack_size) } as *mut u32;

        // SP wouldn't be 8 B align
        if stack_end as usize % 8 != 0 {
            return None;
        }

        Some(Stack {
            stack_end,
            stack_size,
        })
    }
}
