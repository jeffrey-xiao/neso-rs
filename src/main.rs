extern crate nes_wasm;
extern crate sdl2;

use nes_wasm::cpu::Interrupt;
use nes_wasm::Nes;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::fs;

pub fn main() {
    let buffer = fs::read("./tests/dk.nes").unwrap();
    let mut nes = Nes::new();
    nes.load_rom(&buffer);
    for i in 0..30000 {
        nes.execute_cycle();
    }
    println!("DONE EXECUTING MAIN");

    nes.cpu.borrow_mut().trigger_interrupt(Interrupt::NMI);

    for i in 0..20000 {
        nes.execute_cycle();
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    canvas.clear();

    let mut pattern_table = Vec::with_capacity(512);

    for i in 0..512 {
        let mut tile = [0; 64];

        for index in 0..8 {
            let byte = nes.ppu.borrow().memory_map().read_byte(index + i * 16);
            for y in 0..8 {
                tile[index as usize * 8 + 7 - y] |= if byte & 1 << y != 0 { 1 } else { 0 };
            }
        }

        for index in 0..8 {
            let byte = nes.ppu.borrow().memory_map().read_byte(index + 8 + i * 16);
            for y in 0..8 {
                tile[index as usize * 8 + 7 - y] |= if byte & 1 << y != 0 { 2 } else { 0 };
            }
        }

        pattern_table.push(tile);
    }

    let offset = 0x2000;
    for i in offset..offset + 960 {
        let index = nes.ppu.borrow().memory_map().read_byte(i);

        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 256, 256)
            .unwrap();

        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..8 {
                    for x in 0..8 {
                        let offset = y * pitch + x * 3;
                        let val = match pattern_table[index as usize + 256][y * 8 + x] {
                            0 => 255,
                            1 => 200,
                            2 => 150,
                            3 => 100,
                            _ => panic!("ERROR"),
                        };
                        buffer[offset] = val;
                        buffer[offset + 1] = val;
                        buffer[offset + 2] = val;
                    }
                }
            })
            .unwrap();
        canvas
            .copy(
                &texture,
                None,
                Some(Rect::new(
                    (i - offset) as i32 % 32 * 8 * 2,
                    (i - offset) as i32 / 32 * 8 * 2,
                    256 * 2,
                    256 * 2,
                )),
            )
            .unwrap();
    }

    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {},
            }
        }
    }
}
