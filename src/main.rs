extern crate clap;
extern crate nes_wasm;
extern crate sdl2;

use clap::{App, Arg};
use nes_wasm::Nes;
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::fs;
use std::ptr;
use std::slice;
use std::thread;
use std::time::{Duration, Instant};

const KEYS: [Keycode; 8] = [
    Keycode::Q,
    Keycode::W,
    Keycode::E,
    Keycode::R,
    Keycode::Up,
    Keycode::Down,
    Keycode::Left,
    Keycode::Right,
];

pub fn main() {
    let matches = App::new("nes-rs")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Jeffrey Xiao <jeffrey.xiao1998@gmail.com>")
        .about("A NES emulator build with Rust and sdl2.")
        .arg(
            Arg::with_name("rom-path")
                .help("Path to rom.")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name("debug")
                .help("Enable debug views.")
                .short("d")
                .long("debug"),
        )
        .get_matches();

    let debug_enabled = matches.is_present("debug");
    let rom_path = matches.value_of("rom-path").unwrap();

    let mus_per_frame = Duration::from_micros((1.0f64 / 60.0 * 1e6).round() as u64);
    let buffer = fs::read(rom_path).unwrap();
    let mut nes = Nes::new();
    nes.load_rom(&buffer);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let window_dimensions = if debug_enabled {
        (1000, 800)
    } else {
        (480, 512)
    };
    let window = video_subsystem
        .window("nes-rs", window_dimensions.0, window_dimensions.1)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    canvas.present();

    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1),
        samples: Some(1024),
    };
    let device = audio_subsystem
        .open_queue::<f32, _>(None, &desired_spec)
        .unwrap();
    device.resume();

    let mut event_pump = sdl_context.event_pump().unwrap();

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
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(index) = KEYS.iter().position(|key| *key == keycode) {
                        nes.press_button(0, index as u8);
                        nes.press_button(1, index as u8);
                    }
                },
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(index) = KEYS.iter().position(|key| *key == keycode) {
                        nes.release_button(0, index as u8);
                        nes.release_button(1, index as u8);
                    }
                },
                _ => {},
            }
        }

        nes.step_frame();
        let buffer_len = nes.audio_buffer_len();
        let slice = unsafe { slice::from_raw_parts(nes.audio_buffer(), buffer_len) };
        device.queue(&slice[0..buffer_len]);

        canvas.clear();

        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::ABGR8888, 256, 240)
            .unwrap();
        texture
            .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                unsafe {
                    ptr::copy_nonoverlapping(
                        nes.image_buffer(),
                        buffer.as_mut_ptr(),
                        256 * 240 * 4,
                    );
                }
            })
            .unwrap();
        canvas
            .copy(&texture, None, Some(Rect::new(0, 0, 240 * 2, 256 * 2)))
            .unwrap();

        if debug_enabled {
            let mut pattern_table = Vec::with_capacity(512);

            for i in 0..8 {
                let chr_bank = unsafe { slice::from_raw_parts(nes.chr_bank(i), 0x400) };
                for j in 0..64 {
                    let mut tile = [0; 64];

                    for index in 0..8 {
                        let byte = chr_bank[j * 16 + index];
                        for y in 0..8 {
                            tile[index as usize * 8 + 7 - y] |=
                                if byte & 1 << y != 0 { 1 } else { 0 };
                        }
                    }

                    for index in 0..8 {
                        let byte = chr_bank[j * 16 + index + 8];
                        for y in 0..8 {
                            tile[index as usize * 8 + 7 - y] |=
                                if byte & 1 << y != 0 { 2 } else { 0 };
                        }
                    }
                    pattern_table.push(tile);
                }
            }

            for nametable in 0..4 {
                let nametable_bank =
                    unsafe { slice::from_raw_parts(nes.nametable_bank(nametable), 0x800) };
                let mut texture = texture_creator
                    .create_texture_streaming(PixelFormatEnum::RGB24, 256, 240)
                    .unwrap();

                texture
                    .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                        for i in 0..30usize {
                            for j in 0..32usize {
                                let mut index = nametable_bank[i * 32 + j] as usize
                                    + nes.background_chr_bank() * 64;
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
        }

        canvas.present();

        let elapsed = start.elapsed();
        if mus_per_frame > elapsed {
            thread::sleep(mus_per_frame - elapsed);
        }
    }
}
