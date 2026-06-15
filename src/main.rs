use rand::RngExt;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::thread::sleep;
use std::time::Duration;

struct Chip8Engine {
    memory: [u8; 4096],
    program_counter: u16,
    registers: [u8; 16], // 0xV0 - 0xVF
    index: u16,
    stack: Vec<u16>,
    display: [[bool; 64]; 32],
    delay_timer: u8,
    sound_timer: u8,
    old: bool,
    draw: bool,
}

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

impl Chip8Engine {
    fn new(old: bool) -> Self {
        Chip8Engine {
            memory: [0; 4096],
            program_counter: 0x200,
            registers: [0; 16],
            index: 0,
            stack: Vec::new(),
            display: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            delay_timer: 0,
            sound_timer: 0,
            old,
            draw: false,
        }
    }

    fn load_rom(&mut self, data: &[u8]) {
        let mut offset = 0x200;
        for byte in data {
            self.memory[offset] = *byte;
            offset += 1;
        }
    }

    fn tick(&mut self) -> bool {
        if self.program_counter >= 4094 {
            return false;
        }

        let byte1: u8 = self.memory[self.program_counter as usize];
        let byte2: u8 = self.memory[(self.program_counter + 1) as usize];

        let instruction: u16 = (byte1 as u16) << 8 | (byte2 as u16);

        self.program_counter += 2;

        let n1: u8 = ((instruction & 0xF000) >> 12) as u8;
        let n2: u8 = ((instruction & 0x0F00) >> 8) as u8;
        let n3: u8 = ((instruction & 0x00F0) >> 4) as u8;
        let n4: u8 = (instruction & 0x000F) as u8;

        match (n1, n2, n3, n4) {
            // 00E0 - clear screen
            (0x0, 0x0, 0xE, 0x0) => {
                self.display = [[false; 64]; 32];
                self.draw = true;
            }

            // 1NNN - JUMP
            (0x1, ..) => {
                let addr: u16 = instruction & 0x0FFF;
                self.program_counter = addr;
            }

            // 2NNN - jump and add to stack
            (0x2, ..) => {
                self.stack.push(self.program_counter);
                let addr: u16 = instruction & 0x0FFF;
                self.program_counter = addr;
            }

            // 00EE - pop from stack and get back to that addr
            (0x0, 0x0, 0xE, 0xE) => {
                self.program_counter = self.stack.pop().unwrap_or(0x200);
            }

            // 3XNN - skip if VX == NN
            (0x3, ..) => {
                let nn: u16 = instruction & 0x00FF;
                if self.registers[n2 as usize] as u16 == nn {
                    self.program_counter += 2;
                }
            }

            // 4XNN - skip if VX != NN
            (0x4, ..) => {
                let nn: u16 = instruction & 0x00FF;
                if self.registers[n2 as usize] as u16 != nn {
                    self.program_counter += 2;
                }
            }

            // 5XY0 - skip if VX == VY
            (0x5, _, _, 0x0) => {
                if self.registers[n2 as usize] == self.registers[n3 as usize] {
                    self.program_counter += 2;
                }
            }

            // 9XY0 - skip if VX == VY
            (0x9, _, _, 0x0) => {
                if self.registers[n2 as usize] != self.registers[n3 as usize] {
                    self.program_counter += 2;
                }
            }

            // 6XNN - set NN into VX
            (0x6, ..) => {
                let nn: u16 = instruction & 0x00FF;
                self.registers[n2 as usize] = nn as u8;
            }

            // 7XNN - add NN to VX
            (0x7, ..) => {
                let nn: u16 = instruction & 0x00FF;
                self.registers[n2 as usize] = self.registers[n2 as usize].wrapping_add(nn as u8);
            }

            // 8XY0 - set VX to VY
            (0x8, _, _, 0x0) => {
                self.registers[n2 as usize] = self.registers[n3 as usize];
            }

            // 8XY1 - binary or over vx, vy remains unchanged
            (0x8, _, _, 0x1) => {
                let x: u8 = self.registers[n2 as usize];
                let y: u8 = self.registers[n3 as usize];
                self.registers[n2 as usize] = x | y;
            }

            // 8XY2 - binary and over vx, vy remains unchanged
            (0x8, _, _, 0x2) => {
                let x: u8 = self.registers[n2 as usize];
                let y: u8 = self.registers[n3 as usize];
                self.registers[n2 as usize] = x & y;
            }

            // 8XY3 - binary xor over vx, vy remains unchanged
            (0x8, _, _, 0x3) => {
                let x: u8 = self.registers[n2 as usize];
                let y: u8 = self.registers[n3 as usize];
                self.registers[n2 as usize] = x ^ y;
            }

            // 8XY4 - add VY to VX, if it overflows VF is set to 1
            (0x8, _, _, 0x4) => {
                let y: u8 = self.registers[n3 as usize];
                let (result, overflows): (u8, bool) =
                    self.registers[n2 as usize].overflowing_add(y);

                self.registers[n2 as usize] = result;
                self.registers[0xF] = overflows as u8;
            }

            // 8XY5 - VX = VX - VY
            (0x8, _, _, 0x5) => {
                let x = self.registers[n2 as usize];
                let y = self.registers[n3 as usize];
                let (result, underflows) = x.overflowing_sub(y);

                self.registers[n2 as usize] = result;
                self.registers[0xF] = (!underflows) as u8;
            }

            // 8XY7 - VX = VY - VX
            (0x8, _, _, 0x7) => {
                let x = self.registers[n2 as usize];
                let y = self.registers[n3 as usize];
                let (result, underflows) = y.overflowing_sub(x);

                self.registers[n2 as usize] = result;
                self.registers[0xF] = (!underflows) as u8;
            }

            // 8XY6 - Shift VX to the right by 1 / (old) move VY into VX then shift right by one
            (0x8, _, _, 0x6) => {
                let target = if self.old {
                    self.registers[n3 as usize]
                } else {
                    self.registers[n2 as usize]
                };

                let shifted_out = target & 1;
                self.registers[n2 as usize] = target >> 1;
                self.registers[0xF] = shifted_out;
            }

            // 8XYE - Shift VX to the left by 1 // same as above, but shift to left
            (0x8, _, _, 0xE) => {
                let target = if self.old {
                    self.registers[n3 as usize]
                } else {
                    self.registers[n2 as usize]
                };

                let shifted_out = (target & 0x80) >> 7;
                self.registers[n2 as usize] = target << 1;
                self.registers[0xF] = shifted_out;
            }

            // ANNN - Set index to NNN
            (0xA, ..) => {
                let nnn: u16 = instruction & 0x0FFF;

                self.index = nnn;
            }

            // BNNN (old) jump to NNN addr + value in V0 / BXNN (new) jump to XNN + value in VX
            (0xB, ..) => {
                let nnn: u16 = instruction & 0x0FFF;
                if self.old {
                    self.program_counter = nnn + (self.registers[0] as u16);
                } else {
                    self.program_counter = nnn + (self.registers[n2 as usize] as u16);
                }
            }

            // CXNN - generates a random number, applies & to it with NN and puts the result in VX
            (0xC, ..) => {
                let mut rng = rand::rng();
                let nn: u16 = instruction & 0x00FF;
                let random_number: u16 = rng.random::<u8>() as u16;
                let result: u8 = (random_number & nn) as u8;

                self.registers[n2 as usize] = result;
            }

            // DXYN - draw an N pixel tall sprite from the memory location that index is pointing to
            //        at the horizontal X coord inside VX and Y coord inside VY. Pixels that are 'on'
            //        the sprite will flip the pixels on the screen. If any pixels were turned 'off'
            //        VF is set to 1, otherwise it's set to 0
            //        Starting position: wraps ( x % 64 ) | ( y % 32 )
            //        anything else doesn't wrapp, it clips
            (0xD, ..) => {
                let x: u8 = self.registers[n2 as usize] % (SCREEN_WIDTH as u8);
                let y: u8 = self.registers[n3 as usize] % (SCREEN_HEIGHT as u8);
                let height: u8 = n4;

                self.registers[0xF] = 0;
                for height_y in 0..height {
                    let row: u8 = self.memory[(self.index + height_y as u16) as usize];
                    let curr_y: u8 = y + height_y;

                    if curr_y == SCREEN_HEIGHT as u8 {
                        break;
                    }

                    for bit in 0..8 {
                        let curr_bit: bool = (row << bit) & 0b1000_0000 != 0;

                        let curr_x: u8 = x + bit;

                        if curr_x == SCREEN_WIDTH as u8 {
                            break;
                        }

                        if self.display[curr_y as usize][curr_x as usize] && curr_bit {
                            self.display[curr_y as usize][curr_x as usize] = false;
                            self.registers[0xF] = 1;
                        } else if !self.display[curr_y as usize][curr_x as usize] && curr_bit {
                            self.display[curr_y as usize][curr_x as usize] = true;
                        }
                    }
                }

                self.draw = true;
            }
            _ => {}
        }

        true
    }
}

fn main() -> Result<(), Error> {
    let mut engine: Chip8Engine = Chip8Engine::new(true);

    let mut file = File::open("tests/1-chip8-logo.ch8")?;
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer)?;

    engine.load_rom(&buffer);

    loop {
        if !engine.tick() {
            break;
        }

        if engine.draw {
            sleep(Duration::new(0, 50_000_000));
            for i in 0..SCREEN_HEIGHT {
                for j in 0..SCREEN_WIDTH {
                    let display_char: char = if engine.display[i][j] { '#' } else { '.' };
                    print!("{} ", display_char);
                }
                println!();
            }

            std::io::stdout().flush().expect("failed to flush");
            print!("\x1B[2J\x1B[1;1H");

            engine.draw = false;
        }
    }

    Ok(())
}
