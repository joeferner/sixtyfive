use std::{collections::HashMap, fmt};

use super::variable::{Variable, VariableValue};

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Instruction {
    ORA_ZP(u8),
    ASL_ZP(u8),
    PHP,
    ORA_IMM(u8),
    ASL,
    BPL_REL(i8, String),
    CLC,
    JSR_ABS(u16, String),
    BIT_ZP(u8),
    AND_ZP(u8),
    PLP,
    AND_IMM(u8),
    ROL,
    BMI_REL(i8, String),
    AND_ZP_X(u8),
    SEC,
    RTI,
    EOR_ZP(u8),
    LSR_ZP(u8),
    PHA,
    EOR_IMM(u8),
    LSR,
    JMP_ABS(u16, String),
    EOR_ABS(u16),
    RTS,
    ADC_ZP(u8),
    ROR_ZP(u8),
    PLA,
    ADC_IMM(u8),
    ROR,
    ADC_ABS(u16),
    SEI,
    ADC_ABS_X(u16),
    STY_ZP(u8),
    STA_ZP(u8),
    STX_ZP(u8),
    DEY,
    TXA,
    STY_ABS(u16),
    STA_ABS(u16),
    STX_ABS(u16),
    BCC_REL(i8, String),
    STA_IND_Y(u8),
    STY_ZP_X(u8),
    STA_ZP_X(u8),
    TYA,
    STA_ABS_Y(u16),
    TXS,
    STA_ABS_X(u16),
    LDY_IMM(u8),
    LDX_IMM(u8),
    LDY_ZP(u8),
    LDA_ZP(u8),
    LDX_ZP(u8),
    LDA_IMM(u8),
    TAX,
    TAY,
    LDY_ABS(u16),
    LDA_ABS(u16),
    LDX_ABS(u16),
    BCS_REL(i8, String),
    LDA_IND_Y(u8),
    LDY_ZP_X(u8),
    LDA_ZP_X(u8),
    LDA_ABS_Y(u16),
    LDY_ABS_X(u16),
    LDA_ABS_X(u16),
    LDX_ABS_Y(u16),
    CPY_IMM(u8),
    CPY_ZP(u8),
    CMP_ZP(u8),
    DEC_ZP(u8),
    INY,
    CMP_IMM(u8),
    DEX,
    CMP_ABS(u16),
    DEC_ABS(u16),
    BNE_REL(i8, String),
    CMP_ZP_X(u8),
    DEC_ZP_X(u8),
    CLD,
    CMP_ABS_Y(u16),
    CMP_ABS_X(u16),
    DEC_ABS_X(u16),
    CPX_IMM(u8),
    CPX_ZP(u8),
    SBC_ZP(u8),
    INC_ZP(u8),
    INX,
    SBC_IMM(u8),
    INC_ABS(u16),
    BEQ_REL(i8, String),
    INC_ZP_X(u8),
    SBC_ABS_X(u16),
    INC_ABS_X(u16),
    JAM,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut addr_to_variable = HashMap::new();
        return write!(f, "{}", self.to_write_string(&mut addr_to_variable));
    }
}

