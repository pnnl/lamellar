 
 
pub struct Error {
    pub c_err: u32,
    pub kind : ErrorKind,
}

impl std::fmt::Display for Error {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        if matches!(self.kind, ErrorKind::CapabilitiesNotMet)
        {
            write!(f, "Capabilities requested not met")
        }
        else {
            write!(f, "{} (Error {})", crate::utils::error_to_string(self.c_err.into()), self.c_err)
        }
    }
}
impl std::fmt::Debug for Error {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        if matches!(self.kind, ErrorKind::CapabilitiesNotMet)
        {
            write!(f, "Capabilities requested not met")
        }
        else {
            write!(f, "{} (Error {})", crate::utils::error_to_string(self.c_err.into()), self.c_err)
        }
    }
}

impl Error {
    pub(crate) fn from_err_code(c_err: u32) -> Self {
        
        let kind = match c_err {

            libfabric_sys::FI_EPERM => ErrorKind::NotPermitted,
            libfabric_sys::FI_ENOENT => ErrorKind::NotFound,
            libfabric_sys::FI_EINTR => ErrorKind::Interrupted,
            libfabric_sys::FI_EIO => ErrorKind::IoError,
            libfabric_sys::FI_E2BIG => ErrorKind::ArgumentListTooLong,
            libfabric_sys::FI_EBADF => ErrorKind::BadFileNumber,
            libfabric_sys::FI_EAGAIN => ErrorKind::TryAgain,
            libfabric_sys::FI_ENOMEM => ErrorKind::OutOfMemory,
            libfabric_sys::FI_EACCES => ErrorKind::PermissionDenied,
            libfabric_sys::FI_EFAULT => ErrorKind::BadAddress,
            libfabric_sys::FI_EBUSY => ErrorKind::ResourceBusy,
            libfabric_sys::FI_ENODEV => ErrorKind::DeviceNotFound,
            libfabric_sys::FI_EINVAL => ErrorKind::InvalidArgument,
            libfabric_sys::FI_EMFILE => ErrorKind::TooManyFiles,
            libfabric_sys::FI_ENOSPC => ErrorKind::NoSpace,
            libfabric_sys::FI_ENOSYS => ErrorKind::NotImplemented,
            libfabric_sys::FI_ENOMSG => ErrorKind::NoMessage,
            libfabric_sys::FI_ENODATA => ErrorKind::NoData,
            libfabric_sys::FI_EOVERFLOW => ErrorKind::ValueTooLarge,
            libfabric_sys::FI_EMSGSIZE => ErrorKind::MessageTooLong,
            libfabric_sys::FI_ENOPROTOOPT => ErrorKind::ProtocolUnavalailable,
            libfabric_sys::FI_EOPNOTSUPP => ErrorKind::NotSupported,
            libfabric_sys::FI_EADDRINUSE => ErrorKind::AddrInUse,
            libfabric_sys::FI_EADDRNOTAVAIL => ErrorKind::AddrNotAvailalble,
            libfabric_sys::FI_ENETDOWN => ErrorKind::NetworkDown,
            libfabric_sys::FI_ENETUNREACH => ErrorKind::NetworkUnreachable,
            
            libfabric_sys::FI_ECONNABORTED => ErrorKind::ConnectionAborted,
            libfabric_sys::FI_ECONNRESET => ErrorKind::ConnectionReset,
            libfabric_sys::FI_ENOBUFS => ErrorKind::NoBufSpaceAvailable,
            libfabric_sys::FI_EISCONN => ErrorKind::AlreadyConnected,
            libfabric_sys::FI_ENOTCONN => ErrorKind::NotConnected,
            libfabric_sys::FI_ESHUTDOWN => ErrorKind::TransportShutdown,

            libfabric_sys::FI_ETIMEDOUT => ErrorKind::TimedOut,
            libfabric_sys::FI_ECONNREFUSED => ErrorKind::ConnectionRefused,
            libfabric_sys::FI_EHOSTDOWN => ErrorKind::HostDown,
            libfabric_sys::FI_EHOSTUNREACH => ErrorKind::HostUnreachable,
            libfabric_sys::FI_EALREADY => ErrorKind::AlreadyInProgress,
            libfabric_sys::FI_EINPROGRESS => ErrorKind::NowInProgress,
            libfabric_sys::FI_EREMOTEIO => ErrorKind::RemoteIoError,
            libfabric_sys::FI_ECANCELED => ErrorKind::Canceled,
            libfabric_sys::FI_EKEYREJECTED => ErrorKind::KeyRejected,
            libfabric_sys::FI_ETOOSMALL => panic!("TooSmall error to be created but no length is supplied"),
            libfabric_sys::FI_EOPBADSTATE => ErrorKind::BadState, 

            libfabric_sys::FI_EAVAIL => ErrorKind::ErrorAvailable, 
            libfabric_sys::FI_EBADFLAGS => ErrorKind::BadFlags, 
            libfabric_sys::FI_ENOEQ => ErrorKind::NoEventQueue, 
            libfabric_sys::FI_EDOMAIN => ErrorKind::InvalidDomain, 
            libfabric_sys::FI_ENOCQ => ErrorKind::NoCompletionQueue, 
            libfabric_sys::FI_ECRC => ErrorKind::CrcError,
            libfabric_sys::FI_ETRUNC =>  ErrorKind::TruncationError,
            libfabric_sys::FI_ENOKEY => ErrorKind::KeyNotAvailable, 
            libfabric_sys::FI_ENOAV => ErrorKind::NoAddressVector, 
            libfabric_sys::FI_EOVERRUN => ErrorKind::QueueOverrun,
            libfabric_sys::FI_ENORX => ErrorKind::ReceiverNotReady,
            _ => ErrorKind::Other,
        };

        Self { c_err, kind}
    }

    pub(crate) fn caps_error() -> Self {
        Self {
            c_err : 0,
            kind: ErrorKind::CapabilitiesNotMet,
        }
    }
}
    

#[non_exhaustive]
pub enum ErrorKind {
    NotPermitted,
    NotFound,
    Interrupted,
    IoError,
    ArgumentListTooLong,
    BadFileNumber,
    TryAgain,
    OutOfMemory,
    PermissionDenied,
    BadAddress,
    ResourceBusy,
    DeviceNotFound,
    InvalidArgument,
    TooManyFiles,
    NoSpace,
    NotImplemented,
    WouldBlock,
    NoMessage,
    NoData,
    ValueTooLarge,
    MessageTooLong,
    ProtocolUnavalailable,
    NotSupported,
    AddrInUse,
    AddrNotAvailalble,
    NetworkDown,
    NetworkUnreachable,
    ConnectionAborted,
    ConnectionReset,
    NoBufSpaceAvailable,
    AlreadyConnected,
    NotConnected,
    TransportShutdown,
    TimedOut,
    ConnectionRefused,
    HostDown,
    HostUnreachable,
    AlreadyInProgress,
    NowInProgress,
    RemoteIoError,
    Canceled,
    KeyRejected,
    TooSmall(usize),
    BadState,

    ErrorAvailable,
    BadFlags,
    NoEventQueue,
    InvalidDomain,
    NoCompletionQueue,
    
    CrcError,
    TruncationError,
    KeyNotAvailable,
    NoAddressVector,
    QueueOverrun,
    ReceiverNotReady,

    CapabilitiesNotMet,
    Other,
}

#[allow(dead_code)]
fn throw_error() -> Result<u32, Error> {
    std::result::Result::Err(Error::from_err_code(libfabric_sys::FI_EPERM))
}

#[test]
fn test_error() {
    let _res = throw_error().unwrap();
}