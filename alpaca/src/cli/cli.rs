use crate::assembler::pipeline::Pipeline;
use crate::cli::commands::{Cli, Commands};
use crate::packer::alpaca_writer::AlpacaWriter;
use crate::unpacker::alpaca_reader::AlpacaReader;
use crate::walker::walker::Walker;
use anyhow::Result;
use clap::Parser;
use std::fs::{create_dir_all, read, write};

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            input_dir,
            output_dir,
        } => {
            let mut pipeline = Pipeline::new()?;

            pipeline.assemble(&input_dir, &output_dir)?;
        }
        Commands::Pack {
            input_dir,
            output_dir,
            name,
        } => {
            let walker = Walker::create(&input_dir);
            let mut alpaca_writer = AlpacaWriter::create(name, output_dir, 16)?;

            walker.walk(
                |_| true,
                |path, name| {
                    let data = read(path)?;

                    println!("Writing '{}', {} bytes...", name, data.len());
                    alpaca_writer.push(name, &data)
                },
            )?;

            alpaca_writer.pack()?;
        }
        Commands::Unpack {
            input_dir,
            output_dir,
        } => {
            let alpaca_reader = AlpacaReader::parse(&input_dir)?;

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

    Ok(())
}
