use cartridge::Cartridge;
#[cfg(target_arch = "wasm32")]
use debug;
use mapper::Mapper;
use ppu::MirroringMode;

#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Serialize))]
pub enum Variant {
    UNROM,
    UN1ROM,
    Mapper180,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Serialize))]
pub struct UxROM {
    #[cfg_attr(
        not(target_arch = "wasm32"),
        serde(skip, default = "Cartridge::empty_cartridge")
    )]
    cartridge: Cartridge,
    variant: Variant,
    prg_rom_bank: u8,
}

impl UxROM {
    pub fn new(cartridge: Cartridge, variant: Variant) -> Self {
        UxROM {
            cartridge,
            variant,
            prg_rom_bank: 0,
        }
    }
}

impl Mapper for UxROM {
    fn read_byte(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => self.cartridge.read_chr_rom(addr),
            0x8000..=0xBFFF => {
                let bank = match self.variant {
                    Variant::UNROM | Variant::UN1ROM => self.prg_rom_bank as usize,
                    Variant::Mapper180 => 0,
                };
                self.cartridge.read_prg_rom(bank * 0x4000 + addr - 0x8000)
            },
            0xC000..=0xFFFF => {
                let bank = match self.variant {
                    Variant::UNROM | Variant::UN1ROM => self.cartridge.prg_rom_len() / 0x4000 - 1,
                    Variant::Mapper180 => self.prg_rom_bank as usize,
                };
                self.cartridge.read_prg_rom(bank * 0x4000 + addr - 0xC000)
            },
            _ => 0,
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => self.cartridge.write_chr_rom(addr, val),
            0x8000..=0xFFFF => {
                match self.variant {
                    Variant::UNROM | Variant::Mapper180 => self.prg_rom_bank = val & 0x07,
                    Variant::UN1ROM => self.prg_rom_bank = (val >> 2) & 0x07,
                }
                debug!("[UxROM] Write prg rom bank: {}.", self.prg_rom_bank);
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
