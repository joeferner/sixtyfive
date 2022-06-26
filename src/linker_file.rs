use std::{collections::HashMap, fmt, path::PathBuf};

use nom::{
    character::complete::{alpha1, alphanumeric1, char, multispace0, space0},
    error::{context, ErrorKind, FromExternalError, ParseError as NomParseError, VerboseError},
    sequence::tuple,
    AsChar, Err as NomErr, IResult, InputTakeAtPosition, Needed,
};
use nom_supreme::ParserExt;
use nom_supreme::{multi::parse_separated_terminated, tag::complete::tag};

#[derive(Debug)]
pub enum ReadLinkerFileError {
    MissingFile(PathBuf),
    IoError(std::io::Error),
    ParseError(VerboseError<String>),
    ParseFailure(VerboseError<String>),
    ParseIncomplete(Needed),
}

impl From<std::io::Error> for ReadLinkerFileError {
    fn from(err: std::io::Error) -> Self {
        return ReadLinkerFileError::IoError(err);
    }
}

fn verbose_error_to_string(err: VerboseError<&str>) -> VerboseError<String> {
    let mut result = VerboseError::from_error_kind(String::from(""), ErrorKind::Alpha);
    result.errors.clear();
    for err_item in err.errors {
        result.errors.push((String::from(err_item.0), err_item.1));
    }
    return result;
}

impl From<nom::Err<VerboseError<&str>>> for ReadLinkerFileError {
    fn from(err: NomErr<VerboseError<&str>>) -> Self {
        return match err {
            nom::Err::Error(err) => ReadLinkerFileError::ParseError(verbose_error_to_string(err)),
            nom::Err::Failure(err) => {
                ReadLinkerFileError::ParseFailure(verbose_error_to_string(err))
            }
            nom::Err::Incomplete(needed) => ReadLinkerFileError::ParseIncomplete(needed),
        };
    }
}

