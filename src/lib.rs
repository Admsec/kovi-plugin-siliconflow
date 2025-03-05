use std::{error::Error, sync::Arc};
use kovi::{async_move, serde_json::{from_value, json, Value}, Message, MsgEvent, PluginBuilder as plugin, RuntimeBot};
use reqwest::Client;
use config::Config;
use response::{ChatCompletion, R1ChatCompletion, UserProFile, V3ChatCompletion};

mod config;
mod response;

const API_URL: &str = "https://api.siliconflow.cn/v1/chat/completions";
const USER_PROFILE_URL: &str = "https://api.siliconflow.cn/v1/user/info";
const PLUGIN_NAME: &str = "kovi-plugin-siliconflow";
const HELP: &str = ".sc help 帮助\n.sc info config 列出当前配置\n.sc info user 获取用户信息\n.sc update <api_key> 更新api_key\n.sc hint <提示词> 更新提示词";

#[kovi::plugin]
async fn main() {
    let bot = plugin::get_runtime_bot();
    plugin::on_msg(async_move!(e; bot;{
        if !e.raw_message.starts_with('%') {
            return;
        }
        ask_question_main(e, bot).await;
    }));
    plugin::on_admin_msg(async_move!(e;bot;{
        // 填 Api、查余额
        let prefix = ".sc";
        if !e.raw_message.starts_with(prefix){
            return ;
        }
        manager_plugin(e, bot).await
    }));
}

async fn ask_question_main(e: Arc<MsgEvent>, bot: Arc<RuntimeBot>){
    let config_path = bot.get_data_path().join("config.json");
    let cfg = Arc::new(Config::new(&config_path).expect("Failed to load config"));
    let raw_msg = e.get_text();
    match parse_prefix(raw_msg.as_str()) {
        Ok((prefix, msg)) => {
            if msg.is_empty(){
                return ;
            }
            if cfg.api_key_is_null() {
                e.reply("api_key 是空的, 请使用命令: .sc update <api_key> 或手动填写 api_key");
            }
            send_poke(&bot, &e).await;
            handle_message(e, cfg, prefix, msg.as_str()).await;
        },
        Err(_) => reply_error(e, "无法解析消息前缀"),
    }
}

async fn manager_plugin(e: Arc<MsgEvent>, bot: Arc<RuntimeBot>){
    let raw_msg: Vec<&str> = e.raw_message.as_str().split_whitespace().collect();
    let config_path = bot.get_data_path().join("config.json");
    let cfg = Arc::new(Config::new(&config_path).expect("Failed to load config"));
    if cfg.api_key_is_null(){
        e.reply("喵发现你的 api_key 是空的哟, 你可以使用以下命令更新你的 api_key, 否则功能受限哟喵~\n.sc update <api_key>");
    }

    match raw_msg.as_slice() {
        [_, "info", "config"] => {
            e.reply(Message::from(cfg.to_string()));
        }, 
        [_, "info", "user"] => {
            if cfg.api_key_is_null() {
                reply_error(e, "api_key 为空呢喵~");
                return ;
            }
            let api_key = &cfg.api_key;
            let user_data = get_user_profile(api_key).await;
            match user_data {
                Ok(data) => e.reply(reply_user_profile(data)),
                Err(err) => {
                    reply_error(e, err.to_string().as_str())
                }
            }
            
        },
        [_, "update", new_api_key] => {
            let result = cfg.set_api_key(new_api_key.to_string(), &config_path);
            if let Err(err) = result {
                reply_error(e, err.to_string().as_str());
            }
        },
        [_, "hint", new_hint] => {
            let result = cfg.set_api_hint(new_hint.to_string(), &config_path);
            if let Err(err) = result {
                reply_error(e, err.to_string().as_str());
            }
        }
        _ => e.reply(HELP),
    }
}

async fn get_user_profile(api_key: &str) -> Result<UserProFile, reqwest::Error> {
    let client = Client::new();
    let response: UserProFile = client.get(USER_PROFILE_URL)
    .header("Authorization", format!("Bearer {}", api_key))
    .send()
    .await?
    .json()
    .await?;
    println!("{response:#?}");
    Ok(response)
}
fn reply_user_profile(user_data: UserProFile) -> Message{
    let mut msg = Message::new();
    let user_data = user_data.data;
    let text = format!("用户名: {}\n邮箱: {}\n赠送余额: {}\n状态: {}\n总余额: {}",
    &user_data.name,&user_data.email, &user_data.balance, &user_data.status, &user_data.totalBalance);
    msg.push_image(&user_data.image);
    msg.push_text(text);
    msg
}

async fn send_poke(bot: &Arc<RuntimeBot>, e: &Arc<MsgEvent>) {
    let params = if e.is_group() {
        json!({ "group_id": e.group_id, "user_id": e.user_id })
    } else {
        json!({ "user_id": e.user_id })
    };
    bot.send_api("send_poke", params);
}

async fn handle_message(
    e: Arc<MsgEvent>,
    cfg: Arc<config::Config>,
    prefix: u8,
    msg: &str
) {
    let model = match prefix {
        1 => "deepseek-ai/DeepSeek-R1",
        0 => "deepseek-ai/DeepSeek-V3",
        _ => {
            reply_error(e, "未知的模型");
            panic!("未知的模型")
        }
    };
    if let Err(err) = process_answer_data(&e, model, msg, cfg).await {
        reply_error(e, err.to_string().as_str());
    }
}

async fn process_answer_data(e: &Arc<MsgEvent> , model: &str, msg: &str, cfg: Arc<config::Config>) -> Result<(), Box<dyn Error>>{
    let response = send_chat_request(msg, model, cfg.api_key.as_str(), cfg.hint.as_str()).await?;
    match  model {
        "deepseek-ai/DeepSeek-R1" => {
            let r1_json: R1ChatCompletion = from_value(response)?;
            reply_success(e, r1_json.get_plain_msg()).await;
            Ok(())
        },
        "deepseek-ai/DeepSeek-V3" => {
            let v3_json: V3ChatCompletion = from_value(response)?;
            reply_success(e, v3_json.get_plain_msg()).await;
            Ok(())
        },
        _ => Ok(())
    }
}

async fn reply_success(e: &Arc<MsgEvent>, msg: &str) {
    // let msg = format!("{}\n—{}", msg, model);
    e.reply(Message::from(msg));
}

fn reply_error(e: Arc<MsgEvent>, msg: &str) {
    let formatted = format!("[-]{}: {}", PLUGIN_NAME, msg);
    e.reply(Message::from(formatted));
}

fn parse_prefix(raw: &str) -> Result<(u8, String), ()> {
    raw.strip_prefix("%%")
        .map(|msg| (1, msg.to_string()))
        .or_else(|| raw.strip_prefix('%').map(|msg| (0, msg.to_string())))
        .ok_or(())
}

async fn send_chat_request(
    message: &str,
    model: &str,
    api_key: &str,
    hint: &str,
) -> Result<Value, reqwest::Error> {
    let client = Client::new();
    let messages = if hint.is_empty() {
        vec![json!({"role": "user", "content": message})]
    } else {
        vec![
            json!({"role": "system", "content": hint}),
            json!({"role": "user", "content": message})
        ]
    };

    let payload = json!({
        "messages": messages,
        "model": model,
        "stream": false
    });

    let response = client.post(API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await?
        .json().await;
    response
}