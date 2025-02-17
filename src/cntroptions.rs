// use std::marker::PhantomData;

// pub struct On;
// pub struct Off;

// pub struct WaitNone;
// pub struct WaitNoRetrieve;
// pub struct WaitRetrieve;

// pub trait CntrConfig{}

// pub struct Options<WAIT,WAITFD> {
//     wait: PhantomData<WAIT>,
//     fd: PhantomData<WAITFD>,
// }

// impl Options<WaitNoRetrieve,Off>  {
//     pub(crate) fn new() -> Self {
//         Self {
//             wait: PhantomData,
//             fd: PhantomData,
//         }
//     }
// }

// impl<WAIT, WAITFD> Options <WAIT, WAITFD> {

//     pub(crate) fn wait_retrievable(self) -> Options<WaitRetrieve, Off> {
//         Options::<WaitRetrieve, Off> {
//             wait: PhantomData,
//             fd: PhantomData,
//         }
//     }

//     pub(crate) fn no_wait(self) -> Options<WaitNone, Off> {
//         Options::<WaitNone, Off> {
//             wait: PhantomData,
//             fd: PhantomData,
//         }
//     }

//     pub(crate) fn wait_no_retrieve(self) -> Options<WaitNoRetrieve, Off> {
//         Options::<WaitNoRetrieve, Off> {
//             wait: PhantomData,
//             fd: PhantomData,
//         }
//     }

//     pub(crate) fn wait_fd(self) -> Options<WaitRetrieve, On> {
//         Options::<WaitRetrieve, On> {
//             wait: PhantomData,
//             fd: PhantomData,
//         }
//     }
// }

// impl<WAITFD> crate::Waitable for Options<WaitNoRetrieve, WAITFD> {}
// impl<WAITFD> crate::Waitable for Options<WaitRetrieve, WAITFD> {}
// impl<WAITFD> crate::WaitRetrievable for Options<WaitRetrieve, WAITFD> {}

// // impl<WAIT, WAITFD> Options<Off, WAIT, WAITFD> {}

// // impl Options<WaitRetrieve, Off> {}

// impl crate::FdRetrievable for Options<WaitRetrieve, On> {}

// impl<WAIT, WAITFD> CntrConfig for Options<WAIT, WAITFD> {}
