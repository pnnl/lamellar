use clap::Parser;

use crate::Variant;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct TcCli {
    #[arg(short, long)]
    pub graph_file: String,

    #[arg(short, long, default_value_t = 3)]
    pub iterations: usize,

    #[arg(short, long, default_value_t = 1)]
    pub launch_threads: usize,

    #[arg(short, long, num_args(1..), default_values_t = vec![10000])]
    pub buffer_size: Vec<usize>,

    #[arg(value_enum, short, long, num_args(0..))]
    pub variants: Option<Vec<Variant>>,
}

impl TcCli {
    pub fn describe(&self) {
        println!("graph file: {}", self.graph_file);
        println!("iterations: {}", self.iterations);
        println!("launch threads: {}", self.launch_threads);
        println!("buffer size: {:?}", self.buffer_size);
        println!("variants: {:?}", self.variants);
    }

    pub fn max_variant_len(&self) -> usize {
        if let Some(variants) = &self.variants {
            variants
                .iter()
                .map(|v| format! {"{v:?}"}.len())
                .max()
                .unwrap()
        } else {
            "BufferedCasDartRemote".len()
        }
    }
}
