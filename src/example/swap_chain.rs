use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::window::Window;
use wgpu::{BackendBit, RequestAdapterOptions, PowerPreference, DeviceDescriptor, Features, TextureUsage, TextureFormat, PresentMode, CommandBufferDescriptor, CommandEncoderDescriptor, RenderPassDescriptor};

struct State{
    surface : wgpu::Surface,
    device : wgpu::Device,
    queue : wgpu::Queue,
    sc_desc : wgpu::SwapChainDescriptor,
    swap_chain : wgpu::SwapChain,
    size : winit::dpi::PhysicalSize<u32>,
    clear_color : wgpu::Color
}

impl State{
    async fn new(window: &Window) -> State
    {
        let size = window.inner_size();
        let ins = wgpu::Instance::new(BackendBit::PRIMARY);
        let surface = unsafe{ ins.create_surface(window) };
        
        let adapter = ins.request_adapter(&RequestAdapterOptions{
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface)
        }).await.unwrap();

        let (device,queue) = adapter.request_device(&DeviceDescriptor{
            label: None,
            features: Features::empty(),
            limits: Default::default()
        },None).await.unwrap();

        let sc_desc = wgpu::SwapChainDescriptor{
            usage: TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo
        };

        let swap_chain = device.create_swap_chain(&surface,&sc_desc);

        let clear_color = wgpu::Color::BLACK;

        State{
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            clear_color
        }
    }

    fn resize(&mut self,size:winit::dpi::PhysicalSize<u32>)
    {
        if size.width > 0 && size.height > 0
        {
            self.size = size;
            self.sc_desc.width = size.width;
            self.sc_desc.height = size.height;
            self.swap_chain = self.device.create_swap_chain(&self.surface,&self.sc_desc);
        }
    }

    fn input(&mut self,event:&WindowEvent) -> bool
    {
        match event{
            WindowEvent::CursorMoved {position,..} => {
                self.clear_color = wgpu::Color{
                    r: position.x / self.size.width as f64 ,
                    g: position.y / self.size.height as f64,
                    b: 1.0,
                    a: 1.0
                };
                true
            }
            _ => { false }
        }
    }

    fn update(&mut self) {}
    fn render(&mut self) -> Result<(),wgpu::SwapChainError>
    {
        let frame = self.swap_chain.get_current_frame().unwrap().output;

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor{
            label: Some("Render Encoder")
        });
        {
            let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor{
                label: Some("Render Pass"),
                color_attachments: &[
                    wgpu::RenderPassColorAttachment{
                        view: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations{
                            load: wgpu::LoadOp::Clear(self.clear_color),
                            store: true
                        }
                    }
                ],
                depth_stencil_attachment: None
            });
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("swap chain")
        .build(&event_loop).unwrap();

    let mut state = pollster::block_on(State::new(&window));

    event_loop.run(move |e,_,control_flow|{
        match e {
            Event::WindowEvent { window_id,event} => {
                if window_id == window.id() {
                    if !state.input(&event) {
                        match event {
                            WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                                input: KeyboardInput {
                                    state: ElementState::Released,
                                    virtual_keycode: Some(VirtualKeyCode::Escape), ..
                                }, ..
                            } => {
                                *control_flow = ControlFlow::Exit;
                            }
                            WindowEvent::Resized(size) => {
                                state.resize(size);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                // new_inner_size is &mut so w have to dereference it twice
                                state.resize(*new_inner_size);
                            }
                            _ => {}
                        }
                    }
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _=>{}
        }
    });
}
