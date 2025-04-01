#![allow(warnings)]
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
pub struct  RequestResponse {
    pub answer: HashMap<String, String>,
    pub reason: bool
}

impl RequestResponse {
    pub fn new(ans: HashMap<String, String> , reason: bool ) -> Self {
        Self { answer: ans, reason: reason }
    }
}

pub enum ChatCompletions {
    ReasonChatCompletion,
    GeneralCompletions
}

#[derive(Debug, Serialize, Deserialize)]
// 无思考内容的
pub struct GeneralCompletions {
  pub id: String,
  pub object: String,
  pub created: u64,
  pub model: String,
  pub choices: Vec<GeneralChoice>,
  pub usage: Usage,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub system_fingerprint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralChoice {
    pub index: u32,
    pub message: GeneralMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralMessage {
    pub role: String,
    pub content: String,
}


// 有思考内容的
#[derive(Debug, Serialize, Deserialize)]
pub struct ReasonChatCompletion {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ReasonChoice>,
    pub usage: Usage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReasonChoice {
    pub index: u32,
    pub message: ReasonMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReasonMessage {
    pub role: String,
    pub content: String,
    pub reasoning_content: String,
}

// 通用的
#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
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
pub struct V3ChatCompletion {
    #[serde(flatten)]
    base: ChatCompletionBase,
    pub choices: Vec<V3Choice>,
}

#[derive(Debug, Deserialize)]
struct V3Choice {
    index: u32,
    message: V3Message,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct V3Message {
    role: String,
    content: String,
}

#[derive(Error, Debug)]
pub enum ResponseError {
  #[error("响应体解析错误")]
  ParseResBodyError
}