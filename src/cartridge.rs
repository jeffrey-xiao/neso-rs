use crate::info;
use crate::ppu::MirroringMode;
#[cfg(not(target_arch = "wasm32"))]
use serde_derive::{Deserialize, Serialize};

const CARTRIDGE_HEADER: u32 = 0x1A53_454E;

#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Serialize))]
pub struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    pub is_chr_ram: bool,
    pub has_battery: bool,
    pub mapper: u8,
    pub mirroring_mode: MirroringMode,
}

impl Cartridge {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn empty_cartridge() -> Self {
        Cartridge {
            prg_rom: Vec::new(),
            chr_rom: Vec::new(),
            prg_ram: Vec::new(),
            is_chr_ram: false,
            has_battery: false,
            mapper: 0,
            mirroring_mode: MirroringMode::default(),
        }
    }

    pub fn from_buffer(mut buffer: &[u8]) -> Self {
        let header = u32::from(buffer[0])
            | (u32::from(buffer[1]) << 8)
            | (u32::from(buffer[2]) << 16)
            | (u32::from(buffer[3]) << 24);
        assert_eq!(
            header, CARTRIDGE_HEADER,
            "Error reading cartridge: expected header[0..4] = 0x1A53454E."
        );

        let mut is_zero = true;
        for val in buffer[11..=15].iter() {
            is_zero &= *val == 0;
        }

        let prg_rom_len = buffer[4] as usize * 0x4000;
        info!("[CARTRIDGE] PRG ROM length: {} bytes.", prg_rom_len);
        let chr_rom_len = buffer[5] as usize * 0x2000;
        info!("[CARTRIDGE] CHR ROM length: {} bytes.", chr_rom_len);
        let mut prg_ram_len = buffer[8] as usize * 0x2000;
        info!("[CARTRIDGE] PRG RAM length: {} bytes.", prg_ram_len);

        if prg_ram_len == 0 {
            prg_ram_len = 0x4000;
        }

        let flags_6 = buffer[6];
        let flags_7 = if is_zero { buffer[7] } else { 0 };

        buffer = buffer.split_at(16).1;

        if flags_6 & 0x04 != 0 {
            info!("[CARTRIDGE] Trainer present.");
            buffer = buffer.split_at(512).1;
        }

        let (prg_rom_buffer, buffer) = buffer.split_at(prg_rom_len);
        let prg_rom = prg_rom_buffer.to_vec();

        let (is_chr_ram, chr_rom) = if chr_rom_len > 0 {
            info!("[CARTRIDGE] Using CHR ROM.");
            let (chr_rom_buffer, _) = buffer.split_at(chr_rom_len);
            (false, chr_rom_buffer.to_vec())
        } else {
            info!("[CARTRIDGE] Using CHR RAM.");
            (true, vec![0; 0x2000])
        };

        let has_battery = flags_6 & 0x10 != 0;
        info!("[CARTRIDGE] Has battery: {}.", has_battery);

        let mapper = (flags_7 & 0xF0) | (flags_6 >> 4);
        info!("[CARTRIDGE] Mapper: {}.", mapper);

        let mirroring_mode = {
            if flags_6 & 0x08 != 0 {
                MirroringMode::None
            } else if flags_6 & 0x01 != 0 {
                MirroringMode::Vertical
            } else {
                MirroringMode::Horizontal
            }
        };
        info!("[CARTRIDGE] Mirroring mode: {:?}.", mirroring_mode);

        Cartridge {
            prg_rom,
            chr_rom,
            prg_ram: vec![0; prg_ram_len],
            is_chr_ram,
            has_battery,
            mapper,
            mirroring_mode,
        }
    }

    pub fn prg_rom_len(&self) -> usize {
        self.prg_rom.len()
    }

    pub fn read_prg_rom(&self, addr: usize) -> u8 {
        let len = self.prg_rom_len();
        self.prg_rom[addr % len]
    }

    pub fn chr_rom_len(&self) -> usize {
        self.chr_rom.len()
    }

    pub fn read_chr_rom(&self, addr: usize) -> u8 {
        let len = self.chr_rom_len();
        self.chr_rom[addr % len]
    }

    // chr_rom is ram if the size reported in the header is 0.
    pub fn write_chr_rom(&mut self, addr: usize, val: u8) {
        let len = self.chr_rom_len();
        self.chr_rom[addr % len] = val;
    }

    pub fn prg_ram_len(&self) -> usize {
        self.prg_ram.len()
    }

    pub fn read_prg_ram(&self, addr: usize) -> u8 {
        self.prg_ram[addr]
    }

    pub fn write_prg_ram(&mut self, addr: usize, val: u8) {
        self.prg_ram[addr] = val;
    }

    pub fn chr_bank(&self, offset: usize) -> *const u8 {
        unsafe { self.chr_rom.as_ptr().add(offset * 0x400) }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) -> bincode::Result<Option<Vec<u8>>> {
        if !self.has_battery {
            return Ok(None);
        }
        self.save_state().map(Some)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_state(&self) -> bincode::Result<Vec<u8>> {
        let chr_ram_opt = if self.is_chr_ram {
            Some(&self.chr_rom)
        } else {
            None
        };
        bincode::serialize(&(&self.prg_ram, chr_ram_opt))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(&mut self, save_data: &[u8]) -> bincode::Result<()> {
        let (prg_ram, chr_ram_opt) = bincode::deserialize(save_data)?;
        self.prg_ram = prg_ram;
        if let Some(chr_ram) = chr_ram_opt {
            self.chr_rom = chr_ram;
        }
        Ok(())
    }
}
