use std::{collections::HashMap, error::Error, fmt, path::PathBuf};
use kovi::utils::{load_json_data, save_json_data};
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub api_key: String,
    pub hint: String,
    pub forward: bool,
    // prefix(通常是五个字符以内) : model
    pub prefix: HashMap<String, String>,
}


impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let api_key = if self.api_key.trim().is_empty() {
        String::from("None")} else {
          self.api_key.clone()
        };
      let hint = if self.hint.trim().is_empty() {
          String::from("None")} else {
            self.hint.clone()
          };
      write!(f, "api_key: {}\nhint: {}\nforward: {}\nprefix: {:#?}"
        , api_key, hint, self.forward, self.prefix)
    }
}

impl Default for Config {
    fn default() -> Self {
      let mut prefix = HashMap::new();
      prefix.insert(String::from("%"), String::from("deepseek-ai/DeepSeek-V3"));
      prefix.insert(String::from("%%"), String::from("deepseek-ai/DeepSeek-R1"));
        Self{
          api_key: String::new(), hint: String::new(), 
          forward: true, prefix: prefix
        }
    }
}

impl Config {
  // 加载本地插件配置文件，没有则返回默认值并创建默认配置文件
    pub fn load(config_path: &PathBuf) -> Result<Self, Box<dyn Error>> {
      let data: Result<Config, Box<dyn Error>> = load_json_data(Self::default(), config_path);
      match data {
          Ok(data) => return Ok(data),
          Err(e) => return Err(e)
      }
    }
    
    pub fn set_api_key(&self, api_key: String, config_path: &PathBuf) -> Result<String, Box<dyn Error>> {
        let mut cfg = self.clone();
        cfg.api_key = api_key;
        save_json_data(&cfg, config_path)?;
        Ok(String::from("api_key 设置成功!"))
    }

    pub fn set_api_hint(&self, hint: String, config_path: &PathBuf) -> Result<String, Box<dyn Error>> {
      let mut cfg = self.clone();
      cfg.hint = hint;
      save_json_data(&cfg, config_path)?;
      Ok(String::from("提示词设置成功!"))
    }

    pub fn set_forward(&self, forward: bool, config_path: &PathBuf) -> Result<String, Box<dyn Error>>  {
      if forward == self.forward {
        let turn_is_on = if forward {"开启"} else {"关闭"};
        let msg = format!("合并转发状态已{}, 请勿重复操作~", turn_is_on);
        return Ok(msg);
      }
      let mut cfg = self.clone();
      cfg.forward = forward;
      save_json_data(&cfg, config_path)?;
      let turn_is_on = if forward {"开启"} else {"关闭"};
      let msg = format!("合并转发开关设置成功, 当前合并转发开启状态为: {}", turn_is_on);
      Ok(msg)
    }
  
  pub fn set_prefix(&self, prefix: String, model: String, config_path: &PathBuf) -> Result<String, String>   {
    let cfg = self.clone();
    let mut cfg = cfg.clone();
    cfg.prefix.entry(prefix.to_string()).or_insert(model.to_string());
    save_json_data(&cfg, config_path).unwrap();
    let msg = format!("指令设置成功(prefix => model): {} => {}", prefix, model);
    Ok(msg)
  }

  pub fn del_prefix(&self, prefix: String, config_path: &PathBuf) -> Result<String, Box<dyn Error>> {
    let cfg = self.clone();
    let prefix = prefix.trim();
    let mut cfg = cfg.clone();
    match cfg.prefix.remove(prefix) {
      Some(v) => {
        save_json_data(&cfg, config_path).unwrap();
        let msg = format!("触发器删除成功: {} => {}", prefix, v);
        Ok(msg)
      },
      None => {
        let msg  = String::from("触发器删除失败, 原因: 该指令不存在");
        Ok(msg)
      }
    }
  }

}