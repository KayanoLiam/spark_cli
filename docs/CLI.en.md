# spark_cli Command Reference

This document lists all commands, flags, and examples for the current implementation.

> Development entrypoint:
>
> ```bash
> cargo run -- [global flags] [subcommand] [positional]
> ```

## Global flags
- `--config <PATH>`: use an explicit config file (project/user).
- `--provider <NAME>`: override provider for this run.
- `--model <NAME>`: override model for this run.
- `--stream`: stream responses (SSE) when supported.
- `-f, --file <PATH>`: read prompt from file.
- `-o, --output <PATH>`: write output to file.

Positional:
- `[prompt ...]`: one-shot chat when no subcommand is used.

## Config
- Initialize (user):
  ```bash
  cargo run -- config init
  ```
- Initialize (project):
  ```bash
  cargo run -- config init --scope project
  ```
- Set and list:
  ```bash
  cargo run -- config set provider openrouter
  cargo run -- config set api-key sk-xxxx
  cargo run -- config list
  ```

Supported fields in config: `provider`, `api_key`, `model`, `base_url` (for OpenAI-compatible providers)

## Chat
- One-shot:
```bash
cargo run -- "Explain Rust ownership in one sentence"
```
- Subcommand:
```bash
cargo run -- chat "Hello"
```
- With flags:
```bash
cargo run -- --config ./config.toml --model openrouter/auto chat "Hello"
cargo run -- -f prompt.txt -o answer.txt chat
```

## Interactive
```bash
cargo run -- interactive
```

## Sessions
```bash
cargo run -- session new "my project"
cargo run -- session list
cargo run -- session load <id>
cargo run -- session delete <id>
```

## Code workflows
- Generate:
```bash
cargo run -- code generate --lang cpp --type "hello world" --code-only -o hello.cpp
cargo run -- code generate --lang rust --type "small http server" --out-dir out/ --code-only
```
- Review:
```bash
cargo run -- code review --file src/main.rs
```
- Optimize:
```bash
cargo run -- code optimize --file src/lib.rs
```

## Providers
- OpenRouter (default for `provider=openrouter`)
- OpenAI-compatible (DeepSeek/Qwen/OpenAI): set `provider` accordingly and provide `base_url` in config, e.g. `https://api.deepseek.com/v1`

## Troubleshooting
- API keys: set via config or env `OPENROUTER_API_KEY`.
- Smart quotes in keys can cause auth failures; use ASCII quotes.
- For OpenAI-compatible providers, ensure `base_url` is set.
