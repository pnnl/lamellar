use crate::fid::{AsTypedFid, AvRawFid, BorrowedTypedFid, EpRawFid, MrRawFid};
use std::{
    marker::PhantomData,
    os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd},
};

#[allow(unused_imports)]
// use crate::fid::AsFid;
use crate::{
    cq::WaitObjectRetrieve,
    fid::{AsRawFid, AsRawTypedFid, EqRawFid, OwnedEqFid},
    Context, MyRc, MyRefCell, SyncSend,
};
use crate::{
    enums::WaitObjType,
    fabric::FabricImpl,
    fid::RawFid,
    info::{InfoEntry, InfoHints},
    infocapsoptions::Caps,
};
use libfabric_sys::{fi_mutex_cond, FI_AFFINITY, FI_CONNECTED, FI_CONNREQ, FI_SHUTDOWN, FI_WRITE};

/// Represents a connection request event.
pub type ConnReqEvent = EventQueueCmEntry<{ libfabric_sys::FI_CONNREQ }>;
/// Represents a connected event.
pub type ConnectedEvent = EventQueueCmEntry<{ libfabric_sys::FI_CONNECTED }>;
/// Represents a shutdown event.
pub type ShutdownEvent = EventQueueCmEntry<{ libfabric_sys::FI_SHUTDOWN }>;
/// Represents a memory region completion event.
pub type MrCompleteEvent = EventQueueEntry<MrRawFid>;
/// Represents an address vector completion event.
pub type AVCompleteEvent = EventQueueEntry<AvRawFid>;
/// Represents a join completion event.
pub type JoinCompleteEvent = EventQueueEntry<EpRawFid>;

/// An enumeration of the possible event types returned by an [EventQueue]
///
/// Each entry corresponds to a specific type of event that can occur within the event queue.
pub enum Event {
    // Notify(EventQueueEntry<T, NotifyEventFid>),
    ConnReq(ConnReqEvent),
    Connected(ConnectedEvent),
    Shutdown(ShutdownEvent),
    MrComplete(MrCompleteEvent),
    AVComplete(AVCompleteEvent),
    JoinComplete(JoinCompleteEvent),
}

// [TODO]
// #[cfg(feature="threading-thread-safe")]
#[cfg(feature = "thread-safe")]
unsafe impl Sync for Event {}
unsafe impl Send for Event {}

impl Event {
    #[allow(dead_code)]
    pub(crate) fn as_raw(&self) -> libfabric_sys::_bindgen_ty_18 {
        match self {
            // Event::Notify(_) => libfabric_sys::FI_NOTIFY,
            Event::ConnReq(_) => libfabric_sys::FI_CONNREQ,
            Event::Connected(_) => libfabric_sys::FI_CONNECTED,
            Event::Shutdown(_) => libfabric_sys::FI_SHUTDOWN,
            Event::MrComplete(_) => libfabric_sys::FI_MR_COMPLETE,
            Event::AVComplete(_) => libfabric_sys::FI_AV_COMPLETE,
            Event::JoinComplete(_) => libfabric_sys::FI_JOIN_COMPLETE,
        }
    }

    pub(crate) fn get_entry(&self) -> (*const std::ffi::c_void, usize) {
        match self {
            // Event::Notify(entry)|
            Event::MrComplete(entry) => (
                (&entry.c_entry as *const libfabric_sys::fi_eq_entry).cast(),
                std::mem::size_of::<libfabric_sys::fi_eq_entry>(),
            ),
            Event::AVComplete(entry) => (
                (&entry.c_entry as *const libfabric_sys::fi_eq_entry).cast(),
                std::mem::size_of::<libfabric_sys::fi_eq_entry>(),
            ),
            Event::JoinComplete(entry) => (
                (&entry.c_entry as *const libfabric_sys::fi_eq_entry).cast(),
                std::mem::size_of::<libfabric_sys::fi_eq_entry>(),
            ),
            Event::ConnReq(entry) => (
                (&entry.c_entry as *const libfabric_sys::fi_eq_cm_entry).cast(),
                std::mem::size_of::<libfabric_sys::fi_eq_cm_entry>(),
            ),
            Event::Connected(entry) => (
                (&entry.c_entry as *const libfabric_sys::fi_eq_cm_entry).cast(),
                std::mem::size_of::<libfabric_sys::fi_eq_cm_entry>(),
            ),
            Event::Shutdown(entry) => (
                (&entry.c_entry as *const libfabric_sys::fi_eq_cm_entry).cast(),
                std::mem::size_of::<libfabric_sys::fi_eq_cm_entry>(),
            ),
        }
    }

    // pub(crate) fn from_control_value(event: u32, entry: EventQueueEntry<usize>) -> Event {
    //     if event == libfabric_sys::FI_NOTIFY {
    //         Event::Notify(entry)
    //     }
    //     else if event == libfabric_sys::FI_MR_COMPLETE {
    //         Event::MrComplete(entry)
    //     }
    //     else if event == libfabric_sys::FI_AV_COMPLETE {
    //         Event::AVComplete(entry)
    //     }
    //     else if event == libfabric_sys::FI_JOIN_COMPLETE {
    //         Event::JoinComplete(entry)
    //     }
    //     else {
    //         panic!("Unexpected value for Event")
    //     }
    // }

    // pub(crate) fn from_connect_value<const ETYPE: u32>(entry: EventQueueCmEntry<ETYPE>) -> Self {
    //     if ETYPE == libfabric_sys::FI_CONNREQ {
    //         Event::ConnReq(entry)
    //     } else if ETYPE == libfabric_sys::FI_CONNECTED {
    //         Event::Connected(entry)
    //     } else if ETYPE == libfabric_sys::FI_SHUTDOWN {
    //         Event::Shutdown(entry)
    //     } else {
    //         panic!("Unexpected value for Event")
    //     }
    // }
}

