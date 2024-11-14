# Book

The purpose of this book is to provide a overview of my learning journey in building
a real-time operating system (RTOS) from scratch for ARM Cortex-M microcontrollers with Rust.
Focusing on providing a minimal usable userspace for C programs (syscalls, toolchain, loadable programs, etc).
And finally experiencing it with real hardware.

I'm not an RTOS expert, I'm just very interested in learning fundamentals and want to share my experience
of rust for bare metal programming.
There's probably are a lot of other implementations that are more efficient and feature rich rtoses attempts: Tock, embassy, RTIC, Rust for Zephyr oo, etc.

Many things could be improved, non exhaustive list:

- Don't use Box or use separate allocators for different parts of the kernel
- Focus on drivers and peripherals
- Implement more idiomatic Rust API for synchronization primitives
- Implement a proper scheduler
- Minimize stack usage
- Make the kernel more architecture agnostic
- Implement a MMU/MPU

## ARM Cortex M with RUST

- Arch / Supervisor/User
- Linker script and CPU initialization !
- Inline assembly
- Very few assembly code
- No handling in interrupts
- Not much unsafe code ??!

## Bare metal rust

- Toolchain
- Targets (qemu, real hardware)

## RTOS

- Threads
- Stack
- Interrupts (systick)
- Syscalls
- Scheduler
- Allocator: Bump allocator
- Synchronization primitives

## Finally a userspace

- Toolchain (crosstool-ng)
- Loadable programs (elf, PIC, relocation, syscalls)
- Syscalls

## Real hardware
- stm32f4xx
- Upload protocol for loadable programs
- Custom board ??? (teal) -> leds

## Conclusion
- Goind further:
- References/credits
