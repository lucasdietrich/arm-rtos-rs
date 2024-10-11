use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Debug)]
#[repr(C)]
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
    Yield = 0,
    Sleep = 1,
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
            SyscallId::Kernel => KernelSyscallId::from_u32(params.r3).map(|kernel_syscall| {
                Syscall::Kernel(match kernel_syscall {
                    KernelSyscallId::Yield => KernelSyscall::Yield,
                    KernelSyscallId::Sleep => KernelSyscall::Sleep { ms: params.r0 },
                })
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

#[derive(Debug)]
pub enum KernelSyscall {
    Yield,
    Sleep { ms: u32 },
}

#[derive(Debug)]
pub enum IoSyscall {
    Print { ptr: *const u8, len: usize },
}
