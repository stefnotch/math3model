import package::uniforms_0::{time, screen, mouse, extra, instance_id, linear_sampler};
import package::uniforms_model::{material, t_diffuse};

fn sampleObject(input: vec2f) -> vec3f {
    let size = vec2f(textureDimensions(t_diffuse));
    let uv = round(input * size) / size;
    let pos = toSRGB(textureSampleBaseClampToEdge(
        t_diffuse, 
        linear_sampler, 
        uv * material.texture_scale
    )).xyz;
    
    return pos;
}

fn getColor(input: vec2f, base_color: vec3f) -> vec3f {
    return base_color;
}

fn toSRGBComponent(c: f32) -> f32 { 
  if (c <= 0.0) { return 0.0; }
  if (c < 0.0031308) { return 12.95 * c; }
  if (c < 1.0) { return pow(1.055 * c, 0.41666) - 0.055; }
  return 1.0;
}

fn toSRGB(c: vec4f) -> vec4f {
  return vec4f(
     toSRGBComponent(c.r),
     toSRGBComponent(c.g),
     toSRGBComponent(c.b),
     c.a);
}