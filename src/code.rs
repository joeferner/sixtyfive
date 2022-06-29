use std::{fmt, io::Write, mem};

use crate::disassemble::DisassembleError;

#[derive(Debug)]
pub enum AsmCode {
    DataHexU8(u8),
    DataHexU16(u16),
    DataU8(u8),
    DataBinaryU8(u8),
    DataString(String),
    DataSeq(Vec<AsmCode>),
    Used,
}

impl fmt::Display for AsmCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsmCode::DataHexU8(v) => {
                write!(f, ".byte ${:02X?}", v)
            }
            AsmCode::DataHexU16(v) => {
                write!(f, ".byte ${:04X?}", v)
            }
            AsmCode::DataU8(v) => {
                write!(f, ".byte {}", v)
            }
            AsmCode::DataBinaryU8(v) => {
                write!(f, ".byte {:#010b}", v)
            }
            AsmCode::DataString(str) => {
                write!(f, ".byte \"{}\"", str)
            }
            AsmCode::DataSeq(v) => {
                return write!(
                    f,
                     ".byte {}",
                            v.iter()
                                .map(|i| match i {
                                    AsmCode::DataHexU8(v) => format!("${:02X?}", v),
                                    AsmCode::DataU8(v) => format!("{}", v),
                                    AsmCode::DataBinaryU8(v) => format!("{:#010b}", v),
                                    AsmCode::DataString(str) => format!("\"{}\"", str),
                                    v => panic!(
                                        "data sequence can only contain data elements. found: {}",
                                        v
                                    ),
                                })
                                .collect::<Vec<String>>()
                                .join(", ")
                );
            }
            AsmCode::Used => write!(f, ""),
        }
    }
}

impl AsmCode {
    pub fn is_eq_u8(&self, arg: u8) -> bool {
        if let AsmCode::DataHexU8(v) = self {
            return *v == arg;
        }
        return false;
    }

    fn to_u8(&self) -> Result<u8, DisassembleError> {
        return match self {
            AsmCode::DataHexU8(v) => Result::Ok(*v),
            AsmCode::DataU8(v) => Result::Ok(*v),
            AsmCode::DataBinaryU8(v) => Result::Ok(*v),
            _ => Result::Err(DisassembleError::ParseError(format!(
                "unexpected asm code {}",
                self
            ))),
        };
    }
}

pub struct Statement {
    pub asm_code: AsmCode,
    pub comment: Option<String>,
}

pub struct Code {
    stmts: Vec<Statement>,
}

impl Code {
    pub fn new(data: Vec<u8>) -> Code {
        let mut stmts = Vec::new();
        for value in data {
            stmts.push(Statement {
                asm_code: AsmCode::DataHexU8(value),
                comment: Option::None,
            });
        }

        return Code { stmts };
    }

    pub fn is_eq_u8(&self, offset: usize, d: u8) -> bool {
        return self.stmts[offset].asm_code.is_eq_u8(d);
    }

    pub fn take(&mut self, offset: usize) -> Result<Statement, DisassembleError> {
        return Result::Ok(mem::replace(
            &mut self.stmts[offset],
            Statement {
                asm_code: AsmCode::Used,
                comment: Option::None,
            },
        ));
    }

    pub fn set(&mut self, offset: usize, stmt: Statement) -> Result<(), DisassembleError> {
        self.stmts[offset] = stmt;
        return Result::Ok(());
    }

    pub fn replace(&mut self, range: std::ops::Range<usize>, new_code: AsmCode) {
        for i in range.clone() {
            self.stmts[i].asm_code = AsmCode::Used;
        }
        self.stmts[range.start].asm_code = new_code;
    }

    pub fn replace_with_u8(&mut self, offset: usize) -> Result<u8, DisassembleError> {
        let result = self.stmts[offset].asm_code.to_u8()?;
        self.stmts[offset].asm_code = AsmCode::DataU8(result);
        return Result::Ok(result);
    }

    pub fn replace_with_binary_u8(&mut self, offset: usize) -> Result<u8, DisassembleError> {
        let result = self.stmts[offset].asm_code.to_u8()?;
        self.stmts[offset].asm_code = AsmCode::DataBinaryU8(result);
        return Result::Ok(result);
    }

    pub fn set_comment(&mut self, offset: usize, comment: &str) {
        self.stmts[offset].comment = Option::Some(comment.to_string());
    }

    pub fn write(&self, mut out: Box<dyn Write>) -> Result<(), DisassembleError> {
        for c in &self.stmts {
            if let AsmCode::Used = c.asm_code {
                continue;
            }
            let asm = format!("{}", c.asm_code);
            writeln!(out, "{}", with_comment(asm, &c.comment))?;
        }
        return Result::Ok(());
    }
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
