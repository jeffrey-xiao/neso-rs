#![feature(nll)]

#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        #[cfg(console_error_panic_hook)]
        extern crate console_error_panic_hook;
        extern crate wasm_bindgen;

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = console)]
            fn debug(s: &str);
            #[wasm_bindgen(js_namespace = console)]
            fn log(s: &str);
        }

        macro_rules! debug {
            ($($t:tt)*) => (debug(&format_args!($($t)*).to_string()))
        }

        macro_rules! info {
            ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
        }
    } else {
        #[macro_use]
        extern crate log;
        #[macro_use]
        extern crate serde_derive;
        extern crate serde;
        extern crate bincode;

        use std::fmt;
        use std::marker::PhantomData;
        use serde::ser::{Serialize, Serializer, SerializeTuple};
        use serde::de::{Deserialize, Deserializer, Visitor, SeqAccess, Error};

        trait BigArray<'de>: Sized {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer;
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: Deserializer<'de>;
        }

        macro_rules! big_array {
            ($($len:expr$(,)*)+) => {
                $(
                    impl<'de, T> BigArray<'de> for [T; $len]
                        where T: Default + Copy + Serialize + Deserialize<'de>
                    {
                        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                            where S: Serializer
                        {
                            let mut seq = serializer.serialize_tuple(self.len())?;
                            for elem in &self[..] {
                                seq.serialize_element(elem)?;
                            }
                            seq.end()
                        }

                        fn deserialize<D>(deserializer: D) -> Result<[T; $len], D::Error>
                            where D: Deserializer<'de>
                        {
                            struct ArrayVisitor<T> {
                                element: PhantomData<T>,
                            }

                            impl<'de, T> Visitor<'de> for ArrayVisitor<T>
                                where T: Default + Copy + Deserialize<'de>
                            {
                                type Value = [T; $len];

                                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                    formatter.write_str(concat!("an array of length ", $len))
                                }

                                fn visit_seq<A>(self, mut seq: A) -> Result<[T; $len], A::Error>
                                    where A: SeqAccess<'de>
                                {
                                    let mut arr = [T::default(); $len];
                                    for i in 0..$len {
                                        arr[i] = seq.next_element()?
                                            .ok_or_else(|| Error::invalid_length(i, &self))?;
                                    }
                                    Ok(arr)
                                }
                            }

                            let visitor = ArrayVisitor { element: PhantomData };
                            deserializer.deserialize_tuple($len, visitor)
                        }
                    }
                )+
            }
        }

        big_array! { 0x100, 0x800, 0x2000 }
    }
}

mod apu;
mod bus;
mod cartridge;
mod controller;
mod cpu;
mod mapper;
mod ppu;

