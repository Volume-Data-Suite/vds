use std::sync::Arc;

use eframe::{
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu},
};
use egui::{epaint::Shadow, Pos2};
use glam::{vec3, Vec2, Vec3, Vec4};

struct RayMarchingRendererResources {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer_projection_view_model_matrix: wgpu::Buffer,
    uniform_buffer_view_model_matrix_without_model_scale: wgpu::Buffer,
    uniform_buffer_threshold: wgpu::Buffer,
    uniform_buffer_sample_step_length: wgpu::Buffer,
    uniform_buffer_focal_length: wgpu::Buffer,
    uniform_buffer_aspect_ratio: wgpu::Buffer,
    uniform_buffer_viewport_size: wgpu::Buffer,
    uniform_buffer_viewport_position: wgpu::Buffer,
    uniform_buffer_ray_origin: wgpu::Buffer,
    uniform_buffer_top_aabb: wgpu::Buffer,
    uniform_buffer_bottom_aabb: wgpu::Buffer,
    uniform_buffer_camera_position: wgpu::Buffer,
    bind_group_texture: wgpu::BindGroup,
    bind_group_uniforms_vertex_shader_stage: wgpu::BindGroup,
    bind_group_uniforms_fragment_shader_stage: wgpu::BindGroup,
}

impl RayMarchingRendererResources {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        projection_view_model_matrix: glam::Mat4,
        view_model_matrix_without_model_scale: glam::Mat4,
        threshold: f32,
        sample_step_length: f32,
        focal_length: f32,
        aspect_ratio: f32,
        viewport_size: glam::Vec2,
        viewport_position: glam::Vec2,
        ray_origin: glam::Vec3,
        top_aabb: glam::Vec3,
        bottom_aabb: glam::Vec3,
        camera_position: glam::Vec3,
    ) {
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(INDICES));
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(VERTICES));
        queue.write_buffer(
            &self.uniform_buffer_threshold,
            0,
            bytemuck::cast_slice(&[threshold]),
        );
        queue.write_buffer(
            &self.uniform_buffer_sample_step_length,
            0,
            bytemuck::cast_slice(&[sample_step_length]),
        );
        queue.write_buffer(
            &self.uniform_buffer_projection_view_model_matrix,
            0,
            bytemuck::cast_slice(&[projection_view_model_matrix]),
        );
        queue.write_buffer(
            &self.uniform_buffer_view_model_matrix_without_model_scale,
            0,
            bytemuck::cast_slice(&[view_model_matrix_without_model_scale]),
        );
        queue.write_buffer(
            &self.uniform_buffer_focal_length,
            0,
            bytemuck::cast_slice(&[focal_length]),
        );
        queue.write_buffer(
            &self.uniform_buffer_aspect_ratio,
            0,
            bytemuck::cast_slice(&[aspect_ratio]),
        );
        queue.write_buffer(
            &self.uniform_buffer_viewport_size,
            0,
            bytemuck::cast_slice(&[viewport_size]),
        );
        queue.write_buffer(
            &self.uniform_buffer_viewport_position,
            0,
            bytemuck::cast_slice(&[viewport_position]),
        );
        queue.write_buffer(
            &self.uniform_buffer_ray_origin,
            0,
            bytemuck::cast_slice(&[ray_origin]),
        );
        queue.write_buffer(
            &self.uniform_buffer_top_aabb,
            0,
            bytemuck::cast_slice(&[top_aabb]),
        );
        queue.write_buffer(
            &self.uniform_buffer_bottom_aabb,
            0,
            bytemuck::cast_slice(&[bottom_aabb]),
        );
        queue.write_buffer(
            &self.uniform_buffer_camera_position,
            0,
            bytemuck::cast_slice(&[camera_position]),
        );
    }

    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        render_pass.set_pipeline(&self.render_pipeline);

        // uniforms for vertex shader stage
        render_pass.set_bind_group(0, &self.bind_group_uniforms_vertex_shader_stage, &[]);
        // volume data texture
        render_pass.set_bind_group(1, &self.bind_group_texture, &[]);
        // uniforms for fragment shader stage
        render_pass.set_bind_group(2, &self.bind_group_uniforms_fragment_shader_stage, &[]);
        // geometry
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    }
}

