use std::{error, fmt};

use clap::{Parser, Subcommand};
use mcrs::Coordinate;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Clear a 3D block region (set every block to air)
    ///
    /// Order of bounding coordinates do not matter; they will be normalized
    Clear {
        /// First corner of 3D block region
        #[arg(value_parser = parse_coordinate)]
        origin: Coordinate,
        /// Second corner of 3D block region
        #[arg(value_parser = parse_coordinate)]
        bound: Coordinate,
    },

    /// Store a 3D block region to a file
    ///
    /// File will include coordinates and block data in a binary format
    ///
    /// Order of bounding coordinates do not matter; they will be normalized
    Save {
        /// Name of binary file to save to
        filename: String,
        /// First corner of 3D block region
        #[arg(value_parser = parse_coordinate)]
        origin: Coordinate,
        /// Second corner of 3D block region
        #[arg(value_parser = parse_coordinate)]
        bound: Coordinate,
    },

    /// Load a 3D block region from a file
    ///
    /// Always loads region at same position it was saved
    Load {
        /// Name of binary file to load from
        filename: String,
    },
}

#[derive(Debug)]
struct ParseCoordinateError;
impl error::Error for ParseCoordinateError {}
impl fmt::Display for ParseCoordinateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Must be of the form `x,y,z`")
    }
}

fn parse_coordinate(arg: &str) -> Result<Coordinate, ParseCoordinateError> {
    let mut parts = arg.split(',').map(|part| part.parse::<i32>().ok());
    let Some(x) = parts.next().flatten() else {
        return Err(ParseCoordinateError);
    };
    let Some(y) = parts.next().flatten() else {
        return Err(ParseCoordinateError);
    };
    let Some(z) = parts.next().flatten() else {
        return Err(ParseCoordinateError);
    };
    Ok(Coordinate { x, y, z })
}
