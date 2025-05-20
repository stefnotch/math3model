# Interactive Web Editing for Parametric Objects and Geometry Images

Aka "Geometry Images" plus [Fast Rendering of Parametric Objects on Modern GPUs](https://www.cg.tuwien.ac.at/research/publications/2024/unterguggenberger-2024-fropo/)

I wonder when I'll do the "web" part.

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