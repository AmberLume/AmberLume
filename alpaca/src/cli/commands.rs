use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "alpaca")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Generate {
        input_dir: PathBuf,
        output_dir: PathBuf,
    },
    Pack {
        input_dir: PathBuf,
        output_dir: PathBuf,
        name: String,
    },
    Unpack {
        input_dir: PathBuf,
        output_dir: PathBuf,
    },
}
