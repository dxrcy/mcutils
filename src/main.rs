mod args;

use std::fs;
use std::io;

use anyhow::Result;
use clap::Parser;
use mcrs::Block;
use mcrs::Connection;

use crate::args::Command;
use mcutils::{read_data, write_data};

fn main() -> Result<()> {
    let args = args::Args::parse();

    let mut mc = Connection::new().expect("Failed to connect to Minecraft server");

    match args.command {
        Command::Clear { origin, bound } => {
            let chunk = mc.get_blocks(origin, bound)?;
            let size = origin.size_between(bound);

            for i in 0..size.volume() {
                let coord = origin + size.index_to_offset(i);
                let block = Block::AIR;
                let current_block = chunk
                    .get_worldspace(coord)
                    .expect("Chunk should contain coordinate");
                if block != current_block {
                    mc.set_block(coord, block)?;
                }
            }

            println!("Successfully cleared {:?} chunk at {}.", size, origin);
        }

        Command::Save {
            filename,
            origin,
            bound,
        } => {
            let file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(filename)?;
            let mut writer = io::BufWriter::new(file);

            let mut chunk = mc.get_blocks_stream(origin, bound)?;
            write_data(&mut writer, &mut chunk)?;

            println!(
                "Successfully saved {:?} chunk at {}.",
                chunk.size(),
                chunk.origin(),
            );
        }

        Command::Load { filename } => {
            let file = fs::OpenOptions::new().read(true).open(filename)?;
            let mut reader = io::BufReader::new(file);

            let mut entries = read_data(&mut reader)?;

            let chunk = mc.get_blocks(entries.origin(), entries.bound())?;

            for entry in &mut entries {
                let (coord, block) = entry?;
                let current_block = chunk
                    .get_worldspace(coord)
                    .expect("Chunk should contain coordinate");
                if block != current_block {
                    mc.set_block(coord, block)?;
                }
            }

            println!(
                "Successfully loaded {:?} chunk at {}.",
                entries.size(),
                entries.origin()
            );
        }
    }

    Ok(())
}
