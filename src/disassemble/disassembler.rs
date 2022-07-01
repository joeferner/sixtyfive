use crate::code::{AsmCode, Code, Instruction};

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

    pub fn disassemble<F1: Fn(usize) -> usize, F2: Fn(usize) -> usize>(
        &mut self,
        addr: usize,
        name: &str,
        label_prefix: String,
        addr_map_fn: F1,
        addr_rev_map_fn: F2,
    ) -> Result<(), DisassembleError> {
        let mut addr = addr_map_fn(addr);
        self.code
            .set_label(addr, format!("{}_{}", label_prefix, name).as_str());
        println!("{} -> 0x{:02x}", name, addr);
        println!("{}", self.code.to_string(addr)?);

        loop {
            let op = self.code.get_u8(addr)?;
            match op {
                // JSR ABS
                0x20 => {
                    let l = self.code.get_u8(addr + 1)? as u16;
                    let h = self.code.get_u8(addr + 2)? as u16;
                    let jsr_addr = (h << 8) | l;
                    let label = format!("{}_{:04x}", label_prefix, jsr_addr);
                    self.code.replace_with_instr(addr, 2, |_| {
                        Result::Ok(Instruction::JSR_ABS(label.clone()))
                    })?;
                    addr = addr_map_fn(jsr_addr as usize);
                    self.code.set_label(addr, label.as_str());
                }

                // BNE REL
                0xd0 => {
                    let rel = self.code.get_i8(addr + 1)?;
                    let addr16 = addr_rev_map_fn(addr) as u16;
                    let bne_addr = addr16.wrapping_add(rel as u16);
                    println!(
                        "------- 0x{:04x} + 0x{:02x} = 0x{:04x} (0x{:04x})",
                        addr16,
                        rel,
                        bne_addr,
                        0 // addr_map_fn(bne_addr as usize)
                    );
                    let label = format!("{}_{:04x}", label_prefix, bne_addr);
                    self.code.replace_with_instr(addr, 1, |_| {
                        Result::Ok(Instruction::BNE_REL(label.clone()))
                    })?;
                    break;
                }

                // Other
                _ => {
                    addr += match op {
                        0x29 => self.code.replace_with_instr(addr, 1, |args| {
                            Result::Ok(Instruction::AND_IMM(args[0].to_u8()?))
                        })?,
                        0x48 => self
                            .code
                            .replace_with_instr(addr, 0, |_| Result::Ok(Instruction::PHA))?,
                        0x85 => self.code.replace_with_instr(addr, 1, |args| {
                            Result::Ok(Instruction::STA_ZP(args[0].to_u8()?))
                        })?,
                        0x8d => self.code.replace_with_instr(addr, 2, |args| {
                            Result::Ok(Instruction::STA_ABS(to_u16(&args[0], &args[1])?))
                        })?,
                        0xa0 => self.code.replace_with_instr(addr, 1, |args| {
                            Result::Ok(Instruction::LDY_IMM(args[0].to_u8()?))
                        })?,
                        0xa5 => self.code.replace_with_instr(addr, 1, |args| {
                            Result::Ok(Instruction::LDA_ZP(args[0].to_u8()?))
                        })?,
                        0xa9 => self.code.replace_with_instr(addr, 1, |args| {
                            Result::Ok(Instruction::LDA_IMM(args[0].to_u8()?))
                        })?,
                        0xb1 => self.code.replace_with_instr(addr, 1, |args| {
                            Result::Ok(Instruction::LDA_IND_Y(args[0].to_u8()?))
                        })?,
                        0xae => self.code.replace_with_instr(addr, 2, |args| {
                            Result::Ok(Instruction::LDX_ABS(to_u16(&args[0], &args[1])?))
                        })?,
                        _ => {
                            // TODO i => return Result::Err(DisassembleError::UnhandledInstruction(i))
                            println!("unhandled: 0x{:02x}", op);
                            break;
                        }
                    };
                }
            }
        }

        return Result::Ok(());
    }
}

fn to_u16(arg0: &AsmCode, arg1: &AsmCode) -> Result<u16, DisassembleError> {
    let l = arg0.to_u8()? as u16;
    let h = arg1.to_u8()? as u16;
    return Result::Ok((h << 8) | l);
}
