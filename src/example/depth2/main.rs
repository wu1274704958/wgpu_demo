use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::window::Window;
use wgpu::{BackendBit, RequestAdapterOptions, PowerPreference, DeviceDescriptor, Features, TextureUsage, TextureFormat, PresentMode, CommandBufferDescriptor, CommandEncoderDescriptor, RenderPassDescriptor, ShaderModuleDescriptor, ShaderFlags, PipelineLayoutDescriptor, RenderPipelineDescriptor, VertexState, FragmentState, ColorTargetState, BlendState, BlendComponent, PrimitiveState, PrimitiveTopology, FrontFace, Face, PolygonMode, MultisampleState, ShaderModule, SwapChainDescriptor, BufferUsage, VertexBufferLayout, InputStepMode, IndexFormat, TextureView, Texture, Sampler, TextureDescriptor, Extent3d, TextureDimension, ImageDataLayout, ImageCopyTexture, Origin3d, TextureViewDescriptor, TextureViewDimension, TextureAspect, SamplerDescriptor, AddressMode, FilterMode, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStage, BindingType, TextureSampleType, BindGroupDescriptor, BindGroupEntry, BindingResource, BindGroupLayout, BindGroup, BufferDescriptor, BufferBindingType, Buffer, Device, CompareFunction, DepthStencilState, RenderPassDepthStencilAttachment, Operations, LoadOp, RenderPipeline};
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use std::mem::size_of;
use image::{ImageError, GenericImageView};
use std::num::NonZeroU32;
use cgmath::{Matrix4, SquareMatrix, Vector3, Zero, Rad, Quaternion, Rotation3, Vector2};
use winit::dpi::{Pixel, PhysicalPosition};
use std::ops::Range;
use wgpu::LoadOp::Clear;
use utils::{from_raw_parts, from_raw_parts_ex};

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

const DEPTH_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.9, -0.9, 0.0],
        tex_coords: [0.0, 1.0],
        color : [1.0,1.0,1.0,1.0]
    },
    Vertex {
        position: [0.9, -0.9, 0.0],
        tex_coords: [1.0, 1.0],
        color : [1.0,1.0,1.0,1.0]
    },
    Vertex {
        position: [0.9, 0.9, 0.0],
        tex_coords: [1.0, 0.0],
        color : [1.0,1.0,1.0,1.0]
    },
    Vertex {
        position: [-0.9, 0.9, 0.0],
        tex_coords: [0.0, 0.0],
        color : [1.0,1.0,1.0,1.0]
    },
];

const DEPTH_INDICES: &[u32] = &[0, 1, 2, 0, 2, 3];

