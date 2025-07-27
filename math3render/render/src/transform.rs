use glam::{Mat4, Vec3};
use nanoserde::{DeJson, SerJson};

#[derive(Debug, Copy, Clone, PartialEq, DeJson, SerJson)]
pub struct Transform {
    #[nserde(proxy = "Vec3Nano")]
    pub position: Vec3,
    #[nserde(proxy = "QuatNano")]
    pub rotation: glam::Quat,
    pub scale: f32,
}
//aa
impl Transform {
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            Vec3::new(self.scale, self.scale, self.scale),
            self.rotation,
            self.position,
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: glam::Quat::IDENTITY,
            scale: 1.0,
        }
    }
}

#[derive(DeJson, SerJson)]
pub struct Vec2Nano([f32; 2]);
impl From<&glam::Vec2> for Vec2Nano {
    fn from(value: &glam::Vec2) -> Self {
        Self(value.to_array())
    }
}
impl From<&Vec2Nano> for glam::Vec2 {
    fn from(value: &Vec2Nano) -> Self {
        Self::from_array(value.0)
    }
}
#[derive(DeJson, SerJson)]
pub struct Vec3Nano([f32; 3]);
impl From<&Vec3> for Vec3Nano {
    fn from(value: &Vec3) -> Self {
        Self(value.to_array())
    }
}
impl From<&Vec3Nano> for Vec3 {
    fn from(value: &Vec3Nano) -> Self {
        Self::from_array(value.0)
    }
}
#[derive(DeJson, SerJson)]
pub struct QuatNano([f32; 4]);

impl From<&glam::Quat> for QuatNano {
    fn from(value: &glam::Quat) -> Self {
        Self(value.to_array())
    }
}
impl From<&QuatNano> for glam::Quat {
    fn from(value: &QuatNano) -> Self {
        Self::from_array(value.0)
    }
}
