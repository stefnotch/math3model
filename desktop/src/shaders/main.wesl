import package::util::rotate_3u;

@compute
fn main(@builtin(global_invocation_id) index: vec3u) -> vec3u {
    return rotate_3u(index);
}