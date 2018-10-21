use cartridge::Cartridge;
use mapper::Mapper;
use ppu::MirroringMode;
use std::mem;

#[derive(Debug)]
enum PrgRomBankMode {
    // prg rom is one switchable 32K bank
    Switch32K,
    // prg rom is one fixed 16K bank on the first bank and one switchable 16K bank
    FixFirstBank,
    // prg rom is one switchable 16K bank and one fixed 16K bank on the last bank
    FixLastBank,
}

impl Default for PrgRomBankMode {
    fn default() -> Self {
        PrgRomBankMode::FixLastBank
    }
}

#[derive(Debug)]
enum ChrRomBankMode {
    // chr rom is one switchable 8K bank
    Switch8K,
    // chr rom is two switchable 4K banks
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
        println!("[MMC1] Push shift register.");
        let is_full = self.sr & 0x01 == 1;

        // Clear sr to original state
        if val & 0x80 != 0 {
            self.sr = 0x10;
            println!("[MMC1] Clear shift register.");
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
        self.mirroring_mode = match val {
            0x00 => MirroringMode::Lower,
            0x01 => MirroringMode::Upper,
            0x02 => MirroringMode::Vertical,
            0x03 => MirroringMode::Horizontal,
            _ => panic!("[MMC1] Invalid mirroring mode."),
        };
        println!("[MMC1] Write mirroring mode: {:?}.", self.mirroring_mode);
    }

    pub fn write_prg_rom_bank_mode(&mut self, val: u8) {
        self.prg_rom_bank_mode = match val {
            0x00 | 0x01 => PrgRomBankMode::Switch32K,
            0x02 => PrgRomBankMode::FixFirstBank,
            0x03 => PrgRomBankMode::FixLastBank,
            _ => panic!("[MMC1] Invalid prg rom bank mode."),
        };
        println!(
            "[MMC1] Write prg rom bank mode: {:?}.",
            self.prg_rom_bank_mode
        );
    }

    pub fn write_chr_rom_bank_mode(&mut self, val: u8) {
        self.chr_rom_bank_mode = match val {
            0x00 => ChrRomBankMode::Switch8K,
            0x01 => ChrRomBankMode::Switch4K,
            _ => panic!("[MMC1] Invalid chr rom bank mode."),
        };
        println!(
            "[MMC1] Write chr rom bank mode: {:?}.",
            self.chr_rom_bank_mode
        );
    }

    pub fn write_control(&mut self, val: u8) {
        self.write_mirroring_mode(val & 0x03);
        self.write_prg_rom_bank_mode((val >> 2) & 0x03);
        self.write_chr_rom_bank_mode((val >> 4) & 0x01);
    }

    pub fn write_prg_bank(&mut self, val: u8) {
        self.prg_rom_bank = val & 0x0F;
        println!("[MMC1] Write prg rom bank: {}.", self.prg_rom_bank);
        self.prg_ram_enabled = val & 0x10 == 0;
        println!("[MMC1] Write prg ram enabled: {}.", self.prg_ram_enabled);
    }
}

impl Default for Registers {
    fn default() -> Self {
        Registers::new()
    }
}

pub struct MMC1 {
    cartridge: Cartridge,
    r: Registers,
}

impl MMC1 {
    pub fn new(cartridge: Cartridge) -> Self {
        MMC1 {
            cartridge,
            r: Registers::default(),
        }
    }
}

// TODO: Handle differences between variants.
impl Mapper for MMC1 {
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
            0x6000..=0x7FFF if self.r.prg_ram_enabled => {
                let addr = (addr - 0x6000) % self.cartridge.prg_ram_len();
                self.cartridge.read_prg_ram(addr)
            },
            0x8000..=0xBFFF => {
                let bank = match self.r.prg_rom_bank_mode {
                    PrgRomBankMode::Switch32K => self.r.prg_rom_bank as usize & !0x01,
                    PrgRomBankMode::FixFirstBank => 0,
                    PrgRomBankMode::FixLastBank => self.r.prg_rom_bank as usize,
                };
                self.cartridge.read_prg_rom(bank * 0x4000 + addr - 0x8000)
            },
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

        match addr {
            0x0000..=0x0FFF => {
                let bank = match self.r.chr_rom_bank_mode {
                    ChrRomBankMode::Switch8K => self.r.chr_rom_bank_0 as usize & !0x01,
                    ChrRomBankMode::Switch4K => self.r.chr_rom_bank_0 as usize,
                } as usize;
                self.cartridge.write_chr_rom(bank * 0x1000 + addr, val);
            },
            0x1000..=0x1FFF => {
                let bank = match self.r.chr_rom_bank_mode {
                    ChrRomBankMode::Switch8K => self.r.chr_rom_bank_0 as usize | 0x01,
                    ChrRomBankMode::Switch4K => self.r.chr_rom_bank_1 as usize,
                };
                self.cartridge
                    .write_chr_rom(bank * 0x1000 + addr - 0x1000, val)
            },
            0x6000..=0x7FFF if self.r.prg_ram_enabled => {
                let addr = (addr - 0x6000) % self.cartridge.prg_ram_len();
                self.cartridge.write_prg_ram(addr, val);
            },
            0x8000..=0xFFFF => {
                let val = match self.r.push_val(val) {
                    Some(val) => val,
                    None => return,
                };
                match addr {
                    0x8000..=0x9FFF => self.r.write_control(val),
                    0xA000..=0xBFFF => {
                        self.r.chr_rom_bank_0 = val;
                        println!("[MMC1] Write chr rom bank 0: {}.", self.r.chr_rom_bank_0);
                    },
                    0xC000..=0xDFFF => {
                        self.r.chr_rom_bank_1 = val;
                        println!("[MMC1] Write chr rom bank 1: {}.", self.r.chr_rom_bank_1);
                    },
                    0xE000..=0xFFFF => self.r.write_prg_bank(val),
                    _ => {},
                }
            },
            _ => {},
        }
    }

    fn mirroring_mode(&self) -> MirroringMode {
        self.r.mirroring_mode
    }
}
