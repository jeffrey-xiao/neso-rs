extern crate nes_wasm;
extern crate sdl2;

use nes_wasm::Nes;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::fs;
use std::ptr;
use std::thread;
use std::time::{Duration, Instant};

pub fn main() {
    let mus_per_frame = Duration::from_micros((1.0f64 / 60.0 * 1e6).round() as u64);

    let buffer = fs::read("./tests/games/180/crazy_climber.nes").unwrap();
    let mut nes = Nes::new();
    nes.load_rom(&buffer);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 1000, 800)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut step_scanline = false;

    'running: loop {
        let start = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => nes.cpu.borrow_mut().controller.press_button(0),
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => nes.cpu.borrow_mut().controller.press_button(1),
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => nes.cpu.borrow_mut().controller.press_button(2),
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => nes.cpu.borrow_mut().controller.press_button(3),
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => nes.cpu.borrow_mut().controller.press_button(4),
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => nes.cpu.borrow_mut().controller.press_button(5),
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => nes.cpu.borrow_mut().controller.press_button(6),
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => nes.cpu.borrow_mut().controller.press_button(7),
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => step_scanline = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Q),
                    ..
                } => nes.cpu.borrow_mut().controller.release_button(0),
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => nes.cpu.borrow_mut().controller.release_button(1),
                Event::KeyUp {
                    keycode: Some(Keycode::E),
                    ..
                } => nes.cpu.borrow_mut().controller.release_button(2),
                Event::KeyUp {
                    keycode: Some(Keycode::R),
                    ..
                } => nes.cpu.borrow_mut().controller.release_button(3),
                Event::KeyUp {
                    keycode: Some(Keycode::Up),
                    ..
                } => nes.cpu.borrow_mut().controller.release_button(4),
                Event::KeyUp {
                    keycode: Some(Keycode::Down),
                    ..
                } => nes.cpu.borrow_mut().controller.release_button(5),
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => nes.cpu.borrow_mut().controller.release_button(6),
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => nes.cpu.borrow_mut().controller.release_button(7),
                Event::KeyUp {
                    keycode: Some(Keycode::Space),
                    ..
                } => step_scanline = false,
                _ => {},
            }
        }

        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 256, 240)
            .unwrap();

        texture
            .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                unsafe {
                    let ppu = nes.ppu.borrow();
                    // println!("{}", ppu.frame);
                    ptr::copy_nonoverlapping(
                        ppu.image.as_ptr(),
                        buffer.as_mut_ptr(),
                        256 * 240 * 3,
                    );
                }
            })
            .unwrap();

        if step_scanline {
            nes.step_scanline();
        } else {
            nes.step_frame();
        }

        let mut pattern_table = Vec::with_capacity(512);

        for i in 0..512 {
            let mut tile = [0; 64];

            for index in 0..8 {
                let byte = nes.ppu.borrow().read_byte(index + i * 16);
                for y in 0..8 {
                    tile[index as usize * 8 + 7 - y] |= if byte & 1 << y != 0 { 1 } else { 0 };
                }
            }

            for index in 0..8 {
                let byte = nes.ppu.borrow().read_byte(index + 8 + i * 16);
                for y in 0..8 {
                    tile[index as usize * 8 + 7 - y] |= if byte & 1 << y != 0 { 2 } else { 0 };
                }
            }

            pattern_table.push(tile);
        }

        canvas.clear();
        canvas
            .copy(&texture, None, Some(Rect::new(0, 0, 240 * 2, 256 * 2)))
            .unwrap();

        for (nametable, offset) in [0x2000, 0x2400, 0x2800, 0x2C00].iter().enumerate() {
            let mut texture = texture_creator
                .create_texture_streaming(PixelFormatEnum::RGB24, 256, 240)
                .unwrap();

            texture
                .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                    for i in 0..30usize {
                        for j in 0..32usize {
                            let mut index = nes
                                .ppu
                                .borrow()
                                .read_byte(offset + i as u16 * 32 + j as u16)
                                as usize;
                            if nes.ppu.borrow().r.background_pattern_table_address == 0x1000 {
                                index += 256;
                            }
                            for x in 0..8usize {
                                for y in 0..8usize {
                                    let offset = ((i * 8 + x) * 256 + j * 8 + y) * 3;
                                    let val = match pattern_table[index][x * 8 + y] {
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
                        }
                    }
                })
                .unwrap();

            canvas
                .copy(
                    &texture,
                    None,
                    Some(Rect::new(
                        240 * 2 + 240 * (nametable as i32 % 2),
                        256 * (nametable as i32 / 2),
                        240,
                        256,
                    )),
                )
                .unwrap();
        }

        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 256, 128)
            .unwrap();

        texture
            .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                for i in 0..16usize {
                    for j in 0..32usize {
                        for x in 0..8usize {
                            for y in 0..8usize {
                                let row = (i * 2) % 16 + j / 16;
                                let col = if i < 8 { j % 16 } else { j % 16 + 16 };
                                let offset = ((row * 8 + x) * 256 + col * 8 + y) * 3;
                                let val = match pattern_table[i * 32 + j][x * 8 + y] {
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
                    }
                }
            })
            .unwrap();
        canvas
            .copy(
                &texture,
                None,
                Some(Rect::new(0, 256 * 2, 256 * 2, 128 * 2)),
            )
            .unwrap();

        canvas.present();

        let elapsed = start.elapsed();
        if mus_per_frame > elapsed {
            thread::sleep(mus_per_frame - elapsed);
        }
    }
}
