# Rust ARM RTOS playground

This personal *learning* project is an attempt to build a simple/naive RTOS for ARM Cortex-M microcontrollers in Rust.
Targeted microcontrollers are the `mps2_an385` platform and the `stm32f429zi` MCU.
I've already worked on an RTOS for AVR 8 bits microcontrollers, written in C: <https://github.com/lucasdietrich/AVRTOS>

## Ressources

- [The embedonomicon](https://docs.rust-embedded.org/embedonomicon/preface.html)

- [QEMU / System Emulation / Generic Loader](https://www.qemu.org/docs/master/system/generic-loader.html)
- [crate: cortex_m_rt](https://docs.rs/cortex-m-rt/latest/cortex_m_rt/)

## Desired features

- [ ] Architecture: ARM Cortex-M3 (`thumbv7em-none-eabihf`), devices:
    - [ ] mps2_an385
    - [ ] stm32f4xx

- [x] Cortex M3/M4 initialization
    - [x] RAM initialization
    - [ ] Vector table
    - [x] Reset handler
    - [ ] Interrupts
    - [ ] SysTick
- [ ] Peripherals: UART
    - [x] mps2_an385
    - [ ] stm32f4xx
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
- ~~understand why .data MYVAR is already initialized in QEMU~~ -> QEMU loads the .data section from the ELF file to RAM
- ~~understand why .data .bss appears in the ELF file~~ -> QEMU loads the .bss section from the ELF file to RAM
