pub struct BindCache<T> {
    pub bindgroup: T,
    last_bindgroup: LastBindgroup,
}

macro_rules! bind_cache {
    ($name:path {
        $($field_name:ident: $field_value:expr,)*
    }) => {{
        $name {
            $($field_name: $field_value,)*
        }
    }};
}

struct LastBindgroup {
    bindings: Vec<OwnedBinding>,
}

#[derive(Clone, PartialEq, Eq)]
enum OwnedBinding {
    Buffer {
        buffer: wgpu::Buffer,
        offset: wgpu::BufferAddress,
        size: Option<wgpu::BufferSize>,
    },
    Texture {
        view: wgpu::TextureView,
    },
}

impl<'a> From<wgpu::BufferBinding<'a>> for OwnedBinding {
    fn from(value: wgpu::BufferBinding<'a>) -> Self {
        OwnedBinding::Buffer {
            buffer: value.buffer.clone(),
            offset: value.offset,
            size: value.size,
        }
    }
}

impl<'a> From<&'a wgpu::TextureView> for OwnedBinding {
    fn from(value: &'a wgpu::TextureView) -> Self {
        OwnedBinding::Texture {
            view: value.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CuteLayout {
        foo: u32,
        bar: f32,
    }

    #[test]
    fn test_macro() {
        let foo = bind_cache!(CuteLayout {
            foo: 33,
            bar: 1f32.abs(),
        });
    }
}