#[repr(C)]
#[derive(Debug,Copy, Clone)]
struct Uniform {
    projection: Matrix4<f32>,
    view: Matrix4<f32>,
}
#[repr(C)]
#[derive(Debug,Copy, Clone)]
struct Uniform2 {
    projection: Matrix4<f32>,
    view: Matrix4<f32>,
    model: Matrix4<f32>,
}
struct Instance{
    pos: Vector3<f32>,
    quaternion: Quaternion<f32>,
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
    bind_groups : Vec<BindGroup>,
    uniform : Uniform,
    uniform_buf : Buffer,
    instances: Vec<Instance>,
    instance_buf: Vec<Matrix4<f32>>,
    rotate : Vector3<f32>,
    instance_buffer: Buffer,
    left_btn_down: bool,
    last_cursor_pos : Vector2<f32>,
    depth_stencil : (Texture,TextureView,Sampler),
    mesh_quad: (Buffer,Buffer),
    depth_pipeline:RenderPipeline,
    depth_uniform:(Uniform2,Buffer),
    depth_bind_group_layout: BindGroupLayout,
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
        let shader_depth = device.create_shader_module(&ShaderModuleDescriptor{
            label: Some("Shader Depth"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_depth.wgsl").into()),
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
        
        let depth_vertices = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Depth Vertices"),
            contents: bytemuck::cast_slice(DEPTH_VERTICES),
            usage: BufferUsage::VERTEX
        });

        let depth_indices = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Depth Indices"),
            contents: bytemuck::cast_slice(DEPTH_INDICES),
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
            label: Some("Depth Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry{ binding: 0, resource: BindingResource::TextureView(&texture_view) },
                BindGroupEntry{ binding: 1, resource: BindingResource::Sampler(&sampler) }
            ]
        });

        let depth_stencil = Self::create_depth_stencil(&device,&sc_desc);

        let depth_bind_group_layout =  device.create_bind_group_layout(&BindGroupLayoutDescriptor{
            label: Some("Depth Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry{
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Depth,
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

        let depth_bind_group = device.create_bind_group(&BindGroupDescriptor{
            label: Some("Depth Bind Group"),
            layout: &depth_bind_group_layout,
            entries: &[
                BindGroupEntry{ binding: 0, resource: BindingResource::TextureView(&depth_stencil.1) },
                BindGroupEntry{ binding: 1, resource: BindingResource::Sampler(&depth_stencil.2) }
            ]
        });

        let uniform = Uniform::new(60.0,size.width as f32 / 2.0f32 / size.height as f32 );
        let uniform2 = Uniform2::new(60.0,size.width as f32 / 2.0f32 / size.height as f32 );

        let uniform_buf = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Uniform Buffer"),
            contents: unsafe { from_raw_parts(&uniform) },
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST
        });

        let uniform2_buf = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Depth Uniform Buffer"),
            contents: unsafe { from_raw_parts(&uniform2) },
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST
        });

        let instances = Instance::gen_instances(255,15,Vector3::new(0.0,0.03,0.0),1.0);
        let instance_buf:Vec<_> = instances.iter().map(|it|{
            it.to_matrix()

        }).collect();

        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Instance Buffer"),
            contents: unsafe { from_raw_parts_ex(instance_buf.as_slice()) },
            usage: BufferUsage::VERTEX
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

        let depth_vertex_binding_group = device.create_bind_group(&BindGroupDescriptor{
            label: Some("Vertex binding Gropu"),
            layout: &vertex_binding_group_layout,
            entries: &[
                BindGroupEntry{
                    binding: 0,
                    resource: uniform2_buf.as_entire_binding()
                }
            ]
        });

        let pipeline = Self::create_pipeline(&device,&shader,&sc_desc,&[&bind_group_layout,
            &vertex_binding_group_layout],&[VertexBufferLayout{
            array_stride: size_of::<Matrix4<f32>>() as _,
            step_mode: InputStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![ 5 => Float32x4,6 => Float32x4,7 => Float32x4,8 => Float32x4 ]
        }],true);

        let depth_pipeline = Self::create_pipeline(&device,&shader_depth,&sc_desc,&[
            &depth_bind_group_layout,&vertex_binding_group_layout
        ],&[],false);

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
            bind_groups: vec![bind_group,vertex_binding_group,depth_bind_group,depth_vertex_binding_group],
            uniform,
            uniform_buf,
            rotate: Vector3::zero(),
            instances,
            instance_buf,
            instance_buffer,
            left_btn_down :false,
            last_cursor_pos: Vector2::zero(),
            depth_stencil,
            mesh_quad: (depth_vertices,depth_indices),
            depth_pipeline,
            depth_uniform : (uniform2,uniform2_buf),
            depth_bind_group_layout
        }
    }

    fn create_depth_stencil(device:&Device,sc_desc:&SwapChainDescriptor)-> (Texture,TextureView,Sampler)
    {
        let tex = device.create_texture(&TextureDescriptor{
            label: Some("Depth Stencil Tex"),
            size: Extent3d{
                width: sc_desc.width,
                height: sc_desc.height,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED
        });
        let tex_view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor{
            label: Some("Depth Stencil Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: -100 as _,
            lod_max_clamp: 100 as _,
            compare: None,
            anisotropy_clamp: None,
            border_color: None
        });
        (tex,tex_view,sampler)
    }

    fn create_pipeline(
        device:& wgpu::Device,shader:&ShaderModule,
        sc_desc:&SwapChainDescriptor,
        bind_group_layouts:&'_[&'_ BindGroupLayout],
        added_vertex_buffer:&[VertexBufferLayout],
        has_depth_stencil:bool) -> wgpu::RenderPipeline
    {
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor{
            label: Some("Render Pipeline Layout"),
            bind_group_layouts,
            push_constant_ranges: &[]
        });

        let attributes = wgpu::vertex_attr_array![ 0 => Float32x3, 1=> Float32x4, 2=> Float32x2 ];
        let mut vertex_buffers = vec![ VertexBufferLayout{
            array_stride: size_of::<Vertex>() as _,
            step_mode: InputStepMode::Vertex,
            attributes: &attributes
        }];
        if added_vertex_buffer.len() > 0{
            vertex_buffers.extend_from_slice(added_vertex_buffer);
        }

        device.create_render_pipeline(&RenderPipelineDescriptor{
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState{
                module: &shader,
                entry_point: "main",
                buffers: vertex_buffers.as_slice()
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
                cull_mode: None,
                clamp_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false
            },
            depth_stencil: if has_depth_stencil { Some(DepthStencilState{
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default()
            }) }else{None},
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
            self.depth_stencil = Self::create_depth_stencil(&self.device,&self.sc_desc);

            self.bind_groups[2] = self.device.create_bind_group(&BindGroupDescriptor{
                label: Some("Depth Bind Group"),
                layout: &self.depth_bind_group_layout,
                entries: &[
                    BindGroupEntry{ binding: 0, resource: BindingResource::TextureView(&self.depth_stencil.1) },
                    BindGroupEntry{ binding: 1, resource: BindingResource::Sampler(&self.depth_stencil.2) }
                ]
            });
        }
    }

    fn input(&mut self,event:&WindowEvent,window:&Window) -> bool
    {
        match event{
            &WindowEvent::KeyboardInput{ input:KeyboardInput{
                virtual_keycode:Some(VirtualKeyCode::Space),state:ElementState::Released,..
            },.. } => {
                true
            }
            &WindowEvent::MouseInput {
                button:MouseButton::Left,
                state,..
            } => {

                self.left_btn_down = match state{
                    ElementState::Pressed => {true}
                    ElementState::Released => {false}
                };
                true
            }
            &WindowEvent::CursorMoved{position:PhysicalPosition::<f64> {x,y},..} =>
            {
                if self.left_btn_down{
                    let offset = Vector2::new(x as f32,y as f32) - self.last_cursor_pos;
                    self.rotate.y += offset.x * 0.001;
                    self.rotate.x += offset.y * 0.001;
                    self.last_cursor_pos = Vector2::new(x as f32,y as f32);
                }else{
                    self.last_cursor_pos = Vector2::new(x as f32,y as f32);
                }
                true
            }
            _ => { false }
        }
    }

    fn update(&mut self) {
        self.uniform.set_rotate(self.rotate);
        unsafe { self.queue.write_buffer(&self.uniform_buf, 0, from_raw_parts(&self.uniform)) }
        //self.depth_uniform.0.set_rotate(self.rotate);
        //unsafe { self.queue.write_buffer(&self.depth_uniform.1, 0, from_raw_parts(&self.depth_uniform.0)) }
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
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment{
                    view: &self.depth_stencil.1,
                    depth_ops: Some(Operations::<f32>{ load: LoadOp::Clear(1f32), store: true }),
                    stencil_ops: None
                })
            });
            render_pass.set_viewport(0.0,0.0,self.sc_desc.width as f32 / 2f32,self.sc_desc.height as f32,0.0,1.0);
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0,&self.bind_groups[0],&[]);
            render_pass.set_bind_group(1,&self.bind_groups[1],&[]);
            render_pass.set_vertex_buffer(0,self.vertices.slice(..));
            render_pass.set_vertex_buffer(1,self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.indices.slice(..),IndexFormat::Uint32);
            //render_pass.draw(0..VERTICES.len() as u32,0..1);
            render_pass.draw_indexed(0..INDICES.len() as u32,0,0..self.instance_buf.len() as _);
        }
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor{
                label: Some("Render Pass depth"),
                color_attachments: &[
                    wgpu::RenderPassColorAttachment{
                        view: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations{
                            load: wgpu::LoadOp::Load,
                            store: true
                        }
                    }
                ],
                depth_stencil_attachment: None
            });
            render_pass.set_viewport(self.sc_desc.width as f32 / 2f32,0.0,self.sc_desc.width as f32 / 2f32,self.sc_desc.height as f32,0.0,1.0);
            render_pass.set_pipeline(&self.depth_pipeline);
            render_pass.set_bind_group(0,&self.bind_groups[2],&[]);
            render_pass.set_bind_group(1,&self.bind_groups[3],&[]);
            render_pass.set_vertex_buffer(0,self.mesh_quad.0.slice(..));
            render_pass.set_index_buffer(self.mesh_quad.1.slice(..),IndexFormat::Uint32);
            //render_pass.draw(0..VERTICES.len() as u32,0..1);
            render_pass.draw_indexed(0..DEPTH_INDICES.len() as u32,0,0..1);
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
                    if !state.input(&event,&window) {
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
                                println!("Resized {:?}",size);
                                state.resize(size);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                // new_inner_size is &mut so w have to dereference it twice
                                println!("Resized {:?}",new_inner_size);
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
        let proj = cgmath::perspective(cgmath::Deg(fovy),aspect,0.1,100f32);
        Uniform{
            projection : proj,
            view : cgmath::Matrix4::from_translation(Vector3::new(0.0,0.0,-9f32)),
        }
    }

    fn set_rotate(&mut self,r:Vector3<f32>)
    {
        let mat =
            cgmath::Matrix4::from_translation(Vector3::new(0.0,0.0,-9f32)) *
            Matrix4::from_angle_x(Rad(r.x)) *
            Matrix4::from_angle_y(Rad(r.y)) *
            Matrix4::from_angle_z(Rad(r.z));
        self.view = mat;
    }
}
impl Uniform2{
    fn new(fovy:f32,aspect:f32) -> Uniform2
    {
        let proj = cgmath::perspective(cgmath::Deg(fovy),aspect,0.1,100f32);
        Uniform2{
            projection : proj,
            view : cgmath::Matrix4::from_translation(Vector3::new(0.0,0.0,-2f32)),
            model : cgmath::Matrix4::identity()
        }
    }

