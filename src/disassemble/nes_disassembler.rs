use std::io::Write;

use super::{
    disassembler::Disassembler,
    variable::{Variable, VariableValue},
    DisassembleError, code::{AsmCode, Statement},
};

// https://www.nesdev.org/wiki/NES_2.0
// https://archive.nes.science/nesdev-forums/f2/t10469.xhtml
// https://en.wikibooks.org/wiki/NES_Programming/Initializing_the_NES
// https://www.pagetable.com/c64ref/6502/
const NES_HEADER_LENGTH: usize = 16;
const NES_PRG_ROM_PAGE_LENGTH: usize = 16 * 1024;
const NES_CHR_ROM_PAGE_LENGTH: usize = 8 * 1024;
const NES_PRG_ROM_START_ADDRESS: usize = 0x8000;

pub struct NesDisassembler {
    d: Disassembler,
    prg_rom_count: u8,
    chr_rom_count: u8,
    flags6: u8,
    flags7: u8,
    mapper: u8,
    prg_chr_rom_size: u8,
    prg_ram_eeprom_size: u8,
    chr_ram_size: u8,
    cpu_ppu_timing: u8,
    flags13: u8,
    misc_roms: u8,
    default_expansion_device: u8,
}

impl NesDisassembler {
    pub fn is_handled(data: &Vec<u8>) -> bool {
        return data[0] == b'N' && data[1] == b'E' && data[2] == b'S' && data[3] == 0x1a;
    }

    pub fn disassemble(data: Vec<u8>, out: Box<dyn Write>) -> Result<(), super::DisassembleError> {
        let mut d = NesDisassembler {
            d: Disassembler::new(data),
            prg_rom_count: 0,
            chr_rom_count: 0,
            flags6: 0,
            flags7: 0,
            mapper: 0,
            prg_chr_rom_size: 0,
            prg_ram_eeprom_size: 0,
            chr_ram_size: 0,
            cpu_ppu_timing: 0,
            flags13: 0,
            misc_roms: 0,
            default_expansion_device: 0,
        };

        d.set_variables();
        d.parse_header()?;
        d.parse_chr_rom()?;
        d.disassemble_entry_points()?;

        d.d.code.write(out)?;

        return Result::Ok(());
    }

