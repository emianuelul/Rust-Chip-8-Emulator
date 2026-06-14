struct Chip8Engine {
    memory: [u8; 4096],
    program_counter: u16,
    registers: [u8; 16], // 0xV0 - 0xVF
    index: u16,
    stack: Vec<u16>,
    display: [[bool; 64]; 32],
    delay_timer: u8,
    sound_timer: u8,
    old: bool
}

impl Chip8Engine {
    fn new(old: bool) -> Self {
        Chip8Engine {
            memory: [0; 4096],
            program_counter: 0x200,
            registers: [0; 16],
            index: 0,
            stack: Vec::new(),
            display: [[false; 64]; 32],
            delay_timer: 0,
            sound_timer: 0,
            old
        }
    }

    fn load_rom(&mut self, data: &[u8]) {
        let mut offset = 0x200;
        for byte in data {
            self.memory[offset] = *byte;
            offset += 1;
        }
    }

    fn tick(&mut self) {
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
            }

            // 1NNN - JUMP
            (0x1, ..) => {
                let addr: u16 = (instruction & 0x0FFF);
                self.program_counter = addr;
            }

            // 2NNN - jump and add to stack
            (0x2, ..) => {
                self.stack.push(self.program_counter);
                let addr: u16 = (instruction & 0x0FFF);
                self.program_counter = addr;
            }

            // 00EE - pop from stack and get back to that addr
            (0x0, 0x0, 0xE, 0xE) => {
                self.program_counter = if let Some(addr) = self.stack.pop() {
                    addr
                } else {
                    0x200 // reset app if error occurs
                };
            }

            // 3XNN - skip if VX == NN
            (0x3, ..) => {
                let nn: u16 = (instruction & 0x00FF);
                if self.registers[n2 as usize] as u16 == nn {
                    self.program_counter += 2;
                }
            }

            // 4XNN - skip if VX != NN
            (0x4, ..) => {
                let nn: u16 = (instruction & 0x00FF);
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
                let nn: u16 = (instruction & 0x00FF);
                self.registers[n2 as usize] = nn as u8;
            }

            // 7XNN - add NN to VX
            (0x7, ..) => {
                let nn: u16 = (instruction & 0x00FF);
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

            // 8XY6 - Shift VX to the right by 1 /
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

            // 8XYE - Shift VX to the left by 1
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

            // BNNN - 
            _ => {}
        }
    }
}

fn main() {

}
