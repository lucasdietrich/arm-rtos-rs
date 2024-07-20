# Rust ARM RTOS playground

This personal *playground* project is an attempt to build a simple/naive RTOS for ARM Cortex-M microcontrollers in Rust.

## Ressources

- [The embedonomicon](https://docs.rust-embedded.org/embedonomicon/preface.html)

- [QEMU / System Emulation / Generic Loader](https://www.qemu.org/docs/master/system/generic-loader.html)
- [crate: cortex_m_rt](https://docs.rs/cortex-m-rt/latest/cortex_m_rt/)

## Desired features

- [ ] Architecture: ARM Cortex-M3 (`thumbv7em-none-eabihf`)
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

TODO:

- Target triplet ? `thumbv7em-none-eabihf`, or maybe `thumbv7m-none-eabi` is enough ?
- understand why .data MYVAR is already initialized in QEMU
- understand why .data .bss appears in the ELF file
