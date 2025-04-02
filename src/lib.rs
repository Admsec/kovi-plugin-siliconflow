use std::{collections::HashMap, error::Error, sync::Arc};
use kovi::{async_move, serde_json::{from_str, json}, Message, MsgEvent, PluginBuilder as plugin, RuntimeBot};
use reqwest::Client;
use config::Config;
use response::{GeneralCompletions, ReasonChatCompletion, RequestResponse, UserProFile};
use kovi_plugin_expand_napcat::NapCatVec;
mod config;
mod response;

const HELP: &str = ".sc help: 帮助
.sc info config: 列出当前配置
.sc info user: 获取用户信息
.sc api_key set <api_key>: 更新api_key
.sc hint set <提示词>: 更新提示词
.sc forward set <true|false>: 开启/关闭消息转发
.sc prefix set <prefix> <model>: 设置触发器和对应模型
.sc prefix del <prefix>: 删除触发器";

#[kovi::plugin]
async fn main() {
    let bot = plugin::get_runtime_bot();
    plugin::on_msg(async_move!(e; bot;{
        ask_question_main(e, bot).await;
    }));
    plugin::on_admin_msg(async_move!(e;bot;{
        manager_plugin(e, bot).await
    }));
}

// 收发 Ai 回答内容
async fn ask_question_main(e: Arc<MsgEvent>, bot: Arc<RuntimeBot>){
  let config_path = bot.get_data_path().join("config.json");
  let cfg = Config::load(&config_path).map_err(|err|{
    let msg = format!("加载配置文件出错了, 原因: {}", err.to_string());
    e.reply_and_quote(msg);
    return ;
  }).unwrap();
  let text = e.borrow_text();
  match text {
      Some(pre_match_str) => {
        let match_prefix = cfg.prefix.keys().filter(|&x|{
          let x_len = x.chars().count();
          x == &pre_match_str.chars().take(x_len).collect::<String>()
        });
        if match_prefix.clone().count() < 1{
          return ;
        }
        let mut match_prefix_vec: Vec<&String> = match_prefix.clone().collect();
        match_prefix_vec.sort_by(|&a, &b| b.len().cmp(&a.len()));
        let prefix = match_prefix_vec[0];
        let question = e.raw_message.trim_start_matches(prefix);
        let model = cfg.prefix.get(prefix).unwrap();
        let uid = &e.self_id.to_string();
        let nickname = &e.get_sender_nickname();
        send_poke(&bot, &e).await;
        let ans = get_ans_from_api(&e, cfg.api_key, model.to_string(), cfg.hint, question.to_string())
        .await;
        match ans {
            Ok(req_result) => {
              if cfg.forward{
                let mut nodes = Vec::new();
                nodes.push_fake_node_from_content(uid, nickname, Message::from(&req_result.answer["message"]));
                if req_result.reason {
                  nodes.push_fake_node_from_content(uid, nickname, Message::from(&req_result.answer["reason_message"]));
                }
                e.reply(nodes);
              } else {
                // 没开启消息转发的话，直接发送回答内容
                e.reply(Message::from(&req_result.answer["message"]));
              }
            },
            Err(err) => {
              let msg = format!("Api请求出错了, {}", err.to_string());
              e.reply_and_quote(msg);
            }
        }
      },
      None => {}
  }

}

