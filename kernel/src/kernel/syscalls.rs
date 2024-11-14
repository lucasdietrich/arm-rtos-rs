use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use super::timeout::Timeout;

#[derive(Debug)]
pub struct SVCCallParams {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,        // Contains the exact function within the SyscallId
    pub syscall_id: u8, // Contains the SyscallId
}

#[repr(u8)]
#[derive(FromPrimitive)]
pub enum SyscallId {
    Test = 0,
    Kernel = 1,
    Io = 2,
    Driver = 3,
}

#[repr(u32)]
#[derive(FromPrimitive)]
pub enum KernelSyscallId {
    // Yield CPU
    Yield = 0,
    // Make thread sleep for the specified duration
    Sleep = 1,
    // Create synchronization object
    SyncCreate = 3,
    // Synchronize (i.e.) first thread pending on a kernel object
    Sync = 4,
    // Make thread pend on a kernel object
    Pend = 5,
    // Cancel a pending operation on a kernel object
    Cancel = 6,
    // Stop the current thread
    Stop = 7,
    // Allocate memory for a new thread
    MemoryAlloc = 8,
    // Free memory for a thread
    MemoryFree = 9,
    // Clone the current thread into a new thread
    Fork = 10,
    // // Uptime
    // Uptime = 100,
}

#[repr(u32)]
#[derive(FromPrimitive)]
pub enum IoSyscallId {
    Write = 0,
    Read = 1,
    HexPrint = 10,
    Read1 = 11,
}

#[derive(Debug)]
pub enum Syscall {
    Test { r0: u32, r1: u32, r2: u32, r3: u32 },
    Kernel(KernelSyscall),
    Io(IoSyscall),
    Driver,
}

impl Syscall {
    pub fn from_svc_params(params: SVCCallParams) -> Option<Syscall> {
        SyscallId::from_u8(params.syscall_id).and_then(|syscall_id| match syscall_id {
            SyscallId::Test => Some(Syscall::Test {
                r0: params.r0,
                r1: params.r1,
                r2: params.r2,
                r3: params.r3,
            }),
            SyscallId::Kernel => KernelSyscallId::from_u32(params.r3).and_then(|kernel_syscall| {
                match kernel_syscall {
                    KernelSyscallId::Yield => Some(KernelSyscall::Yield),
                    KernelSyscallId::Sleep => Some(KernelSyscall::Sleep { ms: params.r0 }),
                    KernelSyscallId::SyncCreate => {
                        SyncPrimitiveType::from_u32(params.r2).map(|sync_prim_type| {
                            KernelSyscall::SyncCreate {
                                prim: match sync_prim_type {
                                    SyncPrimitiveType::Sync => SyncPrimitiveCreate::Sync,
                                    SyncPrimitiveType::Signal => SyncPrimitiveCreate::Signal,
                                    SyncPrimitiveType::Semaphore => {
                                        SyncPrimitiveCreate::Semaphore {
                                            init: params.r0,
                                            max: params.r1,
                                        }
                                    }
                                    SyncPrimitiveType::Mutex => SyncPrimitiveCreate::Mutex,
                                },
                            }
                        })
                    }
                    KernelSyscallId::Sync => {
                        SyncPrimitiveType::from_u32(params.r2).map(|sync_prim_type| {
                            KernelSyscall::Sync {
                                arg: params.r0,
                                kobj: params.r1 as i32,
                                prim: sync_prim_type,
                            }
                        })
                    }
                    KernelSyscallId::Pend => {
                        SyncPrimitiveType::from_u32(params.r2).map(|sync_prim_type| {
                            KernelSyscall::Pend {
                                timeout: Timeout::try_from(params.r0 as i32).unwrap_or_default(),
                                kobj: params.r1 as i32,
                                prim: sync_prim_type,
                            }
                        })
                    }
                    KernelSyscallId::Cancel => {
                        SyncPrimitiveType::from_u32(params.r2).map(|sync_prim_type| {
                            KernelSyscall::Cancel {
                                kobj: params.r1 as i32,
                                prim: sync_prim_type,
                            }
                        })
                    }
                    KernelSyscallId::Stop => Some(KernelSyscall::Stop),
                    KernelSyscallId::MemoryAlloc => Some(KernelSyscall::MemoryAlloc {
                        size: params.r0 as usize,
                        align: params.r1 as usize,
                    }),
                    KernelSyscallId::MemoryFree => Some(KernelSyscall::MemoryFree {
                        ptr: params.r0 as *mut u8,
                    }),
                    KernelSyscallId::Fork => Some(KernelSyscall::Fork),
                }
                .map(Syscall::Kernel)
            }),
            SyscallId::Io => IoSyscallId::from_u32(params.r3).map(|io_syscall| {
                Syscall::Io(match io_syscall {
                    IoSyscallId::Write => {
                        let ptr = params.r0 as *const u8;
                        let size = params.r1 as usize;
                        let newline = params.r2 != 0;
                        IoSyscall::Print {
                            ptr,
                            len: size,
                            newline,
                        }
                    }
                    IoSyscallId::HexPrint => {
                        let ptr = params.r0 as *const u8;
                        let size = params.r1 as usize;
                        IoSyscall::HexPrint { ptr, len: size }
                    }
                    IoSyscallId::Read => {
                        let ptr = params.r0 as *mut u8;
                        let size = params.r1 as usize;
                        let timeout = Timeout::try_from(params.r2 as i32).unwrap_or_default();
                        IoSyscall::Read {
                            ptr,
                            len: size,
                            timeout,
                        }
                    }
                    IoSyscallId::Read1 => IoSyscall::Read1,
                })
            }),
            SyscallId::Driver => Some(Syscall::Driver),
        })
    }
}

#[repr(u32)]
#[derive(Debug, FromPrimitive, PartialEq, Eq, Clone, Copy)]
pub enum SyncPrimitiveType {
    Sync = 0,
    Signal = 1,
    Semaphore = 2,
    Mutex = 3,
}

#[derive(Debug)]
pub enum SyncPrimitiveCreate {
    Sync,
    Signal,
    Semaphore { init: u32, max: u32 },
    Mutex,
}

#[derive(Debug)]
pub enum KernelSyscall {
    Yield,
    Sleep {
        ms: u32,
    },
    SyncCreate {
        prim: SyncPrimitiveCreate,
    },
    Sync {
        prim: SyncPrimitiveType,
        kobj: i32,
        arg: u32, // Argument to the sync primitive (r0)
    },
    Pend {
        prim: SyncPrimitiveType,
        kobj: i32,
        timeout: Timeout,
    },
    Cancel {
        prim: SyncPrimitiveType,
        kobj: i32,
    },
    MemoryAlloc {
        size: usize,
        align: usize,
    },
    MemoryFree {
        ptr: *mut u8,
    },
    Fork,
    Stop,
}

#[derive(Debug)]
pub enum IoSyscall {
    Print {
        ptr: *const u8,
        len: usize,
        newline: bool,
    },
    Read {
        ptr: *mut u8,
        len: usize,
        timeout: Timeout,
    },
    HexPrint {
        ptr: *const u8,
        len: usize,
    },
    Read1,
}
