# Rust-Chip-8-Emulator

Personal project done as a way to regain my familiarity with Rust and to prepare for my Bachelor's final project - a GBA Emulator.

Chip-8 Emulator written 100% in Rust that runs in the browser through WebAssembly. Basic React frontend.

User can load their own games in the emulator, but only COSMAC VIP, DREAM 6800, and ETI 660 compatible games.

Might return later and implement SUPER-CHIP-48 support later.

# How To Use
- Import Chip-8 Game through the attached button
- Play the game:
    - Controls:

     ORIGINAL              Modern Keyboard
|---|---|---|---|         |---|---|---|---|
| 1 | 2 | 3 | C |         | 1 | 2 | 3 | 4 | 
| 4 | 5 | 6 | D | ----->  | Q | W | E | R | 
| 7 | 8 | 9 | E |         | A | S | D | F |
| A | 0 | B | F |         | Z | X | C | V |

# Known issues:

- Emulator may sometimes crash
- Pressing the escape key prompts an error, simply just refresh the page

# What I learned

- Basic idea of how an emulator is written
- Rust
- Tried to apply SOLID principles when writing stuff
- Basic idea of how WASM (wasm-pack and wasm-bindgen) is used to compile Rust code for browsers

# Resources

- https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#8xy0-set
- http://devernay.free.fr/hacks/chip8/C8TECH10.HTM

\*AI was used in this project only for asking questions about stuff I didn't understand.

\*AI was used to implement the audio web API thing, that was not the main focus of this project
