use cartridge::Cartridge;
#[cfg(target_arch = "wasm32")]
use debug;
use mapper::Mapper;
use ppu::MirroringMode;

#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Serialize))]
pub struct AxROM {
    #[cfg_attr(
        not(target_arch = "wasm32"),
        serde(skip, default = "Cartridge::empty_cartridge")
    )]
    cartridge: Cartridge,
    mirroring_mode: MirroringMode,
    prg_rom_bank: u8,
}

impl AxROM {
    pub fn new(cartridge: Cartridge) -> Self {
        AxROM {
            cartridge,
            mirroring_mode: MirroringMode::Lower,
            prg_rom_bank: 0,
        }
    }
}

impl Mapper for AxROM {
    fn read_byte(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => self.cartridge.read_chr_rom(addr),
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
            0x0000..=0x1FFF => self.cartridge.write_chr_rom(addr, val),
            0x8000..=0xFFFF => {
                self.mirroring_mode = if val & 0x10 == 0 {
                    MirroringMode::Lower
                } else {
                    MirroringMode::Upper
                };
                debug!("[AxROM] Write mirroring mode: {:?}.", self.mirroring_mode);

                self.prg_rom_bank = val & 0x07;
                debug!("[AxROM] Write prg rom bank: {}.", self.prg_rom_bank);
            },
            _ => {},
        }
    }

    fn chr_bank(&self, index: usize) -> *const u8 {
        self.cartridge.chr_bank(index)
    }

    fn mirroring_mode(&self) -> MirroringMode {
        self.mirroring_mode
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
