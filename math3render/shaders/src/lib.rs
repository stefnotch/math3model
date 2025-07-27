pub const DEFAULT_PARAMETRIC: &str = include_str!("../wgsl/DefaultParametric.wgsl");
pub const HEART_SPHERE: &'static str = include_str!("../wgsl/HeartSphere.wgsl");

include!(concat!(env!("OUT_DIR"), "/shaders.rs"));
