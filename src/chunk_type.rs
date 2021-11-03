use std::convert::{ TryFrom, TryInto };
use std::str;
use std::str::FromStr;
use std::fmt;
use std::cmp;

use crate::{Result, Error};

#[derive(cmp::PartialEq, cmp::Eq, Debug)]
pub struct ChunkType {
    chunk: [u8; 4],
}

impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        self.chunk
    }

    pub fn is_valid(&self) -> bool {
        // Other validations are checked during construction
        if self.is_reserved_bit_valid() {
            return true;
        }

        false
    }

    pub fn is_critical(&self) -> bool {
        u8::is_ascii_uppercase(&self.chunk[0])
    }

    pub fn is_public(&self) -> bool {
        u8::is_ascii_uppercase(&self.chunk[1])
    }

    pub fn is_reserved_bit_valid(&self) -> bool {
        u8::is_ascii_uppercase(&self.chunk[2])
    }

    pub fn is_safe_to_copy(&self) -> bool {
        u8::is_ascii_lowercase(&self.chunk[3])
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = Error;

    fn try_from(bytes: [u8; 4]) -> Result<Self> {
        for b in bytes {
            if !u8::is_ascii_lowercase(&b) && !u8::is_ascii_uppercase(&b) {
                return Err(Box::new(ChunkTypeError::InvalidChunkTypeBytes));
            }
        }

        Ok(ChunkType {
            chunk: bytes,
        })
    }
}

impl FromStr for ChunkType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let slice: [u8; 4] = match s.as_bytes().try_into() {
            Ok(slice) => slice,
            Err(_) => {
                return Err(Box::new(ChunkTypeError::InvalidString))
            }
        };

        ChunkType::try_from(slice)
    }
}

impl fmt::Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", str::from_utf8(&self.chunk).unwrap().to_string())
    }
}

#[derive(Debug)]
pub enum ChunkTypeError {
    InvalidString,
    InvalidChunkTypeBytes,
}

impl std::error::Error for ChunkTypeError {}

impl fmt::Display for ChunkTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidString => {
                write!(f, "Invalid string")
            }
            Self::InvalidChunkTypeBytes=> {
                write!(f, "Invalid ChunkType bytes")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}