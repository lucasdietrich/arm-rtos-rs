use core::{ffi::c_void, fmt::Write};

use cortex_m::register::control::Control;
use kernel::{
    cortex_m::{
        arch::CortexM, cortex_m_rt::FCPU, cpu::Cpu, interrupts, irqn::SysIrqn, scb::SCB,
        systick::SysTick,
    },
    kernel::{kernel::Kernel, stack::Stack, thread::Thread, userspace},
    serial::{SerialConfig, SerialTrait},
    serial_utils::Hex,
    soc::mps2_an385::{UartDevice, UART0},
    stdio,
};
use kernel::{print, println};

pub const FREQ_SYS_TICK: u32 = 100; // Hz

const USER_THREAD_SIZE: usize = 16384;

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

    // init kernel
    let systick = SysTick::configure_period::<FCPU, FREQ_SYS_TICK>(true);
    let mut kernel = Kernel::<CortexM, 32>::init(systick);

    // // initialize task1
    // #[link_section = ".noinit"]
    // static mut THREAD_STACK1: Stack<USER_THREAD_SIZE> = Stack::uninit();

    // let stack1 = unsafe { THREAD_STACK1.get_info() };
    // let task1 = Thread::init(&stack1, mytask_entry, 0xaaaa0000 as *mut c_void, 0);

    // kernel.register_thread(&task1);

    // // initialize task2
    // #[link_section = ".noinit"]
    // static mut THREAD_STACK2: Stack<USER_THREAD_SIZE> = Stack::uninit();

    // let stack2 = unsafe { THREAD_STACK2.get_info() };
    // let task2 = Thread::init(&stack2, mytask_shell, 0xbbbb0000 as *mut c_void, 0);

    // kernel.register_thread(&task2);

    // // initialize task3
    // #[link_section = ".noinit"]
    // static mut THREAD_STACK3: Stack<USER_THREAD_SIZE> = Stack::uninit();

    // let stack3 = unsafe { THREAD_STACK3.get_info() };
    // let task3 = Thread::init(&stack3, mytask_entry3, 0xcccc0000 as *mut c_void, 0);

    // kernel.register_thread(&task3);

    // initialize task4
    #[link_section = ".noinit"]
    static mut THREAD_STACK4: Stack<USER_THREAD_SIZE> = Stack::uninit();

    let stack4 = unsafe { THREAD_STACK4.get_info() };
    let task4 = Thread::init(&stack4, mytask_pend, 0xdddd0000 as *mut c_void, 0);

    kernel.register_thread(&task4);

    // initialize task4
    #[link_section = ".noinit"]
    static mut THREAD_STACK5: Stack<USER_THREAD_SIZE> = Stack::uninit();

    let stack5 = unsafe { THREAD_STACK5.get_info() };
    let task5 = Thread::init(&stack5, mytask_sync, 0xeeee0000 as *mut c_void, 0);

    kernel.register_thread(&task5);

    loop {
        kernel.kernel_loop();
    }
}

extern "C" fn mytask_shell(_arg: *mut c_void) -> ! {
    loop {
        if let Some(byte) = stdio::read() {
            println!("recv: {}", Hex::U8(byte));

            let mut syscall_ret = 0;

            match byte {
                b'y' => {
                    println!("yield !");
                    userspace::k_yield();
                }
                b's' => {
                    println!("SVC sleep");
                    syscall_ret = userspace::k_sleep(1000);
                }
                b'w' => {
                    println!("SVC print");
                    let msg = "Hello using SVC !!\n";
                    syscall_ret = userspace::k_print(msg);
                }
                _ => {}
            }

            println!("syscall_ret: {}", Hex::U32(syscall_ret as u32));
        }

        userspace::k_sleep(100);
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

        let svc_ret = userspace::k_sleep(1000);
        println!("svc_ret: {}", svc_ret);
    }
}

extern "C" fn mytask_pend(arg: *mut c_void) -> ! {
    let kobj = userspace::k_sync_create();
    println!("kobj {}", kobj);

    loop {
        let ret = userspace::k_pend(kobj);
        println!("pend {}", ret);

        userspace::k_sleep(500);
    }
}

extern "C" fn mytask_sync(arg: *mut c_void) -> ! {
    let kobj = 0_i32;

    userspace::k_sleep(2000);
    let ret = userspace::k_sync(kobj);
    println!("sync {}", ret);

    loop {
        userspace::k_sleep(10000);
    }
}

extern "C" fn mytask_entry3(arg: *mut c_void) -> ! {
    loop {
        println!("MyTask arg: {}, sleep", Hex::U32(arg as u32),);

        let msg = "Hello using SVC !!\n";
        userspace::k_print(msg);
        userspace::k_sleep(1000);
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
