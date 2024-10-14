use core::time::Duration;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use super::{sync::SwapData, timeout::Timeout};

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
    // // Uptime
    // Uptime = 6,
}

#[repr(u32)]
#[derive(FromPrimitive)]
pub enum IoSyscallId {
    Print = 0,
}

#[derive(Debug)]
pub enum Syscall {
    Kernel(KernelSyscall),
    Io(IoSyscall),
    Driver,
}

impl Syscall {
    pub fn from_svc_params(params: SVCCallParams) -> Option<Syscall> {
        SyscallId::from_u8(params.syscall_id).and_then(|syscall_id| match syscall_id {
            SyscallId::Kernel => KernelSyscallId::from_u32(params.r3).and_then(|kernel_syscall| {
                match kernel_syscall {
                    KernelSyscallId::Yield => Some(KernelSyscall::Yield),
                    KernelSyscallId::Sleep => Some(KernelSyscall::Sleep { ms: params.r0 }),
                    KernelSyscallId::SyncCreate => {
                        SyncPrimitiveType::from_u32(params.r2).map(|sync_prim_type| {
                            KernelSyscall::Create {
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
                    KernelSyscallId::Sync => Some(KernelSyscall::Sync {
                        kobj: params.r2 as i32,
                    }),
                    KernelSyscallId::Pend => Some(KernelSyscall::Pend {
                        kobj: params.r2 as i32,
                        timeout: if params.r1 == 0 {
                            Timeout::Forever
                        } else {
                            Timeout::Duration(params.r0 as u64)
                        },
                    }),
                }
                .map(Syscall::Kernel)
            }),
            SyscallId::Io => IoSyscallId::from_u32(params.r3).map(|io_syscall| {
                Syscall::Io(match io_syscall {
                    IoSyscallId::Print => {
                        let ptr = params.r0 as *const u8;
                        let size = params.r1 as usize;
                        IoSyscall::Print { ptr, len: size }
                    }
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
    Sleep { ms: u32 },
    Create { prim: SyncPrimitiveCreate },
    Sync { kobj: i32 },
    Pend { kobj: i32, timeout: Timeout },
}

#[derive(Debug)]
pub enum IoSyscall {
    Print { ptr: *const u8, len: usize },
}
