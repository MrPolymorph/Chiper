use std::fs::File;
use std::io::{BufReader, Read};
use rand::prelude::*;
pub struct CPU {
    pub screen: [u8; CPU::DISPLAY_WIDTH * CPU::DISPLAY_HEIGHT],
    pub interrupt: bool,
    pub memory: Vec<u8>,
    pub keyboard: [u8; 16],
    pub stack: [u16; 16],
    pub delay_timer: u32,
    pub sound_timer: u32,
    pub registers: [u8; 16],
    pub i: usize, 
    pub pc: u16,
    pub stack_pointer: u8,
    pub instruction: u16,
    pub program_size: usize,
    pub x: usize,
    pub y: usize,
    pub n: usize,
    pub nn: usize,
    pub nnn: usize,
    pub program: Vec<u8>,
    pub run_count: u32
}

impl CPU {
    pub const DISPLAY_WIDTH: usize = 64;
    pub const DISPLAY_HEIGHT: usize = 32;

    pub fn load_rom(file_path: &str) -> Vec<u8> {
        println!("Loading ROM: {}", file_path);
        let f = File::open(file_path).expect("Failed to open file");
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();

        reader.read_to_end(&mut buffer).expect("Failed to read file");
        
        buffer
    }
    
    pub fn run(&mut self, file_path: &str) {
        self.run_count = 0;
        let rom = CPU::load_rom(file_path);
        let mut buffer = vec![0; 9999];
        
        for (i, byte) in rom.iter().enumerate() {
            buffer.insert(i + 0x200, *byte)
        }
        
        self.memory = buffer;
        
        loop {
            self.run_count += 1;
            self.fetch();
            self.execute();
        }
    }

    /// Fetches the next instruction from memory and increments the program counter.
    fn fetch(&mut self) {
        
        if self.pc >= self.memory.len() as u16 {
            panic!("Program counter out of bounds fuck yeah");
        }
        
        let hi_byte = self.memory[self.pc as usize] as u16;
        let lo_byte = self.memory[(self.pc + 1) as usize] as u16;

        self.instruction = (hi_byte << 8) | lo_byte;

        self.nnn = (self.instruction & 0x0FFF) as usize;
        self.nn = (self.instruction & 0x00FF)  as usize;
        self.n = (self.instruction & 0x000F) as usize;
        self.x = ((self.instruction & 0x0F00) >> 8) as usize;
        self.y = ((self.instruction & 0x00F0) >> 4) as usize;

        if !self.interrupt {
            self.pc += 2;
        }
    }