//================== EventQueue (fi_eq) ==================//
pub struct EventQueueImpl<const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD: bool>
{
    pub(crate) c_eq: OwnedEqFid,
    wait_obj: Option<libfabric_sys::fi_wait_obj>,
    event_buffer: MyRefCell<Vec<u8>>,
    pub(crate) _fabric_rc: MyRc<FabricImpl>,
}
impl<const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD: bool> SyncSend
    for EventQueueImpl<WRITE, WAIT, RETRIEVE, FD>
{
}

/// A trait that provides the capabilities of a default [EventQueue]
pub trait ReadEq: AsTypedFid<EqRawFid> + SyncSend {
    /// Reads an event from the event queue in a user provided buffer and event
    ///
    /// Corresponds to `libfabric_sys::fi_eq_read`
    fn read_in(&self, buff: &mut [u8], event: &mut u32) -> Result<usize, crate::error::Error> {
        let ret = unsafe {
            libfabric_sys::inlined_fi_eq_read(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                event,
                buff.as_mut_ptr().cast(),
                buff.len(),
                0,
            )
        };
        if ret < 0 {
            return Err(crate::error::Error::from_err_code(
                (-ret).try_into().unwrap(),
            ));
        }
        Ok(ret as usize)
    }

    /// Peeks an event from the event queue in a user provided buffer and event
    ///
    /// Corresponds to `libfabric_sys::fi_eq_read` with FI_PEEK flag
    fn peek_in(&self, buff: &mut [u8], event: &mut u32) -> Result<usize, crate::error::Error> {
        let ret = unsafe {
            libfabric_sys::inlined_fi_eq_read(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                event,
                buff.as_mut_ptr().cast(),
                buff.len(),
                libfabric_sys::FI_PEEK.into(),
            )
        };

        if ret < 0 {
            Err(crate::error::Error::from_err_code(
                (-ret).try_into().unwrap(),
            ))
        } else {
            Ok(ret as usize)
        }
    }

    /// Reads an error event from the event queue in a user provided buffer
    ///
    /// Corresponds to `libfabric_sys::fi_eq_readerr`
    fn readerr_in(&self, buff: &mut [u8]) -> Result<usize, crate::error::Error> {
        let ret = unsafe {
            libfabric_sys::inlined_fi_eq_readerr(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buff.as_mut_ptr().cast(),
                0,
            )
        };

        if ret < 0 {
            Err(crate::error::Error::from_err_code(
                (-ret).try_into().unwrap(),
            ))
        } else {
            Ok(ret as usize)
        }
    }

    /// Peeks an error event from the event queue in a user provided buffer
    ///
    /// Corresponds to `libfabric_sys::fi_eq_readerr` with FI_PEEK flag
    fn peekerr_in(&self, buff: &mut [u8]) -> Result<usize, crate::error::Error> {
        let ret = unsafe {
            libfabric_sys::inlined_fi_eq_readerr(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buff.as_mut_ptr().cast(),
                libfabric_sys::FI_PEEK.into(),
            )
        };

        if ret < 0 {
            Err(crate::error::Error::from_err_code(
                (-ret).try_into().unwrap(),
            ))
        } else {
            Ok(ret as usize)
        }
    }

    /// Corresponds to `libfabric_sys::fi_eq_strerror`
    fn strerror(&self, entry: &EventError) -> &str {
        let ret = unsafe {
            libfabric_sys::inlined_fi_eq_strerror(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                -entry.c_err.prov_errno,
                entry.c_err.err_data,
                std::ptr::null_mut(),
                0,
            )
        };

        unsafe { std::ffi::CStr::from_ptr(ret).to_str().unwrap() }
    }

    /// Reads an event from the event queue and returns it
    fn read(&self) -> Result<Event, crate::error::Error>;
    /// Peeks an event from the event queue and returns it
    fn peek(&self) -> Result<Event, crate::error::Error>;
    /// Reads an error event from the event queue and returns it
    fn readerr(&self) -> Result<EventError, crate::error::Error>;
    /// Peeks an error event from the event queue and returns it without removing it from the queue
    fn peekerr(&self) -> Result<EventError, crate::error::Error>;
}

/// A trait that provides the operations of an [EventQueue] configured with wait capabilities
pub trait WaitEq: ReadEq {
    /// Blocks until it reads an event from the event queue into a user-provided buffer
    fn sread_in(
        &self,
        buff: &mut [u8],
        event: &mut u32,
        timeout: i32,
    ) -> Result<usize, crate::error::Error> {
        let ret = unsafe {
            libfabric_sys::inlined_fi_eq_sread(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                event,
                buff.as_mut_ptr().cast(),
                buff.len(),
                timeout,
                0,
            )
        };

        if ret < 0 {
            Err(crate::error::Error::from_err_code(
                (-ret).try_into().unwrap(),
            ))
        } else {
            Ok(ret as usize)
        }
    }

    /// Blocks until it peeks an event from the event queue into a user-provided buffer
    fn speek_in(
        &self,
        buff: &mut [u8],
        event: &mut u32,
        timeout: i32,
    ) -> Result<usize, crate::error::Error> {
        let ret = unsafe {
            libfabric_sys::inlined_fi_eq_sread(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                event,
                buff.as_mut_ptr().cast(),
                buff.len(),
                timeout,
                libfabric_sys::FI_PEEK.into(),
            )
        };

        if ret < 0 {
            Err(crate::error::Error::from_err_code(
                (-ret).try_into().unwrap(),
            ))
        } else {
            Ok(ret as usize)
        }
    }

    /// Blocks until it reads an event from the event queue and returns it
    fn sread(&self, timeout: i32) -> Result<Event, crate::error::Error>;
    /// Blocks until it peeks an event from the event queue and returns it
    fn speek(&self, timeout: i32) -> Result<Event, crate::error::Error>;
}

