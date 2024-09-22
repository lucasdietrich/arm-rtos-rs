use core::{ffi::c_void, fmt::Write, intrinsics, mem::MaybeUninit, ptr::addr_of_mut};

use cortex_m::register::control::Control;
use kernel::{
    cortex_m::{
        cortex_m_rt::{k_call_pendsv, FCPU},
        critical_section::{self, Cs, GlobalIrq},
        interrupts::{atomic_restore, atomic_section},
        irqn::SysIrqn,
        scb::SCB,
        systick::SysTickDevice,
    },
    kernel::{
        kernel::{z_current, z_next, Kernel},
        threading::{Stack, Thread},
        userspace::{self, k_svc_sleep},
    },
    serial::{SerialConfig, SerialTrait},
    serial_utils::Hex,
    soc::mps2_an385::{UartDevice, UART0},
    stdio,
};
use kernel::{print, println};

use crate::ref_cast_lifetime;

const FreqSysTick: u32 = 1_000; // Hz

// init kernel
static mut KERNEL: Kernel<FreqSysTick> = Kernel::init();

#[no_mangle]
pub extern "C" fn z_systick() {
    /* Systick interrupt is executed with the highest priority and cannot be preempted
     * This is a *natural* critical section with the maximum degree
     */
    let cs = unsafe { Cs::<GlobalIrq>::new() };

    /* Increment ticks */
    unsafe { KERNEL.increment_ticks(&cs) };
}

fn k_yield() {
    unsafe {
        z_current = KERNEL.current_ptr();
        KERNEL.sched_next_thread();
        z_next = KERNEL.current_ptr();
        k_call_pendsv();
    };
}

#[no_mangle]
pub extern "C" fn _start() {
    // Initialize uart
    let mut uart = UartDevice::<FCPU>::new(UART0);
    let uart_config = SerialConfig::default();
    uart.init(&uart_config);
    let _ = uart.write_str("arm rust RTOS demo starting\n");
    let _ = uart.write_fmt(format_args!("Hello, world: {}\n", Hex::U16(2024)));

    // Set UART0 as main uart
    stdio::set_uart(uart);

    let mut systick = SysTickDevice::<FCPU>::instance();
    systick.configure::<FreqSysTick>(true);

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

    // initialize task1
    #[link_section = ".noinit"]
    static mut THREAD_STACK1: [u8; 4096] = [0; 4096];

    let stack1 = Stack::new(unsafe { &mut THREAD_STACK1 }).unwrap();
    let task1 = Thread::init(&stack1, mytask_entry, 0xaaaa0000 as *mut c_void);

    let task1_ref = &task1;
    let task1_static = unsafe { ref_cast_lifetime(task1_ref) };
    unsafe { KERNEL.register_thread(task1_static) }

    // initialize task2
    #[link_section = ".noinit"]
    static mut THREAD_STACK2: [u8; 4096] = [0; 4096];

    let stack2 = Stack::new(unsafe { &mut THREAD_STACK2 }).unwrap();
    let task2 = Thread::init(&stack2, mytask_entry, 0xbbbb0000 as *mut c_void);

    let task2_ref = &task2;
    let task2_static = unsafe { ref_cast_lifetime(task2_ref) };
    unsafe { KERNEL.register_thread(task2_static) }

    // initialize task3
    #[link_section = ".noinit"]
    static mut THREAD_STACK3: [u8; 4096] = [0; 4096];

    let stack3 = Stack::new(unsafe { &mut THREAD_STACK3 }).unwrap();
    let task3 = Thread::init(&stack3, mytask_entry3, 0xcccc0000 as *mut c_void);

    let task3_ref = &task3;
    let task3_static = unsafe { ref_cast_lifetime(task3_ref) };
    unsafe { KERNEL.register_thread(task3_static) }

    // init kernel
    unsafe { KERNEL.register_main_thread() }

    unsafe { KERNEL.print_tasks() }

    loop {
        if let Some(byte) = stdio::read() {
            println!("recv: {}", Hex::U8(byte));

            println!(
                "[ticks: {}]: cur {}",
                atomic_restore(|cs| unsafe { KERNEL.get_ticks(cs) }),
                unsafe { KERNEL.current() }
            );

            let mut syscall_ret = 0;

            match byte {
                b'b' => unsafe { KERNEL.busy_wait(1000) },
                b'p' => {
                    println!("PendSV !");
                    k_yield();
                }
                b'y' => {
                    println!("SVC yield");
                    syscall_ret = userspace::k_svc_yield();
                }
                b's' => {
                    println!("SVC sleep");
                    syscall_ret = userspace::k_svc_sleep(1000);
                }
                b'v' => {
                    println!("SVC debug");
                    syscall_ret = userspace::k_svc_yield();
                }
                b'w' => {
                    println!("SVC print");
                    let msg = "Hello using SVC !!\n";
                    syscall_ret = userspace::k_svc_print(msg);
                }
                b'a' => {
                    println!("aborting...");
                    intrinsics::abort();
                }
                _ => {}
            }

            println!("syscall_ret: {}", Hex::U32(syscall_ret as u32));
        }
    }
}

extern "C" fn mytask_entry(arg: *mut c_void) -> ! {
    let mut counter: u32 = 0;

    loop {
        println!(
            "MyTask arg: {}, counter: {}",
            Hex::U32(arg as u32),
            Hex::U32(counter)
        );

        k_yield();

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
    if control_register.fpca().is_active() {
        let _ = print!(", FPU");
    }
    let _ = println!();
}
