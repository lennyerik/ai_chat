use std::{fs, io, path::PathBuf};

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
    Serde(#[from] serde_json::Error),
}

impl Config {
    /// Gets all the possible config paths for the current system im order of importance
    /// TODO: This currently only works on linux
    pub fn get_config_paths() -> Result<Vec<PathBuf>, Error> {
        let home = PathBuf::from(std::env::var("HOME").map_err(|_| Error::NoHomeDir)?);
        Ok(vec![
            home.join(".config/ai_chat.toml"),
            home.join(".config/ai_chat/config.toml"),
            home.join(".ai_chat.toml"),
        ])
    }

    /// Opens a cofiguration file on disk or returns `ConfigError::FileDoesNotExist` if it does not exist
    fn open_config_file() -> Result<Option<fs::File>, Error> {
        let possible_paths = Self::get_config_paths()?;

        for path in possible_paths {
            if path.exists() && path.is_file() {
                return Ok(Some(
                    fs::File::open(&path).map_err(|e| Error::FileIO(e, path))?,
                ));
            }
        }

        Ok(None)
    }

    /// Reads the configuration from disk and returns a `Config` object
    pub fn read_from_disk() -> Result<Option<Self>, Error> {
        if let Some(file) = Self::open_config_file()? {
            Ok(Some(serde_json::from_reader(file)?))
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

        let file = fs::File::create(path).map_err(|e| Error::FileIO(e, path.clone()))?;
        serde_json::to_writer_pretty(file, &default_config)?;

        Ok(default_config)
    }
}
