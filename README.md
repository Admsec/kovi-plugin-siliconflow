## 简介

一款对接硅基流动官方的问答插件，仅需硅基流动的 api_key 就可以使用

## 打个广告

现在硅基流动搞活动，注册的人填写邀请码之后，注册人和被邀请人都可以返14块钱代金券，可以用 deepseek-r1 等等大模型(~~~白嫖怪还不赶紧来~~~)

硅基流动官网: https://siliconflow.cn/zh-cn/

我的邀请链接: https://cloud.siliconflow.cn/i/PcVMvRDw (~~~当然邀请码不想用我的也可以~~~)

或者你不想注册的话，闲鱼两块钱买个 api_key 也能用

## 使用方法

1. 添加本插件

```
cargo kovi add kovi-plugin-siliconflow
```

2. 对着机器人使用 .sc api_key set <api_key> 指令更新 api_key ，或者手动填写 api_key，路径是 <kovi-bot>/data/kovi-plugin-siliconflow/config.json

```
.sc api_key set <api_key>
```

3. 本插件还有一些其他功能，命令前缀是 .sc

```
.sc help: 帮助
.sc info config: 列出当前配置
.sc info user: 获取用户信息
.sc api_key set <api_key>: 更新api_key
.sc hint set <提示词>: 更新提示词
.sc forward set <true|false>: 开启/关闭消息转发
.sc prefix set <prefix> <model>: 设置触发器和对应模型
.sc prefix del <prefix>: 删除触发器
```

