use crate::transform::{Transform, Vec2Nano, Vec3Nano};
use glam::{Vec2, Vec3};
use nanoserde::{DeJson, SerJson};

#[derive(Clone, PartialEq, DeJson, SerJson)]
pub struct Model {
    pub name: String,
    pub transform: Transform,
    pub material_info: MaterialInfo,
    pub shader_id: ShaderId,
    pub instance_count: u32,
}

#[derive(Clone, PartialEq, DeJson, SerJson)]
pub struct MaterialInfo {
    #[nserde(proxy = "Vec3Nano")]
    pub color: Vec3,
    #[nserde(proxy = "Vec3Nano")]
    pub emissive: Vec3,
    pub roughness: f32,
    pub metallic: f32,
    pub diffuse_texture: Option<TextureId>,
    #[nserde(proxy = "Vec2Nano")]
    pub texture_scale: Vec2,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, DeJson, SerJson)]
pub struct ShaderId(pub String);

#[derive(Clone)]
pub struct ShaderInfo {
    pub label: String,
    pub code: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, DeJson, SerJson)]
pub struct TextureId(pub String);

pub struct TextureInfo {
    pub width: u32,
    pub height: u32,
    /// RGBA
    pub data: TextureData,
}

pub enum TextureData {
    Bytes(Vec<u8>),
    #[cfg(target_arch = "wasm32")]
    Image(web_sys::ImageBitmap),
}

pub enum SceneUpdate {
    RemoveModel(usize),
    AddModel(Model),
    UpdateModel(usize, Model),
}

impl MaterialInfo {
    pub fn missing() -> Self {
        Self {
            color: Vec3::new(1.0, 0.0, 1.0),
            emissive: Vec3::new(1.0, 0.0, 1.0),
            roughness: 0.7,
            metallic: 0.0,
            diffuse_texture: None,
            texture_scale: Vec2::ONE,
        }
    }
}
impl Default for MaterialInfo {
    fn default() -> Self {
        Self {
            color: Vec3::new(0.0, 0.0, 0.0),
            emissive: Vec3::new(0.0, 0.0, 0.0),
            roughness: 0.0,
            metallic: 0.0,
            diffuse_texture: None,
            texture_scale: Vec2::ONE,
        }
    }
}
