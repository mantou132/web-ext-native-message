## web-ext-native-message

浏览器扩展进程 <-- 标准输入/输出 --> wenm <-- Socket 进程间通信 --> 本机 App

作为一个本机 App 和浏览器通信的中间程序，如果采用 nodejs 编写，打包文件较大，所以使用 rust 编写，
兼容 nodejs 的 node-ipc，用来和 node 端的 node-ipc 通信

### 使用

```bash
# 编译
cargo build --release
# 应用启动时会自动连接 path 为 `/tmp/app.<filename>` 的 unix socket
# 想要自定义 path，只需要重命名二进制文件
```

### 开发

```powershell
# 编译并运行
cargo run
# 启动测试 socket，打印了 `node-ipc` 的 socket 的一些信息
npm i
node ./
```

### node-ipc 通信规范

- [平台](https://nodejs.org/api/net.html#net_ipc_support): UNIX domain / named pipe
- 名称: `\\.\pipe\tmp-app.${id}` / `/tmp/app.${id}`
- 消息格式: [js-message](https://www.npmjs.com/package/js-message)
  - 序列化消息 + 分割符(默认换页符)
  - type
  - data // 支持 json

### web-ext 通信规范

前面 4 个字节存长度，后面写 `utf-8` 编码的字符串内容。

注意：webextension 会自动序列化
