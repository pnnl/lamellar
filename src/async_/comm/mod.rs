use std::time::Instant;

use crate::{cq::ReadCq, error::ErrorKind};

pub mod atomic;
pub mod collective;
pub mod message;
pub mod rma;
pub mod tagged;

#[inline]
fn progress(cq: &(impl ReadCq + ?Sized)) -> Result<(), crate::error::Error> {
    match cq.read(0) {
        Ok(completion) => match completion {
            crate::cq::Completion::Unspec(vec) => {
                assert_eq!(vec.len(), 0);
                Ok(())
            }
            crate::cq::Completion::Ctx(vec) => {
                assert_eq!(vec.len(), 0);
                Ok(())
            }
            crate::cq::Completion::Msg(vec) => {
                assert_eq!(vec.len(), 0);
                Ok(())
            }
            crate::cq::Completion::Data(vec) => {
                assert_eq!(vec.len(), 0);
                Ok(())
            }
            crate::cq::Completion::Tagged(vec) => {
                assert_eq!(vec.len(), 0);
                Ok(())
            }
        },
        Err(cq_error) => {
            if !matches!(cq_error.kind, ErrorKind::TryAgain) {
                Err(cq_error)
            } else {
                Ok(())
            }
        }
    }
}

async fn while_try_again(
    cq: &(impl ReadCq + ?Sized),
    mut foo: impl FnMut() -> Result<(), crate::error::Error>,
) -> Result<(), crate::error::Error> {
    // let mut temp_now = Instant::now();
    // println!("while_try_again started");
    loop {
        match foo() {
            Ok(()) => break,
            Err(error) => {
                if matches!(error.kind, ErrorKind::TryAgain) {
                    progress(cq)?;
                    // println!("while_try_again: TryAgain error, retrying...");
                    async_std::task::yield_now().await;
                } else {
                    return Err(error);
                }
            }
        }
        // if temp_now.elapsed().as_secs() > 5 {
        //     println!("Probably stuck in trying to put/read\n {}", std::backtrace::Backtrace::capture());
        //     temp_now = Instant::now();
        // }
    }
    // println!("while_try_again finished");
    Ok(())
}
