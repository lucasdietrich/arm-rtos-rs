# Rust ARM RTOS playground

This personal *learning* project is an attempt to build a simple/naive RTOS for ARM Cortex-M microcontrollers in Rust.
Targeted microcontrollers are the `mps2_an386` (arm v7 Cortex M4) platform and the `stm32f429zi` MCU (arm v7 Cortex M4).
I've already worked on an RTOS for AVR 8 bits microcontrollers, written in C: <https://github.com/lucasdietrich/AVRTOS>

## Features

- [ ] Architecture: (`thumbv7em-none-eabihf`), devices:
    - [ ] mps2_an385 (armv6 Cortex-M3 )
        - compile flags: `-mfloat-abi=soft -march=armv6m -mfpu=none`
        - target triplet: `thumbv6m-unknown-none-eabi`
        - [AN385]: https://developer.arm.com/documentation/dai0385/latest/
    - [x] mps2_an386 (armv7 Cortex-M4 )
        - compile flags: `-mfloat-abi=softfp -march=armv7m -mfpu=fpv4-sp-d16`
        - target triplet: `thumbv7m-unknown-none-eabi`
        - [AN386]: https://developer.arm.com/documentation/dai0386/latest/
    - [ ] stm32f4xx (armv7 Cortex-M4)
- [x] Cortex M3/M4 initialization
    - [x] RAM initialization
    - [x] Vector table
    - [x] Reset handler
    - [x] PendSV
      - [x] Configure lowest priority (0b111)
    - [x] SVCall
      - [x] Configure lowest priority (0b111)
    - [x] Systick
      - [x] Configure highest priority (0b000)
    - [ ] Other interrupts
- [ ] minimal drivers support
    - [ ] UART
      - [x] mps2_an385
      - [x] mps2_an386
      - [ ] stm32f4xx
- [ ] RTOS features:
    - [x] stacks
        - [x] system stack
        - [x] user stack
        - [ ] irq stack
    - [x] MSP/PSP
    - [x] thread switch (without FPU support)
    - [x] cooperative scheduling
    - [x] preemptive scheduling
    - [x] sleep
    - [x] mutex
    - [x] semaphore
    - [x] syscalls:
        - [x] printf
        - [x] sleep
        - [ ] fork (needs MMU)
        - [x] mutex
        - [x] semaphore 
        - [x] memory allocation
    - [ ] std library (allocator, collections, etc.)
        - [ ] rust
        - [ ] C
- [x] Minimal process: load an application from an elf file and run it
    - [x] parse elf file
    - [x] build toolchain with crosstool-ng for C development
        - [ ] custom linker script ?
    - [ ] write a minimal libc for the os (syscalls)

## Expected output (loadable elf)

```
===========================
arm rust RTOS demo starting
===========================
CPUID base: 0x410fc240
interrupts enabled: true
Systick prio: 0x00
PendSV prio: 0x00
SVC prio: 0x00
Systick prio: 0x00
PendSV prio: 0x07
SVC prio: 0x07
control: 0x00000000  priviledged, MSP, no FPU
elf 1 loaded
elf 2 loaded
Kernel initialized, starting kernel loop and user threads ...
===========================
Test syscall: r0=10005f4a, r1=0, r2=0, r3=0
Test syscall: r0=10005f44, r1=0, r2=0, r3=0
.42 65 6C 6C 6F 20 57 6F 72 6C 64 21 0A 00 00 00 
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 
42 65 6C 6C 6F 20 57 6F 72 6C 64 21 0A 00 00 00 
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 
Bello World!
Bello World!
Test syscall: r0=1, r1=2, r2=3, r3=4
Test syscall: r0=1, r1=2, r2=3, r3=4
Test syscall: r0=0, r1=2000dfc0, r2=20, r3=87654321
Test syscall: r0=0, r1=2000bf08, r2=20, r3=87654321
Test syscall: r0=0, r1=0, r2=0, r3=0
Loadable ELF returned: 2a
Test syscall: r0=0, r1=0, r2=0, r3=0
Loadable ELF returned: 2a
......................................................
```

