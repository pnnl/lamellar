//! UCC collectives for the `ucx` lamellae: the shared implementation in `lamellae/ucc.rs`, bootstrapped over this
//! lamellae's own allocation.
use super::fabric::UcxAlloc;
pub(crate) use crate::lamellae::ucc::*;

impl UccOobAlloc for UcxAlloc {
    fn my_pe(&self) -> usize {
        self.my_pe
    }
    fn num_pes(&self) -> usize {
        self.num_pes
    }
    unsafe fn flags_u8(&self) -> &mut [u8] {
        self.as_mut_slice::<u8>()
    }
    fn wait(&self) {
        UcxAlloc::wait(self)
    }
    unsafe fn put_u8(&self, pe: usize, offset: usize, src: &[u8], blocking: bool, managed: bool) {
        self.put_inner(pe, offset, src, blocking, managed);
    }
    unsafe fn get_u8(&self, pe: usize, offset: usize, blocking: bool, dst: &mut [u8]) {
        self.inner_get(pe, offset, blocking, dst);
    }
}
