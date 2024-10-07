use core::{ffi::c_void, fmt::Write, intrinsics, mem::MaybeUninit, ptr::addr_of_mut};

use cortex_m::register::control::Control;
use kernel::{
    cortex_m::{
        cortex_m_rt::{k_call_pendsv, FCPU},
        cpu::Cpu,
        critical_section::{self, Cs, GlobalIrq},
        interrupts::{atomic_restore, atomic_section},
        irqn::SysIrqn,
        scb::SCB,
        systick::SysTickDevice,
    },
    kernel::{
        kernel::{z_current, z_next, Kernel},
        stack::Stack,
        thread::Thread,
        userspace::{self, k_svc_sleep, k_svc_yield},
    },
    serial::{SerialConfig, SerialTrait},
    serial_utils::Hex,
    soc::mps2_an385::{UartDevice, UART0},
    stdio,
};
use kernel::{print, println};

use crate::ref_cast_lifetime;

const FreqSysTick: u32 = 1_000; // Hz

#[no_mangle]
pub extern "C" fn z_systick() {
    /* Systick interrupt is executed with the highest priority and cannot be preempted
     * This is a *natural* critical section with the maximum degree
     */
    let cs = unsafe { Cs::<GlobalIrq>::new() };

    // /* Increment ticks */
    // unsafe { KERNEL.increment_ticks(&cs) };
}

fn k_yield() {
    unsafe {
        // z_current = KERNEL.current_ptr();
        // KERNEL.sched_next_thread();
        // z_next = KERNEL.current_ptr();
        k_call_pendsv();
        // k_svc_yield();
    };
}

#[no_mangle]
pub extern "C" fn _start() {
    // Initialize uart
    let mut uart = UartDevice::<FCPU>::new(UART0);
    let uart_config = SerialConfig::default();
    uart.init(&uart_config);
    let _ = uart.write_str("arm rust RTOS demo starting\n");

    // Set UART0 as main uart
    stdio::set_uart(uart);

    // let mut systick = SysTickDevice::<FCPU>::instance();
    // systick.configure::<FreqSysTick>(true);

    // Show startup state    let cpuid = SCB::new().get_cpuid();
    let mut scb = SCB::instance();
    println!("CPUID base: {}", Hex::U32(scb.get_cpuid()));

    println!(
        "Systick prio: {}",
        Hex::U8(scb.get_priority(SysIrqn::SYSTICK))
    );
    println!(
        "PendSV prio: {}",
        Hex::U8(scb.get_priority(SysIrqn::PENDSV))
    );
    println!("SVC prio: {}", Hex::U8(scb.get_priority(SysIrqn::SVCALL)));

    scb.set_priority(SysIrqn::PENDSV, 0x7);
    scb.set_priority(SysIrqn::SVCALL, 0x7);
    scb.set_priority(SysIrqn::SYSTICK, 0);

    println!(
        "Systick prio: {}",
        Hex::U8(scb.get_priority(SysIrqn::SYSTICK))
    );
    println!(
        "PendSV prio: {}",
        Hex::U8(scb.get_priority(SysIrqn::PENDSV))
    );
    println!("SVC prio: {}", Hex::U8(scb.get_priority(SysIrqn::SVCALL)));

    display_control_register();

    // Initialize kernel

    // init kernel
    let mut kernel = Kernel::<FreqSysTick>::init();

    // initialize task1
    #[link_section = ".noinit"]
    static mut THREAD_STACK1: [u8; 32768] = [0; 32768];

    let stack1 = Stack::new(unsafe { &mut THREAD_STACK1 }).unwrap();
    let task1 = Thread::init(&stack1, mytask_entry, 0xaaaa0000 as *mut c_void);

    kernel.register_thread(&task1);

    #[cfg(feature = "multiple_threads")]
    {
        // initialize task2
        #[link_section = ".noinit"]
        static mut THREAD_STACK2: [u8; 32768] = [0; 32768];

        let stack2 = Stack::new(unsafe { &mut THREAD_STACK2 }).unwrap();
        let task2 = Thread::init(&stack2, mytask_entry, 0xbbbb0000 as *mut c_void);

        kernel.register_thread(&task2);

        // initialize task3
        #[link_section = ".noinit"]
        static mut THREAD_STACK3: [u8; 32768] = [0; 32768];

        let stack3 = Stack::new(unsafe { &mut THREAD_STACK3 }).unwrap();
        let task3 = Thread::init(&stack3, mytask_entry3, 0xcccc0000 as *mut c_void);

        kernel.register_thread(&task3);
    }

    // init kernel
    kernel.print_tasks();
    kernel.kernel_loop();
    loop {}

    // loop {
    //     if let Some(byte) = stdio::read() {
    //         println!("recv: {}", Hex::U8(byte));

    //         println!(
    //             "[ticks: {}]: cur {}",
    //             atomic_restore(|cs| unsafe { KERNEL.get_ticks(cs) }),
    //             unsafe { KERNEL.current() }
    //         );

    //         let mut syscall_ret = 0;

    //         match byte {
    //             b'b' => unsafe { KERNEL.busy_wait(1000) },
    //             b'p' => {
    //                 println!("PendSV !");
    //                 k_yield();
    //             }
    //             b'y' => {
    //                 println!("SVC yield");
    //                 syscall_ret = userspace::k_svc_yield();
    //             }
    //             b's' => {
    //                 println!("SVC sleep");
    //                 syscall_ret = userspace::k_svc_sleep(1000);
    //             }
    //             b'v' => {
    //                 println!("SVC debug");
    //                 syscall_ret = userspace::k_svc_yield();
    //             }
    //             b'w' => {
    //                 println!("SVC print");
    //                 let msg = "Hello using SVC !!\n";
    //                 syscall_ret = userspace::k_svc_print(msg);
    //             }
    //             b'a' => {
    //                 println!("aborting...");
    //                 intrinsics::abort();
    //             }
    //             _ => {}
    //         }

    //         println!("syscall_ret: {}", Hex::U32(syscall_ret as u32));
    //     }
    // }
}

extern "C" fn mytask_entry(arg: *mut c_void) -> ! {
    let regs = Cpu::registers();
    Cpu::print_registers(&regs);

    let mut counter: u32 = 0;

    println!(
        "MyTask arg: {}, counter: {}",
        Hex::U32(arg as u32),
        Hex::U32(counter)
    );

    loop {
        // k_yield();

        counter = counter.wrapping_add(1);
    }
}

extern "C" fn mytask_entry3(arg: *mut c_void) -> ! {
    loop {
        println!("MyTask arg: {}, sleep", Hex::U32(arg as u32),);

        k_svc_sleep(1000);

        k_yield();
    }
}

fn display_control_register() {
    // Print control (special) register
    let control_register: Control = cortex_m::register::control::read();
    let _ = print!("control: {} ", Hex::U32(control_register.bits()));
    let _ = if control_register.npriv().is_privileged() {
        print!(" priviledged")
    } else {
        print!(" unpriviledged")
    };
    let _ = if control_register.spsel().is_msp() {
        print!(", MSP")
    } else {
        print!(", PSP")
    };
    let _ = if control_register.fpca().is_active() {
        print!(", FPU")
    } else {
        print!(", no FPU")
    };
    let _ = println!();
}
