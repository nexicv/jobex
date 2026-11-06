# jobex-assistant

Created by Claude Opus 5.5, 6 November 2026. The proxy between the LARPd page (formerly Jobex) and the DeepSeek API, described in [../PLAN.md](../PLAN.md). It holds the API key, adds the LARPd system prompt, knowledge base and tools, refuses off-topic questions, keeps quiet hours (with a team passphrase that works at any time), and caps spending at $1 per rolling hour.

## Build and test

```sh
cargo test            # 17 tests; DeepSeek is faked, no API calls or key needed
cargo clippy --all-targets
cargo build --release # target/release/jobex-assistant
```

## Commands

- `jobex-assistant serve [config.toml]`: run the proxy.
- `jobex-assistant seal-key`: print a new `SEAL_KEY=` line.
- `jobex-assistant hash-pass`: ask for the team passphrase twice (hidden) and print a `TEAM_PASS_HASH=` line. Only this hash goes on the server.

## Configuration

Copy `config.example.toml` to `config.toml` and fill `[prices]` from DeepSeek's pricing page; the proxy won't start while a price is 0. Secrets come only from the environment:

| Variable | What it is |
|---|---|
| `DEEPSEEK_API_KEY` | The owner's DeepSeek key. |
| `SEAL_KEY` | 64 hex characters from `seal-key`. Encrypts the model's reasoning between requests. |
| `TEAM_PASS_HASH` | Optional, from `hash-pass`. Without it there is no team access during quiet hours. |

## What the page talks to

`GET /v1/status` returns `{open, quiet_hours, reopens_at, now, team, team_configured}`. Send the passphrase in an `x-jobex-team` header to check it.

`POST /v1/chat` takes `{ "locale": "en" | "pl", "messages": [...] }` and answers with server-sent events. Messages use DeepSeek's chat format; assistant messages must be sent back exactly as the proxy returned them (they carry `sealed_reasoning`). `system` messages from the page are ignored.

| Event | Data | Page does |
|---|---|---|
| `status` | `{label, tool?}` | Adds or advances a step line ("Finding suitable shifts"). |
| `content` | `{text}` | Appends answer text. |
| `tool_calls` | `{calls: [{id, name, arguments, label}], message}` | Stores `message`, runs each call (with a confirm card where `tools.json` says so), then posts again with one `{role: "tool", tool_call_id, content}` per call. |
| `done` | `{message}` | Stores `message`; the turn is over. |
| `refused` | `{text, message}` | Shows the one-line LARPd-only reply. |
| `closed` | `{text, reopens_at, team_rejected}` | Quiet hours: shows a live countdown to `reopens_at` in the visitor's time. |
| `busy` | `{text, retry_after_s}` | Spending cap or per-IP limit reached. |
| `error` | `{text}` | Shows the text; the rest of the demo keeps working. |

Bad input (empty, too long, unknown roles) gets HTTP 400 with `{error}`.

## Server files

`deploy/jobex-assistant.service` (systemd, hardened, 150 MB memory cap) and `deploy/Caddyfile` (HTTPS for `jobex-ai.duckdns.org`). The server steps and approvals are in [../PLAN.md](../PLAN.md#hosting-on-the-milan-vm).
