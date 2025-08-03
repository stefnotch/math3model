struct Material {
    // color.rgb is the color of the material
    // color.a is the roughness of the material
    color_roughness: vec4<f32>,
    // emissive_metallic.rgb is the emissive color of the material
    // emissive_metallic.a is the metallicness of the material
    emissive_metallic: vec4<f32>,
    // is a boolean
    has_texture: u32,
    texture_scale: vec2f
}

@group(1) @binding(0) var<uniform> material: Material;
@group(1) @binding(1) var t_diffuse: texture_2d<f32>;