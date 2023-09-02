use imgui::*;
use futures::executor::block_on;
use wgpu::{Device, Queue};
use wgpu::util::DeviceExt;
use wgpu::VertexStepMode::Instance;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

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
        run_count: 0,
    };

    let event_loop = EventLoop::new();
    let mut hidpi_factor = 1.0;
    let (window, mut size, surface) = {
        let window = Window::new(&event_loop).unwrap();
        window.set_inner_size(LogicalSize {
            width: 640.0,
            height: 320.0,
        });
        window.set_title("chip8-rust");
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);

        (window, size, surface)
    };
    
    let adapter = Instance
        .enumerate_adapters(wgpu::Backends::all)
        .find(|adapter| {
            adapter.is_surface_supported(&surface)
        })
        .unwrap();
    
    let mut features = wgpu::Features::empty();
    features |= wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
    features |= wgpu::Features::ANISOTROPIC_FILTERING;
    
    let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features,
            limits: wgpu::Limits::default(),
        },
        None,
    ).unwrap();
    
    let mut imgui = imgui::Context::create();
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    imgui.set_ini_filename(None);
    cpu.run(file_path);

    assert_eq!(cpu.registers[0], 45);
}
