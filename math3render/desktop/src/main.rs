mod application;
mod config;

use application::run;
use env_logger::Env;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .filter_module("wgpu_hal::vulkan::instance", log::LevelFilter::Warn)
        .filter_module("naga::back::spv::writer", log::LevelFilter::Warn)
        .init();
    run()
}
