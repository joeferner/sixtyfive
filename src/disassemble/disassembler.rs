use crate::code::{Code, Instruction};

use super::DisassembleError;

pub struct Disassembler {
    pub code: Code,
}

impl Disassembler {
    pub fn new(data: Vec<u8>) -> Disassembler {
        return Disassembler {
            code: Code::new(data),
        };
    }

    pub fn disassemble<F: Fn(usize) -> usize>(
        &mut self,
        addr: usize,
        name: String,
        addr_map_fn: F,
    ) -> Result<(), DisassembleError> {
        let mut addr = addr_map_fn(addr);
        self.code.set_label(addr, name.as_str());
        println!("{} -> 0x{:02x}", name, addr);
        println!("{}", self.code.to_string(addr)?);

        loop {
            addr += match self.code.get_u8(addr)? {
                0x48 => self
                    .code
                    .replace_with_instr(addr, 0, |_| Result::Ok(Instruction::PHA))?,
                0xa5 => self.code.replace_with_instr(addr, 1, |args| {
                    Result::Ok(Instruction::LDA_ZP(args[0].to_u8()?))
                })?,
                _ => break, // TODO i => return Result::Err(DisassembleError::UnhandledInstruction(i)),
            }
        }

        return Result::Ok(());
    }
}
