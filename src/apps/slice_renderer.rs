use std::sync::Arc;

use eframe::{
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu},
};

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vector3 {
    data: [f32; 3],
}

impl Vector3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { data: [x, y, z] }
    }
}

struct SliceRenderResources {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer_slice_position: wgpu::Buffer,
    uniform_buffer_fullscreen_factor: wgpu::Buffer,
    texture_bind_group: wgpu::BindGroup,
    fullscreen_factor_bind_group: wgpu::BindGroup,
    bind_group_slice_position: wgpu::BindGroup,
}

impl SliceRenderResources {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        slice_position: f32,
        fullscreen_factor_uniform: Vector3,
    ) {
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(INDICES));
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(VERTICES));
        queue.write_buffer(
            &self.uniform_buffer_fullscreen_factor,
            0,
            bytemuck::cast_slice(&[fullscreen_factor_uniform]),
        );
        queue.write_buffer(
            &self.uniform_buffer_slice_position,
            0,
            bytemuck::cast_slice(&[slice_position]),
        );
    }

    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        render_pass.set_pipeline(&self.render_pipeline);

        // fullscreen_factor
        render_pass.set_bind_group(1, &self.fullscreen_factor_bind_group, &[]);
        // slice position
        render_pass.set_bind_group(2, &self.bind_group_slice_position, &[]);
        // volume data
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    }
}

pub struct SliceRenderer {
    // size: Rect,
    slice_position: f32,
    scale: egui::Rect,
}

impl SliceRenderer {
    fn fullscreen_factor(viewport: egui::Rect, volume: egui::Rect) -> Vector3 {
        let volume_aspect_ratio = volume.width() / volume.height();
        let viewport_aspect_ratio = viewport.width() / viewport.height();

        let mut scale_x: f32 = 1.0;
        let mut scale_y: f32 = 1.0;

        if volume_aspect_ratio > viewport_aspect_ratio {
            scale_y = viewport_aspect_ratio / volume_aspect_ratio;
        } else {
            scale_x = volume_aspect_ratio / viewport_aspect_ratio;
        }

        Vector3::new(scale_x, scale_y, 1.0)
    }

    pub fn new(
        wgpu_render_state: &egui_wgpu::RenderState,
        texture: &crate::apps::Texture,
    ) -> Option<Self> {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        // let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        let device = &wgpu_render_state.device;

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D3,
                            sample_type: wgpu::TextureSampleType::Float { filterable: (true) },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let fullscreen_factor = Vector3::new(1.0, 1.0, 1.0);

        let uniform_buffer_fullscreen_factor =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("fullscreen_factor Buffer"),
                contents: bytemuck::cast_slice(&[fullscreen_factor]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let fullscreen_factor_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("fullscreen_factor_bind_group_layout"),
            });

        let fullscreen_factor_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &fullscreen_factor_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer_fullscreen_factor.as_entire_binding(),
            }],
            label: Some("fullscreen_factor_bind_group"),
        });

        let slice_position: f32 = 0.5;

        let uniform_buffer_slice_position =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Slice position"),
                contents: bytemuck::cast_slice(&[slice_position]),
                // Mapping at creation (as done by the create_buffer_init utility) doesn't require us to to add the MAP_WRITE usage
                // (this *happens* to workaround this bug )
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let bind_group_layout_slice_position =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Slice position"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bind_group_slice_position = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Slice position"),
            layout: &bind_group_layout_slice_position,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer_slice_position.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &fullscreen_factor_bind_group_layout,
                    &bind_group_layout_slice_position,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu_render_state.target_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our `Custom3D` struct, we insert it into the
        // `paint_callback_resources` type map, which is stored alongside the render pass.
        wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(SliceRenderResources {
                render_pipeline,
                vertex_buffer,
                index_buffer,
                uniform_buffer_slice_position,
                uniform_buffer_fullscreen_factor,
                texture_bind_group,
                fullscreen_factor_bind_group,
                bind_group_slice_position,
            });

        Some(Self {
            // size: Rect::EVERYTHING,
            slice_position,
            scale: egui::Rect::from_two_pos(egui::pos2(-1.0, -1.0), egui::pos2(1.0, 1.0)),
        })
    }
}

impl eframe::App for SliceRenderer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                        self.custom_painting(ui);
                    });
                });
        });
    }
}

impl SliceRenderer {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        // TODO: Deal with resize and aspect ratio
        let availbale_size = ui.available_size_before_wrap();
        let (rect, response) = ui.allocate_exact_size(availbale_size, egui::Sense::drag());

        self.slice_position += response.drag_delta().y * 0.01;

        // Clone locals so we can move them into the paint callback:
        let slice_position = self.slice_position;

        let fullscreen_factor = Self::fullscreen_factor(rect, self.scale);

        // The callback function for WGPU is in two stages: prepare, and paint.
        //
        // The prepare callback is called every frame before paint and is given access to the wgpu
        // Device and Queue, which can be used, for instance, to update buffers and uniforms before
        // rendering.
        //
        // You can use the main `CommandEncoder` that is passed-in, return an arbitrary number
        // of user-defined `CommandBuffer`s, or both.
        // The main command buffer, as well as all user-defined ones, will be submitted together
        // to the GPU in a single call.
        //
        // The paint callback is called after prepare and is given access to the render pass, which
        // can be used to issue draw commands.
        let cb = egui_wgpu::CallbackFn::new()
            .prepare(move |device, queue, _encoder, paint_callback_resources| {
                let resources: &SliceRenderResources = paint_callback_resources.get().unwrap();
                resources.prepare(device, queue, slice_position, fullscreen_factor);
                Vec::new()
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let resources: &SliceRenderResources = paint_callback_resources.get().unwrap();
                resources.paint(render_pass);
            });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };

        ui.painter().add(callback);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    // wgpu uses the coordinate systems of D3D and Metal
    // https://github.com/gfx-rs/wgpu#coordinate-systems
    Vertex {
        position: [-1.0, -1.0, 1.0],
        tex_coords: [0.0, 1.0],
    }, // 0
    Vertex {
        position: [1.0, -1.0, 1.0],
        tex_coords: [1.0, 1.0],
    }, // 1
    Vertex {
        position: [1.0, 1.0, 1.0],
        tex_coords: [1.0, 0.0],
    }, // 2
    Vertex {
        position: [-1.0, 1.0, 1.0],
        tex_coords: [0.0, 0.0],
    }, // 3
       // Vertex { position: [-1.0, -1.0, -1.0] },  // 4
       // Vertex { position: [1.0, -1.0, -1.0] },   // 5
       // Vertex { position: [1.0, 1.0, -1.0] },    // 6
       // Vertex { position: [-1.0, 1.0, -1.0] },   // 7
];

const INDICES: &[u16] = &[
    // front
    0, 1, 2, 2, 3,
    0,
    // // right
    // 1, 5, 6, 6, 2, 1,
    // // back
    // 7, 6, 5, 5, 4, 7,
    // // left
    // 4, 0, 3, 3, 7, 4,
    // // bottom
    // 4, 5, 1, 1, 0, 4,
    // // top
    // 3, 2, 6, 6, 7, 3
];
