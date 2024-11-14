use crate::{domain::{DomainBase, DomainBuilder, DomainImplBase}, eq::EventQueueBase, fid::{AsRawFid, AsRawTypedFid, AsTypedFid}, MyRc};

use super::eq::{AsyncReadEq, EventQueue};



pub(crate) type AsyncDomainImpl = DomainImplBase<dyn AsyncReadEq>;
pub type Domain = DomainBase<dyn AsyncReadEq>;

impl DomainImplBase<dyn AsyncReadEq> {

    
    pub(crate) fn bind(&self, eq: MyRc<dyn AsyncReadEq>, async_mem_reg: bool) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_domain_bind(self.as_typed_fid().as_raw_typed_fid(), eq.as_typed_fid().as_raw_fid(), if async_mem_reg {libfabric_sys::FI_REG_MR} else {0})} ;

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            if self._eq_rc.set((eq, async_mem_reg)).is_err() {
                panic!("Domain is alread bound to an EventQueue");
            }
            Ok(())
        }
    } 
}

impl DomainBase<dyn AsyncReadEq> {
    /// Associates an [crate::eq::EventQueue] with the domain.
    /// 
    /// If `async_mem_reg` is true, the provider should perform all memory registration operations asynchronously, with the completion reported through the event queue
    /// 
    /// Corresponds to `fi_domain_bind`, with flag `FI_REG_MR` if `async_mem_reg` is true. 
    pub(crate) fn bind_eq<EQ: AsyncReadEq + 'static>(&self, eq: &EventQueueBase<EQ>, async_mem_reg: bool) -> Result<(), crate::error::Error> {
        self.inner.bind(eq.inner.clone(), async_mem_reg)
    }
}

impl<'a, E> DomainBuilder<'a, E> {

    pub fn build_and_bind_async<EQ: AsyncReadEq + 'static>(self, eq: &EventQueue<EQ>, async_mem_reg: bool) -> Result<DomainBase<dyn AsyncReadEq>, crate::error::Error> {
        let domain = DomainBase::<dyn AsyncReadEq>::new(self.fabric, self.info, self.flags, self.info.domain_attr().clone(), self.ctx)?;
        domain.bind_eq(eq, async_mem_reg)?;
        Ok(domain)
    }
}
