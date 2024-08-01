#![no_std]
#![no_main]
#![feature(stdarch_arm_hints)]
#![feature(stdarch_arm_neon_intrinsics)]
#![feature(core_intrinsics)]
#![feature(maybe_uninit_uninit_array)]
use core::{ffi::c_void, fmt::Write, intrinsics, ptr::addr_of_mut};

use cortex_m::{
    register::{self, control::Control},
    Peripherals,
};
use cortex_m_rt::{enable_irq, pendsv_set};
use kernel::Kernel;
use mps2_an385::{UartDevice, UART0};
use serial::{SerialConfig, SerialTrait};
use serial_utils::Hex;
use threading::{Stack, Thread};
mod cortex_m_rt;
mod io;
mod kernel;
mod serial;
mod serial_utils;
mod task;
mod threading;

#[cfg(feature = "mps2-an385")]
mod mps2_an385;

static RO_MYVAR: u32 = 10;
static mut DATA_MYVAR: u32 = 10;

/* Explicitly place the var in the .bss section
 * but this is not necessary, as all 0-initialized
 * vars are placed in the .bss section by default */
#[link_section = ".bss"]
static mut BSS_MYVAR: u32 = 0;

pub const FCPU: u32 = 25_000_000;

// init kernel
static mut KERNEL: Kernel<2> = Kernel::init();

#[no_mangle]
pub unsafe extern "C" fn _pendsv() {
    unsafe { KERNEL.pendsv_handler() };
}

pub fn _start() {
    // Initialize uart
    let mut uart = UartDevice::<FCPU>::new(UART0);
    let uart_config = SerialConfig::default();
    uart.init(&uart_config);
    let _ = uart.write_str("arm rust RTOS demo starting\n");
    let _ = uart.write_fmt(format_args!("Hello, world: {}\n", Hex::U16(2024)));

    io::set_uart(uart);

    // Show startup state
    let p = cortex_m::Peripherals::take().unwrap();
    display_cpuid(&p);
    display_control_register();

    #[link_section = ".noinit"]
    static mut THREAD_STACK: [u8; 4096] = [0; 4096];

    // initialize task
    let stack = Stack::new(unsafe { &mut THREAD_STACK });
    let task1 = Thread::init(&stack, mytask, 0xbadebeaf as *mut c_void);
    println!("task1: {}", task1);
    if let Err(task) = unsafe { KERNEL.register_thread(task1) } {
        println!("Failed to register task: {}", task)
    }

    loop {
        if let Some(byte) = io::read() {
            println!("recv: {}", Hex::U8(byte));

            match byte {
                b'p' => unsafe {
                    // enable_irq();
                    println!("PendSV !");
                    pendsv_set();
                },
                b'a' => {
                    intrinsics::abort();
                }
                _ => {}
            }
        }
    }
}

extern "C" fn mytask(arg: *mut c_void) -> ! {
    let mut counter: u32 = 0;

    loop {
        println!(
            "MyTask arg: arg: {}, counter: {}",
            Hex::U32(arg as u32),
            Hex::U32(counter)
        );

        unsafe { pendsv_set() };

        counter = counter.wrapping_add(1);
    }
}

// Read CPUID base register
fn display_cpuid(p: &Peripherals) {
    let cpuid_base = p.CPUID.base.read();
    println!("CPUID base: {}", Hex::U32(cpuid_base));
}

fn display_control_register() {
    // Print control (special) register
    let control_register: Control = cortex_m::register::control::read();
    let _ = print!("control:");
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