impl fmt::Display for ReadLinkerFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReadLinkerFileError::MissingFile(path) => {
                write!(f, "Missing linker file {}", path.display())
            }
            ReadLinkerFileError::IoError(err) => {
                write!(f, "Read linker io error {}", err)
            }
            ReadLinkerFileError::ParseError(err) => {
                write!(f, "Parse error {}", err)
            }
            ReadLinkerFileError::ParseFailure(err) => {
                write!(f, "Parse failure {}", err)
            }
            ReadLinkerFileError::ParseIncomplete(needed) => {
                write!(f, "Parse incomplete {:?}", needed)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Symbol {
    symbol_type: Option<String>,
    value: Option<u32>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MemorySegment {
    file: Option<String>,
    start: Option<u32>,
    size: Option<u32>,
    fill: Option<bool>,
    define: Option<bool>,
    segment_type: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Segment {
    load: Option<String>,
    segment_type: Option<String>,
    run: Option<String>,
    define: Option<bool>,
    optional: Option<bool>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Feature {
    feature_type: Option<String>,
    label: Option<String>,
    count: Option<String>,
    segment: Option<String>,
}

#[derive(Debug)]
pub struct LinkerFile {
    symbols: Option<HashMap<String, Symbol>>,
    memory: Option<HashMap<String, MemorySegment>>,
    segments: Option<HashMap<String, Segment>>,
    features: Option<HashMap<String, HashMap<String, Feature>>>,
}

impl PartialEq for LinkerFile {
    fn eq(&self, other: &Self) -> bool {
        self.symbols == other.symbols
            && self.memory == other.memory
            && self.segments == other.segments
            && self.features == other.features
    }
}

pub fn read_linker_file(linker_file: String) -> Result<LinkerFile, ReadLinkerFileError> {
    if linker_file == "nes" {
        let str = include_str!("linker/nes.cfg");
        return read_linker_from_string(str)
            .map_err(|err| ReadLinkerFileError::from(err))
            .map(|res| res.1);
    }

    let file = PathBuf::from(linker_file);
    if !file.as_path().exists() {
        return Result::Err(ReadLinkerFileError::MissingFile(file));
    }

    let str = std::fs::read_to_string(file.as_path())?;
    return read_linker_from_string(str.as_str())
        .map_err(|err| ReadLinkerFileError::from(err))
        .map(|res| res.1);
}

type Res<T, U> = IResult<T, U, VerboseError<T>>;

#[rustfmt::skip]
fn read_linker_from_string(input: &str) -> Res<&str, LinkerFile> {
    return context(
        "linker file",
        memory
    )(input).and_then(|(next_input, _res)| {
        return Result::Ok((
            next_input,
            LinkerFile {
                symbols: Option::None,
                memory: Option::None,
                segments: Option::None,
                features: Option::None,
            }
        ));
    });
}

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
fn memory(input: &str) -> Res<&str, HashMap<String, MemorySegment>> {
    return context(
        "memory",
        tuple((
          tag("MEMORY"),
          multispace0,
          char('{'),
          multispace0,
          parse_separated_terminated(
            memory_segment,
            char(';').delimited_by(space0),
            char('}').preceded_by(space0),
            HashMap::new,
            |mut map, arg| {
                map.insert(arg.0, arg.1);
                map
            },
          ),
        ))
    )(input).and_then(|(next_input, res)| {
        return Result::Ok((
            next_input,
            res.4
        ));
    });
}

#[rustfmt::skip]
fn memory_segment(input: &str) -> Res<&str, (String, MemorySegment)> {
    return context(
        "memory segment",
        tuple((
            alphanumeric1,
            multispace0,
            char(':'),
            multispace0,
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
        ))        
    )(input).and_then(|(next_input, res)| {
        let mut memory_segment_map = res.4;
        let memory_segment = MemorySegment {
            file: memory_segment_map.remove("file").and_then(|v| Option::Some(String::from(v))),
            define: parse_bool(memory_segment_map.remove("define"))?,
            fill: parse_bool(memory_segment_map.remove("fill"))?,
            size: parse_u32(memory_segment_map.remove("size"))?,
            start: parse_u32(memory_segment_map.remove("start"))?,
            segment_type: memory_segment_map.remove("type").and_then(|v| Option::Some(String::from(v)))
        };
        // TODO test res is empty
        
        return Result::Ok((
            next_input,
            (
                String::from(res.0),
                memory_segment
            )
        ));
    });
}

pub fn not_arg_end<T, E: NomParseError<T>>(input: T) -> IResult<T, T, E>
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

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{
        error::{ErrorKind, VerboseError, VerboseErrorKind},
        Err as NomErr,
    };

    #[test]
    fn test_read_linker_from_string() {
        let mut expected_memory = HashMap::new();
        expected_memory.insert(
            String::from("ZP"),
            MemorySegment {
                file: Option::Some(String::from("\"\"")),
                start: Option::Some(0x0002),
                size: Option::Some(0x1a),
                fill: Option::None,
                define: Option::Some(true),
                segment_type: Option::Some(String::from("rw")),
            },
        );
        assert_eq!(
            read_linker_from_string(
                "MEMORY { ZP: file = \"\", start = $0002, size = $001A, type = rw, define = yes; }"
            ),
            Ok((
                "",
                LinkerFile {
                    symbols: Option::None,
                    memory: Option::Some(expected_memory),
                    features: Option::None,
                    segments: Option::None,
                }
            ))
        );
    }

    #[test]
    fn test_memory_segment() {
        assert_eq!(
            memory_segment(
                "ZP: file = \"\", start = $0002, size = $001A, type = rw, define = yes;"
            ),
            Ok((
                "",
                (
                    String::from("ZP"),
                    MemorySegment {
                        file: Option::Some(String::from("\"\"")),
                        start: Option::Some(0x0002),
                        size: Option::Some(0x1a),
                        fill: Option::None,
                        define: Option::Some(true),
                        segment_type: Option::Some(String::from("rw"))
                    }
                )
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
}
