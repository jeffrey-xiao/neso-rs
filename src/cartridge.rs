use ppu::MirroringMode;

const CARTRIDGE_HEADER: u32 = 0x1A53_454E;

pub struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    pub mapper: u8,
    pub mirroring_mode: MirroringMode,
}

impl Cartridge {
    pub fn from_buffer(mut buffer: &[u8]) -> Self {
        let header = u32::from(buffer[0])
            | (u32::from(buffer[1]) << 8)
            | (u32::from(buffer[2]) << 16)
            | (u32::from(buffer[3]) << 24);
        assert_eq!(
            header, CARTRIDGE_HEADER,
            "Error reading cartridge: expected header[0..4] = 0x1A53454E."
        );

        for val in buffer[11..=15].iter() {
            assert_eq!(
                *val, 0,
                "Error reading cartridge: expected header[11..16] = 0x0."
            );
        }

        let prg_rom_len = buffer[4] as usize * 0x4000;
        let chr_rom_len = buffer[5] as usize * 0x2000;
        let mut prg_ram_len = buffer[8] as usize * 0x2000;

        if prg_ram_len == 0 {
            prg_ram_len = 0x4000;
        }

        let flags_6 = buffer[6];
        let flags_7 = buffer[7];
        let flags_9 = buffer[9];
        let flags_10 = buffer[10];

        buffer = buffer.split_at(16).1;

        if flags_6 & 0x04 != 0 {
            buffer = buffer.split_at(512).1;
        }

        let (prg_rom_buffer, buffer) = buffer.split_at(prg_rom_len);
        let prg_rom = prg_rom_buffer.to_vec();

        let chr_rom = if chr_rom_len > 0 {
            let (chr_rom_buffer, _) = buffer.split_at(chr_rom_len);
            chr_rom_buffer.to_vec()
        } else {
            vec![0; 0x2000]
        };

        let mapper = (flags_7 & 0xF0) | (flags_6 >> 4);

        let mirroring_mode = {
            if flags_6 & 0x08 != 0 {
                MirroringMode::None
            } else if flags_6 & 0x01 != 0 {
                MirroringMode::Vertical
            } else {
                MirroringMode::Horizontal
            }
        };

        Cartridge {
            prg_rom,
            chr_rom,
            prg_ram: vec![0; prg_ram_len],
            mapper,
            mirroring_mode,
        }
    }

    pub fn prg_rom_len(&self) -> usize {
        self.prg_rom.len()
    }

    pub fn read_prg_rom(&self, addr: usize) -> u8 {
        self.prg_rom[addr]
    }

    pub fn chr_rom_len(&self) -> usize {
        self.chr_rom.len()
    }

    pub fn read_chr_rom(&self, addr: usize) -> u8 {
        self.chr_rom[addr]
    }

    // chr_rom is ram if the size reported in the header is 0.
    pub fn write_chr_rom(&mut self, addr: usize, val: u8) {
        self.chr_rom[addr] = val;
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
}
