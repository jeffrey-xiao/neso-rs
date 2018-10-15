extern crate nes_wasm;
extern crate sdl2;

use std::time::{Duration, Instant};
use nes_wasm::cpu::Interrupt;
use std::ptr;
use nes_wasm::Nes;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::fs;
use std::thread;


pub fn main() {
    let mus_per_frame = Duration::from_micros((1.0f64 / 60.0 * 1e6).round() as u64);

    let buffer = fs::read("./tests/cpu/01-basics.nes").unwrap();
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
        let start = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                | Event::KeyDown { keycode: Some(Keycode::Q), .. } => nes.cpu.borrow_mut().controller.press_button(0),
                | Event::KeyDown { keycode: Some(Keycode::W), .. } => nes.cpu.borrow_mut().controller.press_button(1),
                | Event::KeyDown { keycode: Some(Keycode::E), .. } => nes.cpu.borrow_mut().controller.press_button(2),
                | Event::KeyDown { keycode: Some(Keycode::R), .. } => nes.cpu.borrow_mut().controller.press_button(3),
                | Event::KeyDown { keycode: Some(Keycode::Up), .. } => nes.cpu.borrow_mut().controller.press_button(4),
                | Event::KeyDown { keycode: Some(Keycode::Down), .. } => nes.cpu.borrow_mut().controller.press_button(5),
                | Event::KeyDown { keycode: Some(Keycode::Left), .. } => nes.cpu.borrow_mut().controller.press_button(6),
                | Event::KeyDown { keycode: Some(Keycode::Right), .. } => nes.cpu.borrow_mut().controller.press_button(7),
                | Event::KeyUp { keycode: Some(Keycode::Q), .. } => nes.cpu.borrow_mut().controller.release_button(0),
                | Event::KeyUp { keycode: Some(Keycode::W), .. } => nes.cpu.borrow_mut().controller.release_button(1),
                | Event::KeyUp { keycode: Some(Keycode::E), .. } => nes.cpu.borrow_mut().controller.release_button(2),
                | Event::KeyUp { keycode: Some(Keycode::R), .. } => nes.cpu.borrow_mut().controller.release_button(3),
                | Event::KeyUp { keycode: Some(Keycode::Up), .. } => nes.cpu.borrow_mut().controller.release_button(4),
                | Event::KeyUp { keycode: Some(Keycode::Down), .. } => nes.cpu.borrow_mut().controller.release_button(5),
                | Event::KeyUp { keycode: Some(Keycode::Left), .. } => nes.cpu.borrow_mut().controller.release_button(6),
                | Event::KeyUp { keycode: Some(Keycode::Right), .. } => nes.cpu.borrow_mut().controller.release_button(7),
                _ => {},
            }
        }


        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 256, 256)
            .unwrap();

        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                unsafe {
                    let ppu = nes.ppu.borrow();
                    // println!("{}", ppu.frame);
                    ptr::copy_nonoverlapping(ppu.image.as_ptr(), buffer.as_mut_ptr(), 256 * 240 * 3);
                }
            })
            .unwrap();
        nes.step_frame();

        canvas.clear();
        canvas
            .copy(&texture, None, Some(Rect::new(0, 0, 240 * 2, 256 * 2)))
            .unwrap();
        canvas.present();

        if mus_per_frame > start.elapsed() {
            thread::sleep(mus_per_frame - start.elapsed());
        }
    }
}
