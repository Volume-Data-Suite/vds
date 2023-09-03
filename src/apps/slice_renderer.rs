use std::sync::Arc;

use eframe::{
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu},
};
use egui::{epaint::Shadow, Pos2};

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
    uniform_buffer_volume_axis: wgpu::Buffer,
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
        axis: i32,
        fullscreen_factor: Vector3,
    ) {
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(INDICES));
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(VERTICES));
        queue.write_buffer(
            &self.uniform_buffer_fullscreen_factor,
            0,
            bytemuck::cast_slice(&[fullscreen_factor]),
        );
        queue.write_buffer(
            &self.uniform_buffer_slice_position,
            0,
            bytemuck::cast_slice(&[slice_position]),
        );
        queue.write_buffer(
            &self.uniform_buffer_volume_axis,
            0,
            bytemuck::cast_slice(&[axis]),
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

#[derive(Clone, Copy, PartialEq)]
pub enum VolumeAxis {
    Axial,
    Coronal,
    Sagittal,
}

impl From<VolumeAxis> for i32 {
    fn from(v: VolumeAxis) -> i32 {
        match v {
            x if x == VolumeAxis::Axial => 0,
            x if x == VolumeAxis::Axial => 1,
            x if x == VolumeAxis::Axial => 2,
            _ => -1,
        }
    }
}

pub struct SliceRenderer {
    id: egui::Id,
    slice_position: u32,
    scale: egui::Rect,
    axis: VolumeAxis,
    dimensions: glam::UVec3,
    pub show_settings_oberlay: bool,
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
    pub fn axial(
        wgpu_render_state: &egui_wgpu::RenderState,
        texture: &crate::apps::Texture,
    ) -> Option<Self> {
        Self::new(wgpu_render_state, texture, VolumeAxis::Axial)
    }
    pub fn saggital(
        wgpu_render_state: &egui_wgpu::RenderState,
        texture: &crate::apps::Texture,
    ) -> Option<Self> {
        Self::new(wgpu_render_state, texture, VolumeAxis::Coronal)
    }
    pub fn coronal(
        wgpu_render_state: &egui_wgpu::RenderState,
        texture: &crate::apps::Texture,
    ) -> Option<Self> {
        Self::new(wgpu_render_state, texture, VolumeAxis::Sagittal)
    }

    fn new(
        wgpu_render_state: &egui_wgpu::RenderState,
        texture: &crate::apps::Texture,
        axis: VolumeAxis,
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

        let slice_position: u32 = match axis {
            VolumeAxis::Axial => texture.dimensions.x / 2,
            VolumeAxis::Coronal => texture.dimensions.y / 2,
            VolumeAxis::Sagittal => texture.dimensions.z / 2,
        };

        let uniform_buffer_slice_position =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Slice position"),
                contents: bytemuck::cast_slice(&[slice_position]),
                // Mapping at creation (as done by the create_buffer_init utility) doesn't require us to to add the MAP_WRITE usage
                // (this *happens* to workaround this bug )
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let axis_for_shader: i32 = axis.into();

        let uniform_buffer_volume_axis =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Volume Axis"),
                contents: bytemuck::cast_slice(&[axis_for_shader]),
                // Mapping at creation (as done by the create_buffer_init utility) doesn't require us to to add the MAP_WRITE usage
                // (this *happens* to workaround this bug )
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let bind_group_layout_slice_position =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Slice position"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let bind_group_slice_position = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Slice position"),
            layout: &bind_group_layout_slice_position,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer_slice_position.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer_volume_axis.as_entire_binding(),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("slice_renderer_shader.wgsl").into()),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
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

        let id = egui::Id::new(uuid::Uuid::new_v4());
        let slice_renderer_resources = SliceRenderResources {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer_slice_position,
            uniform_buffer_volume_axis,
            uniform_buffer_fullscreen_factor,
            texture_bind_group,
            fullscreen_factor_bind_group,
            bind_group_slice_position,
        };

        match wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources
            .entry::<std::collections::HashMap<egui::Id, SliceRenderResources>>()
        {
            type_map::concurrent::Entry::Occupied(mut e) => {
                // The typemap already contains a value of this type, so you can access or modify it here
                e.get_mut().insert(id, slice_renderer_resources);
            }
            type_map::concurrent::Entry::Vacant(e) => {
                // The typemap does not contain a value of this type, so you can insert one here
                e.insert(std::collections::HashMap::new())
                    .insert(id, slice_renderer_resources);
            }
        }

        let height = match axis {
            VolumeAxis::Axial => texture.dimensions.x as f32 * texture.spacing.x,
            VolumeAxis::Coronal => texture.dimensions.z as f32 * texture.spacing.z,
            VolumeAxis::Sagittal => texture.dimensions.z as f32 * texture.spacing.z,
        };
        let width = match axis {
            VolumeAxis::Axial => texture.dimensions.y as f32 * texture.spacing.y,
            VolumeAxis::Coronal => texture.dimensions.y as f32 * texture.spacing.y,
            VolumeAxis::Sagittal => texture.dimensions.x as f32 * texture.spacing.x,
        };
        let scale = egui::Rect::from_two_pos(
            egui::Pos2::new(-width, -height),
            egui::Pos2::new(width, height),
        );

        Some(Self {
            id,
            slice_position,
            scale,
            axis,
            dimensions: texture.dimensions,
            show_settings_oberlay: true,
        })
    }
}