/// A trait that provides the operations of an [EventQueue] configured with write capabilities
pub trait WriteEq: AsTypedFid<EqRawFid> {
    /// Writes raw data to the event queue
    fn write_raw(&self, buff: &[u8], event: u32, flags: u64) -> Result<usize, crate::error::Error> {
        let ret = unsafe {
            libfabric_sys::inlined_fi_eq_write(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                event,
                buff.as_ptr().cast(),
                buff.len(),
                flags,
            )
        };
        if ret < 0 {
            Err(crate::error::Error::from_err_code(
                (-ret).try_into().unwrap(),
            ))
        } else {
            Ok(ret as usize)
        }
    }

    /// Writes an event to the event queue
    fn write(&self, event: Event) -> Result<(), crate::error::Error> {
        let event_val = event.as_raw();
        let (event_entry, event_entry_size) = event.get_entry();

        let ret = unsafe {
            libfabric_sys::inlined_fi_eq_write(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                event_val,
                event_entry,
                event_entry_size,
                0,
            )
        };
        if ret < 0 {
            Err(crate::error::Error::from_err_code(
                (-ret).try_into().unwrap(),
            ))
        } else {
            debug_assert_eq!(ret as usize, event_entry_size);
            Ok(())
        }
    }
}
impl<const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD: bool> ReadEq
    for EventQueueImpl<WRITE, WAIT, RETRIEVE, FD>
{
    fn read(&self) -> Result<Event, crate::error::Error> {
        let mut event = 0;
        #[cfg(feature = "thread-safe")]
        let mut buffer = self.event_buffer.write();
        #[cfg(not(feature = "thread-safe"))]
        let mut buffer = self.event_buffer.borrow_mut();
        let len = self.read_in(&mut buffer, &mut event)?;
        Ok(EventQueueImpl::<WRITE, WAIT, RETRIEVE, FD>::read_eq_entry(
            len, &buffer, &event,
        ))
    }

    fn peek(&self) -> Result<Event, crate::error::Error> {
        let mut event = 0;
        #[cfg(feature = "thread-safe")]
        let mut buffer = self.event_buffer.write();
        #[cfg(not(feature = "thread-safe"))]
        let mut buffer = self.event_buffer.borrow_mut();
        let len = self.peek_in(&mut buffer, &mut event)?;
        Ok(EventQueueImpl::<WRITE, WAIT, RETRIEVE, FD>::read_eq_entry(
            len, &buffer, &event,
        ))
    }

    fn readerr(&self) -> Result<EventError, crate::error::Error> {
        #[cfg(feature = "thread-safe")]
        let mut buffer = self.event_buffer.write();
        #[cfg(not(feature = "thread-safe"))]
        let mut buffer = self.event_buffer.borrow_mut();
        let _len = self.readerr_in(&mut buffer)?;
        let mut err_event = EventError::new();
        err_event.c_err = unsafe { std::ptr::read(buffer.as_ptr().cast()) };
        Ok(err_event)
    }

    fn peekerr(&self) -> Result<EventError, crate::error::Error> {
        #[cfg(feature = "thread-safe")]
        let mut buffer = self.event_buffer.write();
        #[cfg(not(feature = "thread-safe"))]
        let mut buffer = self.event_buffer.borrow_mut();
        let _len = self.peekerr_in(&mut buffer)?;
        let mut err_event = EventError::new();
        err_event.c_err = unsafe { std::ptr::read(buffer.as_ptr().cast()) };
        Ok(err_event)
    }
}

impl<const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD: bool> WaitEq
    for EventQueueImpl<WRITE, WAIT, RETRIEVE, FD>
{
    fn sread(&self, timeout: i32) -> Result<Event, crate::error::Error> {
        let mut event = 0;
        #[cfg(feature = "thread-safe")]
        let mut buff = self.event_buffer.write();
        #[cfg(not(feature = "thread-safe"))]
        let mut buff = self.event_buffer.borrow_mut();
        let len = self.sread_in(&mut buff, &mut event, timeout)?;
        Ok(EventQueueImpl::<WRITE, WAIT, RETRIEVE, FD>::read_eq_entry(
            len, &buff, &event,
        ))
    }

    fn speek(&self, timeout: i32) -> Result<Event, crate::error::Error> {
        let mut event = 0;
        #[cfg(feature = "thread-safe")]
        let mut buff = self.event_buffer.write();
        #[cfg(not(feature = "thread-safe"))]
        let mut buff = self.event_buffer.borrow_mut();
        let len = self.speek_in(&mut buff, &mut event, timeout)?;
        Ok(EventQueueImpl::<WRITE, WAIT, RETRIEVE, FD>::read_eq_entry(
            len, &buff, &event,
        ))
    }
}

impl<'a, const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD: bool>
    WaitObjectRetrieve<'a> for EventQueueImpl<WRITE, WAIT, RETRIEVE, FD>
{
    fn wait_object(&self) -> Result<WaitObjType<'a>, crate::error::Error> {
        if let Some(wait) = self.wait_obj {
            if wait == libfabric_sys::fi_wait_obj_FI_WAIT_FD {
                let mut fd: i32 = 0;
                let err = unsafe {
                    libfabric_sys::inlined_fi_control(
                        self.as_typed_fid_mut().as_raw_fid(),
                        libfabric_sys::FI_GETWAIT as i32,
                        (&mut fd as *mut i32).cast(),
                    )
                };
                if err < 0 {
                    Err(crate::error::Error::from_err_code(
                        (-err).try_into().unwrap(),
                    ))
                } else {
                    Ok(WaitObjType::Fd(unsafe { BorrowedFd::borrow_raw(fd) }))
                }
            } else if wait == libfabric_sys::fi_wait_obj_FI_WAIT_MUTEX_COND {
                let mut mutex_cond = fi_mutex_cond {
                    mutex: std::ptr::null_mut(),
                    cond: std::ptr::null_mut(),
                };

                let err = unsafe {
                    libfabric_sys::inlined_fi_control(
                        self.as_typed_fid_mut().as_raw_fid(),
                        libfabric_sys::FI_GETWAIT as i32,
                        (&mut mutex_cond as *mut fi_mutex_cond).cast(),
                    )
                };
                if err < 0 {
                    Err(crate::error::Error::from_err_code(
                        (-err).try_into().unwrap(),
                    ))
                } else {
                    Ok(WaitObjType::MutexCond(mutex_cond))
                }
            } else if wait == libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC {
                Ok(WaitObjType::Unspec)
            } else {
                panic!("Could not retrieve wait object")
            }
        } else {
            panic!("Should not be reachable! Could not retrieve wait object")
        }
    }
}

