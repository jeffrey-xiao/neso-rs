extern crate nes_wasm;
extern crate sdl2;

use nes_wasm::cpu::Interrupt;
use nes_wasm::Nes;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::fs;
use std::thread;
use std::time::Duration;

pub fn main() {
    let buffer = fs::read("./tests/dk.nes").unwrap();
    let mut nes = Nes::new();
    nes.load_rom(&buffer);

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

        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 256, 256)
            .unwrap();

        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..240 {
                    for x in 0..256 {
                        let offset = y * pitch + x * 3;
                        buffer[offset] = (nes.ppu.borrow().image[y * 256 + x] >> 16) as u8;
                        buffer[offset + 1] =
                            ((nes.ppu.borrow().image[y * 256 + x] >> 8) & 0xFF) as u8;
                        buffer[offset + 2] = (nes.ppu.borrow().image[y * 256 + x] & 0xFF) as u8;
                    }
                }
            })
            .unwrap();
        nes.step_frame();
        nes.step_frame();
        nes.step_frame();
        nes.step_frame();
        nes.step_frame();

        canvas.clear();
        canvas
            .copy(&texture, None, Some(Rect::new(0, 0, 240 * 2, 256 * 2)))
            .unwrap();
        canvas.present();
    }
}
