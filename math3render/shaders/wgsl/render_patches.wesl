import package::utils::{RenderBufferRead, patch_decode};
import package::uniforms_0::{time, screen, mouse, extra, instance_id, linear_sampler};
import package::uniforms_model::{material, t_diffuse};
import package::pbr::{
    LightSource, 
    MaterialInfo, 
    LIGHT_TYPE_DIRECTIONAL, 
    getMetallicRoughnessInfo, 
    getLightIntensity,
    BRDF_specularGGX,
    BRDF_lambertian,
    clamped_dot
};
import package::parametric_fn::{sampleObject, getColor};

alias Vec3Padded = vec4<f32>;

struct Camera {
    world_position: Vec3Padded,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}

struct Lights {
    ambient: Vec3Padded,
    // TODO: Directional light
    points_length: u32,
    points: array<LightSource>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) vertex_index: u32,
}

struct Model {
    model_similarity: mat4x4<f32>,
    object_id: u32
}



@group(0) @binding(5) var<uniform> camera: Camera;
@group(0) @binding(6) var<storage, read> lights: Lights;
@group(1) @binding(2) var<uniform> model: Model;
@group(2) @binding(0) var<storage, read> render_buffer: RenderBufferRead;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) texture_coords: vec2<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    let quad = patch_decode(render_buffer.patches[in.instance_index]);
    let quad_point = mix(quad.min, quad.max, in.uv);
    instance_id = quad.instance;
    let pos = sampleObject(quad_point);
    let world_pos = model.model_similarity * vec4<f32>(pos, 1.0);


    var out: VertexOutput;
    out.clip_position = camera.projection * camera.view * world_pos;
    out.world_position = world_pos.xyz;
    out.texture_coords = quad_point;
    let normal = vec3<f32>(0.0, -1.0, 0.0); // TODO: We'll compute this later
    out.world_normal = (model.model_similarity * vec4<f32>(normal, 0.0)).xyz; // Only uniform scaling
    return out;
}

struct FragmentOutput {
  @location(0) color: vec4f,
  @location(1) object_id: u32,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    if(false) {
      // silly hack to get the auto layout to pick up on the uniforms
      _ = sampleObject(vec2f(0.0)); 
    }
    let v = normalize(camera.world_position.xyz - in.world_position);
    // let n = normalize(in.world_normal);
    let n = normalize(-cross(dpdxFine(in.world_position), dpdyFine(in.world_position)));

    var base_color = material.color_roughness.rgb;
    if material.has_texture != 0u {
        base_color = textureSample(t_diffuse, linear_sampler, in.texture_coords * material.texture_scale).rgb;
    }
    
    var materialInfo = MaterialInfo(
        getColor(in.texture_coords, base_color),
        vec3f(0.04),
        vec3f(1.0),
        vec3f(0.0),
        1.0,
        1.0
    );
    materialInfo = getMetallicRoughnessInfo(materialInfo, material.emissive_metallic.a, material.color_roughness.a);

    var f_diffuse = vec3f(0.0);
    var f_specular = vec3f(0.0);
    for (var i: u32 = 0u; i < lights.points_length; i += 1u) {
        let light = lights.points[i];
        var pointToLight: vec3f;
        if (light.light_type != LIGHT_TYPE_DIRECTIONAL)
        {
            pointToLight = light.position_range.xyz - in.world_position;
        }
        else
        {
            pointToLight = -light.position_range.xyz;
        }


        let l = normalize(pointToLight); // Direction from surface point to light
        let h = normalize(l + v);        // Direction of the vector between l and v, called halfway vector
        let intensity: vec3f = getLightIntensity(light, pointToLight);
        let NdotL = clamped_dot(n, l);
        if(NdotL > 0.0) {
            let NdotV = clamped_dot(n, v);
            let NdotH = clamped_dot(n, h);
            let VdotH = clamped_dot(v, h);
            f_diffuse += intensity * NdotL *  BRDF_lambertian(
                materialInfo.f0, 
                materialInfo.f90, 
                materialInfo.c_diff, 
                materialInfo.specularWeight, 
                VdotH
            );
            f_specular += intensity * NdotL * BRDF_specularGGX(
                materialInfo.f0, 
                materialInfo.f90, 
                materialInfo.alphaRoughness, 
                materialInfo.specularWeight, 
                VdotH, 
                NdotL, 
                NdotV, 
                NdotH
            );
        }
    }

    let ambient: vec3f = lights.ambient.rgb * materialInfo.baseColor;

    let color = f_diffuse * 2.0 
        + f_specular * 2.0 
        + ambient 
        + material.emissive_metallic.rgb;

    var fragmentOutput: FragmentOutput;
    fragmentOutput.color = vec4f(color, 1.0);
    fragmentOutput.object_id = model.object_id;
    return fragmentOutput;
}

 

 