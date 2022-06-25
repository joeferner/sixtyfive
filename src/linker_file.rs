use std::{fmt, iter::Map, path::PathBuf};

use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take},
    character::complete::{alpha1, alphanumeric1, one_of},
    combinator::opt,
    error::{context, ErrorKind, VerboseError},
    multi::{count, many0, many1, many_m_n},
    sequence::{preceded, separated_pair, terminated, tuple},
    AsChar, Err as NomErr, IResult, InputTakeAtPosition,
};

use nom::character::complete::multispace0;
use nom::number::complete::hex_u32;

#[derive(Debug)]
pub enum ReadLinkerFileError {
    MissingFile(PathBuf),
    Invalid(),
    IoError(std::io::Error),
}

impl From<std::io::Error> for ReadLinkerFileError {
    fn from(err: std::io::Error) -> Self {
        return ReadLinkerFileError::IoError(err);
    }
}

impl fmt::Display for ReadLinkerFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReadLinkerFileError::MissingFile(path) => {
                write!(f, "Missing linker file {}", path.display())
            }
            ReadLinkerFileError::Invalid() => {
                write!(f, "Invalid linker file")
            }
            ReadLinkerFileError::IoError(err) => {
                write!(f, "Read linker io error {}", err)
            }
        }
    }
}

#[derive(Debug)]
pub struct Symbol {
    symbolType: Option<String>,
    value: Option<u32>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MemorySegment {
    file: Option<String>,
    start: Option<u32>,
    size: Option<u32>,
    fill: Option<bool>,
    define: Option<bool>,
}

#[derive(Debug)]
pub struct Segment {
    load: Option<String>,
    segmentType: Option<String>,
    run: Option<String>,
    define: Option<bool>,
    optional: Option<bool>,
}

#[derive(Debug)]
pub struct Feature {
    featureType: Option<String>,
    label: Option<String>,
    count: Option<String>,
    segment: Option<String>,
}

#[derive(Debug)]
pub struct LinkerFile {
    symbols: Option<Map<String, Symbol>>,
    memory: Option<Map<String, MemorySegment>>,
    segments: Option<Map<String, Segment>>,
    features: Option<Map<String, Map<String, Feature>>>,
}

pub fn read_linker_file(linker_file: String) -> Result<LinkerFile, ReadLinkerFileError> {
    if linker_file == "nes" {
        let str = include_str!("linker/nes.cfg");
        return read_linker_bytes(str);
    }

    let file = PathBuf::from(linker_file);
    if !file.as_path().exists() {
        return Result::Err(ReadLinkerFileError::MissingFile(file));
    }

    let str = std::fs::read_to_string(file.as_path())?;
    return read_linker_bytes(str.as_str());
}

fn read_linker_bytes(str: &str) -> Result<LinkerFile, ReadLinkerFileError> {
    let result = LinkerFile {
        symbols: Option::None,
        memory: Option::None,
        segments: Option::None,
        features: Option::None,
    };
    return Result::Err(ReadLinkerFileError::Invalid());
}

type Res<T, U> = IResult<T, U, VerboseError<T>>;

// fn memory_statement(input: &str) -> Res<&str, String> {
//     return context("memory statement", file_arg)(input).map(|(next_input, res)| (next_input, res));
// }

// fn file_arg(input: &str) -> Res<&str, String> {
//     return context(
//         "file arg",
//          tuple((
//             tag("file"),
//             multispace0
//         )))(input)
//         .map(|(next_input, res)| (next_input, res));
// }

fn hex_number_u32(input: &[u8]) -> Res<&[u8], u32> {
    return context(
        "hex number u32",
         tuple((
            tag("$"),
            hex_u32
        )))(input)
        .map(|(next_input, res)| (next_input, res.1));
}

// fn start_arg(input: &str) -> Res<&str, u32> {
//     return context(
//         "start arg",
//          tuple((
//             tag("start"),
//             multispace0,
//             tag("="),
//             multispace0,
//             hex_number_u32
//         )))(input)
//         .map(|(next_input, res)| (next_input, res));
// }

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{
        error::{ErrorKind, VerboseError, VerboseErrorKind},
        Err as NomErr,
    };

    #[test]
    fn test_memory_statement() {
        // assert_eq!(
        //     memory_statement("file = \"\", start = $0002, size = $001A, type = rw, define = yes;"),
        //     Ok((
        //         "",
        //         String::from("aaa") // MemorySegment {
        //                             //     file: Option::None,
        //                             //     start: Option::None,
        //                             //     size: Option::None,
        //                             //     fill: Option::None,
        //                             //     define: Option::None,
        //                             // }
        //     ))
        // );
    }

    #[test]
    fn test_hex_number_u32() {
        assert_eq!(hex_number_u32("$0123".as_bytes()), Ok(("".as_bytes(), 0x0123)));
        assert_eq!(hex_number_u32("$abcd".as_bytes()), Ok(("".as_bytes(), 0xabcd)));
        assert_eq!(hex_number_u32("$gggg".as_bytes()),Err(NomErr::Error(VerboseError {
            errors: vec![
                ("gggg".as_bytes(), VerboseErrorKind::Nom(ErrorKind::IsA)),
                ("$gggg".as_bytes(), VerboseErrorKind::Context("hex number u32")),
            ]
        })));
    }
}
