use bytemuck::bytes_of;
use glam::{Mat3, Vec4};
use image::RgbaImage;
use wgpu::util::DeviceExt;

use crate::application::texture::DiffuseImageTexture;

#[derive(Debug)]
pub struct ImageRenderPipelineData {
    pub uniforms: ImageUniforms,
    pub uniforms_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl ImageRenderPipelineData {
    const UNIFORMS_BINDING: u32 = 0;
    const TEXTURE_BINDING: u32 = 1;
    const SAMPLER_BINDING: u32 = 2;

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_configuration: &wgpu::SurfaceConfiguration,
        image: &RgbaImage,
        uniforms: ImageUniforms,
    ) -> Self {
        let uniforms_buffer = device.create_buffer_init(&uniforms.buffer_init_descriptor());
        let texture = DiffuseImageTexture::from_image(device, queue, image, Some("Image Texture"));
        let bind_group_layout_desc = wgpu::BindGroupLayoutDescriptor {
            label: Some("Image Bind Group Layout"),
            entries: &[
                ImageUniforms::bind_group_layout_entry(Self::UNIFORMS_BINDING),
                DiffuseImageTexture::texture_bind_group_layout_entry(Self::TEXTURE_BINDING),
                DiffuseImageTexture::sampler_bind_group_layout_entry(Self::SAMPLER_BINDING),
            ],
        };
        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_desc);
        let uniforms_buffer_bind_group_entry = wgpu::BindGroupEntry {
            binding: Self::UNIFORMS_BINDING,
            resource: uniforms_buffer.as_entire_binding(),
        };
        let bind_group_desc = wgpu::BindGroupDescriptor {
            label: Some("Image Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                uniforms_buffer_bind_group_entry,
                texture.texture_view_bind_group_entry(Self::TEXTURE_BINDING),
                texture.sampler_bind_group_entry(Self::SAMPLER_BINDING),
            ],
        };
        let bind_group = device.create_bind_group(&bind_group_desc);
        let render_pipeline_layout_desc = wgpu::PipelineLayoutDescriptor {
            label: Some("Image Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        };
        let render_pipeline_layout = device.create_pipeline_layout(&render_pipeline_layout_desc);
        let shader_module_desc = wgpu::include_wgsl!("../../assets/shaders/image.wgsl");
        let shader_module = device.create_shader_module(shader_module_desc);
        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: Some("Image Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_configuration.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        };
        let render_pipeline = device.create_render_pipeline(&render_pipeline_desc);

        Self {
            uniforms,
            uniforms_buffer,
            bind_group,
            render_pipeline,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImageUniforms {
    x_col: Vec4,
    y_col: Vec4,
    z_col: Vec4,
}

impl ImageUniforms {
    pub fn new(transform: Mat3) -> Self {
        Self {
            x_col: Vec4::from((transform.x_axis, 0.0)),
            y_col: Vec4::from((transform.y_axis, 0.0)),
            z_col: Vec4::from((transform.z_axis, 0.0)),
        }
    }

    pub fn set_transform(&mut self, transform: Mat3) {
        self.x_col = Vec4::from((transform.x_axis, 0.0));
        self.y_col = Vec4::from((transform.y_axis, 0.0));
        self.z_col = Vec4::from((transform.z_axis, 0.0));
    }

    pub fn buffer_init_descriptor<'a>(&'a self) -> wgpu::util::BufferInitDescriptor<'a> {
        wgpu::util::BufferInitDescriptor {
            label: Some("Image Uniforms Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytes_of(self),
        }
    }

    pub fn bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }
}
