use std::fs::File;
use std::io::Read;
mod cpu;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    //let file_path = &args[1];
    
    //let rom = load_rom(file_path);
    
    let mut cpu = cpu::CPU {
        screen: [0; cpu::CPU::DISPLAY_WIDTH * cpu::CPU::DISPLAY_HEIGHT],
        interrupt: false,
        memory: [0; 4096],
        keyboard: [0; 16],
        stack: [0; 16],
        delay_timer: 0,
        sound_timer: 0,
        registers: [0; 16],
        i: 0,
        pc: 0,
        stack_pointer: 0,
        instruction: 0,
        program_size: 0,
        x: 0,
        y: 0,
        n: 0,
        nn: 0,
        nnn: 0,
        program: vec![0; 0],
    };
    
    cpu.run();

    assert_eq!(cpu.registers[0], 45);
}

// fn load_rom(file_path: &str) -> Vec<u8> {
//     let mut file = File::open(file_path).expect("Failed to open file");
//     let mut buffer = Vec::new();
// 
//     file.read_to_end(&mut buffer).expect("Failed to read file");
//     
//     return buffer
// }