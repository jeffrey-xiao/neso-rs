use cartridge::Cartridge;
use mapper::Mapper;
use ppu::MirroringMode;

#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Serialize))]
pub struct NROM {
    #[cfg_attr(not(target_arch = "wasm32"), serde(skip, default = "Cartridge::empty_cartridge"))]
    cartridge: Cartridge,
}

impl NROM {
    pub fn new(cartridge: Cartridge) -> Self {
        NROM { cartridge }
    }
}

impl Mapper for NROM {
    fn read_byte(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => self.cartridge.read_chr_rom(addr),
            0x6000..=0x7FFF => {
                let addr = (addr - 0x6000) % self.cartridge.prg_ram_len();
                self.cartridge.read_prg_ram(addr)
            },
            0x8000..=0xFFFF => {
                let addr = (addr - 0x8000) % self.cartridge.prg_rom_len();
                self.cartridge.read_prg_rom(addr)
            },
            _ => 0,
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => self.cartridge.write_chr_rom(addr, val),
            0x6000..=0x7FFF => {
                let addr = (addr - 0x6000) % self.cartridge.prg_ram_len();
                self.cartridge.write_prg_ram(addr, val);
            },
            _ => {},
        }
    }

    fn chr_bank(&self, index: usize) -> *const u8 {
        self.cartridge.chr_bank(index)
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
    fn save_state(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&self)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_state(&mut self, mapper_data: &[u8], save_data_opt: Option<Vec<u8>>) -> bincode::Result<()> {
        let mut saved_mapper = bincode::deserialize(mapper_data)?;
        std::mem::swap(self, &mut saved_mapper);
        std::mem::swap(&mut self.cartridge, &mut saved_mapper.cartridge);
        if let Some(save_data) = save_data_opt {
            self.load(&save_data)?;
        }
        Ok(())
    }
}
