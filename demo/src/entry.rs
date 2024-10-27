use core::fmt::Write;

use cortex_m::register::control::Control;
use kernel::{
    cortex_m::{
        arch::CortexM, cortex_m_rt::FCPU, interrupts, irqn::SysIrqn, scb::SCB, systick::SysTick,
    },
    kernel::kernel::Kernel,
    serial::{SerialConfig, SerialTrait},
    serial_utils::Hex,
    soc::mps2_an38x::{UartDevice, UART0},
    stdio,
};
use kernel::{print, println};

use crate::{shell, signal};

pub const FREQ_SYS_TICK: u32 = 100; // Hz

pub const USER_THREAD_SIZE: usize = 16384;

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
    let systick = SysTick::<FREQ_SYS_TICK>::configure_period::<FCPU>(true);
    let mut kernel = Kernel::<CortexM, 32, FREQ_SYS_TICK>::init(systick);

    #[cfg(feature = "signal")]
    let signal_threads = signal::init_threads();
    #[cfg(feature = "signal")]
    for thread in signal_threads.iter() {
        kernel.register_thread(&thread);
    }

    #[cfg(feature = "shell")]
    let shell_thread = shell::init_shell_thread();
    #[cfg(feature = "shell")]
    kernel.register_thread(&shell_thread);

    loop {
        kernel.kernel_loop();
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
