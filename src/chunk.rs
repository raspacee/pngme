use std::{convert::TryFrom, convert::TryInto} ;
use std::fmt;
use crc::crc32::checksum_ieee;

use crate::{Result, Error};
use crate::chunk_type::ChunkType;

#[derive(Debug)]
pub struct Chunk {
    data_length: u32,
    chunk_type: ChunkType,
    chunk_data: Vec<u8>,
    crc: u32,
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, chunk_data: Vec<u8>) -> Chunk {
        let data_length = chunk_data.len() as u32;

        let c: Vec<u8> = chunk_type.bytes()
        .iter()
        .cloned()
        .chain(chunk_data.iter().cloned())
        .collect();
        let crc = checksum_ieee(&c);

        Chunk { 
            data_length, 
            chunk_type, 
            chunk_data, 
            crc,
        }
    }

    pub fn length(&self) -> u32 {
        self.data_length
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        &self.chunk_data
    }

    pub fn crc(&self) -> u32 {
        let c: Vec<u8> = self.chunk_type.bytes()
            .iter()
            .cloned()
            .chain(self.data().iter().cloned())
            .collect();
        checksum_ieee(&c)
    }

    pub fn data_as_string(&self) -> Result<String> {
        match String::from_utf8(self.chunk_data.clone()) {
            Ok(mystr) => {
                Ok(mystr)
            }
            Err(_) => {
                Err(Box::new(ChunkError::UTF8UncompatibleChunk))
            }
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let chunk = self.data_length.to_be_bytes()
            .iter()
            .chain(self.chunk_type.bytes().iter())
            .chain(self.chunk_data.iter())
            .chain(self.crc.to_be_bytes().iter())
            .copied()
            .collect();
        chunk
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        //TODOcheck len of bytes

        // Reading data_length
        let (data_length_buf, bytes) = bytes.split_at(4);
        let data_length = u32::from_be_bytes(data_length_buf.try_into()?);

        // Reading chunk_type
        let (chunk_type_buf, bytes) = bytes.split_at(4);
        let c: [u8; 4] = chunk_type_buf.try_into()?;
        let chunk_type = ChunkType::try_from(c).unwrap();

        // Reading chunk_data
        let (chunk_data, bytes) = bytes.split_at(data_length as usize);
        let chunk_data: Vec<u8> = chunk_data.to_vec();
        
        // Reading crc
        let (bytes, _) = bytes.split_at(4);
        let crc = u32::from_be_bytes(bytes.try_into()?);
        let b: Vec<u8> = chunk_type_buf.iter().chain(chunk_data.iter()).copied().collect();
        let calculated_crc = checksum_ieee(&b);

        if crc == calculated_crc {
            Ok(Chunk {
                data_length,
                chunk_type,
                chunk_data,
                crc,
            })
        } else {
            Err(Box::new(ChunkError::InvalidCRC))
        }
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Chunk {{")?;
        writeln!(f, "   length: {}", self.length())?;
        writeln!(f, "   chunk type: {}", self.chunk_type)?;
        writeln!(f, "   data: {}", self.data().len())?;
        writeln!(f, "   crc: {}", self.crc())?;
        writeln!(f, "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ChunkError {
    InvalidCRC,
    UTF8UncompatibleChunk,
}

impl std::error::Error for ChunkError {}

impl fmt::Display for ChunkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCRC => {
                write!(f, "Invalid CRC")
            }
            Self::UTF8UncompatibleChunk=> {
                write!(f, "Chunk data is not compatible with UTF-8")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        
        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        
        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();
        
        let _chunk_string = format!("{}", chunk);
    }
}
