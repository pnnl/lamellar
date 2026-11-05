use crate::array::ArrayDistribution;
use crate::options::{IndexGatherCli, IndexSize};
use crate::Variant;
use std::time::Duration;

pub(crate) fn print_am_times(
    cli: &IndexGatherCli,
    my_pe: usize,
    num_pes: usize,
    variant: &Variant,
    idx_size: &IndexSize,
    times: (Duration, Duration, Duration, Duration),
) {
    if my_pe == 0 {
        let payload_size = match idx_size {
            IndexSize::Usize => std::mem::size_of::<usize>() + std::mem::size_of::<usize>(),
            IndexSize::U32 => std::mem::size_of::<usize>() + std::mem::size_of::<u32>(),
        };
        let l_num_updates = cli.pe_updates(num_pes);
        let g_num_updates = cli.total_updates(num_pes);
        let variant_col = format!("{:<1$}", format!("{variant:?}"), cli.max_variant_len());
        let idx_size_col = format!(
            "{:<1$}",
            format!("{idx_size:?}"),
            cli.max_index_size_len() + cli.max_array_distribution_len()
        );
        println!(
            "{variant_col} {idx_size_col} lmups {:>9.2?}, gmups {:>9.2?}, lGB/s {:>7.2?}, gGB/s {:>7.2?}, {times:>6.4?}",
            (l_num_updates as f32 / 1_000_000.0) / times.3.as_secs_f32(),
            (g_num_updates as f32 / 1_000_000.0) / times.3.as_secs_f32(),
            ((l_num_updates * payload_size) as f32 / 1_000_000_000.0) / times.3.as_secs_f32(),
            ((g_num_updates * payload_size) as f32 / 1_000_000_000.0) / times.3.as_secs_f32(),
        );
    }
}

pub(crate) fn print_array_times(
    cli: &IndexGatherCli,
    my_pe: usize,
    num_pes: usize,
    variant: &Variant,
    distribution: &ArrayDistribution,
    times: (Duration, Duration, Duration, Duration),
) {
    if my_pe == 0 {
        // alculate the size of indices used by the lamellar array,
        // which is based on the number of elements on each PE (not the total table size)
        let pe_table_size = cli.pe_table_size(num_pes);
        let index_size = usize::BITS as usize - pe_table_size.leading_zeros() as usize;
        let (index_size, index_name) = if index_size > 32 {
            (std::mem::size_of::<usize>(), "usize")
        } else if index_size > 16 {
            (std::mem::size_of::<u32>(), "u32")
        } else if index_size > 8 {
            (std::mem::size_of::<u16>(), "u16")
        } else {
            (std::mem::size_of::<u8>(), "u8")
        };
        let payload_size = std::mem::size_of::<usize>() + index_size;
        let l_num_updates = cli.pe_updates(num_pes);
        let g_num_updates = cli.total_updates(num_pes);

        let variant_col = format!("{:<1$}", format!("{variant:?}"), cli.max_variant_len());
        let dist_col = format!(
            "{:<1$}",
            format!("{distribution:?} {index_name}"),
            cli.max_index_size_len() + cli.max_array_distribution_len()
        );
        println!(
            "{variant_col} {dist_col} lmups {:>9.2?}, gmups {:>9.2?}, lGB/s {:>7.2?}, gGB/s {:>7.2?}, {times:>6.4?}",
            (l_num_updates as f32 / 1_000_000.0) / times.3.as_secs_f32(),
            (g_num_updates as f32 / 1_000_000.0) / times.3.as_secs_f32(),
            ((l_num_updates * payload_size) as f32 / 1_000_000_000.0) / times.3.as_secs_f32(),
            ((g_num_updates * payload_size) as f32 / 1_000_000_000.0) / times.3.as_secs_f32(),
        );
    }
}

pub(crate) fn print_results(
    cli: &IndexGatherCli,
    my_pe: usize,
    num_pes: usize,
    variant: &str,
    sub_variant: &str,
    times: &[(Duration, Duration, Duration, Duration)],
) {
    let g_num_updates = cli.total_updates(num_pes) as f32 / 1_000_000.0;
    if my_pe == 0 {
        let gups = times
            .iter()
            .map(|t| g_num_updates / t.3.as_secs_f32())
            .collect::<Vec<f32>>();
        println!(
            "summary: {variant} {sub_variant} {gups:>6.2?} min: {:>6.2} avg: {:>6.2} max: {:>6.2}",
            gups.iter().fold(f32::INFINITY, |a, &b| a.min(b)),
            gups.iter().sum::<f32>() / gups.len() as f32,
            gups.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b))
        );
    }
}