impl<const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD: bool> WriteEq
    for EventQueueImpl<WRITE, WAIT, RETRIEVE, FD>
{
}

pub type EventQueue<T> = EventQueueBase<T>;

/// Owned wrapper around a libfabric `fid_eq`.
///
/// This type wraps an instance of a `fid_eq`, monitoring its lifetime and closing it when it goes out of scope.
/// To be able to check its configuration at compile this object is extended with a `T:`[`EqConfig`] (e.g. [Options]) that provides this information.
///
/// For more information see the libfabric [documentation](https://ofiwg.github.io/libfabric/v1.22.0/man/fi_eq.3.html).
///
/// Note that other objects that rely on a EventQueue (e.g., an [crate::ep::Endpoint] bound to it) will extend its lifetime until they
/// are also dropped.
pub struct EventQueueBase<EQ: ?Sized> {
    pub(crate) inner: MyRc<EQ>,
}

// pub(crate) trait BindEqImpl<EQ, CQ> {
//     fn bind_mr(&self, mr: &MyRc<MemoryRegionImplBase<EQ>>);

//     fn bind_av(&self, av: &MyRc<AddressVectorImplBase<EQ>>);

//     fn bind_mc(&self, mc: &MyRc<MulticastGroupCollectiveImplBase<EQ, CQ>>);
// }

// impl BindEqImpl<EventQueueImpl, CompletionQueueImpl> for EventQueueImpl {
//     fn bind_mr(&self, mr: &MyRc<MemoryRegionImplBase<EventQueueImpl>>) {
//         self.bind_mr(mr);
//     }

//     fn bind_av(&self, av: &MyRc<AddressVectorImplBase<EventQueueImpl>>) {
//         self.bind_av(av);
//     }

//     fn bind_mc(&self, mc: &MyRc<MulticastGroupCollectiveImplBase<EventQueueImpl, CompletionQueueImpl>>) {
//         self.bind_mc(mc);
//     }
// }

impl<const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD: bool>
    EventQueueImpl<WRITE, WAIT, RETRIEVE, FD>
{
    pub(crate) fn new(
        fabric: &MyRc<crate::fabric::FabricImpl>,
        mut attr: EventQueueAttr,
        context: *mut std::ffi::c_void,
    ) -> Result<Self, crate::error::Error> {
        let mut c_eq: *mut libfabric_sys::fid_eq = std::ptr::null_mut();
        let c_eq_ptr: *mut *mut libfabric_sys::fid_eq = &mut c_eq;

        let err = unsafe {
            libfabric_sys::inlined_fi_eq_open(
                fabric.as_typed_fid_mut().as_raw_typed_fid(),
                attr.get_mut(),
                c_eq_ptr,
                context,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(Self {
                c_eq: OwnedEqFid::from(c_eq),
                wait_obj: Some(attr.c_attr.wait_obj),
                event_buffer: MyRefCell::new(vec![
                    0;
                    std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(
                    )
                ]),
                _fabric_rc: fabric.clone(),
            })
        }
    }

    pub(crate) fn read_eq_entry(bytes_read: usize, buffer: &[u8], event: &u32) -> Event {
        if event == &libfabric_sys::FI_CONNREQ
            || event == &libfabric_sys::FI_CONNECTED
            || event == &libfabric_sys::FI_SHUTDOWN
        {
            debug_assert_eq!(
                bytes_read,
                std::mem::size_of::<libfabric_sys::fi_eq_cm_entry>()
            );

            if *event == FI_CONNREQ {
                let entry = EventQueueCmEntry::<FI_CONNREQ> {
                    c_entry: unsafe { std::ptr::read(buffer.as_ptr().cast()) },
                };
                Event::ConnReq(entry)
            } else if *event == FI_CONNECTED {
                let entry = EventQueueCmEntry::<FI_CONNECTED> {
                    c_entry: unsafe { std::ptr::read(buffer.as_ptr().cast()) },
                };
                Event::Connected(entry)
            } else if *event == FI_SHUTDOWN {
                let entry = EventQueueCmEntry::<FI_SHUTDOWN> {
                    c_entry: unsafe { std::ptr::read(buffer.as_ptr().cast()) },
                };
                Event::Shutdown(entry)
            } else {
                panic!("Unexpected Event type")
            }
        } else {
            debug_assert_eq!(
                bytes_read,
                std::mem::size_of::<libfabric_sys::fi_eq_entry>()
            );
            let c_entry: libfabric_sys::fi_eq_entry =
                unsafe { std::ptr::read(buffer.as_ptr().cast()) };

            if event == &libfabric_sys::FI_NOTIFY {
                panic!("Unexpected event");
            }

            if event == &libfabric_sys::FI_MR_COMPLETE {
                Event::MrComplete(MrCompleteEvent {
                    c_entry,
                    phantom: PhantomData,
                })
            } else if event == &libfabric_sys::FI_AV_COMPLETE {
                Event::AVComplete(AVCompleteEvent {
                    c_entry,
                    phantom: PhantomData,
                })
            } else if event == &libfabric_sys::FI_JOIN_COMPLETE {
                Event::JoinComplete(JoinCompleteEvent {
                    c_entry,
                    phantom: PhantomData,
                })
            } else {
                panic!("Unexpected value for Event")
            }
        }
    }
}

impl<const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD: bool>
    EventQueue<EventQueueImpl<WRITE, WAIT, RETRIEVE, FD>>
{
    pub(crate) fn new(
        fabric: &crate::fabric::Fabric,
        attr: EventQueueAttr,
        context: Option<&mut Context>,
    ) -> Result<Self, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(Self {
            inner: MyRc::new(EventQueueImpl::new(&fabric.inner, attr, c_void)?),
        })
    }
}

