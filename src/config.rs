use std::{error::Error, fmt, path::PathBuf};
use kovi::{serde_json::{from_value, json, Value}, utils::{load_json_data, save_json_data}};
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;

lazy_static! {
    static ref DEFAULT_CONFIG: Value = json!({
        "api_key": "",
        "hint": "",
    });
}



#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub api_key: String,
    pub hint: String
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{  api_key: {}\n  hint: {}}}", self.api_key, self.hint)
    }
}
impl Config {
    pub fn new(config_path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let data = load_json_data(DEFAULT_CONFIG.clone(), config_path)?;
        Ok(from_value(data)?)
    }

    #[inline]
    pub fn api_key_is_null(&self) -> bool {
        self.api_key.is_empty()
    }
    
    pub fn set_api_key(&self, api_key: String, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let cfg = Config{
            api_key: api_key,
            hint: self.hint.clone()
        };
        save_json_data(&cfg, file_path)
    }

    pub fn set_api_hint(&self, hint: String, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let cfg = Config{
            api_key: self.api_key.clone(),
            hint: hint
        };
        save_json_data(&cfg, file_path)
    }

}