    fn set_variables(&mut self) {
        self.d.code.set_variable(
            0x2000,
            Variable {
                name: "PPU_CTRL".to_string(),
                value: VariableValue::U16(0x2000),
            },
        );
        self.d.code.set_variable(
            0x2001,
            Variable {
                name: "PPU_MASK".to_string(),
                value: VariableValue::U16(0x2001),
            },
        );
        self.d.code.set_variable(
            0x2002,
            Variable {
                name: "PPU_STATUS".to_string(),
                value: VariableValue::U16(0x2002),
            },
        );
        self.d.code.set_variable(
            0x2003,
            Variable {
                name: "OAM_ADDR".to_string(),
                value: VariableValue::U16(0x2003),
            },
        );
        self.d.code.set_variable(
            0x2004,
            Variable {
                name: "OAM_DATA".to_string(),
                value: VariableValue::U16(0x2004),
            },
        );
        self.d.code.set_variable(
            0x2005,
            Variable {
                name: "PPU_SCROLL".to_string(),
                value: VariableValue::U16(0x2005),
            },
        );
        self.d.code.set_variable(
            0x2006,
            Variable {
                name: "PPU_ADDR".to_string(),
                value: VariableValue::U16(0x2006),
            },
        );
        self.d.code.set_variable(
            0x2007,
            Variable {
                name: "PPU_DATA".to_string(),
                value: VariableValue::U16(0x2007),
            },
        );

        self.d.code.set_variable(
            0x4000,
            Variable {
                name: "APU_PULSE_1_ENV".to_string(),
                value: VariableValue::U16(0x4000),
            },
        );
        self.d.code.set_variable(
            0x4001,
            Variable {
                name: "APU_PULSE_1_SWEEP".to_string(),
                value: VariableValue::U16(0x4001),
            },
        );
        self.d.code.set_variable(
            0x4002,
            Variable {
                name: "APU_PULSE_1_TIMER".to_string(),
                value: VariableValue::U16(0x4002),
            },
        );
        self.d.code.set_variable(
            0x4003,
            Variable {
                name: "APU_PULSE_1_LEN".to_string(),
                value: VariableValue::U16(0x4003),
            },
        );
        self.d.code.set_variable(
            0x4004,
            Variable {
                name: "APU_PULSE_2_ENV".to_string(),
                value: VariableValue::U16(0x4004),
            },
        );
        self.d.code.set_variable(
            0x4005,
            Variable {
                name: "APU_PULSE_2_SWEEP".to_string(),
                value: VariableValue::U16(0x4005),
            },
        );
        self.d.code.set_variable(
            0x4006,
            Variable {
                name: "APU_PULSE_2_TIMER".to_string(),
                value: VariableValue::U16(0x4006),
            },
        );
        self.d.code.set_variable(
            0x4007,
            Variable {
                name: "APU_PULSE_2_LEN".to_string(),
                value: VariableValue::U16(0x4007),
            },
        );
        self.d.code.set_variable(
            0x4008,
            Variable {
                name: "APU_TRIANGLE_LEN_CR".to_string(),
                value: VariableValue::U16(0x4008),
            },
        );
        self.d.code.set_variable(
            0x4009,
            Variable {
                name: "APU_TRIANGLE_UNUSED".to_string(),
                value: VariableValue::U16(0x4009),
            },
        );
        self.d.code.set_variable(
            0x400a,
            Variable {
                name: "APU_TRIANGLE_TIMER".to_string(),
                value: VariableValue::U16(0x400a),
            },
        );
        self.d.code.set_variable(
            0x400b,
            Variable {
                name: "APU_TRIANGLE_LOAD".to_string(),
                value: VariableValue::U16(0x400b),
            },
        );
        self.d.code.set_variable(
            0x400c,
            Variable {
                name: "APU_NOISE_ENV".to_string(),
                value: VariableValue::U16(0x400c),
            },
        );
        self.d.code.set_variable(
            0x400d,
            Variable {
                name: "APU_NOISE_UNUSED".to_string(),
                value: VariableValue::U16(0x400d),
            },
        );
        self.d.code.set_variable(
            0x400e,
            Variable {
                name: "APU_NOISE_LP".to_string(),
                value: VariableValue::U16(0x400e),
            },
        );
        self.d.code.set_variable(
            0x400f,
            Variable {
                name: "APU_NOISE_LOAD".to_string(),
                value: VariableValue::U16(0x400f),
            },
        );
        self.d.code.set_variable(
            0x4010,
            Variable {
                name: "APU_DMC_IL__RRRR".to_string(),
                value: VariableValue::U16(0x4010),
            },
        );
        self.d.code.set_variable(
            0x4011,
            Variable {
                name: "APU_DMC_LOAD".to_string(),
                value: VariableValue::U16(0x4011),
            },
        );
        self.d.code.set_variable(
            0x4012,
            Variable {
                name: "APU_DMC_SAMPLE_ADDR".to_string(),
                value: VariableValue::U16(0x4012),
            },
        );
        self.d.code.set_variable(
            0x4013,
            Variable {
                name: "APU_DMC_SAMPLE_LEN".to_string(),
                value: VariableValue::U16(0x4013),
            },
        );
        self.d.code.set_variable(
            0x4014,
            Variable {
                name: "OAM_DMA".to_string(),
                value: VariableValue::U16(0x4014),
            },
        );
        self.d.code.set_variable(
            0x4015,
            Variable {
                name: "APU_CH_ENABLE_STATUS".to_string(),
                value: VariableValue::U16(0x4015),
            },
        );
        self.d.code.set_variable(
            0x4017,
            Variable {
                name: "APU_ALL_FRAME_COUNTER".to_string(),
                value: VariableValue::U16(0x4017),
            },
        );
    }

