struct Time {
    elapsed: f32,
    delta: f32,
    frame: u32,
}
struct Screen {
    resolution: vec2<u32>,
    inv_resolution: vec2<f32>,
}
struct Mouse {
    pos: vec2<f32>,
    buttons: u32,
}
struct Extra {
    hot_value: f32
}