pub mod chip_8_emulator;

use crate::chip_8_emulator::*;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::time::Instant;

const FPS: u8 = 60;

const TESTS: [&str; 8] = [
    "../chip-8_tests/1-chip8-logo.ch8",
    "../chip-8_tests/2-ibm-logo.ch8",
    "../chip-8_tests/3-corax+.ch8",
    "../chip-8_tests/4-flags.ch8",
    "../chip-8_tests/5-quirks.ch8",
    "../chip-8_tests/6-keypad.ch8",
    "../chip-8_tests/7-beep.ch8",
    "../chip-8_tests/8-scrolling.ch8",
];

fn main() -> Result<(), String> {
    let mut engine: Chip8Engine = Chip8Engine::new(true);

    engine.load_rom(TESTS[6]);

    let delta: f32 = 1f32 / FPS as f32;

    let mut now = Instant::now();

    loop {
        engine.tick()?;

        if now.elapsed().as_secs_f32() >= delta {
            engine.set_delay_timer(engine.get_delay_timer().saturating_sub(1));
            engine.set_sound_timer(engine.get_sound_timer().saturating_sub(1));

            now = Instant::now();
        }

        if engine.get_draw_flag() {
            for i in 0..SCREEN_HEIGHT {
                for j in 0..SCREEN_WIDTH {
                    let curr_2d_arr_index: usize = i * SCREEN_WIDTH + j;
                    let display_char: char = if engine.get_display()[curr_2d_arr_index] == 1 {
                        '#'
                    } else {
                        '.'
                    };
                    print!("{} ", display_char);
                }
                println!();
            }

            //if pressed_key == esc { break; }

            std::io::stdout().flush().expect("failed to flush");
            print!("\x1B[2J\x1B[1;1H");

            engine.reset_draw_flag();
        }
    }

    Ok(())
}
