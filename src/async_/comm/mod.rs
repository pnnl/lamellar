use crate::{cq::ReadCq, error::ErrorKind};

pub mod atomic;
pub mod collective;
pub mod message;
pub mod rma;
pub mod tagged;

#[inline]
fn progress(cq: &(impl ReadCq + ?Sized)) -> Result<(), crate::error::Error> {
    match cq.read(0) {
        Ok(_) => panic!("cq.read(0) returned a completion"),
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
    loop {
        match foo() {
            Ok(()) => break,
            Err(error) => {
                if matches!(error.kind, ErrorKind::TryAgain) {
                    progress(cq)?;
                    async_std::task::yield_now().await;
                } else {
                    return Err(error);
                }
            }
        }
    }

    Ok(())
}
