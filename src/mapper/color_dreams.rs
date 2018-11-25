use cartridge::Cartridge;
#[cfg(target_arch = "wasm32")]
use debug;
use mapper::Mapper;
use ppu::MirroringMode;

#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Serialize))]
pub struct ColorDreams {
    #[cfg_attr(
        not(target_arch = "wasm32"),
        serde(skip, default = "Cartridge::empty_cartridge")
    )]
    cartridge: Cartridge,
    prg_rom_bank: u8,
    chr_rom_bank: u8,
}

impl ColorDreams {
    pub fn new(cartridge: Cartridge) -> Self {
        ColorDreams {
            cartridge,
            prg_rom_bank: 0,
            chr_rom_bank: 0,
        }
    }
}

impl Mapper for ColorDreams {
    fn read_byte(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => {
                let addr = self.chr_rom_bank as usize * 0x2000 + addr;
                self.cartridge.read_chr_rom(addr)
            },
            0x8000..=0xFFFF => {
                let addr = self.prg_rom_bank as usize * 0x8000 + addr - 0x8000;
                self.cartridge.read_prg_rom(addr)
            },
            _ => 0,
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => {
                let addr = self.chr_rom_bank as usize * 0x2000 + addr;
                self.cartridge.write_chr_rom(addr, val);
            },
            0x8000..=0xFFFF => {
                self.prg_rom_bank = val & 0x03;
                debug!("[ColorDreams] Write prg rom bank: {}.", self.prg_rom_bank);
                self.chr_rom_bank = (val >> 4) & 0x0F;
                debug!("[ColorDreams] Write chr rom bank: {}.", self.chr_rom_bank);
            },
            _ => {},
        }
    }

    fn chr_bank(&self, index: usize) -> *const u8 {
        self.cartridge
            .chr_bank(self.chr_rom_bank as usize * 8 + index)
    }

    fn mirroring_mode(&self) -> MirroringMode {
        self.cartridge.mirroring_mode
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save(&self) -> bincode::Result<Option<Vec<u8>>> {
        self.cartridge.save()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load(&mut self, save_data: &[u8]) -> bincode::Result<()> {
        self.cartridge.load(save_data)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_state(&self) -> bincode::Result<(Vec<u8>, Vec<u8>)> {
        Ok((bincode::serialize(&self)?, self.cartridge.save_state()?))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_state(&mut self, mapper_data: &[u8], save_data: &[u8]) -> bincode::Result<()> {
        let mut saved_mapper = bincode::deserialize(mapper_data)?;
        std::mem::swap(self, &mut saved_mapper);
        std::mem::swap(&mut self.cartridge, &mut saved_mapper.cartridge);
        self.load(&save_data)?;
        Ok(())
    }
}
