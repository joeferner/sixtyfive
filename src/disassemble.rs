use std::{
    fmt,
    fs::File,
    io::{BufReader, Read, Write},
    path::PathBuf,
};

use crate::linker_file::{read_linker_file, ReadLinkerFileError};

#[derive(Debug)]
pub struct DisassembleOptions {
    pub linker_file: String,
    pub in_file: Option<PathBuf>,
    pub out_file: Option<PathBuf>,
}

#[derive(Debug)]
pub enum DisassembleError {
    MissingFile(PathBuf),
    ReadLinkerFileError(ReadLinkerFileError),
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
            DisassembleError::ReadLinkerFileError(err) => write!(f, "{}", err),
            DisassembleError::IoError(err) => write!(f, "io error: {}", err),
            DisassembleError::ParseError(err) => write!(f, "parse error: {}", err),
        }
    }
}

#[derive(Debug)]
enum AsmCode {
    DataHexU8(u8, Option<String>),
    DataU8(u8, Option<String>),
    DataBinaryU8(u8, Option<String>),
    DataString(String, Option<String>),
    DataSeq(Vec<AsmCode>, Option<String>),
    Used,
}

fn with_comment(first: String, comment: &Option<String>) -> String {
    if let Option::Some(comment) = comment {
        if comment.contains("\n") {
            return format!("\n; {}\n{:<25}", comment.replace("\n", "\n; "), first);
        } else {
            return format!("{:<25} ; {}", first, comment);
        }
    } else {
        return first;
    }
}

impl fmt::Display for AsmCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsmCode::DataHexU8(v, comment) => {
                write!(f, "{}", with_comment(format!(".byte ${:02X?}", v), comment))
            }
            AsmCode::DataU8(v, comment) => {
                write!(f, "{}", with_comment(format!(".byte {}", v), comment))
            }
            AsmCode::DataBinaryU8(v, comment) => {
                write!(f, "{}", with_comment(format!(".byte {:#010b}", v), comment))
            }
            AsmCode::DataString(str, comment) => {
                write!(f, "{}", with_comment(format!(".byte \"{}\"", str), comment))
            }
            AsmCode::DataSeq(v, comment) => {
                return write!(
                    f,
                    "{}",
                    with_comment(
                        format!(
                            ".byte {}",
                            v.iter()
                                .map(|i| match i {
                                    AsmCode::DataHexU8(v, _) => format!("${:02X?}", v),
                                    AsmCode::DataU8(v, _) => format!("{}", v),
                                    AsmCode::DataBinaryU8(v, _) => format!("{:#010b}", v),
                                    AsmCode::DataString(str, _) => format!("\"{}\"", str),
                                    _ => panic!("data sequence can only contain data elements"),
                                })
                                .collect::<Vec<String>>()
                                .join(", ")
                        ),
                        comment
                    )
                );
            }
            AsmCode::Used => write!(f, ""),
        }
    }
}

impl AsmCode {
    fn is_eq(&self, arg: u8) -> bool {
        if let AsmCode::DataHexU8(v, _) = self {
            return *v == arg;
        }
        return false;
    }

    fn to_u8(&self) -> Result<u8, DisassembleError> {
        return match self {
            AsmCode::DataHexU8(v, _) => Result::Ok(*v),
            AsmCode::DataU8(v, _) => Result::Ok(*v),
            AsmCode::DataBinaryU8(v, _) => Result::Ok(*v),
            _ => Result::Err(DisassembleError::ParseError(format!(
                "unexpected asm code {}",
                self
            ))),
        };
    }
}

pub fn disassemble(opts: DisassembleOptions) -> Result<(), DisassembleError> {
    let data = read_file_or_stdin(opts.in_file)?;

    let _linker_file = match read_linker_file(opts.linker_file) {
        Result::Err(err) => return Result::Err(DisassembleError::ReadLinkerFileError(err)),
        Result::Ok(res) => res,
    };

    let mut asm_code = Vec::new();
    for value in data {
        asm_code.push(AsmCode::DataHexU8(value, Option::None));
    }

    parse_nes_header(&mut asm_code)?;

    let mut out = open_out_file(opts.out_file)?;
    for c in asm_code {
        if let AsmCode::Used = c {
            continue;
        }
        writeln!(out, "{}", c)?;
    }

    return Result::Ok(());
}

