use bytemuck::bytes_of;
use glam::Vec2;
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct SpotlightRenderPipelineData {
    pub uniforms: SpotlightUniforms,
    pub uniforms_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl SpotlightRenderPipelineData {
    const UNIFORMS_BINDING: u32 = 0;

    pub fn new(
        device: &wgpu::Device,
        surface_configuration: &wgpu::SurfaceConfiguration,
        uniforms: SpotlightUniforms,
    ) -> Self {
        let uniforms_buffer = device.create_buffer_init(&uniforms.buffer_init_descriptor());
        let bind_group_layout_desc = wgpu::BindGroupLayoutDescriptor {
            label: Some("Spotlight Bind Group Layout"),
            entries: &[SpotlightUniforms::bind_group_layout_entry(
                Self::UNIFORMS_BINDING,
            )],
        };
        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_desc);
        let uniforms_buffer_bind_group_entry = wgpu::BindGroupEntry {
            binding: Self::UNIFORMS_BINDING,
            resource: uniforms_buffer.as_entire_binding(),
        };
        let bind_group_desc = wgpu::BindGroupDescriptor {
            label: Some("Spotlight Bind Group"),
            layout: &bind_group_layout,
            entries: &[uniforms_buffer_bind_group_entry],
        };
        let bind_group = device.create_bind_group(&bind_group_desc);
        let render_pipeline_layout_desc = wgpu::PipelineLayoutDescriptor {
            label: Some("Spotlight Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        };
        let render_pipeline_layout = device.create_pipeline_layout(&render_pipeline_layout_desc);
        let shader_module_desc = wgpu::include_wgsl!("../../assets/shaders/spotlight.wgsl");
        let shader_module = device.create_shader_module(shader_module_desc);
        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: Some("Spotlight Render Pipeline"),
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
pub struct SpotlightUniforms {
    pub center_position: Vec2,
    pub radius: f32,
    pub darkness: f32,
    pub aspect_ratio: f32,
    _pad: [u8; 4],
}

impl SpotlightUniforms {
    pub fn new(center_position: Vec2, radius: f32, darkness: f32, aspect_ratio: f32) -> Self {
        Self {
            center_position,
            radius,
            darkness,
            aspect_ratio,
            _pad: [0; 4],
        }
    }

    pub fn buffer_init_descriptor<'a>(&'a self) -> wgpu::util::BufferInitDescriptor<'a> {
        wgpu::util::BufferInitDescriptor {
            label: Some("Spotlight Uniforms Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytes_of(self),
        }
    }

    pub fn bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }
}
