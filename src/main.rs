use imgui::*;
mod cpu;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_path = &args[1];

    let mut cpu = cpu::CPU {
        screen: [0; cpu::CPU::DISPLAY_WIDTH * cpu::CPU::DISPLAY_HEIGHT],
        interrupt: false,
        memory: Vec::<u8>::with_capacity(9999),
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
        run_count: 0
    };
    
    let event_loop = EventLoop::new();
    
    cpu.run(file_path);

    assert_eq!(cpu.registers[0], 45);
}
