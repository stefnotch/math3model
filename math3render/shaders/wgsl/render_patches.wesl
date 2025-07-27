import package::utils::{RenderBufferRead, patch_decode};
import package::uniforms_0::{Time, Screen, Mouse, Extra};
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

//// START sampleObject
fn sampleObject(input: vec2f) -> vec3f {
  let a = time;
  let b = screen;
  let c = mouse;
  let d = extra;
  return vec3(input, 0.0); 
}
//// END sampleObject
//// START getColor
fn getColor(input: vec2f) -> vec3f {
  if material.has_texture != 0u {
    return textureSample(t_diffuse, linear_sampler, input * material.texture_scale).rgb;
  } else {
    return material.color_roughness.rgb;
  }
}
//// END getColor

// following https://www.martinpalko.com/triplanar-mapping/
fn calculateTriplanarColor(input: vec3f, normal: vec3f) -> vec3f {
  if material.has_texture != 0u {
    let yuv = input.xz;
    let zuv = input.xy;
    let xuv = input.zy;

    let yDiff = textureSample (t_diffuse, linear_sampler, yuv).rgb;
    let xDiff = textureSample (t_diffuse, linear_sampler, xuv).rgb;
    let zDiff = textureSample (t_diffuse, linear_sampler, zuv).rgb;
    let blendWeights = normalize(abs(normal));

  	return xDiff * blendWeights.x + yDiff * blendWeights.y + zDiff * blendWeights.z;
  } else {
    return material.color_roughness.rgb;
  }
}

var<private> instance_id: u32;

// Group 0 is for constants that change once per frame at most
@group(0) @binding(0) var<uniform> time : Time;
@group(0) @binding(1) var<uniform> screen : Screen;
@group(0) @binding(2) var<uniform> mouse : Mouse;
@group(0) @binding(3) var<uniform> extra : Extra;

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

@group(0) @binding(4) var<uniform> camera: Camera;
@group(0) @binding(5) var<storage, read> lights: Lights;
@group(0) @binding(6) var linear_sampler: sampler;
@group(1) @binding(1) var<uniform> model: Model;
@group(1) @binding(2) var<storage, read> render_buffer: RenderBufferRead;
@group(1) @binding(3) var<uniform> material: Material;
@group(1) @binding(4) var t_diffuse: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) texture_coords: vec2<f32>,
    @location(3) color: vec4<f32>,
}

const color_options = array<vec4f,8>(
    vec4f(1.0, 0.0, 0.0, 1.0),
    vec4f(0.0, 1.0, 0.0, 1.0),
    vec4f(0.0, 0.0, 1.0, 1.0),
    vec4f(1.0, 1.0, 0.0, 1.0),
    vec4f(1.0, 0.0, 1.0, 1.0),
    vec4f(0.0, 1.0, 1.0, 1.0),
    vec4f(1.0, 1.0, 1.0, 1.0),
    vec4f(0.5, 0.5, 0.5, 1.0),
);

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

    let i = in.instance_index % 8;
    if (i == 0) {
    out.color = color_options[0];
    } else if (i == 1) {
    out.color = color_options[1];
    } else if (i == 2) {
    out.color = color_options[2];
    } else if (i == 3) {
    out.color = color_options[3];
    } else if (i == 4) {
    out.color = color_options[4];
    } else if (i == 5) {
    out.color = color_options[5];
    } else if (i == 6) {
    out.color = color_options[6];
    } else if (i == 7) {
    out.color = color_options[7];
    }else {
    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }
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
      let a = sampleObject(vec2f(0.0)); 
    }
    let v = normalize(camera.world_position.xyz - in.world_position);
    // let n = normalize(in.world_normal);
    let n = normalize(-cross(dpdxFine(in.world_position), dpdyFine(in.world_position)));

    var materialInfo = MaterialInfo(
        getColor(in.texture_coords),
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
    // return in.color; TODO: Why does this cause z-buffer fighting? 
    // Maybe we have z-buffer fighting, and with consistent colors we just don't notice it?
}

 

 