impl<T: SyncSend> SyncSend for EventQueue<T> {}
pub(crate) trait SyncEq {}

impl<T: ReadEq + SyncEq> ReadEq for EventQueue<T> {
    fn read(&self) -> Result<Event, crate::error::Error> {
        self.inner.read()
    }

    fn peek(&self) -> Result<Event, crate::error::Error> {
        self.inner.peek()
    }

    fn readerr(&self) -> Result<EventError, crate::error::Error> {
        self.inner.readerr()
    }

    fn peekerr(&self) -> Result<EventError, crate::error::Error> {
        self.inner.peekerr()
    }

    fn strerror(&self, entry: &EventError) -> &str {
        self.inner.strerror(entry)
    }
}

impl<T: WriteEq> WriteEq for EventQueue<T> {
    fn write(&self, event: Event) -> Result<(), crate::error::Error> {
        self.inner.write(event)
    }
}

impl<T: WaitEq + SyncEq> WaitEq for EventQueue<T> {
    fn sread(&self, timeout: i32) -> Result<Event, crate::error::Error> {
        self.inner.sread(timeout)
    }

    fn speek(&self, timeout: i32) -> Result<Event, crate::error::Error> {
        self.inner.speek(timeout)
    }
}

// impl<T: WaitEq> EventQueue<T> {

//     pub fn sread(&self, timeout: i32) -> Result<Event, crate::error::Error> {
//         self.inner.sread(timeout)
//     }

//     pub fn speek(&self, timeout: i32) -> Result<Event, crate::error::Error> {
//         self.inner.speek(timeout)
//     }
// }

impl<'a, T: WaitObjectRetrieve<'a>> EventQueue<T> {
    pub fn wait_object(&self) -> Result<WaitObjType<'a>, crate::error::Error> {
        self.inner.wait_object()
    }
}

// impl<T: AsFid> AsFid for EventQueueBase<T> {
//     fn as_fid(&self) -> fid::BorrowedFid<'_> {
//         self.inner.as_fid()
//     }
// }

impl<T: AsRawFid> AsRawFid for EventQueueBase<T> {
    fn as_raw_fid(&self) -> RawFid {
        self.inner.as_raw_fid()
    }
}

// impl<const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD: bool> AsFid
//     for EventQueueImpl<WRITE, WAIT, RETRIEVE, FD>
// {
//     fn as_fid(&self) -> fid::BorrowedFid<'_> {
//         self.c_eq.as_fid()
//     }
// }

// impl<const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD: bool> AsRawFid
//     for EventQueueImpl<WRITE, WAIT, RETRIEVE, FD>
// {
//     fn as_raw_fid(&self) -> RawFid {
//         self.c_eq.as_raw_fid()
//     }
// }

impl<T: AsTypedFid<EqRawFid>> AsTypedFid<EqRawFid> for EventQueueBase<T> {
    fn as_typed_fid(&self) -> BorrowedTypedFid<EqRawFid> {
        self.inner.as_typed_fid()
    }
    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<EqRawFid> {
        self.inner.as_typed_fid_mut()
    }
}

impl<T: AsRawTypedFid<Output = EqRawFid>> AsRawTypedFid for EventQueueBase<T> {
    type Output = EqRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.inner.as_raw_typed_fid()
    }
}

impl<const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD: bool> AsTypedFid<EqRawFid>
    for EventQueueImpl<WRITE, WAIT, RETRIEVE, FD>
{
    fn as_typed_fid(&self) -> BorrowedTypedFid<EqRawFid> {
        self.c_eq.as_typed_fid()
    }
    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<EqRawFid> {
        self.c_eq.as_typed_fid_mut()
    }
}

impl<const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD: bool> SyncEq
    for EventQueueImpl<WRITE, WAIT, RETRIEVE, FD>{}

// impl<const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD: bool> AsRawTypedFid
//     for EventQueueImpl<WRITE, WAIT, RETRIEVE, FD>
// {
//     type Output = EqRawFid;

//     fn as_raw_typed_fid(&self) -> Self::Output {
//         self.c_eq.as_raw_typed_fid()
//     }
// }

impl<'a, T: WaitObjectRetrieve<'a> + AsFd> AsFd for EventQueue<T> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

impl<const WRITE: bool> AsFd for EventQueueImpl<WRITE, true, true, true> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        if let WaitObjType::Fd(fd) = self.wait_object().unwrap() {
            fd
        } else {
            panic!("Fabric object object type is not Fd")
        }
    }
}

impl<const WRITE: bool> AsRawFd for EventQueueImpl<WRITE, true, true, true> {
    fn as_raw_fd(&self) -> RawFd {
        if let WaitObjType::Fd(fd) = self.wait_object().unwrap() {
            fd.as_raw_fd()
        } else {
            panic!("Fabric object object type is not Fd")
        }
    }
}


// impl<const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD: bool> BindImpl for EventQueueImpl<WRITE, WAIT, RETRIEVE, FD> {}
// impl<T: ReadEq + 'static> crate::Bind for EventQueue<T> {
//     fn inner(&self) -> MyRc<dyn AsRawFid> {
//         self.inner.clone()
//     }
// }

//================== EventQueue Attribute(fi_eq_attr) ==================//

/// Builder for creating an [EventQueue].
pub struct EventQueueBuilder<
    'a,
    const WRITE: bool,
    const WAIT: bool,
    const RETRIEVE: bool,
    const FD: bool,
