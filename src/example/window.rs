use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};


fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("title")
        .build(&event_loop).unwrap();

    event_loop.run(move |e,_,control_flow|{
        match e {
            Event::WindowEvent { window_id,event} => {
                if window_id == window.id(){
                    match event{
                        WindowEvent::CloseRequested | WindowEvent::KeyboardInput { input:KeyboardInput{
                            state:ElementState::Released,
                            virtual_keycode:Some(VirtualKeyCode::Escape),..
                        }, ..} => {
                            *control_flow = ControlFlow::Exit;
                        }
                        _ =>{}
                    }
                }
            }
            _=>{}
        }
    });
}
