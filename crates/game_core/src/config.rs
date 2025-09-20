#![coverage(off)]

use std::fs::{read_to_string, write};
use std::path::Path;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::key_converter::convert;

// =================================================================================================
//
//                                            Global
//
// =================================================================================================

#[derive(Resource, Deserialize, Serialize, Clone, Debug, Default)]
pub struct GlobalConfig {
    pub graphics_config: GraphicsConfig,
    pub input_config: InputConfig,
}

impl GlobalConfig {

    /// Loads a configuration file and deserializes it into the specified type.
    ///
    /// # Arguments
    /// - `path`: The file path of the configuration file to load.
    ///
    /// # Panics
    /// This function will panic if the file cannot be read or parsed correctly.
    ///
    /// # Returns
    /// - `T`: The deserialized configuration data.
    pub fn load<T: for<'de> Deserialize<'de>>(path: &str) -> T {
        let content = read_to_string(Path::new(path)).expect("Failed to read config file");
        toml::from_str(&content).expect("Failed to parse toml file")
    }

    /// Creates a new `GlobalConfig` instance and loads all configuration files.
    ///
    ///
    /// # Returns
    /// - `GlobalConfig`: A new instance with loaded configurations for game, graphics, input, and audio.
    pub fn new() -> Self {
        Self {
            graphics_config: Self::load("config/graphics.toml"),
            input_config: Self::load("config/input.toml"),
        }
    }

    /// Saves a specified file with his name.
    fn save<T: Serialize>(data: &T, path: &str) {
        let toml_string = toml::to_string_pretty(data).expect("Failed to serialize to TOML");
        write(Path::new(path), toml_string).expect("Failed to write config file");
    }

    /// Saves all known config files that found in config/ folder.
    /// This func used `GlobalConfig::save` for saving.
    pub fn save_all(&self) {
        Self::save(&self.graphics_config, "config/graphics.toml");
    }

}

// =================================================================================================
//
//                                            Graphics
//
// =================================================================================================

#[derive(Resource, Deserialize, Serialize, Clone, Debug)]
pub struct GraphicsConfig {
    pub window_resolution: String,

    pub fullscreen: bool,
    pub vsync: bool,

    pub video_backend: String,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            window_resolution: String::from("1270x720"),
            fullscreen: false,
            vsync: true,
            video_backend: String::from("AUTO")
        }
    }
}

impl GraphicsConfig {
    pub fn get_window_width(&self) -> f32 {
        let (width, _) = parse_resolution(self.window_resolution.as_str())
            .unwrap_or_else(|_| (1280.0, 720.0));
        width
    }

    pub fn get_window_height(&self) -> f32 {
        let (_, height) = parse_resolution(self.window_resolution.as_str())
            .unwrap_or_else(|_| (1280.0, 720.0));
        height
    }
}

// =================================================================================================
//
//                                            Input
//
// =================================================================================================

#[derive(Resource, Deserialize, Serialize, Clone, Debug)]
pub struct InputConfig {
    pub inspector: String,
    pub system_info: String,
    pub gizmos_boxen: String,

    pub movement_forward: String,
    pub movement_backward: String,
    pub movement_left: String,
    pub movement_right: String,
    pub movement_jump: String,
    pub movement_sprint: String,
    pub movement_crouch: String
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            inspector: String::from("F1"),
            system_info: String::from("F3"),
            gizmos_boxen: String::from("F9"),

            movement_forward: String::from("W"),
            movement_backward: String::from("S"),
            movement_left: String::from("A"),
            movement_right: String::from("D"),
            movement_jump: String::from("Space"),
            movement_sprint: String::from("ShiftLeft"),
            movement_crouch: String::from("C")
        }
    }
}

impl InputConfig {
    pub fn get_inspector_key(&self) -> KeyCode {
        convert(self.inspector.as_str()).unwrap_or_else(|| KeyCode::F12)
    }

}

// =================================================================================================
//
//                                         Internal Func
//
// =================================================================================================

#[inline]
fn parse_resolution(s: &str) -> Result<(f32, f32), String> {
    let (w_str, h_str) = s
        .trim()
        .split_once(['x', 'X'])
        .ok_or_else(|| format!("Wrong Format: '{}'. Example z. B. 1280x720", s))?;

    let w: f32 = w_str.trim().parse()
        .map_err(|_| format!("Width is not a number: '{}'", w_str.trim()))?;
    let h: f32 = h_str.trim().parse()
        .map_err(|_| format!("Height is not a number: '{}'", h_str.trim()))?;

    if w <= 0.0 || h <= 0.0 {
        return Err("Width / Height needs a positive number like > 0".into());
    }
    Ok((w, h))
}