use apu::Apu;
use bus::Bus;
use cartridge::Cartridge;
#[cfg(all(target_arch = "wasm32", console_error_panic_hook))]
use console_error_panic_hook::set_once;
use cpu::Cpu;
use mapper::Mapper;
use ppu::{Ppu, COLORS};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Nes {
    apu: Apu,
    cpu: Cpu,
    ppu: Ppu,
    mapper: Option<*mut Mapper>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Nes {
    pub fn new(sample_freq: f32) -> Self {
        #[cfg(all(target_arch = "wasm32", console_error_panic_hook))]
        set_once();

        let apu = Apu::new(sample_freq);
        let cpu = Cpu::new();
        let ppu = Ppu::new();
        let mapper = None;

        Nes {
            apu,
            cpu,
            ppu,
            mapper,
        }
    }

    fn attach_bus(&mut self, mapper: *mut Mapper) {
        let mut bus = Bus::new(&mut self.apu, &mut self.cpu, &mut self.ppu, mapper);
        self.apu.attach_bus(bus.clone());
        self.cpu.attach_bus(bus.clone());
        self.ppu.attach_bus(bus.clone());
        let bus_clone = bus.clone();
        bus.mapper_mut().attach_bus(bus_clone);
        self.mapper = Some(bus.mapper);
    }

    pub fn load_rom(&mut self, buffer: &[u8]) {
        if let Some(mapper) = self.mapper.take() {
            unsafe {
                Box::from_raw(mapper);
            }
        }

        let cartridge = Cartridge::from_buffer(buffer);
        let mapper = Box::into_raw(mapper::from_cartridge(cartridge));
        self.attach_bus(mapper);
        self.apu.initialize();
        self.cpu.initialize();
        self.ppu.initialize();
    }

    fn step(&mut self) {
        self.cpu.step();
        let mapper = unsafe { &mut (*self.mapper.expect("[NES] No ROM loaded.")) };
        for _ in 0..3 {
            self.ppu.step();
            mapper.step();
        }
        self.apu.step();
    }

    pub fn step_frame(&mut self) {
        self.apu.buffer_index = 0;
        let frame = self.ppu.frame;
        while self.ppu.frame == frame {
            self.step();
        }
    }

    pub fn reset(&mut self) {
        self.apu.buffer_index = 0;
        self.ppu.buffer_index = 0;
        self.apu.reset();
        self.cpu.reset();
        self.ppu.reset();
    }

    pub fn image_buffer(&self) -> *const u8 {
        self.ppu.buffer.as_ptr()
    }

    pub fn audio_buffer(&self) -> *const f32 {
        self.apu.buffer.as_ptr()
    }

    pub fn audio_buffer_len(&self) -> usize {
        self.apu.buffer_index
    }

    pub fn colors(&self) -> *const u32 {
        COLORS.as_ptr()
    }

    pub fn palettes(&self) -> *const u8 {
        self.ppu.palettes()
    }

    pub fn chr_bank(&self, index: usize) -> *const u8 {
        assert!(index < 8);
        let mapper = unsafe { &mut (*self.mapper.expect("[NES] No ROM loaded.")) };
        mapper.chr_bank(index)
    }

    pub fn nametable_bank(&self, index: usize) -> *const u8 {
        assert!(index < 8);
        self.ppu.nametable_bank(index)
    }

    pub fn object_attribute_memory(&self) -> *const u8 {
        self.ppu.primary_oam.as_ptr()
    }

    pub fn tall_sprites_enabled(&self) -> bool {
        self.ppu.r.sprite_size.1 == 16
    }

    pub fn background_chr_bank(&self) -> usize {
        if self.ppu.r.background_pattern_table_address == 0x1000 {
            4
        } else {
            0
        }
    }

    pub fn press_button(&mut self, controller_index: usize, button_index: u8) {
        self.cpu.controllers[controller_index].press_button(button_index);
    }

    pub fn release_button(&mut self, controller_index: usize, button_index: u8) {
        self.cpu.controllers[controller_index].release_button(button_index);
    }

    pub fn set_sample_freq(&mut self, sample_freq: f32) {
        self.apu.set_sample_freq(sample_freq);
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Nes {
    pub fn save(&self) -> bincode::Result<Option<Vec<u8>>> {
        let mapper = unsafe { &mut (*self.mapper.expect("[NES] No ROM loaded.")) };
        mapper.save()
    }

    pub fn load(&mut self, save_data: &[u8]) -> bincode::Result<()> {
        let mapper = unsafe { &mut (*self.mapper.expect("[NES] No ROM loaded.")) };
        mapper.load(save_data)
    }

    pub fn save_state(&self) -> bincode::Result<Vec<u8>> {
        let mapper = unsafe { &mut (*self.mapper.expect("[NES] No ROM loaded.")) };
        let (mapper_data, save_data) = mapper.save_state()?;
        bincode::serialize(&(&self.apu, &self.cpu, &self.ppu, mapper_data, save_data))
    }

    pub fn load_state(&mut self, save_state_data: &[u8]) -> bincode::Result<()> {
        let (apu, cpu, ppu, mapper_data, save_data): (Apu, Cpu, Ppu, Vec<u8>, Vec<u8>) =
            bincode::deserialize(save_state_data)?;
        self.cpu = cpu;
        self.apu = apu;
        self.ppu = ppu;
        let mapper = unsafe { &mut (*self.mapper.expect("[NES] No ROM loaded.")) };
        mapper.load_state(&mapper_data, &save_data)?;
        self.attach_bus(mapper);
        Ok(())
    }
}

impl Default for Nes {
    fn default() -> Self {
        Nes::new(44_100.0)
    }
}

impl Drop for Nes {
    fn drop(&mut self) {
        if let Some(mapper) = self.mapper {
            unsafe {
                Box::from_raw(mapper);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use Nes;

    fn run_text_test(nes: &mut Nes) {
        // Run until test status is running by polling $6000.
        let mut addr = 0x6000;
        let mut byte = nes.cpu.read_byte(addr);
        while byte != 0x80 {
            nes.step_frame();
            byte = nes.cpu.read_byte(addr);
        }

        // Run until test status is finished by polling $6000.
        byte = nes.cpu.read_byte(addr);
        while byte == 0x80 {
            nes.step_frame();
            byte = nes.cpu.read_byte(addr);
        }

        // Read output at $6004.
        let mut output = Vec::new();
        addr = 0x6004;
        byte = nes.cpu.read_byte(addr);
        while byte != b'\0' {
            output.push(byte);
            addr += 1;
            byte = nes.cpu.read_byte(addr);
        }

        assert!(String::from_utf8_lossy(&output).contains("Passed"));
    }

    // Test output is at $6004.
    macro_rules! text_tests {
        ($($test_name:ident: $path:expr$(,)*)*) => {
            $(
                #[test]
                fn $test_name() {
                    use std::fs;
                    use Nes;
                    use tests::run_text_test;

                    let buffer = fs::read($path).expect("Expected test rom to exist.");
                    let mut nes = Nes::default();
                    nes.load_rom(&buffer);
                    run_text_test(&mut nes);
                }
            )*
        }
    }

    macro_rules! reset_text_tests {
        ($($test_name:ident: ($path:expr, $frames:expr)$(,)*)*) => {
            $(
                #[test]
                fn $test_name() {
                    use std::fs;
                    use Nes;
                    use tests::run_text_test;

                    let buffer = fs::read($path).expect("Expected test rom to exist.");
                    let mut nes = Nes::default();
                    nes.load_rom(&buffer);

                    for _ in 0..$frames {
                        nes.step_frame();
                    }

                    nes.reset();
                    run_text_test(&mut nes);
                }
            )*
        }
    }

    // Compare hash of nametables after specified frames for graphical output tests.
    macro_rules! graphical_tests {
        ($($test_name:ident: ($path:expr, $frames:expr, $hash:expr)$(,)*)*) => {
            $(
                #[test]
                fn $test_name() {
                    use std::collections::hash_map::DefaultHasher;
                    use std::fs;
                    use std::hash::Hasher;
                    use Nes;

                    let buffer = fs::read($path).expect("Expected test rom to exist.");
                    let mut nes = Nes::default();
                    nes.load_rom(&buffer);

                    for _ in 0..$frames {
                        nes.step_frame();
                    }

                    let mut hasher = DefaultHasher::new();

                    for val in nes.ppu.buffer.iter() {
                        hasher.write_u8(*val);
                    }

                    assert_eq!(hasher.finish(), $hash);
                }
            )*
        }
    }

    mod cpu {
        mod branch_timing {
            fn test_path(file_name: &str) -> String {
                format!("./tests/cpu/branch_timing/{}", file_name)
            }

            graphical_tests!(
                test_01_branch_basics: (test_path("01-branch_basics.nes"), 13, 0xDB8E_7124_029B_C022),
                test_02_backward_branch: (test_path("02-backward_branch.nes"), 15, 0xDF84_2558_1C2B_C9A7),
                test_03_forward_branch: (test_path("03-forward_branch.nes"), 15, 0x528E_9396_828A_8125),
            );
        }

        mod reset {
            fn test_path(file_name: &str) -> String {
                format!("./tests/cpu/reset/{}", file_name)
            }

            reset_text_tests!(
                test_ram_after_reset: (test_path("ram_after_reset.nes"), 135),
                test_registers: (test_path("registers.nes"), 137),
            );
        }

        mod instr_timing {
            fn test_path(file_name: &str) -> String {
                format!("./tests/cpu/instr_timing/{}", file_name)
            }

            text_tests!(
                test_01_instr_timing: test_path("01-instr_timing.nes"),
                test_02_branch_timing: test_path("02-branch_timing.nes"),
            );

            graphical_tests!(
                test_cpu_timing_test: (test_path("timing_test.nes"), 612, 0x2F89_29CE_711F_FBD4),
            );
        }

        mod instr_misc {
            fn test_path(file_name: &str) -> String {
                format!("./tests/cpu/instr_misc/{}", file_name)
            }

            text_tests!(
                test_01_abs_x_wrap: test_path("01-abs_x_wrap.nes"),
                test_02_branch_wrap: test_path("02-branch_wrap.nes"),
            );
        }

        mod instr_tests {
            fn test_path(file_name: &str) -> String {
                format!("./tests/cpu/instr_test/{}", file_name)
            }

            text_tests!(
                test_01_basics: test_path("01-basics.nes"),
                test_02_implied: test_path("02-implied.nes"),
                test_03_immediate: test_path("03-immediate.nes"),
                test_04_zero_page: test_path("04-zero_page.nes"),
                test_05_zp_xy: test_path("05-zp_xy.nes"),
                test_06_absolute: test_path("06-absolute.nes"),
                test_07_abs_xy: test_path("07-abs_xy.nes"),
                test_08_ind_x: test_path("08-ind_x.nes"),
                test_09_ind_y: test_path("09-ind_y.nes"),
                test_10_branches: test_path("10-branches.nes"),
                test_11_stack: test_path("11-stack.nes"),
                test_12_jmp_jsr: test_path("12-jmp_jsr.nes"),
                test_13_rts: test_path("13-rts.nes"),
            );
        }
    }

    mod ppu {
        fn test_path(file_name: &str) -> String {
            format!("./tests/ppu/{}", file_name)
        }

        text_tests!(
            test_oam_read: test_path("oam_read.nes"),
        );

        mod general {
            fn test_path(file_name: &str) -> String {
                format!("./tests/ppu/general/{}", file_name)
            }

            graphical_tests!(
                test_palette_ram: (test_path("palette_ram.nes"), 18, 0x657D_9167_290E_F938),
                test_power_up_palette: (test_path("power_up_palette.nes"), 18, 0x657D_9167_290E_F938),
                test_sprite_ram: (test_path("sprite_ram.nes"), 18, 0x657D_9167_290E_F938),
                test_vram_access: (test_path("vram_access.nes"), 18, 0x657D_9167_290E_F938),
            );
        }

        mod sprite_hit {
            fn test_path(file_name: &str) -> String {
                format!("./tests/ppu/sprite_hit/{}", file_name)
            }

            text_tests!(
                test_01_basics: test_path("01-basics.nes"),
                test_02_alignment: test_path("02-alignment.nes"),
                test_03_corners: test_path("03-corners.nes"),
                test_04_flip: test_path("04-flip.nes"),
                test_05_left_clip: test_path("05-left_clip.nes"),
                test_06_right_edge: test_path("06-right_edge.nes"),
                test_07_screen_bottom: test_path("07-screen_bottom.nes"),
                test_08_double_height: test_path("08-double_height.nes"),
                test_10_timing_order: test_path("10-timing_order.nes"),
            );
        }

        mod sprite_overflow {
            fn test_path(file_name: &str) -> String {
                format!("./tests/ppu/sprite_overflow/{}", file_name)
            }

            text_tests!(
                test_01_basics: test_path("01-basics.nes"),
                test_02_details: test_path("02-details.nes"),
                test_05_emulator: test_path("05-emulator.nes"),
            );
        }

        mod vbl_nmi {
            fn test_path(file_name: &str) -> String {
                format!("./tests/ppu/vbl_nmi/{}", file_name)
            }

            text_tests!(
                test_01_vbl_basics: test_path("01-vbl_basics.nes"),
                test_03_clear_time: test_path("03-vbl_clear_time.nes"),
            );
        }
    }

    mod apu {
        mod reset {
            fn test_path(file_name: &str) -> String {
                format!("./tests/apu/reset/{}", file_name)
            }

            reset_text_tests!(
                test_4015_cleared: (test_path("4015_cleared.nes"), 10),
                test_4017_timing: (test_path("4017_timing.nes"), 18),
                test_irq_flag_cleared: (test_path("irq_flag_cleared.nes"), 10),
                test_len_ctrs_enabled: (test_path("len_ctrs_enabled.nes"), 13),
            );
        }

        mod general {
            fn test_path(file_name: &str) -> String {
                format!("./tests/apu/general/{}", file_name)
            }

            text_tests!(
                test_01_len_ctr: test_path("01-len_ctr.nes"),
                test_02_len_table: test_path("02-len_table.nes"),
                test_03_irq_flag: test_path("03-irq_flag.nes"),
                test_07_dmc_basics: test_path("07-dmc_basics.nes"),
            );
        }
    }
}
