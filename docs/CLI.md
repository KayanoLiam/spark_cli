# spark_cli 命令参考

本文档列出 `spark_cli` 的所有命令、参数与示例，适用于当前实现版本。

> 运行入口（开发模式）：
>
> ```bash
> cargo run -- [全局参数] [子命令] [位置参数]
> ```

## 全局参数
- `--config <PATH>`：显式指定配置文件路径（默认从用户级或项目级自动解析）。
- `--provider <NAME>`：单次运行覆盖服务商（默认读取配置）。
- `--model <NAME>`：单次运行覆盖模型（默认读取配置）。
- `-f, --file <PATH>`：从文件读取提示词作为输入。
- `-o, --output <PATH>`：将输出写入文件。

位置参数：
- `[prompt ...]`：不使用子命令时，直接作为一次性聊天提示输入。

## 配置管理

### 初始化配置
- 用户级（默认）：
  ```bash
  cargo run -- config init
  ```
- 项目级：在当前目录生成 `config.toml`
  ```bash
  cargo run -- config init --scope project
  ```
- 覆盖已存在文件：
  ```bash
  cargo run -- config init --force
  cargo run -- config init --scope project --force
  ```

### 设置与查看
- 设置字段：
  ```bash
  cargo run -- config set provider openrouter
  cargo run -- config set api-key sk-or-v1-xxxx
  ```
- 查看当前配置：
  ```bash
  cargo run -- config list
  ```

说明：
- 支持的配置字段：`provider`, `api_key`, `model`
- 环境变量兜底：`OPENROUTER_API_KEY`

## 聊天与交互

### 一次性聊天（无子命令）
```bash
cargo run -- "用一句话解释Rust的所有权"
```

### chat 子命令
```bash
cargo run -- chat "你好"
```

- 结合全局参数：
  ```bash
  cargo run -- --config ./config.toml --model openrouter/auto chat "你好"
  cargo run -- -f prompt.txt -o answer.txt chat
  ```

交互模式（占位，待实现）：
```bash
cargo run -- interactive
```

## 会话管理

### 新建会话
```bash
cargo run -- session new "项目名称"
```

### 列出会话
```bash
cargo run -- session list
```

### 切换当前会话
```bash
cargo run -- session load <会话ID>
```

### 删除会话
```bash
cargo run -- session delete <会话ID>
```

说明：
- 会话存储在 `~/.spark_cli/sessions/<ID>`
- 历史以 JSON Lines 写入 `history.jsonl`，并记录 `CURRENT` 指向当前会话
- 若存在当前会话，`chat`/一次性聊天会将用户与助手消息自动写入

## 代码相关（占位）
```bash
cargo run -- code generate --lang rust --type "web server"
cargo run -- code review --file src/main.rs
cargo run -- code optimize --file src/lib.rs
```

## 提示与故障排查
- OpenRouter 认证：确保 `provider=openrouter` 且设置了 `api_key`，或使用环境变量 `OPENROUTER_API_KEY`。
- 中文引号问题：粘贴 Key 时避免 `“……”`，本工具已做规范化，但建议使用英文引号或不加引号。
- 覆盖模型：可通过 `--model` 临时覆盖，或在配置文件中设置 `model`。
- 项目级配置：使用 `config init --scope project` 在根目录生成 `config.toml`，并通过 `--config ./config.toml` 指定。
