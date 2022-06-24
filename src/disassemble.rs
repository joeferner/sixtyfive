use std::{
    fmt,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use crate::linker_file::{read_linker_file, ReadLinkerFileError};

#[derive(Debug)]
pub struct DisassembleOptions {
    pub linker_file: String,
    pub in_file: Option<PathBuf>,
    pub out: Option<PathBuf>,
}

#[derive(Debug)]
pub enum DisassembleError {
    MissingFile(PathBuf),
    ReadLinkerFileError(ReadLinkerFileError),
    IoError(std::io::Error),
}

impl From<std::io::Error> for DisassembleError {
    fn from(err: std::io::Error) -> Self {
        return DisassembleError::IoError(err);
    }
}

impl fmt::Display for DisassembleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DisassembleError::MissingFile(path) => write!(f, "Missing file {}", path.display()),
            DisassembleError::ReadLinkerFileError(err) => write!(f, "{}", err),
            DisassembleError::IoError(err) => write!(f, "io error: {}", err),
        }
    }
}

pub fn disassemble(opts: DisassembleOptions) -> Result<(), DisassembleError> {
    let data = read_file_or_stdin(opts.in_file)?;

    let _linker_file = match read_linker_file(opts.linker_file) {
        Result::Err(err) => return Result::Err(DisassembleError::ReadLinkerFileError(err)),
        Result::Ok(res) => res,
    };

    for value in data {
        print!("{}", value);
    }
    println!("");

    return Result::Ok(());
}

fn read_file_or_stdin(f: Option<PathBuf>) -> Result<Vec<u8>, DisassembleError> {
    if let Option::Some(in_file) = f {
        if !in_file.as_path().exists() {
            return Result::Err(DisassembleError::MissingFile(in_file));
        }

        let f = File::open(in_file.as_path())?;
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        return Result::Ok(buffer);
    }

    let mut buffer = Vec::new();
    std::io::stdin().read_to_end(&mut buffer)?;
    return Result::Ok(buffer);
}
