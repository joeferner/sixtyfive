use std::{collections::HashMap, fmt, iter::Map, path::PathBuf};

use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take, take_till},
    character::complete::{alpha1, alphanumeric1, char, hex_digit1, multispace0, one_of, space0},
    combinator::opt,
    error::{context, ErrorKind, FromExternalError, ParseError, VerboseError},
    multi::{count, many0, many1, many_m_n},
    number::complete::hex_u32,
    sequence::{preceded, separated_pair, terminated, tuple},
    AsChar, Err as NomErr, IResult, InputTakeAtPosition,
};
use nom_supreme::multi::parse_separated_terminated;
use nom_supreme::ParserExt;

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
    segmentType: Option<String>,
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

pub fn read_linker_file<'a>(linker_file: String) -> Result<LinkerFile, ReadLinkerFileError> {
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

fn parse_bool(opt_str: Option<&str>) -> Result<Option<bool>, NomErr<VerboseError<&str>>> {
    if let Option::Some(str) = opt_str {
        if str == "yes" {
            return Result::Ok(Option::Some(true));
        } else if str == "no" {
            return Result::Ok(Option::Some(false));
        } else {
            return Result::Err(NomErr::Error(VerboseError::from_error_kind(
                str,
                ErrorKind::Tag,
            )));
        }
    }
    return Result::Ok(Option::None);
}

fn parse_u32(opt_str: Option<&str>) -> Result<Option<u32>, NomErr<VerboseError<&str>>> {
    if let Option::Some(str) = opt_str {
        if str.starts_with("$") {
            return u32::from_str_radix(&str[1..], 16)
                .map_err(|err| {
                    NomErr::Error(VerboseError::from_external_error(
                        str,
                        ErrorKind::HexDigit,
                        err,
                    ))
                })
                .map(|v| Option::Some(v));
        } else {
            return Result::Err(NomErr::Error(VerboseError::from_error_kind(
                str,
                ErrorKind::HexDigit,
            )));
        }
    }
    return Result::Ok(Option::None);
}

#[rustfmt::skip]
fn memory_segment(input: &str) -> Res<&str, MemorySegment> {
    return context(
        "memory segment",
        parse_separated_terminated(
            arg,
            char(',').delimited_by(space0),
            char(';').preceded_by(space0),
            HashMap::new,
            |mut map, arg| {
                map.insert(arg.0, arg.1);
                map
            },
        )
    )(input).and_then(|(next_input, mut res)| {
        return Result::Ok((
            next_input,
            MemorySegment {
                file: res.remove("file").and_then(|v| Option::Some(String::from(v))),
                define: parse_bool(res.remove("define"))?,
                fill: parse_bool(res.remove("fill"))?,
                size: parse_u32(res.remove("size"))?,
                start: parse_u32(res.remove("start"))?,
                segmentType: res.remove("type").and_then(|v| Option::Some(String::from(v)))
            }
        ));
    });
}

pub fn not_arg_end<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    return input.split_at_position1_complete(
        |item| {
            let c = item.as_char();
            return c == ',' || c == ';';
        },
        ErrorKind::TakeUntil,
    );
}

#[rustfmt::skip]
fn arg(input: &str) -> Res<&str, (&str, &str)> {
    return context(
        "arg",
         tuple((
            alpha1,
            multispace0,
            char('='),
            multispace0,
            not_arg_end
        )))(input)
        .map(|(next_input, res)| (next_input, (res.0, res.4)));
}

#[rustfmt::skip]
fn hex_number_u32(input: &str) -> Res<&str, u32> {
    return context(
        "hex number",
         tuple((
            tag("$"),
            hex_digit1
        )))(input)
        .map(|(next_input, res)| (
            next_input,
            u32::from_str_radix(res.1,16).unwrap()
        ));
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
    fn test_memory_segment() {
        assert_eq!(
            memory_segment("file = \"\", start = $0002, size = $001A, type = rw, define = yes;"),
            Ok((
                "",
                MemorySegment {
                    file: Option::Some(String::from("\"\"")),
                    start: Option::Some(0x0002),
                    size: Option::Some(0x1a),
                    fill: Option::None,
                    define: Option::Some(true),
                    segmentType: Option::Some(String::from("rw"))
                }
            ))
        );
    }

    #[test]
    fn test_arg() {
        assert_eq!(arg("file = \"\";"), Ok((";", ("file", "\"\""))));
        assert_eq!(arg("file=%O,"), Ok((",", ("file", "%O"))));
        assert_eq!(
            arg("file=;"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    (";", VerboseErrorKind::Nom(ErrorKind::TakeUntil)),
                    ("file=;", VerboseErrorKind::Context("arg")),
                ]
            }))
        );
    }

    #[test]
    fn test_hex_number_u32() {
        assert_eq!(hex_number_u32("$0123"), Ok(("", 0x0123)));
        assert_eq!(hex_number_u32("$abcd"), Ok(("", 0xabcd)));
        assert_eq!(
            hex_number_u32("$gggg"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("gggg", VerboseErrorKind::Nom(ErrorKind::HexDigit)),
                    ("$gggg", VerboseErrorKind::Context("hex number")),
                ]
            }))
        );
    }
}