> {
    eq_attr: EventQueueAttr,
    fabric: &'a crate::fabric::Fabric,
    ctx: Option<&'a mut Context>,
}

impl<'a> EventQueueBuilder<'a, false, true, false, false> {
    pub fn new(fabric: &'a crate::fabric::Fabric) -> Self {
        Self {
            eq_attr: EventQueueAttr::new(),
            fabric,
            ctx: None,
        }
    }
}

impl<'a, const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD: bool>
    EventQueueBuilder<'a, WRITE, WAIT, RETRIEVE, FD>
{
    /// Sets the size of the event queue
    pub fn size(mut self, size: usize) -> Self {
        self.eq_attr.size(size);
        self
    }

    /// Enables write operations on the event queue
    pub fn write(mut self) -> EventQueueBuilder<'a, true, WAIT, RETRIEVE, FD> {
        self.eq_attr.write();

        EventQueueBuilder {
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: self.ctx,
        }
    }

    /// Disables waiting on the event queue
    pub fn wait_none(mut self) -> EventQueueBuilder<'a, WRITE, false, false, false> {
        self.eq_attr.wait_obj(crate::enums::WaitObj::None);

        EventQueueBuilder {
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: self.ctx,
        }
    }

    /// Enables waiting on the event queue using a file descriptor
    pub fn wait_fd(mut self) -> EventQueueBuilder<'a, WRITE, true, true, true> {
        self.eq_attr.wait_obj(crate::enums::WaitObj::Fd);

        EventQueueBuilder {
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: self.ctx,
        }
    }

    /// Enables waiting on the event queue using a wait set
    pub fn wait_set(
        mut self,
        set: &crate::sync::WaitSet,
    ) -> EventQueueBuilder<'a, WRITE, true, false, false> {
        self.eq_attr.wait_obj(crate::enums::WaitObj::Set(set));

        EventQueueBuilder {
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: self.ctx,
        }
    }

    /// Enables waiting on the event queue using a mutex condition variable
    pub fn wait_mutex(mut self) -> EventQueueBuilder<'a, WRITE, true, true, false> {
        self.eq_attr.wait_obj(crate::enums::WaitObj::MutexCond);

        EventQueueBuilder {
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: self.ctx,
        }
    }

    /// Enables waiting on the event queue using a yield operation
    pub fn wait_yield(mut self) -> EventQueueBuilder<'a, WRITE, true, false, false> {
        self.eq_attr.wait_obj(crate::enums::WaitObj::Yield);

        EventQueueBuilder {
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: self.ctx,
        }
    }

    /// Sets the signaling vector for the event queue
    pub fn signaling_vector(mut self, signaling_vector: i32) -> Self {
        self.eq_attr.signaling_vector(signaling_vector);
        self
    }

    /// Sets the context for the event queue
    pub fn context(self, ctx: &'a mut Context) -> EventQueueBuilder<'a, WRITE, WAIT, RETRIEVE, FD> {
        EventQueueBuilder {
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: Some(ctx),
        }
    }

    /// Builds the event queue.
    pub fn build(
        self,
    ) -> Result<EventQueue<EventQueueImpl<WRITE, WAIT, RETRIEVE, FD>>, crate::error::Error> {
        EventQueue::<EventQueueImpl<WRITE, WAIT, RETRIEVE, FD>>::new(
            self.fabric,
            self.eq_attr,
            self.ctx,
        )
    }
}

pub(crate) struct EventQueueAttr {
    c_attr: libfabric_sys::fi_eq_attr,
}

impl EventQueueAttr {
    pub(crate) fn new() -> Self {
        let c_attr = libfabric_sys::fi_eq_attr {
            size: 0,
            flags: 0,
            wait_obj: libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC,
            signaling_vector: 0,
            wait_set: std::ptr::null_mut(),
        };

        Self { c_attr }
    }

    pub(crate) fn size(&mut self, size: usize) -> &mut Self {
        self.c_attr.size = size;
        self
    }

    pub(crate) fn write(&mut self) -> &mut Self {
        self.c_attr.flags |= FI_WRITE as u64;
        self
    }

    pub(crate) fn wait_obj(&mut self, wait_obj: crate::enums::WaitObj) -> &mut Self {
        if let crate::enums::WaitObj::Set(wait_set) = wait_obj {
            self.c_attr.wait_set = wait_set.as_raw_typed_fid();
        }
        self.c_attr.wait_obj = wait_obj.as_raw();
        self
    }

