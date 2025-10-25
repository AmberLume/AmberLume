use alpaca::packer::alpaca_writer::AlpacaWriter;
use alpaca::unpacker::alpaca_reader::AlpacaReader;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs::{create_dir_all, read, write};
use std::path::PathBuf;
use std::time::Instant;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "alpaca")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

pub fn main() -> Result<()> {
    let cli = Cli::parse();

    let instant = Instant::now();

    match cli.command {
        Commands::Pack {
            input_dir,
            output_dir,
            name,
        } => {
            let mut alpaca_writer = AlpacaWriter::create(name, output_dir, 64 * 1024)?;

            for entry in WalkDir::new(&input_dir)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let name = entry
                    .path()
                    .strip_prefix(&input_dir)?
                    .to_string_lossy()
                    .into_owned();

                let data = read(entry.path())?;

                println!("Writing '{}', {} bytes...", name, data.len());
                alpaca_writer.push(name, &data)?
            }

            alpaca_writer.pack()?;
        }
        Commands::Unpack {
            input_dir,
            output_dir,
        } => {
            let alpaca_reader = AlpacaReader::parse(input_dir)?;

            for entry in &alpaca_reader.entries {
                println!("Creating '{}', {} bytes...", entry.name, entry.size);
                let data_slice = alpaca_reader.read_slice(entry)?;

                let file_path = output_dir.join(&entry.name);

                if let Some(parent) = file_path.parent() {
                    create_dir_all(parent)?;
                }

                write(&file_path, data_slice)?;
            }
        }
    }

    let elapsed = instant.elapsed();
    println!("Время: {:.3?}", elapsed);

    Ok(())
}
