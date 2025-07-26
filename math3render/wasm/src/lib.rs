mod application;
pub mod wasm_abi;

use log::Level;
use wasm_bindgen::prelude::*;
#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Info).unwrap();
}
