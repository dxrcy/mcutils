use std::io::{self, Read, Write};

use mcrs::chunk::ChunkStream;
use mcrs::{Block, Coordinate, Error, Size};

type Result<T> = std::result::Result<T, Error>;

const MAGIC_NUMBER: u16 = 0xa3f9;
const VERSION: u16 = 0x01_00;

pub fn write_data(file: &mut impl Write, chunk: &mut ChunkStream<'_>) -> Result<()> {
    file.write_all(&MAGIC_NUMBER.to_le_bytes())?;
    file.write_all(&VERSION.to_le_bytes())?;

    file.write_all(&chunk.origin().x.to_le_bytes())?;
    file.write_all(&chunk.origin().y.to_le_bytes())?;
    file.write_all(&chunk.origin().z.to_le_bytes())?;

    file.write_all(&chunk.size().x.to_le_bytes())?;
    file.write_all(&chunk.size().y.to_le_bytes())?;
    file.write_all(&chunk.size().z.to_le_bytes())?;

    while let Some(item) = chunk.next()? {
        file.write_all(&item.block().id.to_le_bytes())?;
        file.write_all(&item.block().modifier.to_le_bytes())?;
    }

    Ok(())
}

pub fn read_data<R: Read>(file: &mut R) -> io::Result<BlockReader<R>> {
    check_data_metadata(file)?;

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

pub struct BlockReader<'a, R> {
    reader: &'a mut R,
    index: u32,
    origin: Coordinate,
    size: Size,
}

impl<'a, R> BlockReader<'a, R> {
    fn new(reader: &'a mut R, origin: Coordinate, size: Size) -> Self {
        Self {
            reader,
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
    pub fn bound(&self) -> Coordinate {
        self.origin + self.size
    }
}

impl<'a, R: Read> Iterator for &mut BlockReader<'a, R> {
    type Item = io::Result<(Coordinate, Block)>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = match try_read_u32(self.reader) {
            Ok(Some(id)) => id,
            Ok(None) => return None,
            Err(error) => return Some(Err(error)),
        };
        let modifier = match read_u32(self.reader) {
            Ok(modifier) => modifier,
            Err(error) => return Some(Err(error)),
        };
        let block = Block::new(id, modifier);

        let coordinate = self.origin + self.size.index_to_offset(self.index as usize);

        self.index += 1;
        Some(Ok((coordinate, block)))
    }
}

fn check_data_metadata(file: &mut impl Read) -> io::Result<()> {
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

fn read_u16(file: &mut impl Read) -> io::Result<u16> {
    let mut buf = [0u8; 2];
    file.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_i32(file: &mut impl Read) -> io::Result<i32> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
}

fn read_u32(file: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn try_read_u32(file: &mut impl Read) -> io::Result<Option<u32>> {
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
