use anyhow::*;
use eframe::wgpu;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub dimensions: glam::UVec3,
    pub spacing: glam::Vec3,
    pub extent: glam::Vec3,
    pub origin: glam::Vec3,
}

impl Texture {
    pub fn default<'a>(cc: &'a eframe::CreationContext<'a>) -> Result<Self> {
        let file_name = "init_volume";
        let dimensions = glam::UVec3::new(1, 1, 1);
        let spacing = glam::Vec3::new(1.0, 1.0, 1.0);
        let mut volume_data_bytes: Vec<u8> =
            vec![127; (dimensions.x * dimensions.y * dimensions.z * 2) as usize];

        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap();
        let device = &wgpu_render_state.device;
        let queue = &wgpu_render_state.queue;

        Self::from_u16_bytes(
            device,
            queue,
            &mut volume_data_bytes,
            dimensions,
            spacing,
            Some(file_name),
        )
    }
    pub fn from_u16_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &mut Vec<u8>,
        dimensions: glam::UVec3,
        spacing: glam::Vec3,
        label: Option<&str>,
    ) -> Result<Self> {
        for i in 0..(bytes.len() / 2) {
            let number_u16 = bytes[i * 2] as u16 | (bytes[i * 2 + 1] as u16) << 8;
            let number_f16 = half::f16::from_f32(number_u16 as f32 / u16::MAX as f32);
            let bytes_f16 = half::f16::to_le_bytes(number_f16);
            bytes[i * 2] = bytes_f16[0];
            bytes[i * 2 + 1] = bytes_f16[1];
        }
        Self::from_f16_bytes(device, queue, bytes, dimensions, spacing, label)
    }

    pub fn from_f16_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        dimensions: glam::UVec3,
        spacing: glam::Vec3,
        label: Option<&str>,
    ) -> Result<Self> {
        let size = wgpu::Extent3d {
            width: dimensions.x,
            height: dimensions.y,
            depth_or_array_layers: dimensions.z,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            // use R16Float since R16Unorm is not portable for all backends
            format: wgpu::TextureFormat::R16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                // needs to ba a multiple of 256 according to https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/#getting-data-into-a-texture
                // bytes_per_row: std::num::NonZeroU32::new(2 * dimensions.0),
                // rows_per_image: std::num::NonZeroU32::new(dimensions.1),
                bytes_per_row: Some(2 * dimensions.x),
                rows_per_image: Some(dimensions.y),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let extent = glam::Vec3 {
            x: spacing.x * dimensions.x as f32,
            y: spacing.y * dimensions.y as f32,
            z: spacing.z * dimensions.z as f32,
        };

        let origin = glam::Vec3::default();

        Ok(Self {
            texture,
            view,
            sampler,
            dimensions,
            spacing,
            extent,
            origin,
        })
    }
}
