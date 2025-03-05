#![allow(warnings)]
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ChatCompletionBase {
    id: String,
    object: String,
    created: u64,
    pub model: String,
    pub usage: Usage,
    system_fingerprint: String,
}

#[derive(Debug, Deserialize)]
pub struct UserProFile {
    code: u32,
    message: String,
    status: bool,
    pub data: UserData,
}

#[derive(Debug, Deserialize)]
pub struct UserData {
    id: String,
    pub name: String,
    pub image: String,
    pub email: String,
    isAdmin: bool,
    pub balance: String,
    pub status: String,
    pub introduction: String,
    role: String,
    chargeBalance: String,
    pub totalBalance: String,
    category: String
}

#[derive(Debug, Deserialize)]
pub struct R1ChatCompletion {
    #[serde(flatten)]
    base: ChatCompletionBase,
    pub choices: Vec<R1Choice>,
}

#[derive(Debug, Deserialize)]
pub struct V3ChatCompletion {
    #[serde(flatten)]
    base: ChatCompletionBase,
    pub choices: Vec<V3Choice>,
}

#[derive(Debug, Deserialize)]
struct R1Choice {
    index: u32,
    message: R1Message,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct V3Choice {
    index: u32,
    message: V3Message,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct R1Message {
    role: String,
    content: String,
    reasoning_content: String,
}

#[derive(Debug, Deserialize)]
struct V3Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// 为各类型实现统一的方法
pub trait ChatCompletion {
    fn get_plain_msg(&self) -> &str;
    fn get_model(&self) -> &str;
}

impl ChatCompletion for R1ChatCompletion {
    fn get_plain_msg(&self) -> &str {
        self.choices.first()
            .map(|c| c.message.content.as_str())
            .unwrap_or_default()
    }

    fn get_model(&self) -> &str {
        &self.base.model
    }
}

impl ChatCompletion for V3ChatCompletion {
    fn get_plain_msg(&self) -> &str {
        self.choices.first()
            .map(|c| c.message.content.as_str())
            .unwrap_or_default()
    }

    fn get_model(&self) -> &str {
        &self.base.model
    }
}