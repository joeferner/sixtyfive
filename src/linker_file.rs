use std::{fmt, path::PathBuf};

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

pub struct LinkerFile {}

impl LinkerFile {
    pub fn new() -> Self {
        return Self {};
    }
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
    let result = LinkerFile::new();
    return Result::Err(ReadLinkerFileError::Invalid());
}
