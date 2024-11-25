use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Default)]
pub enum ModelType {
    #[default]
    Duckduckgo,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub model: ModelType,
    pub ddg_chat_model: Option<crate::ddg::DDGChatModel>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("could not determine user home directory")]
    NoHomeDir,

    #[error("could not determine configuration path on this system")]
    NoValidConfigPaths,

    #[error("{0}: {1:?}")]
    FileIO(io::Error, PathBuf),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),
}

impl Config {
    /// Gets all the possible config paths for the current system im order of importance
    /// TODO: This currently only works on linux
    #[cfg(target_os = "linux")]
    pub fn get_config_paths() -> Result<Vec<PathBuf>, Error> {
        let home = PathBuf::from(std::env::var("HOME").map_err(|_| Error::NoHomeDir)?);
        Ok(vec![
            home.join(".config/ai_chat.toml"),
            home.join(".config/ai_chat/config.toml"),
            home.join(".ai_chat.toml"),
        ])
    }

    fn read_config_file_to_string() -> Result<Option<String>, Error> {
        let possible_paths = Self::get_config_paths()?;

        for path in possible_paths {
            if path.exists() && path.is_file() {
                return Ok(Some(
                    fs::read_to_string(&path).map_err(|e| Error::FileIO(e, path))?,
                ));
            }
        }

        Ok(None)
    }

    /// Reads the configuration from disk and returns a `Config` object
    pub fn read_from_disk() -> Result<Option<Self>, Error> {
        if let Some(conf_string) = Self::read_config_file_to_string()? {
            Ok(Some(toml::from_str(&conf_string)?))
        } else {
            Ok(None)
        }
    }

    /// Writes the default configuration to disk and returns the `Config` object
    pub fn write_default() -> Result<Self, Error> {
        let default_config = Self::default();

        let paths = Self::get_config_paths()?;
        let path = paths.first().ok_or(Error::NoValidConfigPaths)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| Error::FileIO(e, parent.to_path_buf()))?;
        }

        let mut file = fs::File::create(path).map_err(|e| Error::FileIO(e, path.clone()))?;
        write!(
            file,
            "{}",
            toml::to_string_pretty(&default_config)
                .expect("Serialisation of default config struct failed")
        )
        .map_err(|e| Error::FileIO(e, path.clone()))?;

        Ok(default_config)
    }
}
