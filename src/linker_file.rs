use std::{collections::HashMap, fmt, path::PathBuf};

use nom::{
    character::complete::{alpha1, alphanumeric1, char, multispace0, space0},
    error::{context, ErrorKind, ParseError as NomParseError, VerboseError},
    multi::many0,
    sequence::tuple,
    AsChar, Err as NomErr, IResult, InputTakeAtPosition, Needed,
};
use nom_supreme::multi::parse_separated_terminated;
use nom_supreme::ParserExt;

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
    let mut result = VerboseError::from_error_kind("".to_string(), ErrorKind::Alpha);
    result.errors.clear();
    for err_item in err.errors {
        result.errors.push((err_item.0.to_string(), err_item.1));
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

#[derive(Debug)]
pub struct Item {
    arguments: HashMap<String, String>,
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.arguments == other.arguments
    }
}

#[derive(Debug)]
pub struct Category {
    items: HashMap<String, Item>,
}

impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

#[derive(Debug)]
pub struct LinkerFile {
    categories: HashMap<String, Category>,
}

impl PartialEq for LinkerFile {
    fn eq(&self, other: &Self) -> bool {
        self.categories == other.categories
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
        many0(category)
    )(input).and_then(|(next_input, res)| {
        let mut categories = HashMap::new();
        for category in res {
            categories.insert(category.0, category.1);
        }
        return Result::Ok((
            next_input,
            LinkerFile {
                categories
            }
        ));
    });
}

#[rustfmt::skip]
fn category(input: &str) -> Res<&str, (String, Category)> {
    return context(
        "category",
        tuple((
          alphanumeric1,
          multispace0,
          char('{'),
          multispace0,
          parse_separated_terminated(
            item,
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
            (res.0.to_string(), Category { items: res.4 })
        ));
    });
}

#[rustfmt::skip]
fn item(input: &str) -> Res<&str, (String, Item)> {
    return context(
        "item",
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
                    map.insert(arg.0.to_string(), arg.1.to_string());
                    map
                },
            )
        ))        
    )(input).and_then(|(next_input, res)| {
        return Result::Ok((
            next_input,
            (
                res.0.to_string(),
                Item {
                    arguments: HashMap::from(res.4)
                }
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
        assert_eq!(
            read_linker_from_string("MEMORY { ZP: file = \"\", start = $0002; }"),
            Ok((
                "",
                LinkerFile {
                    categories: HashMap::from([(
                        "MEMORY".to_string(),
                        Category {
                            items: HashMap::from([(
                                "ZP".to_string(),
                                Item {
                                    arguments: HashMap::from([
                                        ("file".to_string(), "\"\"".to_string()),
                                        ("start".to_string(), "$0002".to_string())
                                    ])
                                }
                            )])
                        }
                    )])
                }
            ))
        );
    }

    #[test]
    fn test_item() {
        assert_eq!(
            item("ZP: file = \"\", start = $0002;"),
            Ok((
                "",
                (
                    String::from("ZP"),
                    Item {
                        arguments: HashMap::from([
                            ("file".to_string(), "\"\"".to_string()),
                            ("start".to_string(), "$0002".to_string())
                        ])
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
