use cartridge::Cartridge;
use mapper::Mapper;
use ppu::MirroringMode;
use std::mem;

#[derive(Debug)]
enum PrgRomBankMode {
    Switch32K,
    FixFirstBank,
    FixLastBank,
}

impl Default for PrgRomBankMode {
    fn default() -> Self {
        PrgRomBankMode::FixLastBank
    }
}

enum ChrRomBankMode {
    Switch8K,
    Switch4K,
}

impl Default for ChrRomBankMode {
    fn default() -> Self {
        ChrRomBankMode::Switch8K
    }
}

struct Registers {
    sr: u8,
    mirroring_mode: MirroringMode,
    prg_rom_bank_mode: PrgRomBankMode,
    chr_rom_bank_mode: ChrRomBankMode,
    chr_rom_bank_0: u8,
    chr_rom_bank_1: u8,
    prg_rom_bank: u8,
    prg_ram_enabled: bool,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            sr: 0x10,
            mirroring_mode: MirroringMode::default(),
            prg_rom_bank_mode: PrgRomBankMode::default(),
            chr_rom_bank_mode: ChrRomBankMode::default(),
            chr_rom_bank_0: 0,
            chr_rom_bank_1: 0,
            prg_rom_bank: 0,
            prg_ram_enabled: false,
        }
    }

    pub fn push_val(&mut self, val: u8) -> Option<u8> {
        println!("PUSH SR");
        let is_full = self.sr & 0x01 == 1;

        // Clear sr to original state
        if val & 0x80 != 0 {
            self.sr = 0x10;
            println!("CLEAR SR");
            return None;
        }

        // Shift bit 0 of val to sr
        else {
            self.sr = (self.sr >> 1) | (val & 0x01) << 4;
        }

        if !is_full {
            return None;
        }

        Some(mem::replace(&mut self.sr, 0x10))
    }

    pub fn write_mirroring_mode(&mut self, val: u8) {
        println!("WRITING MIRRORING MODE: {}", val);
        self.mirroring_mode = match val {
            0x00 => MirroringMode::Lower,
            0x01 => MirroringMode::Upper,
            0x02 => MirroringMode::Vertical,
            0x03 => MirroringMode::Horizontal,
            _ => panic!("[MMC1] Invalid mirroring mode."),
        };
    }

    pub fn write_prg_rom_bank_mode(&mut self, val: u8) {
        println!("WRITING PRG ROM BANK MODE: {}", val);
        self.prg_rom_bank_mode = match val {
            0x00 | 0x01 => PrgRomBankMode::Switch32K,
            0x02 => PrgRomBankMode::FixFirstBank,
            0x03 => PrgRomBankMode::FixLastBank,
            _ => panic!("[MMC1] Invalid prg rom bank mode."),
        };
    }

    pub fn write_chr_rom_bank_mode(&mut self, val: u8) {
        println!("WRITING CHR ROM BANK MODE: {}", val);
        self.chr_rom_bank_mode = match val {
            0x00 => ChrRomBankMode::Switch8K,
            0x01 => ChrRomBankMode::Switch4K,
            _ => panic!("[MMC1] Invalid chr rom bank mode."),
        }
    }

    pub fn write_control(&mut self, val: u8) {
        self.write_mirroring_mode(val & 0x03);
        self.write_prg_rom_bank_mode((val >> 2) & 0x03);
        self.write_chr_rom_bank_mode((val >> 4) & 0x01);
    }

    pub fn write_prg_bank(&mut self, val: u8) {
        println!("WRITING PRG BANK: {}", val & 0x0F);
        self.prg_rom_bank = val & 0x0F;
        self.prg_ram_enabled = val & 0x10 == 0;
    }
}

impl Default for Registers {
    fn default() -> Self {
        Registers::new()
    }
}

pub struct Mmc1 {
    cartridge: Cartridge,
    r: Registers,
}

impl Mmc1 {
    pub fn new(cartridge: Cartridge) -> Self {
        Mmc1 {
            cartridge,
            r: Registers::default(),
        }
    }
}

impl Mapper for Mmc1 {
    fn read_byte(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0x0000..=0x0FFF => {
                let bank = match self.r.chr_rom_bank_mode {
                    ChrRomBankMode::Switch8K => self.r.chr_rom_bank_0 as usize & !0x01,
                    ChrRomBankMode::Switch4K => self.r.chr_rom_bank_0 as usize,
                } as usize;
                self.cartridge.read_chr_rom(bank * 0x1000 + addr)
            },
            0x1000..=0x1FFF => {
                let bank = match self.r.chr_rom_bank_mode {
                    ChrRomBankMode::Switch8K => self.r.chr_rom_bank_0 as usize | 0x01,
                    ChrRomBankMode::Switch4K => self.r.chr_rom_bank_1 as usize,
                };
                self.cartridge.read_chr_rom(bank * 0x1000 + addr - 0x1000)
            },
            0x6000..=0x7FFF => self.cartridge.read_prg_ram(addr - 0x6000),
            0x8000..=0xBFFF => {
                let bank = match self.r.prg_rom_bank_mode {
                    PrgRomBankMode::Switch32K => self.r.prg_rom_bank as usize & !0x01,
                    PrgRomBankMode::FixFirstBank => 0,
                    PrgRomBankMode::FixLastBank => self.r.prg_rom_bank as usize,
                };
                self.cartridge.read_prg_rom(bank * 0x4000 + addr - 0x8000)
            }
            0xC000..=0xFFFF => {
                let bank = match self.r.prg_rom_bank_mode {
                    PrgRomBankMode::Switch32K => self.r.prg_rom_bank as usize | 0x01,
                    PrgRomBankMode::FixFirstBank => self.r.prg_rom_bank as usize,
                    PrgRomBankMode::FixLastBank => self.cartridge.prg_rom_len() / 0x4000 - 1,
                };
                self.cartridge.read_prg_rom(bank * 0x4000 + addr - 0xC000)
            },
            _ => 0,
        }
    }

    // TODO: Handle dummy writes when cpu writes on consecutive cycles:
    // https://wiki.nesdev.com/w/index.php/MMC1#Registers
    fn write_byte(&mut self, addr: u16, val: u8) {
        let addr = addr as usize;

        if 0x0000 <= addr && addr <= 0x1FFF {
            println!("WRITING CHR ROM {:x} {}", addr, val);
            self.cartridge.write_chr_rom(addr, val);
            return;
        }

        if 0x6000 <= addr && addr <= 0x7FFF {
            self.cartridge.write_prg_ram(addr - 0x6000, val);
            return;
        }

        if addr < 0x8000 {
            return;
        }

        println!("addr {:x}", addr);
        let val = match self.r.push_val(val) {
            Some(val) => val,
            None => return,
        };

        match addr {
            0x8000..=0x9FFF => self.r.write_control(val),
            0xA000..=0xBFFF => {
                println!("CHR ROM BANK 0: {}", val);
                self.r.chr_rom_bank_0 = val;
            },
            0xC000..=0xDFFF => {
                println!("CHR ROM BANK 1: {}", val);
                self.r.chr_rom_bank_1 = val;
            },
            0xE000..=0xFFFF => self.r.write_prg_bank(val),
            _ => {},
        }
    }

    fn mirroring_mode(&self) -> MirroringMode {
        self.r.mirroring_mode
    }
}