impl Instruction {
    pub fn to_write_string(&self, addr_to_variable: &mut HashMap<u16, Variable>) -> String {
        return match self {
            Instruction::ORA_ZP(v) => Instruction::to_write_string_zp("ora", v, addr_to_variable),
            Instruction::ASL_ZP(v) => Instruction::to_write_string_zp("asl", v, addr_to_variable),
            Instruction::PHP => format!("php"),
            Instruction::ORA_IMM(v) => format!("ora #${:02x}", v),
            Instruction::ASL => format!("asl"),
            Instruction::BPL_REL(_, v) => format!("bpl {}", v),
            Instruction::CLC => format!("clc"),
            Instruction::JSR_ABS(_addr, v) => format!("jsr {}", v),
            Instruction::BIT_ZP(v) => Instruction::to_write_string_zp("bit", v, addr_to_variable),
            Instruction::AND_ZP(v) => Instruction::to_write_string_zp("and", v, addr_to_variable),
            Instruction::PLP => format!("plp"),
            Instruction::AND_IMM(v) => format!("and #${:02x}", v),
            Instruction::ROL => format!("rol"),
            Instruction::BMI_REL(_, v) => format!("bmi {}", v),
            Instruction::AND_ZP_X(v) => {
                Instruction::to_write_string_zp_x("and", v, addr_to_variable)
            }
            Instruction::SEC => format!("sec"),
            Instruction::RTI => format!("rti"),
            Instruction::EOR_ZP(v) => Instruction::to_write_string_zp("eor", v, addr_to_variable),
            Instruction::LSR_ZP(v) => Instruction::to_write_string_zp("lsr", v, addr_to_variable),
            Instruction::PHA => format!("pha"),
            Instruction::EOR_IMM(v) => format!("eor #${:02x}", v),
            Instruction::LSR => format!("lsr"),
            Instruction::JMP_ABS(_addr, v) => format!("jmp {}", v),
            Instruction::EOR_ABS(v) => Instruction::to_write_string_abs("eor", v, addr_to_variable),
            Instruction::RTS => format!("rts"),
            Instruction::ADC_ZP(v) => Instruction::to_write_string_zp("adc", v, addr_to_variable),
            Instruction::ROR_ZP(v) => Instruction::to_write_string_zp("ror", v, addr_to_variable),
            Instruction::PLA => format!("pla"),
            Instruction::ADC_IMM(v) => format!("adc #${:02x}", v),
            Instruction::ROR => format!("ror"),
            Instruction::ADC_ABS(v) => Instruction::to_write_string_abs("adc", v, addr_to_variable),
            Instruction::SEI => format!("sei"),
            Instruction::ADC_ABS_X(v) => {
                Instruction::to_write_string_abs_x("adc", v, addr_to_variable)
            }
            Instruction::STY_ZP(v) => Instruction::to_write_string_zp("sty", v, addr_to_variable),
            Instruction::STA_ZP(v) => Instruction::to_write_string_zp("sta", v, addr_to_variable),
            Instruction::STX_ZP(v) => Instruction::to_write_string_zp("stx", v, addr_to_variable),
            Instruction::DEY => format!("dey"),
            Instruction::TXA => format!("txa"),
            Instruction::STY_ABS(v) => Instruction::to_write_string_abs("sty", v, addr_to_variable),
            Instruction::STA_ABS(v) => Instruction::to_write_string_abs("sta", v, addr_to_variable),
            Instruction::STX_ABS(v) => Instruction::to_write_string_abs("stx", v, addr_to_variable),
            Instruction::BCC_REL(_, v) => format!("bcc {}", v),
            Instruction::STA_IND_Y(v) => format!("sta (${:02x}),y", v),
            Instruction::STY_ZP_X(v) => {
                Instruction::to_write_string_zp_x("sty", v, addr_to_variable)
            }
            Instruction::STA_ZP_X(v) => {
                Instruction::to_write_string_zp_x("sta", v, addr_to_variable)
            }
            Instruction::TYA => format!("tya"),
            Instruction::STA_ABS_Y(v) => {
                Instruction::to_write_string_abs_y("sta", v, addr_to_variable)
            }
            Instruction::TXS => format!("txs"),
            Instruction::STA_ABS_X(v) => {
                Instruction::to_write_string_abs_x("sta", v, addr_to_variable)
            }
            Instruction::LDY_IMM(v) => format!("ldy #${:02x}", v),
            Instruction::LDX_IMM(v) => format!("ldx #${:02x}", v),
            Instruction::LDY_ZP(v) => Instruction::to_write_string_zp("ldy", v, addr_to_variable),
            Instruction::LDA_ZP(v) => Instruction::to_write_string_zp("lda", v, addr_to_variable),
            Instruction::LDX_ZP(v) => Instruction::to_write_string_zp("ldx", v, addr_to_variable),
            Instruction::LDA_IMM(v) => format!("lda #${:02x}", v),
            Instruction::TAX => format!("tax"),
            Instruction::TAY => format!("tay"),
            Instruction::LDA_IND_Y(v) => format!("lda (${:02x}),y", v),
            Instruction::LDY_ZP_X(v) => {
                Instruction::to_write_string_zp_x("ldy", v, addr_to_variable)
            }
            Instruction::LDA_ZP_X(v) => {
                Instruction::to_write_string_zp_x("lda", v, addr_to_variable)
            }
            Instruction::LDA_ABS_Y(v) => {
                Instruction::to_write_string_abs_y("lda", v, addr_to_variable)
            }
            Instruction::LDY_ABS_X(v) => {
                Instruction::to_write_string_abs_x("ldy", v, addr_to_variable)
            }
            Instruction::LDA_ABS_X(v) => {
                Instruction::to_write_string_abs_x("lda", v, addr_to_variable)
            }
            Instruction::LDX_ABS_Y(v) => {
                Instruction::to_write_string_abs_y("ldx", v, addr_to_variable)
            }
            Instruction::LDY_ABS(v) => Instruction::to_write_string_abs("ldy", v, addr_to_variable),
            Instruction::LDA_ABS(v) => Instruction::to_write_string_abs("lda", v, addr_to_variable),
            Instruction::LDX_ABS(v) => Instruction::to_write_string_abs("ldx", v, addr_to_variable),
            Instruction::BCS_REL(_, v) => format!("bcs {}", v),
            Instruction::CPY_IMM(v) => format!("cpy #${:02x}", v),
            Instruction::CPY_ZP(v) => Instruction::to_write_string_zp("cpy", v, addr_to_variable),
            Instruction::CMP_ZP(v) => Instruction::to_write_string_zp("cmp", v, addr_to_variable),
            Instruction::DEC_ZP(v) => Instruction::to_write_string_zp("dec", v, addr_to_variable),
            Instruction::INY => format!("iny"),
            Instruction::CMP_IMM(v) => format!("cmp #${:02x}", v),
            Instruction::DEX => format!("dex"),
            Instruction::CMP_ABS(v) => Instruction::to_write_string_abs("cmp", v, addr_to_variable),
            Instruction::DEC_ABS(v) => Instruction::to_write_string_abs("dec", v, addr_to_variable),
            Instruction::BNE_REL(_, v) => format!("bne {}", v),
            Instruction::CMP_ZP_X(v) => {
                Instruction::to_write_string_zp_x("dec", v, addr_to_variable)
            }
            Instruction::DEC_ZP_X(v) => {
                Instruction::to_write_string_zp_x("dec", v, addr_to_variable)
            }
            Instruction::CLD => format!("cld"),
            Instruction::CMP_ABS_Y(v) => {
                Instruction::to_write_string_abs_y("cmp", v, addr_to_variable)
            }
            Instruction::CMP_ABS_X(v) => {
                Instruction::to_write_string_abs_x("cmp", v, addr_to_variable)
            }
            Instruction::DEC_ABS_X(v) => {
                Instruction::to_write_string_abs_x("dec", v, addr_to_variable)
            }
            Instruction::CPX_IMM(v) => format!("cpx #${:02x}", v),
            Instruction::CPX_ZP(v) => Instruction::to_write_string_zp("cpx", v, addr_to_variable),
            Instruction::SBC_ZP(v) => Instruction::to_write_string_zp("sbc", v, addr_to_variable),
            Instruction::INC_ZP(v) => Instruction::to_write_string_zp("inc", v, addr_to_variable),
            Instruction::INX => format!("inx"),
            Instruction::SBC_IMM(v) => format!("sbc #${:02x}", v),
            Instruction::INC_ABS(v) => Instruction::to_write_string_abs("inc", v, addr_to_variable),
            Instruction::BEQ_REL(_, v) => format!("beq {}", v),
            Instruction::INC_ZP_X(v) => {
                Instruction::to_write_string_zp_x("inc", v, addr_to_variable)
            }
            Instruction::SBC_ABS_X(v) => {
                Instruction::to_write_string_abs_x("sbc", v, addr_to_variable)
            }
            Instruction::INC_ABS_X(v) => {
                Instruction::to_write_string_abs_x("inc", v, addr_to_variable)
            }
            Instruction::JAM => format!("jam"),
        };
    }

