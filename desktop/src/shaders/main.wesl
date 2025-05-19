import package::util::rotate_3u;

@compute
@workgroup_size(1,1,1)
fn main(@builtin(global_invocation_id) index: vec3u) -> vec3u {
    return rotate_3u(index);
}