// 管理配置文件
async fn manager_plugin(e: Arc<MsgEvent>, bot: Arc<RuntimeBot>){
    if !e.raw_message.starts_with(".sc"){
        return ;
    }
    let raw_msg: Vec<&str> = e.raw_message.as_str().split_whitespace().collect();
    let config_path = bot.get_data_path().join("config.json");
    let cfg = Config::load(&config_path).map_err(|x|{
      let msg = format!("加载配置文件出错了, 原因: {}", x.to_string());
      e.reply_and_quote(msg);
    }).unwrap();

    match raw_msg.as_slice() {
        [_, "info", "config"] => {
            e.reply(Message::from(cfg.to_string()));
        }, 
        [_, "info", "user"] => {
            let api_key = &cfg.api_key;
            let user_data = get_user_profile(api_key).await;
            match user_data {
                Ok(data) => e.reply(reply_user_profile(data)),
                Err(err) => {
                    e.reply_and_quote(err.to_string());
                }
            }
            
        },
        [_, "api_key", "set", new_api_key] => {
            let result = cfg.set_api_key(new_api_key.to_string(), &config_path);
            match result {
                Ok(res) => {
                  e.reply_and_quote(res);
                },
                Err(err) => {
                  e.reply_and_quote(err.to_string());
                }
            }
        },
        [_, "hint", "set", new_hint@ ..] => {
          let new_hint = new_hint.join(" ");
            let result = cfg.set_api_hint(new_hint, &config_path);
            match result {
              Ok(res) => {
                e.reply_and_quote(res);
              },
              Err(err) => {
                e.reply_and_quote(err.to_string());
              }
          }
        },
        [_, "forward", "set", action] => {
          if let Ok(action) = action.parse::<bool>() {
              let result = cfg.set_forward(action, &config_path);
              match result{
                Ok(result) => {
                  e.reply_and_quote(result);
                },
                Err(err) => {
                  e.reply_and_quote(err.to_string());
                }
              }
          } else {
            let msg = format!("消息转发开关只有true和false, 你输入了意外的字符, 请重新设置");
            e.reply_and_quote(msg);
          }
        },
        [_, "prefix", "set", prefix, model] =>{
          let result = cfg.set_prefix(prefix.to_string(), model.to_string(), &config_path);
          match result {
              Ok(result) => e.reply_and_quote(result),
              Err(err) => e.reply_and_quote(err),
          }
        },
        [_, "prefix", "del", prefix] =>{
          let result = cfg.del_prefix(prefix.to_string(), &config_path);
          match result {
            Ok(result) => e.reply_and_quote(result),
            Err(err) => e.reply_and_quote(err.to_string()),
          }

        }
        _ => {
          if cfg.api_key.trim().is_empty(){
            e.reply_and_quote("喵发现你的 api_key 是空的哟, 你可以使用以下命令更新你的 api_key, 否则功能受限哟喵~\n.sc api_key set <api_key>");
        }
        e.reply(HELP);
    }
  }
}

async fn get_user_profile(api_key: &str) -> Result<UserProFile, reqwest::Error> {
    let client = Client::new();
    let user_profile_url = "https://api.siliconflow.cn/v1/user/info";
    let response: UserProFile = client.get(user_profile_url)
    .header("Authorization", format!("Bearer {}", api_key))
    .send()
    .await?
    .json()
    .await?;
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

// 从 Api 获取处理过的消息文本
async fn get_ans_from_api(
    e: &Arc<MsgEvent>,
    api_key: String,
    model: String,
    hint: String,
    qestion: String
) -> Result<RequestResponse, Box<dyn Error>>{
  let client = Client::new();
  let api_url = "https://api.siliconflow.cn/v1/chat/completions";
  let plugin_name= "kovi-plugin-siliconflow";
  let reasonable = || {
    let v = &["Qwen/QwQ-32B"];
    if let Some(_) = &model.find("DeepSeek-R1"){
      return true;
    }
    if v.iter().filter(|&&x| x == &model).count() > 0{
      return true;
    }
    return false;
  };
  let messages = if hint.trim().is_empty() {
      vec![json!({"role": "user", "content": &qestion})]
  } else {
      vec![
          json!({"role": "system", "content": hint}),
          json!({"role": "user", "content": &qestion})
      ]
  };

  let payload = json!({
      "messages": messages,
      "model": model,
      "stream": false
  });

  let response = client.post(api_url)
      .header("Authorization", format!("Bearer {}", api_key))
      .json(&payload)
      .send()
      .await?
      .text().await?;
  
  let result;
  if reasonable() {
    let response_json: ReasonChatCompletion = from_str(&response).map_err(|err| {
      let msg = format!("[{}] 响应体解析错误, {}", plugin_name, err.to_string());
      e.reply_and_quote(msg);
    }).unwrap();
    let mut msg_result = HashMap::new();
    let msg = &response_json.choices.get(0).unwrap().message;
    msg_result.entry(String::from("message")).or_insert(msg.content.clone());
    msg_result.entry(String::from("reason_message")).or_insert(msg.reasoning_content.clone());
    result = RequestResponse::new(msg_result, true);
  } else {
    let response_json: GeneralCompletions = from_str(&response).map_err(|err| {
      let msg = format!("[{}] 响应体解析错误, {}", plugin_name, err.to_string());
      e.reply_and_quote(msg);
    }).unwrap();
    let mut msg_result = HashMap::new();
    let msg = &response_json.choices.get(0).unwrap().message;
    msg_result.entry(String::from("message")).or_insert(msg.content.clone());
    result = RequestResponse::new(msg_result, false);
  }
  Ok(result)

}

