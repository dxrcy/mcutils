use std::fs::{self, File};
use std::io::{self, Read as _, Write as _};
use std::{error, fmt};

use clap::{Parser, Subcommand};
use mcrs::chunk::Size;
use mcrs::{Block, Chunk, Connection, Coordinate};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
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

fn main() {
    let args = Args::parse();

    let mut mc = Connection::new().expect("Failed to connect to MineCraft server");

    match args.command {
        Command::Save {
            filename,
            origin,
            bound,
        } => {
            let chunk = mc.get_blocks(origin, bound).expect("Failed to get blocks");
            let mut file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(filename)
                .expect("Failed to open file");
            write_file(&mut file, &chunk).expect("Failed to write file");
            let size = origin.size_between(bound);
            let origin = origin.min(bound);
            println!("Successfully saved {:?} chunk at {}.", size, origin);
        }

        Command::Load { filename } => {
            let mut file = fs::OpenOptions::new()
                .read(true)
                .open(filename)
                .expect("Failed to open file");
            let entries = read_file(&mut file).expect("Failed to read file");
            let origin = entries.origin();
            let size = entries.size();
            for entry in entries {
                let (coord, block) = entry.expect("Failed to read file");
                mc.set_block(coord, block).expect("Failed to set block");
            }
            println!("Successfully loaded {:?} chunk at {}.", size, origin);
        }
    }
}

const MAGIC_NUMBER: u16 = 0xa3f9;
const VERSION: u16 = 0x01_00;

fn write_file(file: &mut File, chunk: &Chunk) -> io::Result<()> {
    file.write_all(&MAGIC_NUMBER.to_le_bytes())?;
    file.write_all(&VERSION.to_le_bytes())?;

    file.write_all(&chunk.origin().x.to_le_bytes())?;
    file.write_all(&chunk.origin().y.to_le_bytes())?;
    file.write_all(&chunk.origin().z.to_le_bytes())?;

    file.write_all(&chunk.size().x.to_le_bytes())?;
    file.write_all(&chunk.size().y.to_le_bytes())?;
    file.write_all(&chunk.size().z.to_le_bytes())?;

    for item in chunk.iter() {
        file.write_all(&item.block().id.to_le_bytes())?;
        file.write_all(&item.block().modifier.to_le_bytes())?;
    }

    Ok(())
}

struct BlockReader<'a> {
    file: &'a mut File,
    index: u32,
    origin: Coordinate,
    size: Size,
}

impl<'a> BlockReader<'a> {
    pub fn new(file: &'a mut File, origin: Coordinate, size: Size) -> Self {
        Self {
            file,
            index: 0,
            origin,
            size,
        }
    }

    pub fn origin(&self) -> Coordinate {
        self.origin
    }
    pub fn size(&self) -> Size {
        self.size
    }
}

impl<'a> Iterator for BlockReader<'a> {
    type Item = io::Result<(Coordinate, Block)>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = match try_read_u32(self.file) {
            Ok(Some(id)) => id,
            Ok(None) => return None,
            Err(error) => return Some(Err(error)),
        };
        let modifier = match read_u32(self.file) {
            Ok(modifier) => modifier,
            Err(error) => return Some(Err(error)),
        };
        let block = Block::new(id, modifier);

        let y = self.index / self.size.x / self.size.z;
        let x = self.index / self.size.z % self.size.x;
        let z = self.index % self.size.z;
        let coordinate = self.origin + Coordinate::new(x as i32, y as i32, z as i32);

        self.index += 1;
        Some(Ok((coordinate, block)))
    }
}

fn read_file(file: &mut File) -> io::Result<BlockReader> {
    check_metadata(file)?;

    let x = read_i32(file)?;
    let y = read_i32(file)?;
    let z = read_i32(file)?;
    let origin = Coordinate::new(x, y, z);

    let x = read_u32(file)?;
    let y = read_u32(file)?;
    let z = read_u32(file)?;
    let size = Size::new(x, y, z);

    Ok(BlockReader::new(file, origin, size))
}

fn check_metadata(file: &mut File) -> io::Result<()> {
    let magic_number = read_u16(file)?;
    if magic_number != MAGIC_NUMBER {
        panic!("Invalid file format");
    }
    let version = read_u16(file)?;
    if version < VERSION {
        panic!("Outdated file format");
    } else if version > VERSION {
        panic!("Outdated program");
    }
    Ok(())
}

fn read_u16(file: &mut File) -> io::Result<u16> {
    let mut buf = [0u8; 2];
    file.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_i32(file: &mut File) -> io::Result<i32> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
}

fn read_u32(file: &mut File) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn try_read_u32(file: &mut File) -> io::Result<Option<u32>> {
    let mut buf = [0u8; 4];
    let bytes_read = file.read(&mut buf)?;
    if bytes_read == 0 {
        return Ok(None);
    }
    if bytes_read < 4 {
        return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
    }
    Ok(Some(u32::from_le_bytes(buf)))
}