pub struct RayMarchingRenderer {
    id: egui::Id,
    // Uniforms begin
    threshold: f32,
    sample_step_length: f32,
    focal_length: f32,
    aspect_ratio: f32,
    viewport_size: glam::Vec2,
    viewport_position: glam::Vec2,
    ray_origin: glam::Vec3,
    top_aabb: glam::Vec3,
    bottom_aabb: glam::Vec3,
    camera_position: glam::Vec3,
    // Uniforms end
    rotation_matrix: glam::Mat4,
    translation_matrix: glam::Mat4,
    scale_matrix: glam::Mat4,

    dimensions: glam::UVec3,
    spacing: glam::Vec3,
    extent: glam::Vec3,
    pub show_settings_oberlay: bool,
    projection_matrix: glam::Mat4,
    view_matrix: glam::Mat4,
}

impl RayMarchingRenderer {
    pub fn new(
        wgpu_render_state: &egui_wgpu::RenderState,
        texture: &crate::apps::Texture,
    ) -> Option<Self> {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        // let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        let device = &wgpu_render_state.device;

        let projection_view_model_matrix = glam::Mat4::IDENTITY;

        let uniform_buffer_projection_view_model_matrix =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("projection view model matrix"),
                contents: bytemuck::cast_slice(&[projection_view_model_matrix]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let bind_group_layout_uniforms_vertex_shader_stage =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("all vertex shader uniforms"),
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
            });

