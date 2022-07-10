use itertools::Itertools;
use std::{collections::HashMap, fmt, io::Write, mem};

use crate::disassemble::DisassembleError;

use super::instruction::Instruction;
use super::variable::Variable;

#[derive(Debug)]
pub enum AsmCode {
    DataHexU8(u8),
    DataHexU16(u16),
    DataU8(u8),
    DataBinaryU8(u8),
    DataString(String),
    DataSeq(Vec<AsmCode>),
    Instruction(Instruction),
    Used,
}

impl fmt::Display for AsmCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut addr_to_variable = HashMap::new();
        return write!(f, "{}", self.to_write_string(&mut addr_to_variable));
    }
}

impl AsmCode {
    pub fn is_eq_u8(&self, arg: u8) -> bool {
        if let AsmCode::DataHexU8(v) = self {
            return *v == arg;
        }
        return false;
    }

    pub fn to_u8(&self) -> Result<u8, DisassembleError> {
        return match self {
            AsmCode::DataHexU8(v) => Result::Ok(*v),
            AsmCode::DataU8(v) => Result::Ok(*v),
            AsmCode::DataBinaryU8(v) => Result::Ok(*v),
            _ => Result::Err(DisassembleError::ParseError(format!(
                "unexpected asm code \"{:?}\" -> \"{}\"",
                self, self
            ))),
        };
    }

    pub fn to_write_string(&self, addr_to_variable: &mut HashMap<u16, Variable>) -> String {
        return match self {
            AsmCode::DataHexU8(v) => {
                format!(".byte ${:02X?}", v)
            }
            AsmCode::DataHexU16(v) => {
                format!(".byte ${:04X?}", v)
            }
            AsmCode::DataU8(v) => {
                format!(".byte {}", v)
            }
            AsmCode::DataBinaryU8(v) => {
                format!(".byte {:#010b}", v)
            }
            AsmCode::DataString(str) => {
                format!(".byte \"{}\"", str)
            }
            AsmCode::DataSeq(v) => {
                return format!(
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
            AsmCode::Instruction(instr) => {
                format!("    {}", instr.to_write_string(addr_to_variable))
            }
            AsmCode::Used => format!(""),
        };
    }
}

pub struct Statement {
    pub asm_code: AsmCode,
    pub comment: Option<String>,
    pub segment: Option<String>,
    pub label: Option<String>,
}

pub struct Code {
    stmts: Vec<Statement>,
    addr_to_variable: HashMap<u16, Variable>,
}

impl Code {
    pub fn new(data: Vec<u8>) -> Code {
        let mut stmts = Vec::new();
        for value in data {
            stmts.push(Statement {
                asm_code: AsmCode::DataHexU8(value),
                comment: Option::None,
                segment: Option::None,
                label: Option::None,
            });
        }

        return Code {
            stmts,
            addr_to_variable: HashMap::new(),
        };
    }

    pub fn set_variable(&mut self, addr: u16, variable: Variable) {
        self.addr_to_variable.insert(addr, variable);
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
                segment: Option::None,
                label: Option::None,
            },
        ));
    }

    pub fn get_u8(&self, offset: usize) -> Result<u8, DisassembleError> {
        return self.stmts[offset].asm_code.to_u8();
    }

    pub fn get_i8(&self, offset: usize) -> Result<i8, DisassembleError> {
        return Result::Ok(self.get_u8(offset)? as i8);
    }

    pub fn set(&mut self, offset: usize, stmt: Statement) -> Result<(), DisassembleError> {
        self.stmts[offset] = stmt;
        return Result::Ok(());
    }

    pub fn replace(
        &mut self,
        range: std::ops::Range<usize>,
        new_code: AsmCode,
    ) -> Result<(), DisassembleError> {
        for i in range.clone() {
            self.stmts[i].asm_code = AsmCode::Used;
        }
        self.stmts[range.start].asm_code = new_code;
        return Result::Ok(());
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

    pub fn replace_with_instr<F: FnMut(Vec<AsmCode>) -> Result<Instruction, DisassembleError>>(
        &mut self,
        offset: usize,
        args_len: usize,
        mut instr_fn: F,
    ) -> Result<usize, DisassembleError> {
        let mut args = Vec::new();
        for i in 0..args_len {
            args.push(self.take(offset + i + 1)?.asm_code);
        }
        let instr = instr_fn(args)?;
        self.replace(offset..offset + args_len + 1, AsmCode::Instruction(instr))?;
        return Result::Ok(args_len + 1);
    }

    pub fn set_comment(&mut self, offset: usize, comment: &str) {
        self.stmts[offset].comment = Option::Some(comment.to_string());
    }

    pub fn set_segment(&mut self, offset: usize, segment: &str) {
        self.stmts[offset].segment = Option::Some(segment.to_string());
    }

    pub fn set_label(&mut self, offset: usize, label: &str) {
        self.stmts[offset].label = Option::Some(label.to_string());
    }

    pub fn write(&self, mut out: Box<dyn Write>) -> Result<(), DisassembleError> {
        let mut addr_to_variable = self.addr_to_variable.clone();

        for c in &self.stmts {
            c.asm_code.to_write_string(&mut addr_to_variable);
        }

        for v_addr in addr_to_variable.keys().sorted() {
            if let Option::Some(v) = addr_to_variable.get(v_addr) {
                writeln!(out, ".define {:<25} = {}", v.name, v.value)?;
            }
        }

        for c in &self.stmts {
            if let AsmCode::Used = c.asm_code {
                continue;
            }
            if let Option::Some(segment) = &c.segment {
                writeln!(
                    out,
                    "\n; -------------------------- {} -----------------------\n.segment \"{}\"",
                    segment, segment
                )?;
            }
            if let Option::Some(label) = &c.label {
                writeln!(out, "{}:", label)?;
            }
            let asm = c.asm_code.to_write_string(&mut addr_to_variable);
            writeln!(out, "{}", Code::with_comment(asm, &c.comment))?;
        }
        return Result::Ok(());
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

    pub fn is_instruction(&self, offset: usize) -> bool {
        if let AsmCode::Instruction(_) = self.stmts[offset].asm_code {
            return true;
        }
        return false;
    }
}
