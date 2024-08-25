use core::arch::asm;
use core::{ffi::c_void, fmt::Write, intrinsics, ptr::addr_of_mut};

use crate::cortex_m_rt::{enable_irq, k_call_pendsv, FCPU};
use crate::kernel::{z_current, z_next, Kernel};
use crate::mps2_an385::{UartDevice, UART0};
use crate::println;
use crate::serial::{SerialConfig, SerialTrait};
use crate::serial_utils::Hex;
use crate::threading::{Stack, Thread};
use crate::userspace::{k_svc_debug, k_svc_print, k_svc_sleep};
use crate::{io, print};
use cortex_m::{
    register::{self, control::Control},
    Peripherals,
};

// init kernel
static mut KERNEL: Kernel<2> = Kernel::init();

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
        if let Some(byte) = io::read() {
            println!("recv: {}", Hex::U8(byte));

            println!(
                "current thread ({}): {}",
                Hex::U32(unsafe { z_current } as u32),
                unsafe { KERNEL.current() }
            );

            match byte {
                b'p' => unsafe {
                    println!("PendSV !");
                    k_call_pendsv();

                    let temp = z_current;
                    z_current = z_next;
                    z_next = temp;
                },
                b's' => {
                    println!("SVC sleep");
                    k_svc_sleep(1000);
                }
                b'v' => {
                    println!("SVC debug");
                    k_svc_debug();
                }
                b'w' => {
                    println!("SVC print");
                    let msg = "Hello using SVC !!\n";
                    k_svc_print(msg);
                }
                b'a' => {
                    println!("aborting...");
                    intrinsics::abort();
                }
                _ => {}
            }
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

// Read CPUID base register
fn display_cpuid(p: &Peripherals) {
    let cpuid_base = p.CPUID.base.read();
    println!("CPUID base: {}", Hex::U32(cpuid_base));
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
