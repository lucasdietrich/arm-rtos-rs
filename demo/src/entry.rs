use core::{arch::asm, ffi::c_void, fmt::Write, intrinsics, mem::MaybeUninit, ptr::addr_of_mut};

use cortex_m::{interrupt, register::control::Control};
use kernel::{
    cortex_m::{
        arch::CortexM, cortex_m_rt::FCPU, cpu::Cpu, interrupts, irqn::SysIrqn, scb::SCB,
        systick::SysTick,
    },
    kernel::{
        kernel::Kernel,
        stack::Stack,
        thread::Thread,
        userspace::{self, k_svc_print, k_svc_sleep, k_svc_yield},
    },
    serial::{SerialConfig, SerialTrait},
    serial_utils::Hex,
    soc::mps2_an385::{UartDevice, UART0},
    stdio,
};
use kernel::{print, println};

const FreqSysTick: u32 = 100; // Hz

#[no_mangle]
pub extern "C" fn _start() {
    // Initialize uart
    let mut uart = UartDevice::<FCPU>::new(UART0);
    let uart_config = SerialConfig::default();
    uart.init(&uart_config);
    let _ = uart.write_str("arm rust RTOS demo starting\n");

    // Set UART0 as main uart
    stdio::set_uart(uart);

    // Show startup state    let cpuid = SCB::new().get_cpuid();
    let mut scb = SCB::instance();
    println!("CPUID base: {}", Hex::U32(scb.get_cpuid()));

    let primask = interrupts::enabled();
    println!("interrupts enabled: {}", primask);

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
    let systick = SysTick::configure_period::<FCPU, FreqSysTick>(true);
    let mut kernel = Kernel::<CortexM>::init(systick);

    // initialize task1
    #[link_section = ".noinit"]
    static mut THREAD_STACK1: Stack<32768> = Stack::init();

    let stack1 = unsafe { &mut THREAD_STACK1 }.get_info();
    let task1 = Thread::init(&stack1, mytask_entry, 0xaaaa0000 as *mut c_void);

    kernel.register_thread(&task1);

    // initialize task2
    #[link_section = ".noinit"]
    static mut THREAD_STACK2: Stack<32768> = Stack::init();

    let stack2 = unsafe { &mut THREAD_STACK2 }.get_info();
    let task2 = Thread::init(&stack2, mytask_entry, 0xbbbb0000 as *mut c_void);

    kernel.register_thread(&task2);

    // initialize task3
    #[link_section = ".noinit"]
    static mut THREAD_STACK3: Stack<32768> = Stack::init();

    let stack3 = unsafe { &mut THREAD_STACK3 }.get_info();
    let task3 = Thread::init(&stack3, mytask_entry3, 0xcccc0000 as *mut c_void);

    kernel.register_thread(&task3);

    loop {
        kernel.kernel_loop();
    }
}

extern "C" fn mytask_shell(arg: *mut c_void) -> ! {
    loop {
        if let Some(byte) = stdio::read() {
            println!("recv: {}", Hex::U8(byte));

            let mut syscall_ret = 0;

            match byte {
                b'y' => {
                    println!("yield !");
                    userspace::k_svc_yield();
                }
                b's' => {
                    println!("SVC sleep");
                    syscall_ret = userspace::k_svc_sleep(1000);
                }
                b'w' => {
                    println!("SVC print");
                    let msg = "Hello using SVC !!\n";
                    syscall_ret = userspace::k_svc_print(msg);
                }
                _ => {}
            }

            println!("syscall_ret: {}", Hex::U32(syscall_ret as u32));
        }
    }
}

extern "C" fn mytask_entry(arg: *mut c_void) -> ! {
    println!("MyTask arg: {}", Hex::U32(arg as u32),);

    display_control_register();

    let regs = Cpu::registers();
    Cpu::print_registers(&regs);

    let mut counter: u32 = 0;

    loop {
        println!("[{}] counter: {}", Hex::U32(arg as u32), Hex::U32(counter));
        counter = counter.wrapping_add(1);

        k_svc_yield();
    }
}

extern "C" fn mytask_entry3(arg: *mut c_void) -> ! {
    loop {
        println!("MyTask arg: {}, sleep", Hex::U32(arg as u32),);

        let msg = "Hello using SVC !!\n";
        k_svc_print(msg);
        k_svc_sleep(1000);
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
