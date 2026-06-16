pub mod engine;

use crate::engine::{Chip8Engine, SCREEN_HEIGHT, SCREEN_WIDTH};
use std::fs::File;
use std::io::{Error, Read, Write};
use std::time::Instant;

const FPS: u8 = 60;

const TESTS: [&str; 8] = [
    "tests/1-chip8-logo.ch8",
    "tests/2-ibm-logo.ch8",
    "tests/3-corax+.ch8",
    "tests/4-flags.ch8",
    "tests/5-quirks.ch8",
    "tests/6-keypad.ch8",
    "tests/7-beep.ch8",
    "tests/8-scrolling.ch8"
];

fn main() -> Result<(), Error> {
    let mut engine: Chip8Engine = Chip8Engine::new(true);

    let mut file = File::open(TESTS[7])?;
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer)?;

    engine.load_rom(&buffer);

    let delta: f32 = 1f32 / FPS as f32;

    let mut now = Instant::now();

    loop {
        engine.tick()?;

        if now.elapsed().as_secs_f32() >= delta  {
            engine.delay_timer = engine.delay_timer.saturating_sub(1);
            engine.sound_timer = engine.sound_timer.saturating_sub(1);

            now = Instant::now();
        }

        if engine.draw {
            for i in 0..SCREEN_HEIGHT {
                for j in 0..SCREEN_WIDTH {
                    let display_char: char = if engine.display[i][j] { '#' } else { '.' };
                    print!("{} ", display_char);
                }
                println!();
            }

            //if pressed_key == esc { break; }

            std::io::stdout().flush().expect("failed to flush");
            print!("\x1B[2J\x1B[1;1H");

            engine.draw = false;
        }
    }

    Ok(())
}