fn parse_nes_header(asm_code: &mut Vec<AsmCode>) -> Result<(), DisassembleError> {
    if asm_code[0].is_eq(b'N')
        && asm_code[1].is_eq(b'E')
        && asm_code[2].is_eq(b'S')
        && asm_code[3].is_eq(0x1a)
    {
        asm_code[0] = AsmCode::DataSeq(
            vec![
                AsmCode::DataString("NES".to_string(), Option::None),
                AsmCode::DataHexU8(0x1a, Option::None),
            ],
            Option::None,
        );
        asm_code[1] = AsmCode::Used;
        asm_code[2] = AsmCode::Used;
        asm_code[3] = AsmCode::Used;
    } else {
        return Result::Err(DisassembleError::ParseError(
            "invalid nes header".to_string(),
        ));
    }

    asm_code[4] = AsmCode::DataU8(
        asm_code[4].to_u8()?,
        Option::Some("PRG ROM count".to_string()),
    );

    asm_code[5] = AsmCode::DataU8(
        asm_code[5].to_u8()?,
        Option::Some("CHR ROM count".to_string()),
    );

    asm_code[6] = AsmCode::DataBinaryU8(
        asm_code[6].to_u8()?,
        Option::Some(
            "Flags 6
      ++++------ Mapper Number D0..D3
      ||||        1: Yes
      ||||        0: No
      ||||+----- Hard-wired four-screen mode
      |||||       1: Present between Header and PRG-ROM data
      |||||       0: Not present
      |||||+---- 512-byte Trainer
      ||||||      1: Present
      ||||||      0: Not present
      ||||||+--- \"Battery\" and other non-volatile memory
      |||||||     1: Vertical
      |||||||     0: Horizontal or mapper-controlled
      |||||||+-- Hard-wired nametable mirroring type
      NNNNFTBM"
                .to_string(),
        ),
    );

    asm_code[7] = AsmCode::DataBinaryU8(
        asm_code[7].to_u8()?,
        Option::Some(
            "Flags 7
      ++++------ Mapper Number D4..D7
      ||||++---- NES 2.0 identifier
      ||||||      3: Extended Console Type
      ||||||      2: Nintendo Playchoice 10
      ||||||      1: Nintendo Vs. System
      ||||||      0: Nintendo Entertainment System/Family Computer
      ||||||++-- Console type
      NNNN10TT"
                .to_string(),
        ),
    );

    asm_code[8] = AsmCode::DataBinaryU8(
        asm_code[8].to_u8()?,
        Option::Some(
            "Mapper MSB/Submapper
      ++++------ Submapper number
      ||||++++-- Mapper number D8..D11
      SSSSNNNN"
                .to_string(),
        ),
    );

    asm_code[9] = AsmCode::DataBinaryU8(
        asm_code[9].to_u8()?,
        Option::Some(
            "PRG-ROM/CHR-ROM size MSB
      ++++------ CHR-ROM size MSB
      ||||++++-- PRG-ROM size MSB
      CCCCPPPP"
                .to_string(),
        ),
    );

    asm_code[10] = AsmCode::DataBinaryU8(
        asm_code[10].to_u8()?,
        Option::Some(
            "PRG-RAM/EEPROM size            
  If the shift count is zero, there is no PRG-(NV)RAM.
  If the shift count is non-zero, the actual size is
  \"64 << shift count\" bytes, i.e. 8192 bytes for a shift count of 7.
      ++++------ PRG-NVRAM/EEPROM (non-volatile) shift count
      ||||++++-- PRG-RAM (volatile) shift count
      ppppPPPP"
                .to_string(),
        ),
    );

    asm_code[11] = AsmCode::DataBinaryU8(
        asm_code[11].to_u8()?,
        Option::Some(
            "CHR-RAM size
  If the shift count is zero, there is no CHR-(NV)RAM.
  If the shift count is non-zero, the actual size is
  \"64 << shift count\" bytes, i.e. 8192 bytes for a shift count of 7.
      ++++------ CHR-NVRAM size (non-volatile) shift count
      ||||++++-- CHR-RAM size (volatile) shift count
      ccccCCCC"
                .to_string(),
        ),
    );

    asm_code[12] = AsmCode::DataBinaryU8(
        asm_code[12].to_u8()?,
        Option::Some(
            "CPU/PPU Timing
            ++- CPU/PPU timing mode
            ||   0: RP2C02 (\"NTSC NES\")
            ||   1: RP2C07 (\"Licensed PAL NES\")
            ||   2: Multiple-region
            ||   3: UMC 6527P (\"Dendy\")
      ......VV"
                .to_string(),
        ),
    );

    asm_code[13] = AsmCode::DataBinaryU8(
        asm_code[13].to_u8()?,
        Option::Some(
            "When Byte 7 AND 3 =1: Vs. System Type
      ++++------ Vs. Hardware Type
      ||||++++-- Vs. PPU Type
      MMMMPPPP"
                .to_string(),
        ),
    );

    asm_code[14] = AsmCode::DataBinaryU8(
        asm_code[14].to_u8()?,
        Option::Some(
            "Miscellaneous ROMs
            ++- Number of miscellaneous ROMs present
      ......RR"
                .to_string(),
        ),
    );

    asm_code[15] = AsmCode::DataBinaryU8(
        asm_code[15].to_u8()?,
        Option::Some(
            "Default Expansion Device
        ++++++- Default Expansion Device
      ..DDDDDD"
                .to_string(),
        ),
    );

    return Result::Ok(());
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
