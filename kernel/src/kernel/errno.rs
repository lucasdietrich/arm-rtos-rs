use num_derive::FromPrimitive;

#[repr(i32)]
#[derive(FromPrimitive)]
pub enum Kerr {
    Success = 0,    /* No error */
    EPERM = -1,     /* Operation not permitted */
    NoEntry = -2,   /* No such file or directory */
    ESRCH = -3,     /* No such process */
    EINTR = -4,     /* Interrupted system call */
    IO = -5,        /* I/O error */
    ENXIO = -6,     /* No such device or address */
    E2BIG = -7,     /* Argument list too long */
    ENOEXEC = -8,   /* Exec format error */
    EBADF = -9,     /* Bad file number */
    ECHILD = -10,   /* No child processes */
    TryAgain = -11, /* Try again */
    NoMemory = -12, /* Out of memory */
    EACCES = -13,   /* Permission denied */
    EFAULT = -14,   /* Bad address */
    ENOTBLK = -15,  /* Block device required */
    EBUSY = -16,    /* Device or resource busy */
    EEXIST = -17,   /* File exists */
    EXDEV = -18,    /* Cross-device link */
    ENODEV = -19,   /* No such device */
    ENOTDIR = -20,  /* Not a directory */
    EISDIR = -21,   /* Is a directory */
    EINVAL = -22,   /* Invalid argument */
    ENFILE = -23,   /* File table overflow */
    EMFILE = -24,   /* Too many open files */
    ENOTTY = -25,   /* Not a typewriter */
    ETXTBSY = -26,  /* Text file busy */
    EFBIG = -27,    /* File too large */
    ENOSPC = -28,   /* No space left on device */
    ESPIPE = -29,   /* Illegal seek */
    EROFS = -30,    /* Read-only file system */
    EMLINK = -31,   /* Too many links */
    EPIPE = -32,    /* Broken pipe */
    EDOM = -33,     /* Math argument out of domain of func */
    ERANGE = -34,   /* Math result not representable */

    EDEADLK = -35,       /* Resource deadlock would occur */
    ENAMETOOLONG = -36,  /* File name too long */
    ENOLCK = -37,        /* No record locks available */
    NoSuchSyscall = -38, /* Function not implemented */
    ENOTEMPTY = -39,     /* Directory not empty */
    ELOOP = -40,         /* Too many symbolic links encountered */
    ENOMSG = -42,        /* No message of desired type */
    EIDRM = -43,         /* Identifier removed */
    ECHRNG = -44,        /* Channel number out of range */
    EL2NSYNC = -45,      /* Level 2 not synchronized */
    EL3HLT = -46,        /* Level 3 halted */
    EL3RST = -47,        /* Level 3 reset */
    ELNRNG = -48,        /* Link number out of range */
    EUNATCH = -49,       /* Protocol driver not attached */
    ENOCSI = -50,        /* No CSI structure available */
    EL2HLT = -51,        /* Level 2 halted */
    EBADE = -52,         /* Invalid exchange */
    EBADR = -53,         /* Invalid request descriptor */
    EXFULL = -54,        /* Exchange full */
    ENOANO = -55,        /* No anode */
    EBADRQC = -56,       /* Invalid request code */
    EBADSLT = -57,       /* Invalid slot */

    TimedOut = -116, /* Connection timed out */

    NotSupported = -524, /* Operation not supported */
}

pub const EWOULDBLOCK: Kerr = Kerr::TryAgain; /* Operation would block */

pub type KResult<T> = Result<T, Kerr>;
