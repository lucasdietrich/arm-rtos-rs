use core::{marker::PhantomData, u64};

use crate::{
    cortex_m::{
        critical_section::{Cs, GlobalIrq},
        interrupts,
    },
    list, println,
    serial_utils::Hex,
    stdio,
};

use super::{thread::Thread, CpuVariant};

// CPU: CPU variant
// F: systick frequency (Hz)
#[repr(C)]
pub struct Kernel<'a, CPU: CpuVariant, const F: u32 = 1> {
    tasks: list::List<'a, Thread<'a, CPU>>,
    count: usize,
    current: usize,

    // Ticks counter: period: P (ms)
    ticks: u64,

    _cpu: PhantomData<CPU>,
}

impl<'a, CPU: CpuVariant, const F: u32> Kernel<'a, CPU, F> {
    pub const fn init() -> Kernel<'a, CPU, F> {
        Kernel {
            tasks: list::List::empty(),
            count: 0, // main thread
            current: 0,
            ticks: 0,
            _cpu: PhantomData,
        }
    }

    pub fn register_thread(&mut self, thread: &'a Thread<'a, CPU>) {
        self.tasks.push_front(&thread);
        thread.state.set(super::thread::ThreadState::Running);
        self.count += 1;
    }

    pub fn print_tasks(&self) {
        println!("print_tasks (cur: {} count: {})", self.current, self.count);
        for task in self.tasks.iter() {
            println!("{}", task);
        }
    }

    pub fn current(&self) -> &'a Thread<'a, CPU> {
        for (index, task) in self.tasks.iter().enumerate() {
            if self.current == index {
                return task;
            }
        }
        panic!("Invalid current index");
    }

    pub fn current_ptr(&self) -> *mut Thread<CPU> {
        let current = self.current();
        current as *const Thread<CPU> as *mut Thread<CPU>
    }

    // TODO: Remove the Cs parameter, access to Kernel is already atomic
    pub fn increment_ticks(&mut self, _cs: &Cs<GlobalIrq>) {
        self.ticks += 1;
    }

    // TODO: Remove the Cs parameter, access to Kernel is already atomic
    pub fn get_ticks(&self, _cs: &Cs<GlobalIrq>) -> u64 {
        self.ticks
    }

    // TODO: Remove the cs, access to Kernel is already atomic
    pub fn busy_wait(&self, ms: u32) {
        let end = interrupts::atomic_restore(|cs| self.get_ticks(cs))
            .saturating_add(((ms * F) / 1000) as u64);
        while interrupts::atomic_restore(|cs| self.get_ticks(cs)) < end {}
    }

    pub fn sched_next_thread(&mut self) {
        self.current = (self.current + 1) % self.count;
    }

    pub fn kernel_loop(&mut self) {
        let task = self.current();

        let process_sp = task.stack_ptr.get();
        println!("PSP: 0x{}", Hex::U32(process_sp as u32));

        let process_context = task.context.as_ptr();

        let process_sp = unsafe { CPU::switch_to_user(process_sp, process_context) };

        stdio::write_bytes(&[b'!']);

        println!("PSP: 0x{}", Hex::U32(process_sp as u32));

        task.stack_ptr.set(process_sp);

        println!("Returned from switch_to_user");

        self.sched_next_thread();
    }
}
