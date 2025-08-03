use super::FrameData;
use crate::{
    buffer::{DeviceBufferExt, TypedBuffer},
    camera::Camera,
    time::FrameTime,
};
use glam::{Mat4, UVec2, Vec2, Vec4};
use shaders::{compute_patches, pbr, render_patches, uniforms_0};

pub struct SceneData {
    pub time_buffer: TypedBuffer<uniforms_0::Time>,
    pub screen_buffer: TypedBuffer<uniforms_0::Screen>,
    pub mouse_buffer: TypedBuffer<uniforms_0::Mouse>,
    pub extra_buffer: TypedBuffer<uniforms_0::Extra>,
    pub camera_buffer: TypedBuffer<render_patches::Camera>,
    pub light_buffer: TypedBuffer<render_patches::Lights>,
    pub linear_sampler: wgpu::Sampler,

    pub scene_bind_group: render_patches::bind_groups::BindGroup0,
    // TODO: Remove duplication
    pub scene_bind_group_compute: compute_patches::bind_groups::BindGroup0,
}

impl SceneData {
    pub fn new(device: &wgpu::Device) -> Self {
        let time_buffer = device.uniform_buffer(
            "Time Buffer",
            &uniforms_0::Time {
                elapsed: 0.0,
                delta: 1000.0 / 60.0,
                frame: 0,
            },
            wgpu::BufferUsages::COPY_DST,
        );
        let screen_buffer = device.uniform_buffer(
            "Screen Buffer",
            &uniforms_0::Screen {
                resolution: UVec2::ONE,
                inv_resolution: Vec2::ONE,
            },
            wgpu::BufferUsages::COPY_DST,
        );
        let mouse_buffer = device.uniform_buffer(
            "Mouse Buffer",
            &uniforms_0::Mouse {
                pos: Vec2::ZERO,
                buttons: 0,
            },
            wgpu::BufferUsages::COPY_DST,
        );
        let extra_buffer = device.uniform_buffer(
            "Mouse Buffer",
            &uniforms_0::Extra { hot_value: 0. },
            wgpu::BufferUsages::COPY_DST,
        );
        let camera_buffer = device.uniform_buffer(
            "Camera Buffer",
            &render_patches::Camera {
                view: Mat4::IDENTITY,
                projection: Mat4::IDENTITY,
                world_position: Vec4::ZERO,
            },
            wgpu::BufferUsages::COPY_DST,
        );
        let light_buffer = device.storage_buffer(
            "Light Buffer",
            &render_patches::Lights {
                ambient: Vec4::new(0.05, 0.05, 0.05, 0.0),
                points_length: 4,
                points: vec![
                    pbr::LightSource {
                        position_range: glam::Vec3::new(1.0, -4.0, 1.0).normalize().extend(1.0),
                        color_intensity: Vec4::new(0.5, 0.55, 0.5, 0.9),
                        light_type: pbr::LIGHT_TYPE_DIRECTIONAL,
                    },
                    pbr::LightSource {
                        position_range: Vec4::new(0.0, 8.0, 4.0, 80.0),
                        color_intensity: Vec4::new(1.0, 1.0, 1.0, 1.0),
                        light_type: pbr::LIGHT_TYPE_POINT,
                    },
                    pbr::LightSource {
                        position_range: Vec4::new(1.0, 8.0, -6.0, 70.0),
                        color_intensity: Vec4::new(1.0, 1.0, 1.0, 1.5),
                        light_type: pbr::LIGHT_TYPE_POINT,
                    },
                    pbr::LightSource {
                        position_range: Vec4::new(0.0, -8.0, 0.0, 80.0),
                        color_intensity: Vec4::new(0.8, 0.8, 1.0, 0.9),
                        light_type: pbr::LIGHT_TYPE_POINT,
                    },
                ],
            },
            wgpu::BufferUsages::COPY_DST,
        );
        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let scene_bind_group = render_patches::bind_groups::BindGroup0::from_bindings(
            device,
            render_patches::bind_groups::BindGroupLayout0 {
                camera: camera_buffer.as_buffer_binding(),
                time: time_buffer.as_buffer_binding(),
                screen: screen_buffer.as_buffer_binding(),
                extra: extra_buffer.as_buffer_binding(),
                mouse: mouse_buffer.as_buffer_binding(),
                lights: light_buffer.as_buffer_binding(),
                linear_sampler: &linear_sampler,
            },
        );
        let scene_bind_group_compute = compute_patches::bind_groups::BindGroup0::from_bindings(
            device,
            compute_patches::bind_groups::BindGroupLayout0 {
                time: time_buffer.as_buffer_binding(),
                screen: screen_buffer.as_buffer_binding(),
                extra: extra_buffer.as_buffer_binding(),
                mouse: mouse_buffer.as_buffer_binding(),
                linear_sampler: &linear_sampler,
            },
        );
        Self {
            time_buffer,
            screen_buffer,
            mouse_buffer,
            extra_buffer,
            camera_buffer,
            light_buffer,
            linear_sampler,
            scene_bind_group,
            scene_bind_group_compute,
        }
    }

    pub fn update(
        &self,
        size: UVec2,
        render_data: &FrameData,
        frame_time: &FrameTime,
        queue: &wgpu::Queue,
    ) {
        self.time_buffer.write_buffer(
            queue,
            &uniforms_0::Time {
                elapsed: frame_time.elapsed.0,
                delta: frame_time.delta.0,
                frame: frame_time.frame as u32,
            },
        );
        self.screen_buffer.write_buffer(
            queue,
            &uniforms_0::Screen {
                resolution: size,
                inv_resolution: Vec2::new(1.0 / (size.x as f32), 1.0 / (size.y as f32)),
            },
        );
        self.mouse_buffer.write_buffer(
            queue,
            &uniforms_0::Mouse {
                pos: render_data.mouse_pos,
                buttons: if render_data.mouse_held { 1 } else { 0 },
            },
        );
        self.camera_buffer
            .write_buffer(queue, &render_data.camera.to_shader(size));
    }
}

impl Camera {
    fn to_shader(&self, size: UVec2) -> render_patches::Camera {
        render_patches::Camera {
            view: self.view_matrix(),
            projection: self.projection_matrix(size),
            world_position: self.position.extend(1.0),
        }
    }
}
