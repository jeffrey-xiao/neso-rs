use cartridge::Cartridge;
#[cfg(target_arch = "wasm32")]
use debug;
use mapper::Mapper;
use ppu::MirroringMode;

#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Serialize))]
pub struct CNROM {
    #[cfg_attr(
        not(target_arch = "wasm32"),
        serde(skip, default = "Cartridge::empty_cartridge")
    )]
    cartridge: Cartridge,
    chr_rom_bank: u8,
}

impl CNROM {
    pub fn new(cartridge: Cartridge) -> Self {
        CNROM {
            cartridge,
            chr_rom_bank: 0,
        }
    }
}

impl Mapper for CNROM {
    fn read_byte(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => {
                let addr = self.chr_rom_bank as usize * 0x2000 + addr;
                self.cartridge.read_chr_rom(addr)
            },
            0x8000..=0xFFFF if self.cartridge.prg_rom_len() == 0x8000 => {
                self.cartridge.read_prg_rom(addr - 0x8000)
            },
            0x8000..=0xFFFF => self.cartridge.read_prg_rom((addr - 0x8000) % 0x4000),
            _ => 0,
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        let addr = addr as usize;
        if let 0x8000..=0xFFFF = addr {
            self.chr_rom_bank = val & 0x03;
            debug!("[CNROM] Write chr rom bank: {}.", self.chr_rom_bank);
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
    fn save_state(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&self)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_state(
        &mut self,
        mapper_data: &[u8],
        save_data_opt: Option<Vec<u8>>,
    ) -> bincode::Result<()> {
        let mut saved_mapper = bincode::deserialize(mapper_data)?;
        std::mem::swap(self, &mut saved_mapper);
        std::mem::swap(&mut self.cartridge, &mut saved_mapper.cartridge);
        if let Some(save_data) = save_data_opt {
            self.load(&save_data)?;
        }
        Ok(())
    }
}
