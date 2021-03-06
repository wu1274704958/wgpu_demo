use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::window::Window;
use wgpu::{BackendBit, RequestAdapterOptions, PowerPreference, DeviceDescriptor, Features, TextureUsage, TextureFormat, PresentMode, CommandBufferDescriptor, CommandEncoderDescriptor, RenderPassDescriptor, ShaderModuleDescriptor, ShaderFlags, PipelineLayoutDescriptor, RenderPipelineDescriptor, VertexState, FragmentState, ColorTargetState, BlendState, BlendComponent, PrimitiveState, PrimitiveTopology, FrontFace, Face, PolygonMode, MultisampleState, ShaderModule, SwapChainDescriptor, BufferUsage, VertexBufferLayout, InputStepMode, IndexFormat, TextureView, Texture, Sampler, TextureDescriptor, Extent3d, TextureDimension, ImageDataLayout, ImageCopyTexture, Origin3d, TextureViewDescriptor, TextureViewDimension, TextureAspect, SamplerDescriptor, AddressMode, FilterMode, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStage, BindingType, TextureSampleType, BindGroupDescriptor, BindGroupEntry, BindingResource, BindGroupLayout, BindGroup, BufferDescriptor, BufferBindingType, Buffer};
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use std::mem::size_of;
use image::{ImageError, GenericImageView};
use std::num::NonZeroU32;
use cgmath::{Matrix4, SquareMatrix, Vector3, Zero, Rad};
use winit::dpi::Pixel;

#[repr(C)]
pub struct FatPtr
{
    ptr: *const u8,
    len: usize
}

impl FatPtr{
    pub fn new<T>(t:&T) -> FatPtr
    {
        FatPtr{
            ptr : unsafe { std::mem::transmute::<*const T, *const u8>(t as *const T) },
            len : std::mem::size_of::<T>()
        }
    }
    pub unsafe fn as_ref_arr(&self) -> &[u8]
    {
        let ptr  = std::mem::transmute::<*const FatPtr,*const u128>(self as *const FatPtr);
        std::mem::transmute::<u128,&[u8]>(*ptr)
    }
}

#[repr(C)]
#[derive(Debug,Copy, Clone,bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex{
    position: [f32;3],
    color: [f32;4],
    tex_coords: [f32;2]
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], color:  [0.5, 0.0, 0.5,1.0],tex_coords: [0.4131759, 0.00759614]   ,}, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 1.0, 0.5,1.0],tex_coords: [0.0048659444, 0.43041354],}, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color:[0.0, 0.5, 0.5,1.0],tex_coords: [0.28081453, 0.949397]    ,}, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], color:  [0.0, 0.5, 1.0,1.0],tex_coords: [0.85967, 0.84732914]     ,}, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], color:   [0.5, 1.0, 0.5,1.0],tex_coords: [0.9414737, 0.2652641]    , }, // E
];

const INDICES: &[u32] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

#[repr(C)]
#[derive(Debug,Copy, Clone)]
struct Uniform {
    projection: Matrix4<f32>,
    view: Matrix4<f32>,
    model: Matrix4<f32>
}


struct State{
    surface : wgpu::Surface,
    device : wgpu::Device,
    queue : wgpu::Queue,
    sc_desc : wgpu::SwapChainDescriptor,
    swap_chain : wgpu::SwapChain,
    size : winit::dpi::PhysicalSize<u32>,
    clear_color : wgpu::Color,
    pipeline : wgpu::RenderPipeline,
    vertices : wgpu::Buffer,
    indices : wgpu::Buffer,
    img1: (Texture,TextureView,Sampler),
    bind_group_layouts: Vec<BindGroupLayout>,
    bind_groups : Vec<BindGroup>,
    uniform : Uniform,
    uniform_buf : Buffer,
    rotate : Vector3<f32>,
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

        let shader = device.create_shader_module(&ShaderModuleDescriptor{
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            flags: ShaderFlags::all()
        });