        let bind_group_uniforms_vertex_shader_stage =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("all vertex shader uniforms"),
                layout: &bind_group_layout_uniforms_vertex_shader_stage,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer_projection_view_model_matrix.as_entire_binding(),
                }],
            });

        let bind_group_texture_layout =
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
                label: Some("bind_group_texture_layout"),
            });

        let bind_group_texture = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_texture_layout,
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

        let view_model_matrix_without_model_scale = glam::Mat4::IDENTITY;

        let uniform_buffer_view_model_matrix_without_model_scale =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("view model matrix without model scale"),
                contents: bytemuck::cast_slice(&[view_model_matrix_without_model_scale]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let threshold: f32 = 0.05;

        let uniform_buffer_threshold =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("threshold"),
                contents: bytemuck::cast_slice(&[threshold]),
                // Mapping at creation (as done by the create_buffer_init utility) doesn't require us to to add the MAP_WRITE usage
                // (this *happens* to workaround this bug )
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let sample_step_length: f32 = 0.005;

        let uniform_buffer_sample_step_length =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("threshold"),
                contents: bytemuck::cast_slice(&[sample_step_length]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let focal_length: f32 = 0.0;

        let uniform_buffer_focal_length =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("focal length"),
                contents: bytemuck::cast_slice(&[focal_length]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let aspect_ratio: f32 = 1.0;

        let uniform_buffer_aspect_ratio =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("aspect ratio"),
                contents: bytemuck::cast_slice(&[aspect_ratio]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let viewport_size = glam::Vec2::default();

        let uniform_buffer_viewport_size =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("viewport size"),
                contents: bytemuck::cast_slice(&[viewport_size]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let viewport_position = glam::Vec2::default();

        let uniform_buffer_viewport_position =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("viewport size"),
                contents: bytemuck::cast_slice(&[viewport_position]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let ray_origin = glam::Vec3::default();

        let uniform_buffer_ray_origin =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ray origin"),
                contents: bytemuck::cast_slice(&[ray_origin]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let top_aabb = glam::Vec3::default();

        let uniform_buffer_top_aabb =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("top aabb"),
                contents: bytemuck::cast_slice(&[top_aabb]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let bottom_aabb = glam::Vec3::default();

        let uniform_buffer_bottom_aabb =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("bottom aabb"),
                contents: bytemuck::cast_slice(&[bottom_aabb]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let camera_position = glam::Vec3::default();

        let uniform_buffer_camera_position =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("camera position"),
                contents: bytemuck::cast_slice(&[camera_position]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let bind_group_layout_uniforms_fragment_shader_stage =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("all fragment shader uniforms"),
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 8,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 9,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 10,
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

        let bind_group_uniforms_fragment_shader_stage =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("all fragment shader uniforms"),
                layout: &bind_group_layout_uniforms_fragment_shader_stage,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer_view_model_matrix_without_model_scale
                            .as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: uniform_buffer_threshold.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: uniform_buffer_sample_step_length.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: uniform_buffer_focal_length.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: uniform_buffer_aspect_ratio.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: uniform_buffer_viewport_size.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: uniform_buffer_viewport_position.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: uniform_buffer_ray_origin.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: uniform_buffer_top_aabb.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: uniform_buffer_bottom_aabb.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 10,
                        resource: uniform_buffer_camera_position.as_entire_binding(),
                    },
                ],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ray_marching_shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &bind_group_layout_uniforms_vertex_shader_stage,
                    &bind_group_texture_layout,
                    &bind_group_layout_uniforms_fragment_shader_stage,
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
        let slice_renderer_resources = RayMarchingRendererResources {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer_projection_view_model_matrix,
            uniform_buffer_view_model_matrix_without_model_scale,
            uniform_buffer_threshold,
            uniform_buffer_sample_step_length,
            uniform_buffer_focal_length,
            uniform_buffer_aspect_ratio,
            uniform_buffer_viewport_size,
            uniform_buffer_viewport_position,
            uniform_buffer_ray_origin,
            uniform_buffer_top_aabb,
            uniform_buffer_bottom_aabb,
            uniform_buffer_camera_position,
            bind_group_uniforms_vertex_shader_stage,
            bind_group_texture,
            bind_group_uniforms_fragment_shader_stage,
        };

        match wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources
            .entry::<std::collections::HashMap<egui::Id, RayMarchingRendererResources>>()
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

        let eye = vec3(0.0, 0.0, -2.0);
        let center = vec3(0.0, 0.0, 0.0);
        let up = vec3(0.0, 1.0, 0.0);
        let view_matrix = glam::Mat4::look_at_lh(eye, center, up);

        Some(Self {
            id,
            threshold,
            sample_step_length,
            focal_length,
            aspect_ratio,
            viewport_size,
            viewport_position,
            ray_origin,
            top_aabb,
            bottom_aabb,
            camera_position,
            rotation_matrix: glam::Mat4::IDENTITY,
            translation_matrix: glam::Mat4::IDENTITY,
            scale_matrix: glam::Mat4::IDENTITY,
            projection_matrix: glam::Mat4::IDENTITY,
            view_matrix,
            dimensions: texture.dimensions,
            spacing: texture.spacing,
            extent: texture.extent,
            show_settings_oberlay: true,
        })
    }
}

impl RayMarchingRenderer {
    fn get_arc_ball_vector(position: glam::Vec2, viewport: glam::Vec3) -> glam::Vec3 {
        let mut arc_ball_vector = vec3(
            1.0 * position.x / viewport.x * 2.0 - 1.0,
            1.0 * position.y / viewport.y * 2.0 - 1.0,
            0.0,
        );
        arc_ball_vector.y = -arc_ball_vector.y;

        let op_squared =
            arc_ball_vector.x * arc_ball_vector.x + arc_ball_vector.y * arc_ball_vector.y;

        if op_squared <= 1.0 {
            arc_ball_vector.z = (1.0 - op_squared).sqrt();
        } else {
            arc_ball_vector = arc_ball_vector.normalize();
        }

        return arc_ball_vector;
    }

    // pub fn custom_painting(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
    pub fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let availbale_size = ui.available_size_before_wrap();
        let (rect, response) =
            ui.allocate_exact_size(availbale_size, egui::Sense::click_and_drag());

        // Clone locals so we can move them into the paint callback:
        let id = self.id;

        let screen_rect = ui.ctx().screen_rect();
        let aspect_ratio = rect.width() / rect.height();
        let viewport_size = Vec2 {
            x: rect.width(),
            y: rect.height(),
        };
        let viewport_position = Vec2 {
            x: rect.left_top().x,
            y: rect.left_top().y,
        };

        let z_near = 0.1;
        let z_far = 10.0;
        let fov_y_radians = 90.0;
        let projection_matrix =
            glam::Mat4::perspective_lh(fov_y_radians, aspect_ratio, z_far, z_near);

        let projection_view_model_matrix = projection_matrix
            * self.view_matrix
            * (self.rotation_matrix * self.translation_matrix * self.scale_matrix);
        let view_model_matrix_without_model_scale =
            self.view_matrix * (self.rotation_matrix * self.translation_matrix);

        let threshold: f32 = self.threshold;
        let sample_step_length = self.sample_step_length;

        // const float projectionMatrixValue1x1 = m_projectionMatrix->constData()[1 * 4 + 1];
        // const float fov = std::atan(1.0f / projectionMatrixValue1x1);
        // const GLfloat focalLength = 1.0f / std::tan(fov);
        let projection_matrix_value1x1: f32 = projection_matrix.y_axis.y;
        let fov: f32 = (1.0 / projection_matrix_value1x1).atan();
        let focal_length = 1.0 / fov.tan();

        let ray_origin_vec4 = view_model_matrix_without_model_scale.clone().inverse()
            * glam::vec4(0.0, 0.0, 0.0, 0.0);
        let ray_origin = ray_origin_vec4.truncate();

        let top_aabb = self.extent;
        let bottom_aabb = -self.extent;

        let camera_position_vec4 =
            projection_view_model_matrix.clone().inverse() * Vec4::new(0.0, 0.0, 2.0, 1.0);
        let camera_position = glam::Vec3::new(
            camera_position_vec4.x,
            camera_position_vec4.y,
            camera_position_vec4.z,
        );

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
                let resources: &std::collections::HashMap<egui::Id, RayMarchingRendererResources> =
                    paint_callback_resources.get().unwrap();
                let slice_render_resources = resources.get(&id).unwrap();
                slice_render_resources.prepare(
                    device,
                    queue,
                    projection_view_model_matrix,
                    view_model_matrix_without_model_scale,
                    threshold,
                    sample_step_length,
                    focal_length,
                    aspect_ratio,
                    viewport_size,
                    viewport_position,
                    ray_origin,
                    top_aabb,
                    bottom_aabb,
                    camera_position,
                );
                Vec::new()
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let resources: &std::collections::HashMap<egui::Id, RayMarchingRendererResources> =
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
                    ui.add(egui::Slider::new(&mut self.threshold, 0.0..=1.0).text("Threshold"));
                });
            ui.ctx().set_visuals(original_visuals);
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

const VERTICES: &[Vertex] = &[
    // wgpu uses the coordinate systems of D3D and Metal
    // https://github.com/gfx-rs/wgpu#coordinate-systems
    Vertex {
        position: [-1.0, -1.0, 1.0],
    }, // 0
    Vertex {
        position: [1.0, -1.0, 1.0],
    }, // 1
    Vertex {
        position: [1.0, 1.0, 1.0],
    }, // 2
    Vertex {
        position: [-1.0, 1.0, 1.0],
    }, // 3
    Vertex {
        position: [-1.0, -1.0, -1.0],
    }, // 4
    Vertex {
        position: [1.0, -1.0, -1.0],
    }, // 5
    Vertex {
        position: [1.0, 1.0, -1.0],
    }, // 6
    Vertex {
        position: [-1.0, 1.0, -1.0],
    }, // 7
];

const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0, // front
    1, 5, 6, 6, 2, 1, // right
    7, 6, 5, 5, 4, 7, // back
    4, 0, 3, 3, 7, 4, // left
    4, 5, 1, 1, 0, 4, // bottom
    3, 2, 6, 6, 7, 3, // top
];
