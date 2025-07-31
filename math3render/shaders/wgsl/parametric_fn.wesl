import package::uniforms_0::{time, screen, mouse, extra};

fn sampleObject(input: vec2f) -> vec3f {
    let a = time;
    let b = screen;
    let c = mouse;
    let d = extra;
    return vec3(input, 0.0);
}

fn getColor(input: vec2f, base_color: vec3f) -> vec3f {
    return base_color;
}