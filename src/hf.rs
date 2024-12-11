use crate::config::huggingface_config;
use crate::file_handler::FileHandler;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::path::Path;

pub struct VoiceManager {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Language {
    pub code: String,
    region: String,
    name_native: String,
    name_english: String,
    country_english: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Voice {
    pub name: String,
    pub key: String,
    pub language: Language,
    pub quality: String,
    #[serde(default)]
    pub downloaded: bool,
    pub files: HashMap<String, File>,
}

impl VoiceManager {
    pub fn list_all_avaliable_voices() -> Result<BTreeMap<String, Voice>, Box<dyn Error>> {
        let voices_url = huggingface_config::get_voices_url();
        let voices_file = FileHandler::download_file(voices_url)?;
        let raw_json = voices_file.text()?;

        let value_data: Value = serde_json::from_str(&raw_json)?;
        let mut voices: BTreeMap<String, Voice> = serde_json::from_value(value_data.clone())?;

        let downloaded_voices = Self::list_downloaded_voices()?;
        downloaded_voices.iter().for_each(|f| {
            if let Some(voice) = voices.get_mut(f) {
                voice.downloaded = true;
            }
        });

        Ok(voices)
    }

    pub fn list_downloaded_voices() -> Result<Vec<String>, Box<dyn Error>> {
        let downloaded_voices =
            FileHandler::get_all_file_names(&huggingface_config::get_download_path())?;
        let downloaded_voices: Vec<String> = downloaded_voices
            .iter()
            .map(|f| f.split(".").next().unwrap_or(f).to_string())
            .collect();

        Ok(downloaded_voices)
    }

    pub fn download_voice(voice_files: &HashMap<String, File>) -> Result<(), Box<dyn Error>> {
        for (file_path, _) in voice_files {
            if file_path.ends_with(".onnx") {
                // voice file
                let voice_url = huggingface_config::get_voice_url(&file_path);
                let mut voice_res = FileHandler::download_file(voice_url)?;
                let file_name = Path::new(file_path)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .ok_or("Failed to properly extract file name from path")?;

                FileHandler::save_file(
                    &huggingface_config::get_voice_file_path(file_name),
                    &mut voice_res,
                )?;

                // voice json config
                let mut voice_config_url = huggingface_config::get_voice_url(&file_path);
                voice_config_url.push_str(".json");
                let mut config_res = FileHandler::download_file(voice_config_url)?;
                let file_name = Path::new(file_path)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .ok_or("Failed to properly extract file name from path")?;

                FileHandler::save_file(
                    &huggingface_config::get_voice_file_path(file_name),
                    &mut config_res,
                )?;
            }
        }

        Ok(())
    }

    pub fn delete_voice(voice_files: &HashMap<String, File>) -> Result<(), Box<dyn Error>> {
        for (file_path, _) in voice_files {
            if file_path.ends_with(".onnx") {
                let file_name = Path::new(file_path)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .ok_or("Failed to properly extract file name from path")?;

                FileHandler::remove_file(&huggingface_config::get_voice_file_path(file_name))?
            }
        }

        Ok(())
    }
}