## Ressources

- [The embedonomicon](https://docs.rust-embedded.org/embedonomicon/preface.html)
- [A Practical guide to ARM Cortex-M Exception Handling](https://interrupt.memfault.com/blog/arm-cortex-m-exceptions-and-nvic)

- [QEMU / System Emulation / Generic Loader](https://www.qemu.org/docs/master/system/generic-loader.html)
- [crate: cortex_m_rt](https://docs.rs/cortex-m-rt/latest/cortex_m_rt/)
- [Embedded Systems Security and TrustZone](https://embeddedsecurity.io/)
- Inline assembly:
  - [**The Rust Reference: Inline assembly**](https://doc.rust-lang.org/reference/inline-assembly.html)
  - [Nightly: Inline assembly](https://doc.rust-lang.org/nightly/rust-by-example/unsafe/asm.html)
- [The Embedded Rust Book: Concurrency](https://docs.rust-embedded.org/book/concurrency/)
- [LLVM-embedded-toolchain-for-Arm/CMakeLists.txt](https://github.com/ARM-software/LLVM-embedded-toolchain-for-Arm/blob/main/CMakeLists.txt)
- [Position-Independent Code with GCC for ARM Cortex-M](https://mcuoneclipse.com/2021/06/05/position-independent-code-with-gcc-for-arm-cortex-m/)
- [Support for position independent code](https://developer.arm.com/documentation/100748/0623/Mapping-Code-and-Data-to-the-Target/Support-for-position-independent-code)
    - [Bare-metal Position Independent Executables](https://developer.arm.com/documentation/100748/0623/Mapping-Code-and-Data-to-the-Target/Bare-metal-Position-Independent-Executables?lang=en)
    - [Procedure Call Standard for the Arm® Architecture](https://github.com/ARM-software/abi-aa/blob/main/aapcs32/aapcs32.rst)
- [GCC ARM Options](https://gcc.gnu.org/onlinedocs/gcc-6.1.0/gcc/ARM-Options.html)
- [OpenBSD Position Independent Executable (PIE)](https://www.openbsd.org/papers/nycbsdcon08-pie/)

### Datasheets:

- [Procedure Call Standard for the ARM® Architecture](https://web.eecs.umich.edu/~prabal/teaching/resources/eecs373/ARM-AAPCS-EABI-v2.08.pdf)
- [Cortex-M3 Technical Reference Manual](https://documentation-service.arm.com/static/5e8e107f88295d1e18d34714?token=)
- [Deprecated Features in ARMv7-M](https://documentation-service.arm.com/static/5f8fedcbf86e16515cdbf30f?token=)

---

## Questions/ideas/problems

TODO:

- ~~understand why .data MYVAR is already initialized in QEMU~~ -> QEMU loads the .data section from the ELF file to RAM
- ~~understand why .data .bss appears in the ELF file~~ -> QEMU loads the .bss section from the ELF file to RAM
- Add the noinit section to the linker script
- If symbol gets wiped out of the elf, gdb won't find it, we need to force the symbol to be kept in the elf file -> how to ? (e.g. _thread_switch)
- Proper Systick/FCPU and FREQ_SYS_TICK handling
- Choose distinct allocators for multiple Box<T> (e.g. one for the kernel, one for the user)
- Implement synchronization primitives cancellation mecanism (trait function + syscall)
- Implement identifier for synchronization primitives (number, str ??)

## Notes

### Rust toolchain

Tested with the following toolchains:

- `nightly-2024-10-01`
- `nightly-2024-07-08`

To install a new toolchain:

```
rustup toolchain add nightly-2024-10-01 --profile minimal
```

### Static and const

```rs
const FOO: u32 = 42; // Const is a compile-time constant
static BAR: u32 = 42; // Static is a runtime constant
static mut BAZ: u32 = 42; // Static mutable 
```

### Export symbols

Disable name mangling for a function:

```rs

#[export_name = "switch_to_user"]
fn switch_to_user() {
    // ...
}

#[export_name = "my_symbol"]
extern "C" fn my_function() {
    // ...
}

```

### Links to a section
    
```rs
#[link_section = ".kvars"]
static mut BAZ: u32 = 42;
```

### Make static variable extern

In order to export the symbol of a static variable, it must be declared with `#[used]`:
The `no_mangle` attribute make sure the symbol name is not mangled by the compiler (e.g. demo::entry::z_current -> z_current)

```rs
#[used]
#[no_mangle]
pub static mut z_current: *mut Thread = core::ptr::null_mut();
```

### Rename symbol

!!! warning "TODO"
    What is bellow is probably wrong

`link_name` must only be used on statics and functions that are in an `extern` block.

```rs
extern "C" {
    #[link_name = "z_current"]
    static mut z_current: *mut Thread;
}
```

### Write ASM in rust code

Following inline assembly code is equivalent to the rust code:

```rs
use use core::arch::global_asm;
global_asm!(
    "
    .section .text, \"ax\"
    .global _pendsv
    .thumb_func
_pendsv:
    push {{r7, lr}}
    mov	r7, sp
    nop
    pop	{{r7, pc}}
    "
);

extern "C" {
    pub fn _pendsv();
}
```

Pure rust:

```rs
use use core::arch::asm;
#[no_mangle]
pub unsafe extern "C" fn _pendsv() {
    asm!("nop");
}
```

It's currently impossible to write naked functions in Rust, see <https://github.com/rust-lang/rust/issues/90957> for support for `#[naked]` functions.

### Static initialization

A static variable can be initialized using a `const` function:

```rs
pub struct Kernel;

impl Kernel {
    pub const fn init() -> Kernel {
        Kernel {}
    }
}

fn main() {
    static mut KERNEL: Kernel = Kernel::init();
}
```

### cortex-debug: Watch variables

If you want to watch a static rust variable, you need to use its full name, for example:

![pics/cortex-debug-watch-rust-static.png](pics/cortex-debug-watch-rust-static.png)

The full names can be found in the output of `nm`: e.g. `2000000c 00000014 d demo::KERNEL`

### PhantomData of non generic (TODO)

What is the purpose of `PhantomData` in the following code ?

```rs
pub struct SCB {
    _marker: PhantomData<*const ()>,
}
```

### Force inlining

Feel free to help the compiler to inline a function by using the `#[inline(always)]` attribute:

```rs
impl<D: CsDomain> Cs<D> {
    #[inline(always)]
    /* This is the only method to obtain a critical session object */
    pub unsafe fn new() -> Self {
        Cs {
            domain: PhantomData,
        }
    }
}
```

### Mark as uninit

Set variable in the `.noinit` section:

```rs
#[link_section = ".noinit"]
static mut THREAD_STACK1: u32 = 0;
```

### Define a KernelSpecs trait

It would be great to have: 

```rs
pub trait KernelSpecs {
    const FREQ_SYS_TICK: u32 = 100;
    const KOBJS: usize = 32;
}

// CPU: CPU variant
pub struct Kernel<'a, CPU: CpuVariant, Specs: KernelSpecs>
where
    [(); Specs::KOBJS]:,
    [(); Specs::FREQ_SYS_TICK as usize]:,
{
    tasks: sl::List<'a, Thread<'a, CPU>, Runqueue>,

    // systick
    systick: SysTick<{ Specs::FREQ_SYS_TICK }>,

    // Ticks counter: period: P (ms)
    ticks: u64,

    idle: Thread<'a, CPU>,
    // Idle thread

    // Kernel objects (Sync) for synchronization
    kobj: [Option<Box<dyn KernelObjectTrait<'a, CPU> + 'a>>; Specs::KOBJS],
}
```

However the feature is not well supported today, it needs `#![feature(generic_const_exprs)]`
This is discussed here: <https://github.com/rust-lang/rust/issues/76560>
