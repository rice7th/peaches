use winit::{
    *,
    event::*,
    event_loop::*, window::{WindowBuilder, Window},
};

use wgpu::*;

struct State {
    surf: Surface,
    device: Device,
    queue: Queue,
    conf: SurfaceConfiguration,
    size: dpi::PhysicalSize<u32>,
    win: Window,
    render_pipeline: wgpu::RenderPipeline,
}

impl State {
    async fn new(win: Window) -> Self {
        let size = win.inner_size();
        let instance = Instance::new(InstanceDescriptor {
            backends: Backend::Dx12.into(), dx12_shader_compiler: Default::default()// CHANGEME
        });

        let surf = unsafe { instance.create_surface(&win).unwrap() };
        let adapter = instance.request_adapter(
            &RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance, // Maybe low power?
                force_fallback_adapter: false, // Wonder what will happen..
                compatible_surface: Some(&surf)
            }
        ).await.unwrap();

        instance
            .enumerate_adapters(Backends::all())
            .map(|adapter| {
                // Check if this adapter supports our surface
                dbg!(adapter.is_surface_supported(&surf));
                dbg!(adapter)
            })
        ;

        let (device, queue) = adapter.request_device(
            &DeviceDescriptor {
                features: Features::empty(),
                limits: Limits::default(),
                label: None,
            }, None).await.unwrap();

        let surf_caps = surf.get_capabilities(&adapter);
        let surf_format = surf_caps.formats.iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surf_caps.formats[0])
        ;
        let conf = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surf_format,
            width: size.width,
            height: size.height,
            //present_mode: surf_caps.present_modes[0],
            present_mode: PresentMode::AutoVsync,
            alpha_mode: surf_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surf.configure(&device, &conf);

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[]
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[]
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: conf.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })]
            }),
            multiview: None
        });

        return Self {
            win,
            surf,
            device,
            queue,
            conf,
            size,
            render_pipeline
        }
    }

    pub fn window(&self) -> &Window {
        &self.win
    }

    fn resize(&mut self, new_size: dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.conf.width = new_size.width;
            self.conf.height = new_size.height;
            self.surf.configure(&self.device, &self.conf);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surf.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }
    
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    
        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let el = EventLoop::new();
    let win = WindowBuilder::new().build(&el).unwrap();
    let mut state = State::new(win).await;

    el.run(move |ev, _, cf| match ev {
        Event::WindowEvent { window_id, ref event }
        if window_id == state.window().id() => if !state.input(event) {
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                    ..
                } => *cf = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                },
                WindowEvent::ScaleFactorChanged {
                    new_inner_size,
                    ..
                } => state.resize(**new_inner_size),
                _ => {},
            }
        },
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *cf = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            state.window().request_redraw();
        }
        _ => {}
    })
}