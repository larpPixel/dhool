use std::{fs, path::PathBuf};

use dhool_config::Config;

pub fn export_config(config: &Config) -> Result<Option<PathBuf>, String> {
    let default_name = format!("{}.yaml", sanitize_stem(&config.scene.name));
    let Some(path) = rfd::FileDialog::new()
        .add_filter("YAML config", &["yaml", "yml"])
        .set_file_name(&default_name)
        .save_file()
    else {
        return Ok(None);
    };

    let yaml = serde_yaml::to_string(config)
        .map_err(|err| format!("failed to serialize config: {err}"))?;
    fs::write(&path, yaml).map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    Ok(Some(path))
}

pub fn load_config() -> Result<Option<(PathBuf, Config)>, String> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("YAML config", &["yaml", "yml"])
        .pick_file()
    else {
        return Ok(None);
    };

    let config = Config::load(&path).map_err(|err| err.to_string())?;
    config.validate().map_err(|err| err.to_string())?;
    Ok(Some((path, config)))
}

fn sanitize_stem(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}
