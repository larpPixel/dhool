mod app;
mod export;
mod panels;
mod render_thread;
mod timeline;
mod video_render;

use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use dhool_config::Config;

const DEFAULT_CONFIG_YAML: &str = include_str!("../../configs/inferno.production.yaml");

#[derive(Debug)]
struct Cli {
    config_path: Option<PathBuf>,
}

fn main() -> eframe::Result<()> {
    let cli = parse_args().unwrap_or_else(|message| {
        eprintln!("{message}");
        eprintln!("usage: dhool-live [--config <path>]");
        std::process::exit(2);
    });
    let config = load_initial_config(cli.config_path.as_deref()).unwrap_or_else(|message| {
        eprintln!("{message}");
        std::process::exit(2);
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([960.0, 620.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    let source_path = cli.config_path.clone();

    eframe::run_native(
        "dhool live preview",
        options,
        Box::new(move |cc| Ok(Box::new(app::LiveApp::new(cc, config, source_path)))),
    )
}

fn parse_args() -> Result<Cli, String> {
    let mut config_path = None;
    let mut args = env::args_os().skip(1);
    while let Some(arg) = args.next() {
        if arg == OsStr::new("--help") || arg == OsStr::new("-h") {
            println!("usage: dhool-live [--config <path>]");
            std::process::exit(0);
        }
        if arg == OsStr::new("--config") || arg == OsStr::new("-c") {
            let path = args
                .next()
                .ok_or_else(|| "--config requires a path".to_string())?;
            config_path = Some(PathBuf::from(path));
        } else {
            return Err(format!("unknown argument: {}", arg.to_string_lossy()));
        }
    }
    Ok(Cli { config_path })
}

fn load_initial_config(path: Option<&Path>) -> Result<Config, String> {
    let config = match path {
        Some(path) => Config::load(path).map_err(|err| err.to_string())?,
        None => serde_yaml::from_str(DEFAULT_CONFIG_YAML)
            .map_err(|err| format!("failed to parse bundled default config: {err}"))?,
    };
    config.validate().map_err(|err| err.to_string())?;
    Ok(config)
}