        let vertices = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Vertices"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsage::VERTEX
        });

        let indices = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Indices"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsage::INDEX
        });
        let img_data = include_bytes!("../textures/happy-tree.png");
        let (texture,texture_view,sampler) = Self::load_texture(&device,&queue,img_data).unwrap();

        let bind_group_layout =  device.create_bind_group_layout(&BindGroupLayoutDescriptor{
            label: Some("Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry{
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float{ filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false
                    },
                    count: None
                },
                BindGroupLayoutEntry{
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler{
                        filtering: true,
                        comparison: false
                    },
                    count: None
                }
            ]
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor{
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry{ binding: 0, resource: BindingResource::TextureView(&texture_view) },
                BindGroupEntry{ binding: 1, resource: BindingResource::Sampler(&sampler) }
            ]
        });
        let uniform = Uniform::new(45.0,size.width as f32 / size.height as f32 );

        let uniform_buf = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Uniform Buffer"),
            contents: //unsafe { std::slice::from_raw_parts(std::mem::transmute::<_, *const u8>(&uniform), size_of::<Uniform>()) },
            unsafe { FatPtr::new(&uniform).as_ref_arr() },
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST
        });
        let vertex_binding_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor{
            label: Some("Vertex Binding Group"),
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                }
            ]
        });
        
        let vertex_binding_group = device.create_bind_group(&BindGroupDescriptor{
            label: Some("Vertex binding Gropu"),
            layout: &vertex_binding_group_layout,
            entries: &[
                BindGroupEntry{
                    binding: 0,
                    resource: uniform_buf.as_entire_binding()
                }
            ]
        });

        let pipeline = Self::create_pipeline(&device,&shader,&sc_desc,&[&bind_group_layout,
        &vertex_binding_group_layout]);

        State{
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            clear_color,
            pipeline,
            vertices,
            indices,
            img1: (texture,texture_view,sampler),
            bind_group_layouts: vec![ bind_group_layout,vertex_binding_group_layout],
            bind_groups: vec![bind_group,vertex_binding_group],
            uniform,
            uniform_buf,
            rotate: Vector3::zero(),
        }
    }

    fn create_pipeline(device:& wgpu::Device,shader:&ShaderModule,
                       sc_desc:&SwapChainDescriptor,
                       bind_group_layouts:&'_[&'_ BindGroupLayout]) -> wgpu::RenderPipeline
    {
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor{
            label: Some("Render Pipeline Layout"),
            bind_group_layouts,
            push_constant_ranges: &[]
        });

        device.create_render_pipeline(&RenderPipelineDescriptor{
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState{
                module: &shader,
                entry_point: "main",
                buffers: &[ VertexBufferLayout{
                    array_stride: size_of::<Vertex>() as _,
                    step_mode: InputStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![ 0 => Float32x3, 1=> Float32x4, 2=> Float32x2 ]
                } ]
            },
            fragment: Some(FragmentState{
                module: &shader,
                entry_point: "main",
                targets: &[ColorTargetState{
                    format: sc_desc.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: wgpu::ColorWrite::all()
                }]
            }),
            primitive: PrimitiveState{
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                clamp_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false
            },
            depth_stencil: None,
            multisample: MultisampleState{
                count: 1,
                mask: u64::MAX,
                alpha_to_coverage_enabled: false
            }
        })
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
            &WindowEvent::KeyboardInput{ input:KeyboardInput{
                virtual_keycode:Some(VirtualKeyCode::Space),state:ElementState::Released,..
            },.. } => {
                true
            }
            _ => { false }
        }
    }

    fn update(&mut self) {
        self.rotate.y += 0.1;
        self.uniform.set_model(cgmath::Matrix4::from_angle_y(Rad(self.rotate.y.sin())));
        unsafe { self.queue.write_buffer(&self.uniform_buf, 0, FatPtr::new(&self.uniform).as_ref_arr()); }
    }
    fn render(&mut self) -> Result<(),wgpu::SwapChainError>
    {
        let frame = self.swap_chain.get_current_frame().unwrap().output;

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor{
            label: Some("Render Encoder")
        });
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor{
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
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0,&self.bind_groups[0],&[]);
            render_pass.set_bind_group(1,&self.bind_groups[1],&[]);
            render_pass.set_vertex_buffer(0,self.vertices.slice(..));
            render_pass.set_index_buffer(self.indices.slice(..),IndexFormat::Uint32);
            //render_pass.draw(0..VERTICES.len() as u32,0..1);
            render_pass.draw_indexed(0..INDICES.len() as u32,0,0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    fn load_texture(device:&wgpu::Device, queue:&wgpu::Queue, data:&[u8]) -> Result<(Texture,TextureView,Sampler),ImageError>
    {
        let img = image::load_from_memory(data)?;
        let dimensions = img.dimensions();
        let size = Extent3d{
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1
        };
        let texture = device.create_texture(&TextureDescriptor{
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST
        });
        queue.write_texture(ImageCopyTexture{
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO
        }, img.as_rgba8().unwrap(), ImageDataLayout{
            offset: 0,
            bytes_per_row: NonZeroU32::new(size.width * 4),
            rows_per_image: NonZeroU32::new(size.height)
        }, size);
        let view = texture.create_view(&TextureViewDescriptor{
            label: Some("Texture View"),
            format: Some(TextureFormat::Rgba8UnormSrgb),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::default(),
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None
        });
        let sampler = device.create_sampler(&SamplerDescriptor{
            label: Some("Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        Ok((texture,view,sampler))
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


impl Uniform{
    fn new(fovy:f32,aspect:f32) -> Uniform
    {
        let proj = cgmath::perspective(cgmath::Deg(fovy),aspect,0.1,1000f32);
        Uniform{
            projection : proj,
            view : cgmath::Matrix4::from_translation(Vector3::new(0.0,0.0,-2f32)),
            model : cgmath::Matrix4::identity()
        }
    }
    pub fn set_model(&mut self,model:Matrix4<f32>)
    {
        self.model = model;
    }

}