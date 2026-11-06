use crate::array::ArrayDistribution;
use crate::options::{IndexSize, RandPermCli};
use crate::Variant;
use std::time::Duration;

pub(crate) fn print_am_times(
    cli: &RandPermCli,
    my_pe: usize,
    variant: &Variant,
    idx_size: &IndexSize,
    times: (Duration, Duration, Duration, usize),
) {
    if my_pe == 0 {
        let variant_col = format!("{:<1$}", format!("{variant:?}"), cli.max_variant_len());
        let idx_size_col = format!(
            "{:<1$}",
            format!("{idx_size:?}"),
            cli.max_index_size_len() + cli.max_array_distribution_len() + 1
        );
        let t0 = format!("{:<6.4?}", times.0);
        let t1 = format!("{:<6.4?}", times.1);
        let t2 = format!("{:<6.4?}", times.2);
        println!(
            "{variant_col} {idx_size_col} {t0:<13} {t1:<13} {t2:<13} {}",
            times.3
        );
    }
}

pub(crate) fn print_array_times(
    cli: &RandPermCli,
    my_pe: usize,
    num_pes: usize,
    variant: &Variant,
    distribution: &ArrayDistribution,
    times: (Duration, Duration, Duration, usize),
) {
    if my_pe == 0 {
        // alculate the size of indices used by the lamellar array,
        // which is based on the number of elements on each PE (not the total table size)
        let pe_table_size = cli.pe_table_size(num_pes);
        let index_size = usize::BITS as usize - pe_table_size.leading_zeros() as usize;
        let (_index_size, index_name) = if index_size > 32 {
            (std::mem::size_of::<usize>(), "usize")
        } else if index_size > 16 {
            (std::mem::size_of::<u32>(), "u32")
        } else if index_size > 8 {
            (std::mem::size_of::<u16>(), "u16")
        } else {
            (std::mem::size_of::<u8>(), "u8")
        };
        let variant_col = format!("{:<1$}", format!("{variant:?}"), cli.max_variant_len());
        let dist_col = format!(
            "{:<1$}",
            format!("{distribution:?} {index_name}"),
            cli.max_index_size_len() + cli.max_array_distribution_len() + 1
        );
        let t0 = format!("{:<6.4?}", times.0);
        let t1 = format!("{:<6.4?}", times.1);
        let t2 = format!("{:<6.4?}", times.2);
        println!(
            "{variant_col} {dist_col}  {t0:<13} {t1:<13} {t2:<13} {}",
            times.3
        );
    }
}

pub(crate) fn print_results(
    my_pe: usize,
    variant: &str,
    sub_variant: &str,
    times: &[(Duration, Duration, Duration, usize)],
) {
    if my_pe == 0 {
        let secs = times
            .iter()
            .map(|t| t.2.as_secs_f64())
            .collect::<Vec<f64>>();
        println!(
            "summary: {variant} {sub_variant} {secs:<6.6?} min: {:<6.6} avg: {:<6.6} max: {:<6.6}",
            secs.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            secs.iter().sum::<f64>() / secs.len() as f64,
            secs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
        );
    }
}
