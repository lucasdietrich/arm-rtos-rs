# Rust ARM RTOS playground

This personal *playground* project is an attempt to build a simple/naive RTOS for ARM Cortex-M microcontrollers in Rust.

## Desired features

- [ ] Architecture: ARM Cortex-M4 (`thumbv7em-none-eabihf`)
- [ ] Device: mps2_an385, stm32f4xx

- [ ] Cortex M3/M4 initialization
    - [ ] Stack initialization
    - [ ] Vector table
    - [ ] Reset handler
    - [ ] Interrupts
    - [ ] SysTick
- [ ] Peripherals: UART
- [ ] RTOS features:
    - [ ] thread switch (without FPU support)
    - [ ] cooperative scheduling
    - [ ] preemptive scheduling
    - [ ] sleep
    - [ ] mutex
    - [ ] semaphore
    - [ ] minimal drivers support for UART and GPIO
    - [ ] syscalls:
        - [ ] printf
        - [ ] sleep
        - [ ] fork
        - [ ] mutex
        - [ ] semaphore 
- [ ] Minimal process: load an application from an elf file and run it
    - [ ] parse elf file
    - [ ] toolchain for build the application (C/Rust + linker script + relocation? + syscalls)

## Questions/ideas

Target triplet ?

- `thumbv7em-none-eabihf`, or maybe `thumbv7m-none-eabi` is enough ?
