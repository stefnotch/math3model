# Math3Model

> A spiritual successor to [Math2Model](https://github.com/cg-tuwien/Math2Model)

We combine "Geometry Images" with [Fast Rendering of Parametric Objects on Modern GPUs](https://www.cg.tuwien.ac.at/research/publications/2024/unterguggenberger-2024-fropo/) to let you create fancy 3D shapes directly on the GPU!

Right now, the project is just being set up. Big overarching list is

- [x] Set up project with [wgpu](https://github.com/gfx-rs/wgpu), [wesl](https://wesl-lang.dev/) and [hot reloading](https://docs.rs/subsecond/0.7.0-alpha.0/subsecond/index.html)
- [ ] Integrate parametric rendering
- [ ] Render geometry image
- [ ] Integrate geometry images generator
- [ ] Integrate Dioxus for GUI
- [ ] Run in web

## Running

```
cargo run
```

## Development

This project uses the Dioxus hot patching magic. Run it with
```
dx serve --hot-patch --package masters-desktop
```
after installing the latest Dioxus CLI (`cargo binstall dioxus-cli@0.7.0-alpha.0`)

TODO: https://github.com/DioxusLabs/dioxus/tree/main/packages/subsecond
and https://github.com/jkelleyrtp/subsecond-bevy-demo/blob/main/src/main.rs