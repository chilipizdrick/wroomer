use crate::application::vec2d::Vec2f32;

// Gpu uniforms buffer
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::NoUninit)]
pub struct Uniforms {
    pub canvas_size: Vec2f32,
    pub image_size: Vec2f32,
    pub image_offset: Vec2f32,
    pub cursor_position: Vec2f32,
    pub spotlight_color: [f32; 4],
    pub zoom_factor: f32,
    pub spotlight_radius_multiplier: f32,
    _padding: [u8; 12],
}

impl Uniforms {
    pub fn with_canvas_and_image_size(canvas_size: Vec2f32, image_size: Vec2f32) -> Self {
        Self {
            canvas_size,
            image_size,
            ..Default::default()
        }
    }

    pub fn buffer_init_descriptor<'a>(&'a self) -> wgpu::util::BufferInitDescriptor<'a> {
        wgpu::util::BufferInitDescriptor {
            label: Some("Uniforms Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::bytes_of(self),
        }
    }

    pub fn bind_group_layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniforms Bind Group Layout Descriptor"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        }
    }
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            canvas_size: Vec2f32::new(0.0, 0.0),
            image_size: Vec2f32::new(0.0, 0.0),
            image_offset: Vec2f32::new(0.0, 0.0),
            cursor_position: Vec2f32::new(0.0, 0.0),
            spotlight_color: [0.0, 0.0, 0.0, 0.0],
            zoom_factor: 1.0,
            spotlight_radius_multiplier: 1.0,
            _padding: [0; 12],
        }
    }
}
