import package::uniforms_0::{time, screen, mouse, extra, linear_sampler};
import package::uniforms_model::{material, t_diffuse};

fn sampleObject(input: vec2f) -> vec3f {
    _ = time;
    _ = screen;
    _ = mouse;
    _ = extra;
    _ = material;
    _ = t_diffuse;
    _ = linear_sampler;
    return vec3(input, 0.0);
}

fn getColor(input: vec2f, base_color: vec3f) -> vec3f {
    return base_color;
}