mod disassembler;
mod nes_disassembler;

use std::{
    fmt,
    fs::File,
    io::{BufReader, Read, Write},
    path::PathBuf,
};

use self::nes_disassembler::NesDisassembler;

#[derive(Debug)]
pub struct DisassembleOptions {
    pub in_file: Option<PathBuf>,
    pub out_file: Option<PathBuf>,
}

#[derive(Debug)]
pub enum DisassembleError {
    MissingFile(PathBuf),
    IoError(std::io::Error),
    ParseError(String),
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
            DisassembleError::IoError(err) => write!(f, "io error: {}", err),
            DisassembleError::ParseError(err) => write!(f, "parse error: {}", err),
        }
    }
}

pub fn disassemble(opts: DisassembleOptions) -> Result<(), DisassembleError> {
    let data = read_file_or_stdin(opts.in_file)?;
    let out = open_out_file(opts.out_file)?;

    if NesDisassembler::is_handled(&data) {
        return NesDisassembler::disassemble(data, out);
    } else {
        return Result::Err(DisassembleError::ParseError(
            "unhandled file format".to_string(),
        ));
    }
}

fn open_out_file(f: Option<PathBuf>) -> Result<Box<dyn Write>, DisassembleError> {
    if let Option::Some(out_file) = f {
        let f = File::create(out_file.as_path())?;
        return Result::Ok(Box::new(f) as Box<dyn Write>);
    }

    return Result::Ok(Box::new(std::io::stdout()) as Box<dyn Write>);
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