impl SliceRenderer {
    // pub fn custom_painting(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
    pub fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let availbale_size = ui.available_size_before_wrap();
        let (rect, _response) = ui.allocate_exact_size(availbale_size, egui::Sense::hover());

        // Clone locals so we can move them into the paint callback:
        let axis = match self.axis {
            VolumeAxis::Axial => 0,
            VolumeAxis::Coronal => 1,
            VolumeAxis::Sagittal => 2,
        };
        let slice_position: f32 = match self.axis {
            VolumeAxis::Axial => self.slice_position as f32 / self.dimensions.x as f32,
            VolumeAxis::Coronal => self.slice_position as f32 / self.dimensions.y as f32,
            VolumeAxis::Sagittal => self.slice_position as f32 / self.dimensions.z as f32,
        };

        let fullscreen_factor = Self::fullscreen_factor(rect, self.scale);

        let id = self.id;

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
                let resources: &std::collections::HashMap<egui::Id, SliceRenderResources> =
                    paint_callback_resources.get().unwrap();
                let slice_render_resources = resources.get(&id).unwrap();
                slice_render_resources.prepare(
                    device,
                    queue,
                    slice_position,
                    axis,
                    fullscreen_factor,
                );
                Vec::new()
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let resources: &std::collections::HashMap<egui::Id, SliceRenderResources> =
                    paint_callback_resources.get().unwrap();
                let slice_render_resources = resources.get(&id).unwrap();
                slice_render_resources.paint(render_pass);
            });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };

        ui.painter().add(callback);

        // Paint overlay
        if self.show_settings_oberlay {
            let original_visuals = ui.visuals().clone();
            let mut visuals = ui.visuals().clone();
            visuals.window_shadow = Shadow::NONE;
            // TODO: Implement overlay transparency
            // visuals.window_fill = visuals.window_fill().gamma_multiply(0.5);
            ui.ctx().set_visuals(visuals);

            let overlay_position = Pos2 {
                x: rect.left_top().x + 2.0,
                y: rect.left_top().y + 2.0,
            };
            egui::Window::new("Settings:")
                .id(ui.next_auto_id())
                .fixed_pos(overlay_position)
                .default_open(false)
                .resizable(false)
                .show(ui.painter().ctx(), |ui| {
                    match self.axis {
                        VolumeAxis::Axial => ui.add(
                            egui::Slider::new(&mut self.slice_position, 1..=self.dimensions.x)
                                .text("Slice Position"),
                        ),
                        VolumeAxis::Coronal => ui.add(
                            egui::Slider::new(&mut self.slice_position, 1..=self.dimensions.y)
                                .text("Slice Position"),
                        ),
                        VolumeAxis::Sagittal => ui.add(
                            egui::Slider::new(&mut self.slice_position, 1..=self.dimensions.z)
                                .text("Slice Position"),
                        ),
                    };
                });

            ui.ctx().set_visuals(original_visuals);
        }
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