    pub(crate) fn signaling_vector(&mut self, signaling_vector: i32) -> &mut Self {
        self.c_attr.flags |= FI_AFFINITY as u64;
        self.c_attr.signaling_vector = signaling_vector;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_eq_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_eq_attr {
        &mut self.c_attr
    }
}

impl Default for EventQueueAttr {
    fn default() -> Self {
        Self::new()
    }
}

//================== EqErrEntry (fi_eq_err_entry) ==================//
#[repr(C)]
#[derive(Debug)]
/// Wrapper around a libfabric `fi_eq_err_entry`.
pub struct EventError {
    pub(crate) c_err: libfabric_sys::fi_eq_err_entry,
}

//[TODO]
#[cfg(feature = "thread-safe")]
unsafe impl Sync for EventError {}
unsafe impl Send for EventError {}

impl EventError {
    pub fn new() -> Self {
        let c_err = libfabric_sys::fi_eq_err_entry {
            fid: std::ptr::null_mut(),
            context: std::ptr::null_mut(),
            data: 0,
            err: 0,
            prov_errno: 0,
            err_data: std::ptr::null_mut(),
            err_data_size: 0,
        };

        Self { c_err }
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_eq_err_entry {
        &self.c_err
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_eq_err_entry {
        &mut self.c_err
    }

    // pub fn get_fid(&self) -> BorrowedFid {
    //     unsafe { BorrowedFid::borrow_raw(self.c_err.fid) }
    // }

    /// Returns the data associated with the error entry.
    pub fn data(&self) -> u64 {
        self.c_err.data
    }

    /// Returns the error code associated with the error entry.
    pub fn error(&self) -> crate::error::Error {
        crate::error::Error::from_err_code(self.c_err.err as u32)
    }

    /// Returns the provider-specific error number associated with the error entry.
    pub fn prov_errno(&self) -> i32 {
        self.c_err.prov_errno
    }
}

impl Default for EventError {
    fn default() -> Self {
        Self::new()
    }
}

//================== EventQueueEntry (fi_eq_entry) ==================//

#[repr(C)]
#[derive(Clone)]
pub struct EventQueueEntry<F> {
    pub(crate) c_entry: libfabric_sys::fi_eq_entry,
    phantom: PhantomData<F>,
}

impl<F: AsRawFid> EventQueueEntry<F> {
    // const SIZE_OK: () = assert!(std::mem::size_of::<T>() == std::mem::size_of::<usize>(),
    // "The context of an EventQueueEntry must always be of size equal to usize datatype.");

    pub fn new(event_fid: F) -> Self {
        // let _ = Self::SIZE_OK;
        let c_entry = libfabric_sys::fi_eq_entry {
            fid: event_fid.as_raw_fid(),
            context: std::ptr::null_mut(),
            data: 0,
        };

        Self {
            c_entry,
            phantom: PhantomData,
        }
    }

    /// Returns a reference to the raw event queue identifier.
    pub fn fid(&self) -> &RawFid {
        &self.c_entry.fid
    }

    /// Sets the context for the event queue entry.
    pub fn set_context<T>(&mut self, context: &mut Context) -> &mut Self {
        let context_writable: *mut *mut std::ffi::c_void = &mut (self.c_entry.context);
        let context_writable_usize: *mut usize = context_writable as *mut usize;
        unsafe { *context_writable_usize = *(context.inner_mut() as *mut usize) };
        self
    }

    /// Sets the data for the event queue entry.
    pub fn set_data(&mut self, data: u64) -> &mut Self {
        self.c_entry.data = data;
        self
    }

    /// Returns the data associated with the event queue entry.
    pub fn data(&self) -> u64 {
        self.c_entry.data
    }

    /// Returns a reference to the context associated with the event queue entry.
    pub fn context<T>(&self) -> &T {
        let context_ptr: *mut *mut T = &mut (self.c_entry.context as *mut T);
        unsafe { std::mem::transmute_copy::<T, &T>(&*(context_ptr as *const T)) }
    }

    /// Checks if the context associated with the event queue entry is equal to the given context.
    pub fn is_context_equal(&self, ctx: &Context) -> bool {
        std::ptr::eq(self.c_entry.context, ctx.inner())
    }
}

// impl<T> Default for EventQueueEntry<T> {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// }

//================== EventQueueCmEntry (fi_eq_cm_entry) ==================//
#[repr(C)]
pub struct EventQueueCmEntry<const ETYPE: libfabric_sys::_bindgen_ty_18> {
    pub(crate) c_entry: libfabric_sys::fi_eq_cm_entry,
}

impl<const ETYPE: libfabric_sys::_bindgen_ty_18> EventQueueCmEntry<ETYPE> {
    // pub(crate) fn new() -> EventQueueCmEntry<ETYPE> {
    //     let c_entry = libfabric_sys::fi_eq_cm_entry {
    //         fid: std::ptr::null_mut(),
    //         info: std::ptr::null_mut(),
    //         data: libfabric_sys::__IncompleteArrayField::<u8>::new(),
    //     };

    //     Self { c_entry }
    // }

    pub fn fid(&self) -> libfabric_sys::fid_t {
        self.c_entry.fid
    }

    //[TODO] Should returen the proper type of info entry
    /// Returns a reference to the info entry associated with the event queue entry.
    pub fn info<E: Caps>(&self) -> Result<InfoEntry<E>, crate::error::Error> {
        let caps = E::bitfield();
        if caps & unsafe { (*self.c_entry.info).caps } == caps {
            Ok(InfoEntry::<E>::new(self.c_entry.info))
        } else {
            Err(crate::error::Error::caps_error())
        }
    }

    pub(crate) fn info_handle(&self) -> libfabric_sys::fid_t {
        unsafe { (*self.c_entry.info).handle}
    }

    //[TODO] Should returen the proper type of info entry
    /// Returns a new InfoEntry with the capabilities requested, if possible
    pub fn info_from_caps<E: Caps>(
        &self,
        _caps: &InfoHints<E>,
    ) -> Result<InfoEntry<E>, crate::error::Error> {
        let caps = E::bitfield();
        if caps & unsafe { (*self.c_entry.info).caps } == caps {
            Ok(InfoEntry::<E>::new(self.c_entry.info))
        } else {
            Err(crate::error::Error::caps_error())
        }
    }
}

impl<const ETYPE: libfabric_sys::_bindgen_ty_18> Drop for EventQueueCmEntry<ETYPE> {
    fn drop(&mut self) {
        unsafe { libfabric_sys::fi_freeinfo(self.c_entry.info) }
    }
}

// impl Default for EventQueueCmEntry {
//     fn default() -> Self {
//         Self::new()
//     }
// }

//================== Async Stuff ==============================//

//================== EventQueue related tests ==================//

#[cfg(test)]
mod tests {

    use crate::info::Info;

    use super::EventQueueBuilder;

    // #[test]
    // fn eq_write_read_self() {
    //     let info = Info::new().request().unwrap();
    //     let entries = info.get();
    //     let fab = crate::fabric::FabricBuilder::new(&entry).build().unwrap();
    //     let eq = EventQueueBuilder::new(&fab)
    //         .size(32)
    //         .write()
    //         .wait_fd()
    //         .build().unwrap();

    //     for mut i in 0_usize ..5 {
    //         let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
    //         if i & 1 == 1 {
    //             entry.fid(&fab);
    //         }
    //         else {
    //             entry.fid(&eq);
    //         }

    //         entry.context(&mut i);
    //         eq.write(Event::Notify(entry)).unwrap();
    //     }
    //     for i in 0..10 {

    //         let ret = if i & 1 == 1 {
    //             eq.read().unwrap()
    //         }
    //         else {
    //             eq.peek().unwrap()
    //         };

    //         if let crate::eq::Event::Notify(entry) = ret {

    //             if entry.get_context() != i /2 {
    //                 panic!("Unexpected context {} vs {}", entry.get_context(), i/2);
    //             }

    //             if entry.get_fid() != if i & 2 == 2 {fab.as_raw_fid()} else {eq.as_raw_fid()} {
    //                 panic!("Unexpected fid {:?}", entry.get_fid());
    //             }
    //         }
    //         else {
    //             panic!("Unexpected EventType");
    //         }
    //     }

    //     let ret = eq.read();
    //     if let Err(ref err) = ret {
    //         if !matches!(err.kind, crate::error::ErrorKind::TryAgain) {
    //             ret.unwrap();
    //         }
    //     }

    // }

    // #[test]
    // fn eq_size_verify() {
    //     let info = Info::new().request().unwrap();
    //     let entries = info.get();
    //     let fab = crate::fabric::FabricBuilder::new(&entry).build().unwrap();
    //     let eq = EventQueueBuilder::new(&fab)
    //         .size(32)
    //         .write()
    //         .wait_fd()
    //         .build().unwrap();

    //     for mut i in 0_usize .. 32 {
    //         let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
    //         entry
    //             .fid(&fab)
    //             .context(&mut i);
    //         eq.write(Event::Notify(entry)).unwrap();
    //     }
    // }

    // #[test]
    // fn eq_write_sread_self() {
    //     let info = Info::new().request().unwrap();
    //     let entries = info.get();
    //     let fab = crate::fabric::FabricBuilder::new(&entry).build().unwrap();
    //     let eq = EventQueueBuilder::new(&fab)
    //         .size(32)
    //         .write()
    //         .wait_fd()
    //         .build().unwrap();

    //     for mut i in 0_usize ..5 {
    //         let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
    //         if i & 1 == 1 {
    //             entry.fid(&fab);
    //         }
    //         else {
    //             entry.fid(&eq);
    //         }

    //         entry.context(&mut i);
    //         eq.write(Event::Notify(entry)).unwrap();
    //     }
    //     for i in 0..10 {
    //         let event = if (i & 1) == 1 { eq.sread(2000) } else { eq.speek(2000) }.unwrap();

    //         if let crate::eq::Event::Notify(entry) = event {

    //             if entry.get_context() != i /2 {
    //                 panic!("Unexpected context {} vs {}", entry.get_context(), i/2);
    //             }

    //             if entry.get_fid() != if i & 2 == 2 {fab.as_raw_fid()} else {eq.as_raw_fid()} {
    //                 panic!("Unexpected fid {:?}", entry.get_fid());
    //             }
    //         }
    //         else {
    //             panic!("Unexpected EventType");
    //         }
    //     }

    //     let ret = eq.read();
    //     if let Err(ref err) = ret {
    //         if !matches!(err.kind, crate::error::ErrorKind::TryAgain) {
    //             ret.unwrap();
    //         }
    //     }

    // }

    // #[test]
    // fn eq_readerr() {
    //     let info = Info::new().request().unwrap();
    //     let entries = info.get();
    //     let fab = crate::fabric::FabricBuilder::new(&entry).build().unwrap();
    //     let eq = EventQueueBuilder::new(&fab)
    //         .size(32)
    //         .write()
    //         .wait_fd()
    //         .build().unwrap();

    //     for mut i in 0_usize ..5 {
    //         let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
    //         entry.fid(&fab);

    //         entry.context(&mut i);
    //         eq.write(Event::Notify(entry)).unwrap();
    //     }

    //     for i in 0..5 {
    //         let event = eq.read().unwrap();

    //         if let Event::Notify(entry) = event {

    //             if entry.get_context() != i  {
    //                 panic!("Unexpected context {} vs {}", entry.get_context(), i/2);
    //             }

    //             if entry.get_fid() != fab.as_raw_fid() {
    //                 panic!("Unexpected fid {:?}", entry.get_fid());
    //             }
    //         }
    //         else {
    //             panic!("Unexpected EventQueueEntryFormat");
    //         }
    //     }
    //     let ret = eq.readerr();
    //     if let Err(ref err) = ret {
    //         if !matches!(err.kind, crate::error::ErrorKind::TryAgain) {
    //             ret.unwrap();
    //         }
    //     }
    // }

    #[test]
    fn eq_open_close_sizes() {
        let info = Info::new(&crate::info::libfabric_version()).get().unwrap();
        let entry = info.into_iter().next().unwrap();

        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        for i in -1..17 {
            let size = if i == -1 { 0 } else { 1 << i };
            let _eq = EventQueueBuilder::new(&fab)
                .wait_fd()
                .size(size)
                .build()
                .unwrap();
        }
    }
}

#[cfg(test)]
mod libfabric_lifetime_tests {

    use crate::info::Info;

    use super::EventQueueBuilder;

    #[test]
    fn eq_drops_before_fabric() {
        let info = Info::new(&crate::info::libfabric_version()).get().unwrap();
        let entry = info.into_iter().next().unwrap();

        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let mut eqs = Vec::new();
        for i in -1..17 {
            let size = if i == -1 { 0 } else { 1 << i };
            let eq = EventQueueBuilder::new(&fab)
                .wait_fd()
                .size(size)
                .build()
                .unwrap();
            eqs.push(eq);
        }

        drop(fab);
    }
}
