use rand::prelude::*;
pub struct CPU {
    pub screen: [u8; CPU::DISPLAY_WIDTH * CPU::DISPLAY_HEIGHT],
    pub interrupt: bool,
    pub memory: [u8; 4096],
    pub keyboard: [u8; 16],
    pub stack: [u16; 16],
    pub delay_timer: u32,
    pub sound_timer: u32,
    pub registers: [u8; 16],
    pub i: u16, 
    pub pc: u16,
    pub stack_pointer: u8,
    pub instruction: u16,
    pub program_size: usize,
    pub x: usize,
    pub y: u8,
    pub n: u8,
    pub nn: u8,
    pub nnn: u16,
    pub program: Vec<u8>,
}

impl CPU {
    pub const DISPLAY_WIDTH: usize = 64;
    pub const DISPLAY_HEIGHT: usize = 32;
    const INSTRUCTION_START_ADDRESS: usize = 0x200;
    
    
    
    pub fn run(&mut self) {
        loop {
            self.fetch();
            self.execute();
        }
    }
    
    /// Fetches the next instruction from memory and increments the program counter.
    fn fetch(&mut self) {
        let hiByte = self.memory[self.pc as usize] as u16;
        let loByte = self.memory[(self.pc + 1) as usize] as u16;
        
        self.instruction = (hiByte << 8) | loByte;
        
        self.nnn = self.instruction & 0x0FFF;
        self.nn = (self.instruction & 0x00FF) as u8;
        self.n = (self.instruction & 0x000F) as u8;
        self.x = ((self.instruction & 0x0F00) >> 8) as u8;
        self.y = ((self.instruction & 0x00F0) >> 4) as u8;
        
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
            0x1000...0x1FFF => self.pc = self.nnn, //jump to address nn
            0x2000...0x2FFF => self.call(), //call subroutine at address nn
            0x3000...0x3FFF => {
                if self.registers[self.x] == self.nn {
                    self.pc += 2;
                }
            }, 
            0x4000...0x4FFF => {
                if self.registers[self.x] != self.nn {
                    self.pc += 2;
                }
            }, //skip next instruction if vx != nn
            05000...0x5FFF => {
                if self.registers[self.x] == self.registers[self.y] {
                    self.pc += 2;
                }
            }, //skip next instruction if vx == vy
            0x6000...0x6FFF => self.registers[self.x] = self.nn, //load nn into vx
            0x7000...0x7FFF => { 
                self.registers[self.x] += self.nn;
            }, //add nn to vx
            0x8000...0x8FFF => {
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
            0x9000...0x9FFF => {
                if self.registers[self.x] != self.registers[self.y] {
                    self.pc += 2;
                }
            },
            0xA000...0xAFFF => self.i = self.nnn,
            0xB000...0xBFFF => self.pc = self.nnn + self.registers[0] as u16,
            0xC000...0xCFFF => {
                let mut rng = thread_rng();
                let rand = rng.gen_range(0, 255);
                self.registers[self.x] = rand;
            },
            0xD000...0xDFFF => self.drw(),
            0xE000...0xEFFF => {
                match self.nn {
                    0x9E => self.skp(),
                    0xA1 => self.sknp(),
                    _ => return
                }
            },
            0xF000...0xFFFF => {
                match self.nn {
                    0x07 => self.ld_vx_dt(),
                    0x0A => self.wait(),
                    0x15 => self.set_delay(),
                    0x18 => self.set_sound(),
                    0x1E => self.i_add_vx(),
                    0x29 => self.ld_f_vx(),
                    0x33 => self.bcd(),
                    0x55 => self.dump(),
                    0x65 => self.load(),
                    _ => return
                }
            },
            _ => {}
        }
    }
    
    
    pub fn reset(&mut self) {
        self.registers.iter_mut().for_each(|m| *m = 0);
        self.stack.iter_mut().for_each(|m| *m = 0);
        self.keyboard.iter_mut().for_each(|m| *m = 0);
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.i = 0;
        self.pc = CPU::INSTRUCTION_START_ADDRESS as u16;
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
