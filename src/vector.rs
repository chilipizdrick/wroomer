use winit::dpi::{PhysicalPosition, PhysicalSize};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, bytemuck::NoUninit)]
pub struct Vec2f32 {
    pub x: f32,
    pub y: f32,
}

impl Vec2f32 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl std::ops::Add for Vec2f32 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for Vec2f32 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Mul for Vec2f32 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl std::ops::Mul<f32> for Vec2f32 {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl std::ops::Div for Vec2f32 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}

impl std::ops::Div<f32> for Vec2f32 {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl std::ops::Neg for Vec2f32 {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl std::ops::AddAssign for Vec2f32 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl std::ops::SubAssign for Vec2f32 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl std::ops::MulAssign for Vec2f32 {
    fn mul_assign(&mut self, other: Self) {
        self.x *= other.x;
        self.y *= other.y;
    }
}

impl std::ops::MulAssign<f32> for Vec2f32 {
    fn mul_assign(&mut self, other: f32) {
        self.x *= other;
        self.y *= other;
    }
}

impl std::ops::DivAssign for Vec2f32 {
    fn div_assign(&mut self, other: Self) {
        self.x /= other.x;
        self.y /= other.y;
    }
}

impl std::ops::DivAssign<f32> for Vec2f32 {
    fn div_assign(&mut self, other: f32) {
        self.x /= other;
        self.y /= other;
    }
}

impl From<(f32, f32)> for Vec2f32 {
    fn from(value: (f32, f32)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<[f32; 2]> for Vec2f32 {
    fn from(value: [f32; 2]) -> Self {
        Self::new(value[0], value[1])
    }
}

impl From<PhysicalPosition<f64>> for Vec2f32 {
    fn from(value: PhysicalPosition<f64>) -> Self {
        Self::new(value.x as f32, value.y as f32)
    }
}

impl From<PhysicalSize<u32>> for Vec2f32 {
    fn from(value: PhysicalSize<u32>) -> Self {
        Self::new(value.width as f32, value.height as f32)
    }
}
