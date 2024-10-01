use crate::{
    av::AddressVectorBase, cq::{CompletionQueue, ReadCq}, ep::{ActiveEndpoint, Address, BaseEndpoint, EndpointBase, EndpointImplBase, IncompleteBindCntr}, eq::{Event, EventQueueBase, EventQueueCmEntry, ReadEq}, fid::{AsFid, AsRawFid, AsRawTypedFid, EpRawFid}, info::InfoEntry, utils::check_error, Context, MyRc
};
    
pub struct UnconnectedEndpointBase<EP> {
    pub(crate) inner: MyRc<EP>,
}
impl<EP> UnconnectedEndpointBase<EndpointImplBase<EP, dyn ReadEq, dyn ReadCq>> {
    pub fn bind_eq<T: ReadEq + 'static>(&self, eq: &EventQueueBase<T>) -> Result<(), crate::error::Error>  {
        self.inner.bind_eq(&eq.inner)
    }

    pub fn bind_cntr(&self) -> IncompleteBindCntr<EP, dyn ReadEq, dyn ReadCq> {
        self.inner.bind_cntr()
    }

    pub fn bind_av<EQ: ?Sized + ReadEq + 'static>(&self, av: &AddressVectorBase<EQ>) -> Result<(), crate::error::Error> {
        self.inner.bind_av(av)
    }

    pub fn bind_shared_cq<T: AsRawFid + ReadCq + 'static>(&self, cq: &CompletionQueue<T>, selective: bool) -> Result<(), crate::error::Error> {
        self.inner.bind_shared_cq(&cq.inner, selective)
    }

    pub fn bind_separate_cqs<T: AsRawFid + ReadCq + 'static>(&self, tx_cq: &CompletionQueue<T>, tx_selective: bool, rx_cq: &CompletionQueue<T>, rx_selective: bool) -> Result<(), crate::error::Error> {
        self.inner.bind_separate_cqs(&tx_cq.inner, tx_selective, &rx_cq.inner, rx_selective)
    }
}


impl<EP: AsRawFid> AsRawFid for UnconnectedEndpointBase<EP> {
    fn as_raw_fid(&self) -> crate::fid::RawFid {
        self.inner.as_raw_fid()
    }
}

impl<EP: AsFid> AsFid for UnconnectedEndpointBase<EP> {
    fn as_fid(&self) -> crate::fid::BorrowedFid {
        self.inner.as_fid()
    }
}

impl<EP: AsRawTypedFid<Output = EpRawFid>> AsRawTypedFid for UnconnectedEndpointBase<EP> {
    type Output = EpRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.inner.as_raw_typed_fid()
    }
}

impl<EP: BaseEndpoint> BaseEndpoint for UnconnectedEndpointBase<EP> {}
pub type UnconnectedEndpoint<T>  = UnconnectedEndpointBase<EndpointImplBase<T, dyn ReadEq , dyn ReadCq>>;

impl UnconnectedEndpoint<()> {
    pub fn new<E, DEQ:?Sized + 'static>(domain: &crate::domain::DomainBase<DEQ>, info: &InfoEntry<E>, flags: u64, context: Option<&mut Context>) -> Result< UnconnectedEndpoint<E>, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(
            UnconnectedEndpointBase::<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>> {
                inner:MyRc::new(EndpointImplBase::new(&domain.inner, info, flags, c_void)?),
            }
        )
    }
}

impl<EP: AsRawTypedFid<Output = EpRawFid>> UnconnectedEndpointBase<EP> {

    pub fn connect_with<T>(&self, addr: &Address, param: &[T]) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_connect(
                self.as_raw_typed_fid(),
                addr.as_bytes().as_ptr().cast(),
                param.as_ptr().cast(),
                param.len(),
            )
        };

        check_error(err.try_into().unwrap())
    }

    pub fn connect(&self, addr: &Address) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_connect(
                self.as_raw_typed_fid(),
                addr.as_bytes().as_ptr().cast(),
                std::ptr::null_mut(),
                0,
            )
        };

        check_error(err.try_into().unwrap())
    }

    pub fn accept_with<T0>(&self, param: &[T0]) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_accept(
                self.as_raw_typed_fid(),
                param.as_ptr().cast(),
                param.len(),
            )
        };

        check_error(err.try_into().unwrap())
    }

    pub fn accept(&self) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_accept(self.as_raw_typed_fid(), std::ptr::null_mut(), 0)
        };

        check_error(err.try_into().unwrap())
    }
}

impl<E> UnconnectedEndpoint<E> {
    pub fn connect_complete(self, connected_event: Event) -> ConnectedEndpoint<E> { // TODO: Create a type specifically for each event type
        let event = match connected_event {
            Event::Connected(event) => event,
            _ => panic!("Only \"Connected\" events are allowed"),
        };

        assert_eq!(event.get_fid(), self.as_raw_fid());

        ConnectedEndpoint{inner: self.inner.clone()}
    }
}

impl<E> ActiveEndpoint for UnconnectedEndpoint<E> {}

pub trait ConnectedEp {}

pub type ConnectedEndpointBase<EP> = EndpointBase<EP, true> ;

pub type ConnectedEndpoint<T> = ConnectedEndpointBase<EndpointImplBase<T, dyn ReadEq , dyn ReadCq>>;

impl<EP> ConnectedEp for  ConnectedEndpointBase<EP> {}

impl<EP: AsRawTypedFid<Output = EpRawFid>> ConnectedEndpointBase<EP> {

    pub fn shutdown(&self) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_shutdown(self.as_raw_typed_fid(), 0) };

        check_error(err.try_into().unwrap())
    }

    pub fn peer(&self) -> Result<Address, crate::error::Error> {
        let mut len = 0;
        let err = unsafe {
            libfabric_sys::inlined_fi_getpeer(
                self.as_raw_typed_fid(),
                std::ptr::null_mut(),
                &mut len,
            )
        };

        if -err as u32 == libfabric_sys::FI_ETOOSMALL {
            let mut address = vec![0; len];
            let err = unsafe {
                libfabric_sys::inlined_fi_getpeer(
                    self.as_raw_typed_fid(),
                    address.as_mut_ptr().cast(),
                    &mut len,
                )
            };
            if err != 0 {
                Err(crate::error::Error::from_err_code(
                    (-err).try_into().unwrap(),
                ))
            } else {
                Ok(Address { address })
            }
        } else {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        }
    }
}