    fn execute(&mut self) {
        match self.instruction {
            0x0 => return,
            0xE0 => self.cls(), //clear the screen
            0xEE => {
                self.pc = self.stack[self.stack_pointer as usize];
                self.stack_pointer -= 1;
            },
            0x1000..=0x1FFF => self.pc = self.nnn as u16, //jump to address nn
            0x2000..=0x2FFF => {
                self.stack_pointer += 1;
                self.stack[self.stack_pointer as usize] = self.pc;
                self.pc = self.nnn as u16;
            }, //call subroutine at address nn
            0x3000..=0x3FFF => {
                if self.registers[self.x] == self.nn as u8 {
                    self.pc += 2;
                }
            },
            0x4000..=0x4FFF => {
                if self.registers[self.x] != self.nn as u8 {
                    self.pc += 2;
                }
            }, //skip next instruction if vx != nn
            05000..=0x5FFF => {
                if self.registers[self.x] == self.registers[self.y] {
                    self.pc += 2;
                }
            }, //skip next instruction if vx == vy
            0x6000..=0x6FFF => self.registers[self.x] = self.nn as u8, //load nn into vx
            0x7000..=0x7FFF => {
                self.registers[self.x] += self.nn as u8;
            }, //add nn to vx
            0x8000..=0x8FFF => {
                match self.n {
                    0 => self.registers[self.x] = self.registers[self.y], //load vy into vx
                    1 => self.registers[self.x] |= self.registers[self.y], //vx = vx | vy
                    2 => self.registers[self.x] &= self.registers[self.y], //vx = vx & vy
                    3 => self.registers[self.x] ^= self.registers[self.y], //vx = vx ^ vy
                    4 => {
                        let sum = self.registers[self.x] as u16 + self.registers[self.y] as u16;
                        self.registers[0xF] = if sum > 255 { 1 } else { 0 };
                        self.registers[self.x] = sum as u8;
                    },
                    5 => {
                        self.registers[0xF] = if self.registers[self.x] > self.registers[self.y] { 1 } else { 0 };
                        self.registers[self.x] -= self.registers[self.y];
                    },
                    6 => {
                        self.registers[0xF] = self.registers[self.x] & 0x1;
                        self.registers[self.x] >>= 1;
                    },
                    7 => {
                        self.registers[0xF] = if self.registers[self.y] > self.registers[self.x] { 1 } else { 0 };
                        self.registers[self.x] = self.registers[self.y] - self.registers[self.x];
                    },
                    0xE => {
                        self.registers[0xF] = (self.registers[self.x] & 0x80) >> 7;
                        self.registers[self.x] <<= 1;
                    },
                    _ => return
                }
            }
            0x9000..=0x9FFF => {
                if self.registers[self.x] != self.registers[self.y] {
                    self.pc += 2;
                }
            },
            0xA000..=0xAFFF => self.i = self.nnn,
            0xB000..=0xBFFF => self.pc = (self.nnn + self.registers[0] as usize) as u16,
            0xC000..=0xCFFF => self.registers[self.x] = thread_rng().gen_range(0..255) as u8,
            0xD000..=0xDFFF => self.drw(),
            0xE000..=0xEFFF => {
                match self.nn {
                    0x9E => {
                        if self.keyboard[self.registers[self.x] as usize] == 1 {
                            self.pc += 2;
                        }
                    },
                    0xA1 => {
                        if self.keyboard[self.registers[self.x] as usize] == 0 {
                            self.pc += 2;
                        }
                    },
                    _ => return
                }
            },
            0xF000..=0xFFFF => {
                match self.nn {
                    0x07 => self.registers[self.x] = self.delay_timer as u8,
                    0x0A => self.wait(),
                    0x15 => self.delay_timer = self.registers[self.x] as u32,
                    0x18 => self.sound_timer = self.registers[self.x] as u32,
                    0x1E => self.i += self.registers[self.x] as usize,
                    0x29 => self.i = self.registers[self.x] as usize,
                    0x33 => self.bcd(),
                    0x55 => self.dump(),
                    0x65 => self.load(),
                    _ => return
                }
            },
            _ => {}
        }
    }

    fn wait(&mut self) {
        for (i, key) in self.keyboard.iter().enumerate() {
            if *key != 0 {
                self.registers[self.x] = i as u8;
                self.interrupt = false;
            } else {
                self.interrupt = true;
            }
        }

        self.pc -= 2;
    }

    fn drw(&mut self) {
        println!("Run Count {}", self.run_count);
        let vx = self.registers[self.x] as usize;
        let vy = self.registers[self.y] as usize;
        self.registers[0] = 0;
        
        for y_line in 0..self.n {
            let sprite = self.memory[self.i + y_line];
            
            for x_line in 0..8 {
                if (sprite & (0x80 >> x_line)) == u8::from(true) {

                    if x_line + vx >= CPU::DISPLAY_WIDTH {
                        return;
                    }
                    
                    if self.screen[(x_line + vx) + (y_line + vy) * CPU::DISPLAY_WIDTH] == 1 {
                        self.registers[0xF] = 1;
                    }
                    
                    self.screen[(x_line + vx) + (y_line + vy) * CPU::DISPLAY_WIDTH] ^= 1;
                }
            }
        }
        
    }

    fn bcd(&mut self) {
        self.memory[self.i as usize] = self.registers[self.x] / 100;
        self.memory[(self.i + 1) as usize] = (self.registers[self.x] / 10) % 10;
        self.memory[(self.i + 2) as usize] = self.registers[self.x] % 10;
    }
    
    fn dump(&mut self){
        for i in 0..self.x {
            self.memory[self.i + i] = self.registers[i];
        }
    }
    
    fn load(&mut self){
        for i in 0..self.x {
            self.registers[i] = self.memory[self.i + i];
        }
    }
    
    pub fn reset(&mut self) {
        self.registers.iter_mut().for_each(|m| *m = 0);
        self.stack.iter_mut().for_each(|m| *m = 0);
        self.keyboard.iter_mut().for_each(|m| *m = 0);
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.i = 0;
        self.pc = 0x200;
        self.stack_pointer = 0;
        self.instruction = 0;
        self.x = 0;
        self.y = 0;
        self.n = 0;
        self.nn = 0;
        self.nnn = 0;
        
        self.cls();
    }

    fn cls(&mut self) {
        self.screen.iter_mut().for_each(|m| *m = 0);
    }
}
