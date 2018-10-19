use cartridge::Cartridge;
use mapper::Mapper;
use ppu::MirroringMode;
use std::mem;

enum PrgRomBankMode {
    Switch32K,
    FixFirstBank,
    FixLastBank,
}

impl Default for PrgRomBankMode {
    fn default() -> Self {
        PrgRomBankMode::Switch32K
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
        let is_full = self.sr & 0x01 == 1;

        // Clear sr to original state
        if val & 0x80 != 0 {
            self.sr = 0x10;
        }

        // Shift bit 0 of val to sr
        else {
            self.sr = (self.sr >> 1) | (val & 0x01) << 5;
        }

        if !is_full {
            return None;
        }

        Some(mem::replace(&mut self.sr, 0x10))
    }

    pub fn write_mirroring_mode(&mut self, val: u8) {
        self.mirroring_mode = match val {
            0x00 => MirroringMode::Lower,
            0x01 => MirroringMode::Upper,
            0x02 => MirroringMode::Vertical,
            0x03 => MirroringMode::Horizontal,
            _ => panic!("[MMC1] Invalid mirroring mode."),
        };
    }

    pub fn write_prg_rom_bank_mode(&mut self, val: u8) {
        self.prg_rom_bank_mode = match val {
            0x00 | 0x01 => PrgRomBankMode::Switch32K,
            0x02 => PrgRomBankMode::FixFirstBank,
            0x03 => PrgRomBankMode::FixLastBank,
            _ => panic!("[MMC1] Invalid prg rom bank mode."),
        };
    }

    pub fn write_chr_rom_bank_mode(&mut self, val: u8) {
        self.chr_rom_bank_mode = match val {
            0x00 => ChrRomBankMode::Switch8K,
            0x01 => ChrRomBankMode::Switch4K,
            _ => panic!("[MMC1] Invalid chr rom bank mode."),
        }
    }

    pub fn write_control(&mut self, val: u8) {
        println!("WRITE CONTROL {:08b}", val);
        self.write_mirroring_mode(val & 0x03);
        self.write_prg_rom_bank_mode((val >> 2) & 0x03);
        self.write_chr_rom_bank_mode((val >> 4) & 0x01);
    }

    pub fn write_prg_bank(&mut self, val: u8) {
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
        match addr {
            0x0000..=0x0FFF => {
                let addr = match self.r.chr_rom_bank_mode {
                    ChrRomBankMode::Switch8K => (self.r.chr_rom_bank_0 >> 1) as u16 * 0x1000 + addr,
                    ChrRomBankMode::Switch4K => self.r.chr_rom_bank_0 as u16 * 0x1000 + addr,
                };
                self.cartridge.read_chr_rom(addr)
            },
            0x1000..=0x1FFF => {
                let addr = match self.r.chr_rom_bank_mode {
                    ChrRomBankMode::Switch8K => (self.r.chr_rom_bank_0 >> 1) as u16 * 0x1000 + addr,
                    ChrRomBankMode::Switch4K => self.r.chr_rom_bank_1 as u16 * 0x1000 + addr - 0x1000,
                };
                self.cartridge.read_chr_rom(addr)
            },
            0x6000..=0x7FFF => self.cartridge.read_prg_ram(addr - 0x6000),
            0x8000..=0xBFFF => {
                let addr = match self.r.prg_rom_bank_mode {
                    PrgRomBankMode::Switch32K => (self.r.prg_rom_bank >> 1) as u16 * 0x8000 + addr - 0x8000,
                    PrgRomBankMode::FixFirstBank => addr - 0x8000,
                    PrgRomBankMode::FixLastBank => self.r.prg_rom_bank as u16 * 0x4000 + addr - 0x8000,
                };
                self.cartridge.read_prg_rom(addr)
            }
            0xC000..=0xFFFF => {
                let addr = match self.r.prg_rom_bank_mode {
                    PrgRomBankMode::Switch32K => (self.r.prg_rom_bank >> 1) as u16 * 0x8000 + addr - 0x8000,
                    PrgRomBankMode::FixFirstBank => self.r.prg_rom_bank as u16 * 0x4000 + addr - 0xC000,
                    PrgRomBankMode::FixLastBank => self.cartridge.prg_rom_len() as u16 - 0x8000 + addr - 0xC000,
                };
                self.cartridge.read_prg_rom(addr)
            },
            _ => 0,
        }
    }

    // TODO: Handle dummy writes when cpu writes on consecutive cycles:
    // https://wiki.nesdev.com/w/index.php/MMC1#Registers
    fn write_byte(&mut self, addr: u16, val: u8) {
        if addr < 0x8000 {
            return;
        }

        let val = match self.r.push_val(val) {
            Some(val) => val,
            None => return,
        };

        match addr {
            0x8000..=0x9FFF => self.r.write_control(val),
            0xA000..=0xBFFF => self.r.chr_rom_bank_0 = val,
            0xC000..=0xDFFF => self.r.chr_rom_bank_1 = val,
            0xE000..=0xFFFF => self.r.write_prg_bank(val),
            _ => {},
        }
    }

    fn mirroring_mode(&self) -> MirroringMode {
        self.r.mirroring_mode
    }
}
