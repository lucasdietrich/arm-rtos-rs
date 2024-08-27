use core::{ffi::c_void, fmt::Write, intrinsics, ptr::addr_of_mut};

use cortex_m::register::control::Control;
use kernel::{
    cortex_m::{
        cortex_m_rt::{k_call_pendsv, FCPU},
        critical_section::{self, Cs, GlobalIrq},
        interrupts::atomic_section,
        irqn::SysIrqn,
        scb::SCB,
        systick::SysTickDevice,
    },
    kernel::{
        kernel::{z_current, z_next, Kernel},
        threading::{Stack, Thread},
        userspace,
    },
    serial::{SerialConfig, SerialTrait},
    serial_utils::Hex,
    soc::mps2_an385::{UartDevice, UART0},
    stdio,
};
use kernel::{print, println};

const FST: u32 = 1_000; // Hz

// init kernel
static mut KERNEL: Kernel<2, FST> = Kernel::init();

#[no_mangle]
pub extern "C" fn z_systick() {
    /* Systick interrupt is executed with the highest priority and cannot be preempted
     * This is a *natural* critical section with the maximum degree
     */
    let cs = unsafe { Cs::<GlobalIrq>::new() };

    /* Increment ticks */
    unsafe { KERNEL.increment_ticks(&cs) };
}

#[no_mangle]
pub extern "C" fn _start() {
    // Invoke kernel crate

    atomic_section::<false, _, _>(|_cs| {});

    kernel::test();

    // Initialize uart
    let mut uart = UartDevice::<FCPU>::new(UART0);
    let uart_config = SerialConfig::default();
    uart.init(&uart_config);
    let _ = uart.write_str("arm rust RTOS demo starting\n");
    let _ = uart.write_fmt(format_args!("Hello, world: {}\n", Hex::U16(2024)));

    // Set UART0 as main uart
    stdio::set_uart(uart);

    let mut systick = SysTickDevice::<FCPU>::instance();
    systick.configure::<FST>(true);

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

    #[link_section = ".noinit"]
    static mut THREAD_STACK: [u8; 4096] = [0; 4096];

    // initialize task
    let stack = Stack::new(unsafe { &mut THREAD_STACK }).unwrap();
    let task1 = Thread::init(&stack, mytask_entry, 0xbadebeaf as *mut c_void);
    println!("task1: {}", task1);
    match unsafe { KERNEL.register_thread(task1) } {
        Ok(thread_ptr) => {
            println!("Register thread: {}", Hex::U32(thread_ptr as u32));

            // set next thread
            unsafe { z_next = thread_ptr };
        }
        Err(task) => println!("Failed to register task: {}", task),
    }

    unsafe { z_current = KERNEL.get_current_ptr() };

    loop {
        if let Some(byte) = stdio::read() {
            println!("recv: {}", Hex::U8(byte));

            println!(
                "[ticks: {}] \tcurrent thread ({}): {}",
                unsafe { KERNEL.get_ticks() },
                Hex::U32(unsafe { z_current } as u32),
                unsafe { KERNEL.current() }
            );

            let mut syscall_ret = 0;

            match byte {
                b'b' => unsafe { KERNEL.busy_wait(1000) },
                b'p' => unsafe {
                    println!("PendSV !");
                    k_call_pendsv();

                    let temp = z_current;
                    z_current = z_next;
                    z_next = temp;
                },
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
            "MyTask arg: arg: {}, counter: {}",
            Hex::U32(arg as u32),
            Hex::U32(counter)
        );

        unsafe {
            let temp = z_current;
            z_current = z_next;
            z_next = temp;
        }

        unsafe { k_call_pendsv() };

        counter = counter.wrapping_add(1);
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
