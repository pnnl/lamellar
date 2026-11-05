use crate::Variant;
use std::time::Duration;

pub(crate) fn print_am_times(
    my_pe: usize,
    variant: &Variant,
    buf_size: usize,
    times: (Duration, Duration, Duration),
) {
    if my_pe == 0 {
        println!("{variant:?} buf_size: {buf_size:?} {times:?} ",);
    }
}

pub(crate) fn print_results(
    my_pe: usize,
    variant: &str,
    buf_size: &str,
    times: &[(Duration, Duration, Duration)],
) {
    if my_pe == 0 {
        let secs = times
            .iter()
            .map(|t| t.2.as_secs_f64())
            .collect::<Vec<f64>>();
        println!(
            "summary: {variant} {buf_size} {secs:<6.6?} min: {:<6.6} avg: {:<6.6} max: {:<6.6}",
            secs.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            secs.iter().sum::<f64>() / secs.len() as f64,
            secs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
        );
    }
}
