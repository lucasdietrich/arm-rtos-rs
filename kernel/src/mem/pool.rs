pub struct MemoryPool<T, const N: usize> {
    buffer: [Option<T>; N],
}

impl<T, const N: usize> MemoryPool<T, N> {
    pub const fn init() -> Self {
        Self {
            buffer: [const { None }; N],
        }
    }

    pub fn alloc(&mut self, value: T) -> Option<&mut T> {
        for i in 0..N {
            if self.buffer[i].is_none() {
                self.buffer[i] = Some(value);
                return self.buffer[i].as_mut();
            }
        }
        None
    }
}
