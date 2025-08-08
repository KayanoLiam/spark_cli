# spark_cli

A Rust-based AI-assisted command-line tool with provider-agnostic design and a strong developer workflow. It currently supports OpenRouter for chat/completions and adds batteries-included UX: interactive mode, streaming output, session history, config layers, and code-focused workflows.

## Features
- Unified CLI with subcommands
- Providers: OpenRouter (extensible architecture)
- Config system: user-level and project-level, explicit `--config` override
- Secrets: environment fallback `OPENROUTER_API_KEY`, smart quote normalization
- Interactive chat: `interactive` mode with history recording
- Streaming output: `--stream` (SSE) with smooth printing
- Session management: new/list/load/delete, JSONL history per session
- File I/O: `-f/--file` input, `-o/--output` output
- Code workflows: `code generate/review/optimize`, progress spinner, stream support
- Code extraction: `--code-only`, multi-block write to `--out-dir`

## Quickstart
1) Install Rust toolchain.
2) Clone and build:
```bash
cargo build
```
3) Initialize config:
```bash
# user-level (default)
cargo run -- config init
# or project-level
cargo run -- config init --scope project
```
4) Set provider and API key (OpenRouter):
```bash
cargo run -- config set provider openrouter
cargo run -- config set api-key sk-or-v1-XXXX
```
Or export environment variable:
```bash
export OPENROUTER_API_KEY=sk-or-v1-XXXX
```

## Usage
- One-shot chat:
```bash
cargo run -- "Explain Rust ownership in one sentence"
```
- Streaming output:
```bash
cargo run -- --stream "Hello there"
```
- Interactive mode:
```bash
cargo run -- interactive
```
- Code generate:
```bash
cargo run -- code generate --lang cpp --type "hello world" --code-only -o hello.cpp
# or multiple files
cargo run -- code generate --lang rust --type "small http server" --out-dir out/ --code-only
```
- Session management:
```bash
cargo run -- session new "my project"
cargo run -- session list
cargo run -- session load <id>
cargo run -- session delete <id>
```

## Configuration
- User-level: `~/.spark_cli/config.toml`
- Project-level: `./config.toml` (use `config init --scope project`)
- Explicit file: `--config <path>`

Example `config.example.toml`:
```toml
provider = "openrouter"
api_key = ""
model = "openrouter/auto"
```
Copy it to your desired location and fill in the API key.

## Development
- Format and lint with rustfmt/clippy
- Logs use `tracing`; enable with env `RUST_LOG=info`
- Streaming uses Reqwest `stream` feature

## Roadmap
- More providers (OpenAI, Anthropic, Gemini, Ollama)
- Plugin system and hooks
- Request cache and connection pooling improvements
- Tests (unit/integration) and CI

## Security
- Do not commit real API keys.
- Prefer environment variables for secrets in CI/CD.

## License
MIT
