#[cfg(not(any(
    feature = "with-pmi1",
    feature = "with-pmi2",
    feature = "with-pmix"
)))]
compile_error!("At least one of the features 'with-pmi1', 'with-pmi2' or 'with-pmix' must be enabled");
pub mod pmi;
#[cfg(feature = "with-pmi1")]
pub mod pmi1;
#[cfg(feature = "with-pmi2")]
pub mod pmi2;
#[cfg(feature = "with-pmix")]
pub mod pmix;
