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
    Save {
        filename: String,
        #[arg(value_parser = parse_coordinate)]
        origin: Coordinate,
        #[arg(value_parser = parse_coordinate)]
        bound: Coordinate,
    },

    Load {
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
