use std::marker::PhantomData;

pub struct On;
pub struct Off;

pub struct WaitNone;
pub struct WaitNoRetrieve;
pub struct WaitRetrieve;


pub trait EqWritable{}
pub trait EqConfig{}

pub struct Options<WRITE,WAIT,WAITFD> {
    write: PhantomData<WRITE>,
    wait: PhantomData<WAIT>,
    fd: PhantomData<WAITFD>,
}

impl Options<Off,WaitNoRetrieve,Off>  {
    pub(crate) fn new() -> Self {
        Self {
            write: PhantomData,
            wait: PhantomData,
            fd: PhantomData,
        }
    }
}

impl<WRITE, WAIT, WAITFD> Options <WRITE, WAIT, WAITFD> {
    
    pub(crate) fn wait_retrievable(self) -> Options<WRITE, WaitRetrieve, Off> {
        Options::<WRITE, WaitRetrieve, Off> {
            write: PhantomData,
            wait: PhantomData,
            fd: PhantomData,
        }
    }

    pub(crate) fn no_wait(self) -> Options<WRITE, WaitNone, Off> {
        Options::<WRITE, WaitNone, Off> {
            write: PhantomData,
            wait: PhantomData,
            fd: PhantomData,
        }
    }

    pub(crate) fn wait_no_retrieve(self) -> Options<WRITE, WaitNoRetrieve, Off> {
        Options::<WRITE, WaitNoRetrieve, Off> {
            write: PhantomData,
            wait: PhantomData,
            fd: PhantomData,
        }
    }

    pub(crate) fn wait_fd(self) -> Options<WRITE, WaitRetrieve, On> {
        Options::<WRITE, WaitRetrieve, On> {
            write: PhantomData,
            wait: PhantomData,
            fd: PhantomData,
        }
    }

    pub(crate) fn writable(self) -> Options<On, WAIT, WAITFD> {
        Options::<On, WAIT, WAITFD> {
            write: PhantomData,
            wait: PhantomData,
            fd: PhantomData,
        }
    }
}

impl<WRITE, WAITFD> crate::Waitable for Options<WRITE, WaitNoRetrieve, WAITFD> {}
impl<WRITE, WAITFD> crate::Waitable for Options<WRITE, WaitRetrieve, WAITFD> {}
impl<WRITE, WAITFD> crate::WaitRetrievable for Options<WRITE, WaitRetrieve, WAITFD> {}


impl<WAIT, WAITFD> Options<Off, WAIT, WAITFD> {

}

impl<WAIT, WAITFD> EqWritable for Options<On, WAIT, WAITFD> {}

// impl<WRITE> Options<WRITE, WaitRetrieve, Off> {}

// impl<WRITE> crate::FdRetrievable for Options<WRITE, WaitRetrieve, On> {}

impl<WRITE, WAIT, WAITFD> EqConfig for Options<WRITE, WAIT, WAITFD> {}