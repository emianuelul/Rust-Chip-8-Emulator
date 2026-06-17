use rand::RngExt;
use std::fs::File;
use std::io::Read;
use wasm_bindgen::prelude::wasm_bindgen;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const FONT_SET: [u8; 80] = [
    // 0x50
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0 - 0x50
    0x20, 0x60, 0x20, 0x20, 0x70, // 1 - 0x50 + 5
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2 - 0x50 + 10
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3 - ...
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    // 0x1FF
];

#[wasm_bindgen]
pub struct Chip8Engine {
    memory: Vec<u8>,
    program_counter: u16,
    registers: Vec<u8>,  // 0xV0 - 0xVF
    keyboard: Vec<bool>, // true - held, false - released
    key_pressed: (bool, u8),
    index: u16,
    stack: Vec<u16>,
    display: Vec<bool>,
    delay_timer: u8,
    sound_timer: u8,
    old: bool,
    draw: bool,
}

#[wasm_bindgen]
impl Chip8Engine {
    pub fn new(old: bool) -> Self {
        let mut result = Chip8Engine {
            memory: [0; 4096].to_vec(),
            program_counter: 0x200,
            registers: [0; 16].to_vec(),
            keyboard: [false; 16].to_vec(),
            key_pressed: (false, 0),
            index: 0,
            stack: Vec::new(),
            display: [false; SCREEN_WIDTH * SCREEN_HEIGHT].to_vec(),
            delay_timer: 0,
            sound_timer: 0,
            old,
            draw: false,
        };

        result.memory[0x50..0x50 + 80].copy_from_slice(&FONT_SET);

        result
    }

    fn load_bytes(&mut self, data: &[u8]) {
        for (offset, byte) in (0x200..).zip(data.iter()) {
            self.memory[offset] = *byte;
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let mut file = File::open(path).unwrap();
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        self.load_bytes(&buffer);
    }

    pub fn tick(&mut self) -> Result<bool, String> {
        if self.program_counter >= 4094 {
            return Err("Program Counter tried to go over allowed memory limit".to_string());
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
                self.display = [false; SCREEN_WIDTH * SCREEN_HEIGHT].to_vec();
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
            //        anything else doesn't wrap, it clips
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

                        let curr_2darr_index: usize = (curr_y as u16 * SCREEN_WIDTH as u16 + curr_x as u16) as usize;

                        if self.display[curr_2darr_index] && curr_bit {
                            self.display[curr_2darr_index] = false;
                            self.registers[0xF] = 1;
                        } else if !self.display[curr_y as usize] && curr_bit {
                            self.display[curr_2darr_index] = true;
                        }
                    }
                }

                self.draw = true;
            }

            //EX9E - Skip one if VX is pressed
            (0xE, _, 0x9, 0xE) => {
                if self.keyboard[self.registers[n2 as usize] as usize] {
                    self.program_counter += 2;
                }
            }

            //EXA1 - Skip one if VX is NOT pressed
            (0xE, _, 0xA, 0x1) => {
                if !self.keyboard[self.registers[n2 as usize] as usize] {
                    self.program_counter += 2;
                }
            }

            // FX07 - sets VX to curr value of delay timer
            (0xF, _, 0x0, 0x7) => {
                self.registers[n2 as usize] = self.delay_timer;
            }

            // FX15 - sets delay timer to VX
            (0xF, _, 0x1, 0x5) => {
                self.delay_timer = self.registers[n2 as usize];
            }

            // FX18 - sets sound timer to VX
            (0xF, _, 0x1, 0x8) => {
                self.sound_timer = self.registers[n2 as usize];
            }

            // FX1E - add VX to I
            (0xF, _, 0x1, 0xE) => {
                self.index += self.registers[n2 as usize] as u16;
            }

            // FX0A - wait for keypress, add it to VX
            (0xF, _, 0x0, 0xA) => {
                self.program_counter -= 2;
                if self.key_pressed.0 {
                    self.registers[n2 as usize] = self.key_pressed.1;
                    self.key_pressed.0 = false;
                    self.program_counter += 2;
                }
            }

            // FX29 - I is set to the addr of the hex char value of VX
            (0xF, _, 0x2, 0x9) => {
                self.index = (0x50 + self.registers[n2 as usize] * 5) as u16;
            }

            // FX33 - Takes number in VX and converts it to 3 decimal digits, stores it in I
            //        ex.: If VX contains 156 (0x9C), it would store 1 inside I, 5 in I+1, 6 in I+2
            (0xF, _, 0x3, 0x3) => {
                let number: u8 = self.registers[n2 as usize];
                self.memory[self.index as usize] = number / 100;
                self.memory[(self.index + 1) as usize] = (number % 100) / 10;
                self.memory[(self.index + 2) as usize] = number % 10;
            }

            // FX55 - every value from V0 to VX (inclusive) will be stored in memory, starting from I
            (0xF, _, 0x5, 0x5) => {
                // old increments I, new ones don't
                if self.old {
                    let vars = &self.registers[0x0..(n2 + 1) as usize];

                    for value in vars.iter() {
                        self.memory[self.index as usize] = *value;
                        self.index += 1;
                    }
                } else {
                    let vars = &self.registers[0x0..(n2 + 1) as usize];

                    for (offset, value) in (self.index..).zip(vars.iter()) {
                        self.memory[offset as usize] = *value;
                    }
                }
            }

            // FX65 - takes X values from memory and stores into their respective register, starting from I
            (0xF, _, 0x6, 0x5) => {
                // old increments I, new ones don't
                if self.old {
                    let vars =
                        &self.memory[self.index as usize..(self.index + (n2 + 1) as u16) as usize];

                    for (index, value) in vars.iter().enumerate() {
                        self.registers[index] = *value;
                        self.index += 1;
                    }
                } else {
                    let vars =
                        &self.memory[self.index as usize..(self.index + (n2 + 1) as u16) as usize];

                    for (index, value) in vars.iter().enumerate() {
                        self.registers[index] = *value;
                    }
                }
            }

            _ => {
                let err_msg: String = format!("Unknown instruction: {:04X}", instruction);
                return Err(err_msg);
            }
        }

        Ok(true)
    }
}

#[wasm_bindgen]
impl Chip8Engine {
    pub fn press_key(&mut self, key: usize) {
        if key < 16 {
            self.keyboard[key] = true;
        }
    }

    pub fn release_key(&mut self, key: usize) {
        if key < 16 {
            self.keyboard[key] = false;
        }
    }

    pub fn set_key_pressed(&mut self, key: u8) {
        self.key_pressed = (true, key);
    }

    pub fn clear_key_pressed(&mut self) {
        self.key_pressed.0 = false;
    }

    pub fn get_delay_timer(&self) -> u8 {
        self.delay_timer
    }

    pub fn set_delay_timer(&mut self, value: u8) {
        self.delay_timer = value;
    }

    pub fn get_sound_timer(&self) -> u8 {
        self.sound_timer
    }

    pub fn set_sound_timer(&mut self, value: u8) {
        self.sound_timer = value;
    }

    pub fn get_draw_flag(&self) -> bool {
        self.draw
    }

    pub fn reset_draw_flag(&mut self) {
        self.draw = false;
    }

    pub fn is_old(&self) -> bool {
        self.old
    }

    pub fn get_display(&self) -> Vec<u8> {
        self.display.iter().map(|&pixel| if pixel {1} else {0}).collect()
    }

}