    fn to_write_string_zp(
        instr: &str,
        zp_addr: &u8,
        addr_to_variable: &mut HashMap<u16, Variable>,
    ) -> String {
        let addr = *zp_addr as u16;
        if let Option::Some(var) = addr_to_variable.get(&addr) {
            return format!("{} {}", instr, var.name);
        } else {
            addr_to_variable.insert(
                addr,
                Variable {
                    name: format!("ZP_{:02X}", zp_addr),
                    value: VariableValue::U8(*zp_addr),
                },
            );
            return format!("{} ${:02x}", instr, zp_addr);
        }
    }

    fn to_write_string_zp_x(
        instr: &str,
        zp_addr: &u8,
        addr_to_variable: &mut HashMap<u16, Variable>,
    ) -> String {
        let addr = *zp_addr as u16;
        if let Option::Some(var) = addr_to_variable.get(&addr) {
            return format!("{} {},x", instr, var.name);
        } else {
            addr_to_variable.insert(
                addr,
                Variable {
                    name: format!("ZP_{:02X}", zp_addr),
                    value: VariableValue::U8(*zp_addr),
                },
            );
            return format!("{} ${:02x},x", instr, zp_addr);
        }
    }

    fn to_write_string_abs(
        instr: &str,
        addr: &u16,
        addr_to_variable: &mut HashMap<u16, Variable>,
    ) -> String {
        if let Option::Some(var) = addr_to_variable.get(&addr) {
            return format!("{} {}", instr, var.name);
        } else {
            addr_to_variable.insert(
                *addr,
                Variable {
                    name: format!("ABS_{:04X}", addr),
                    value: VariableValue::U16(*addr),
                },
            );
            return format!("{} ${:04x}", instr, addr);
        }
    }

    fn to_write_string_abs_x(
        instr: &str,
        addr: &u16,
        addr_to_variable: &mut HashMap<u16, Variable>,
    ) -> String {
        if let Option::Some(var) = addr_to_variable.get(&addr) {
            return format!("{} {}", instr, var.name);
        } else {
            addr_to_variable.insert(
                *addr,
                Variable {
                    name: format!("ABS_{:04X}", addr),
                    value: VariableValue::U16(*addr),
                },
            );
            return format!("{} ${:04x},x", instr, addr);
        }
    }

    fn to_write_string_abs_y(
        instr: &str,
        addr: &u16,
        addr_to_variable: &mut HashMap<u16, Variable>,
    ) -> String {
        if let Option::Some(var) = addr_to_variable.get(&addr) {
            return format!("{} {}", instr, var.name);
        } else {
            addr_to_variable.insert(
                *addr,
                Variable {
                    name: format!("ABS_{:04X}", addr),
                    value: VariableValue::U16(*addr),
                },
            );
            return format!("{} ${:04x},y", instr, addr);
        }
    }
}
