pub const DEFAULT_PARAMETRIC: &str = include_str!("../wgsl/samples/DefaultParametric.wgsl");
pub const HEART_SPHERE: &str = include_str!("../wgsl/samples/HeartSphere.wgsl");

include!(concat!(env!("OUT_DIR"), "/shaders.rs"));