    fn parse_header(&mut self) -> Result<(), DisassembleError> {
        if self.d.code.is_eq_u8(0, b'N')
            && self.d.code.is_eq_u8(1, b'E')
            && self.d.code.is_eq_u8(2, b'S')
            && self.d.code.is_eq_u8(3, 0x1a)
        {
            self.d.code.replace(
                0..4,
                AsmCode::DataSeq(vec![
                    AsmCode::DataString("NES".to_string()),
                    AsmCode::DataHexU8(0x1a),
                ]),
            )?;
            self.d.code.set_segment(0, "HEADER");
        } else {
            return Result::Err(DisassembleError::ParseError(
                "invalid nes header".to_string(),
            ));
        }

        self.prg_rom_count = self.d.code.replace_with_u8(4)?;
        self.d.code.set_comment(4, "PRG ROM count");

        self.chr_rom_count = self.d.code.replace_with_u8(5)?;
        self.d.code.set_comment(5, "CHR ROM count");

        self.flags6 = self.d.code.replace_with_binary_u8(6)?;
        self.d.code.set_comment(
            6,
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
      NNNNFTBM",
        );

        self.flags7 = self.d.code.replace_with_binary_u8(7)?;
        self.d.code.set_comment(
            7,
            "Flags 7
      ++++------ Mapper Number D4..D7
      ||||++---- NES 2.0 identifier
      ||||||      3: Extended Console Type
      ||||||      2: Nintendo Playchoice 10
      ||||||      1: Nintendo Vs. System
      ||||||      0: Nintendo Entertainment System/Family Computer
      ||||||++-- Console type
      NNNN10TT",
        );

        self.mapper = self.d.code.replace_with_binary_u8(8)?;
        self.d.code.set_comment(
            8,
            "Mapper MSB/Submapper
      ++++------ Submapper number
      ||||++++-- Mapper number D8..D11
      SSSSNNNN",
        );

        self.prg_chr_rom_size = self.d.code.replace_with_binary_u8(9)?;
        self.d.code.set_comment(
            9,
            "PRG-ROM/CHR-ROM size MSB
      ++++------ CHR-ROM size MSB
      ||||++++-- PRG-ROM size MSB
      CCCCPPPP",
        );

        self.prg_ram_eeprom_size = self.d.code.replace_with_binary_u8(10)?;
        self.d.code.set_comment(
            10,
            "PRG-RAM/EEPROM size            
  If the shift count is zero, there is no PRG-(NV)RAM.
  If the shift count is non-zero, the actual size is
  \"64 << shift count\" bytes, i.e. 8192 bytes for a shift count of 7.
      ++++------ PRG-NVRAM/EEPROM (non-volatile) shift count
      ||||++++-- PRG-RAM (volatile) shift count
      ppppPPPP",
        );

        self.chr_ram_size = self.d.code.replace_with_binary_u8(11)?;
        self.d.code.set_comment(
            11,
            "CHR-RAM size
  If the shift count is zero, there is no CHR-(NV)RAM.
  If the shift count is non-zero, the actual size is
  \"64 << shift count\" bytes, i.e. 8192 bytes for a shift count of 7.
      ++++------ CHR-NVRAM size (non-volatile) shift count
      ||||++++-- CHR-RAM size (volatile) shift count
      ccccCCCC",
        );

        self.cpu_ppu_timing = self.d.code.replace_with_binary_u8(12)?;
        self.d.code.set_comment(
            12,
            "CPU/PPU Timing
            ++- CPU/PPU timing mode
            ||   0: RP2C02 (\"NTSC NES\")
            ||   1: RP2C07 (\"Licensed PAL NES\")
            ||   2: Multiple-region
            ||   3: UMC 6527P (\"Dendy\")
      ......VV",
        );

        self.flags13 = self.d.code.replace_with_binary_u8(13)?;
        self.d.code.set_comment(
            13,
            "When Byte 7 AND 3 =1: Vs. System Type
      ++++------ Vs. Hardware Type
      ||||++++-- Vs. PPU Type
      MMMMPPPP",
        );

        self.misc_roms = self.d.code.replace_with_binary_u8(14)?;
        self.d.code.set_comment(
            14,
            "Miscellaneous ROMs
            ++- Number of miscellaneous ROMs present
      ......RR",
        );

        self.default_expansion_device = self.d.code.replace_with_binary_u8(15)?;
        self.d.code.set_comment(
            15,
            "Default Expansion Device
        ++++++- Default Expansion Device
      ..DDDDDD",
        );

        return Result::Ok(());
    }

    fn parse_chr_rom(&mut self) -> Result<(), DisassembleError> {
        let chr_rom_start_addr =
            NES_HEADER_LENGTH + ((self.prg_rom_count as usize) * NES_PRG_ROM_PAGE_LENGTH);
        let mut addr = chr_rom_start_addr;
        for chr_rom_index in 0..self.chr_rom_count {
            let chr_rom_start_addr = addr;
            let chr_rom_end_addr = addr + NES_CHR_ROM_PAGE_LENGTH;
            while addr < chr_rom_end_addr {
                let mut bytes = Vec::new();
                for i in 0..16 {
                    let old_value = self.d.code.take(addr + i)?;
                    bytes.push(old_value.asm_code);
                }
                // TODO create .neschr with values split out to visualize
                self.d.code.set(
                    addr,
                    Statement {
                        asm_code: AsmCode::DataSeq(bytes),
                        comment: Option::None,
                        segment: Option::None,
                        label: Option::None,
                    },
                )?;
                addr += 16;
            }
            self.d.code.set_segment(
                chr_rom_start_addr,
                format!("CHRROM{}", chr_rom_index).as_str(),
            );
        }
        return Result::Ok(());
    }

    fn disassemble_entry_points(&mut self) -> Result<(), DisassembleError> {
        let mut offset = NES_HEADER_LENGTH;
        for prg_rom_idx in 0..self.prg_rom_count {
            let nmi = self.decode_vector(offset + NES_PRG_ROM_PAGE_LENGTH - 6, "NMI")?;
            let reset = self.decode_vector(offset + NES_PRG_ROM_PAGE_LENGTH - 4, "RESET")?;
            let irq = self.decode_vector(offset + NES_PRG_ROM_PAGE_LENGTH - 2, "IRQ")?;

            let addr_to_offset_fn = |a: u16| {
                let mut addr = (a as usize) - NES_PRG_ROM_START_ADDRESS + NES_HEADER_LENGTH;
                // TODO I think this should only happen if prg rom pages are mirrored
                if addr > NES_PRG_ROM_PAGE_LENGTH {
                    addr = addr - NES_PRG_ROM_PAGE_LENGTH;
                }
                return addr as usize;
            };

            let offset_to_addr_fn = |offset: usize| {
                return (offset - NES_HEADER_LENGTH + NES_PRG_ROM_START_ADDRESS) as u16;
            };

            self.d.disassemble(
                nmi,
                "nmi",
                format!("prgrom{}", prg_rom_idx).as_str(),
                &addr_to_offset_fn,
                &offset_to_addr_fn,
            )?;
            self.d.disassemble(
                reset,
                "reset",
                format!("prgrom{}", prg_rom_idx).as_str(),
                &addr_to_offset_fn,
                &offset_to_addr_fn,
            )?;
            self.d.disassemble(
                irq,
                "irq",
                format!("prgrom{}", prg_rom_idx).as_str(),
                &addr_to_offset_fn,
                &offset_to_addr_fn,
            )?;

            self.d
                .code
                .set_segment(offset, format!("PRGROM{}", prg_rom_idx).as_str());

            offset += NES_PRG_ROM_PAGE_LENGTH;
        }

        return Result::Ok(());
    }

    fn decode_vector(&mut self, offset: usize, name: &str) -> Result<u16, DisassembleError> {
        let low = self.d.code.take(offset)?.asm_code.to_u8()? as u16;
        let high = self.d.code.take(offset + 1)?.asm_code.to_u8()? as u16;
        let addr = low | (high << 8);
        self.d
            .code
            .replace(offset..offset + 2, AsmCode::DataHexU16(addr))?;
        self.d.code.set_comment(offset, name);
        return Result::Ok(addr);
    }
}