    fn set_rotate(&mut self,r:Vector3<f32>)
    {
        let mat =
            cgmath::Matrix4::from_translation(Vector3::new(0.0,0.0,-2f32)) *
                Matrix4::from_angle_x(Rad(r.x)) *
                Matrix4::from_angle_y(Rad(r.y)) *
                Matrix4::from_angle_z(Rad(r.z));
        self.view = mat;
    }
}


impl Instance {
    fn to_matrix(&self) -> Matrix4<f32>
    {
        Matrix4::<f32>::from_translation(self.pos) * Matrix4::<f32>::from(self.quaternion)
    }

    fn gen_instances(len:usize,row:usize,rot_off:Vector3<f32>,space:f32) -> Vec<Instance>
    {
        let mut res = Vec::new();
        let mut col = len / row;
        if len % row > 0 { col += 1 };
        let mut pos = Vector3::new(
            ((row - 1) as f32 * space) / 2f32,
            ((col - 1) as f32 * space) / 2f32,
                0f32
        );
        let mut rot = Vector3::<f32>::zero();
        for i in 0..len {
            res.push(Instance{
                pos,quaternion: Quaternion::from_angle_x(Rad(rot.x)) *
                    Quaternion::from_angle_y(Rad(rot.y)) * Quaternion::from_angle_z(Rad(rot.z))
            });
            if (i + 1) % row == 0
            {
                pos.x = ((row - 1) as f32 * space) / 2f32;
                pos.y -= space;
            }else{
                pos.x -= space;
            }
            rot += rot_off;
        }
        res
    }
}
