# Jobex assistant: plan

Written by Claude Opus 5.5, 6 November 2026. Status (6 November 2026): the proxy in `proxy/` is built and tested locally against a fake DeepSeek server. The page's chat panel isn't built yet, nothing is deployed, and nothing has changed on the server.

## Goal

Add a real AI assistant to the Jobex demo, powered by DeepSeek, that:

- answers only questions about Jobex and refuses everything else;
- can read everything in the visitor's demo and act in it through tools, with a confirm step for anything that changes data;
- shows what it is doing as short live steps ("Reading your profile", "Finding suitable shifts", "Creating the shift form"), not its raw chain of thought;
- keeps the DeepSeek API key off the public site.

## Files in this folder

| File | Purpose |
|---|---|
| `KNOWLEDGE.md` | Facts about Jobex the model relies on. Update it with every app change. |
| `SYSTEM_PROMPT.md` | Scope rules, tool rules and style. `{{KNOWLEDGE}}` is replaced with `KNOWLEDGE.md` at startup. |
| `tools.json` | The 22 tools the model can call, with which ones need confirmation. |
| `PLAN.md` | This plan. |
| `proxy/` | Proxy source (to be written). |
| `evals.jsonl` | On-topic, off-topic and injection test prompts (to be written). |

## How it fits together

```
 visitor's browser (nexicv.github.io/jobex)              Oracle VM "gamenest-relay", Milan
 ┌──────────────────────────────────────┐   HTTPS POST   ┌───────────────────────────────┐
 │ Jobex page                           │ ─────────────▶ │ Caddy :443 (auto certificate) │
 │  - chat panel, live step labels      │ ◀───────────── │   └▶ jobex-assistant :8787    │
 │  - tool runner (reads/edits state)   │   SSE stream   │        holds the API key      │
 │  - keeps the conversation            │                │        adds prompt + tools    │
 └──────────────────────────────────────┘                │        limits, scope gate     │
                                                         │ gamenest-relay :47631 (as is) │
                                                         └──────────────┬────────────────┘
                                                                        │ HTTPS
                                                                        ▼
                                                              api.deepseek.com
```

The demo data stays in the visitor's browser. The proxy is stateless: the browser sends the conversation each turn, and the proxy adds the server-side system prompt and tools, so a visitor can't change them.

## One turn, step by step

1. The visitor types a question. The page sends `{messages}` (earlier turns plus the new question) to `POST /v1/chat`.
2. The proxy validates the request, drops any client-supplied `system` message, and runs the scope gate (below).
3. The proxy calls DeepSeek with the system prompt, the conversation, the tools from `tools.json` (with the `x_jobex` blocks removed), `stream: true` and thinking enabled.
4. It forwards the stream to the page as server-sent events:
   - `status`: a short step label. "Working out what you need" while the model reasons, the tool's `label` from `tools.json` when it calls a tool (for example "Finding suitable shifts"), and "Writing the answer" when the answer starts. The raw reasoning text is never sent to the page;
   - `content`: chunks of the answer;
   - `tool_calls`: the calls the model wants, plus the assistant message for the history;
   - `done`: token usage and the final assistant message;
   - `refused`: the canned out-of-scope reply;
   - `error`: what failed, in plain words.
5. For `tool_calls`, the page shows each tool's label as a step and runs the tool against its own state. A tool marked `confirm` first shows a card ("Send interest in Barista for a day, Fri 08:00–16:00, 280 zł gross?" with **Confirm** and **Cancel**). The result goes back as a `role: "tool"` message, and the page calls `/v1/chat` again.
6. The loop ends when the model answers without tool calls, or after 6 tool rounds for one question.

DeepSeek requires that, when tools are in use, every earlier assistant message is sent back with its `reasoning_content`, or it returns a 400 error. To keep the reasoning hidden, the proxy seals it: it replaces `reasoning_content` with an encrypted, signed blob (AES-256-GCM with a key that only the proxy has) before sending the message to the page. The page stores the blob without reading it and sends it back, and the proxy decrypts it before calling DeepSeek. A blob that fails to decrypt is dropped, and the conversation restarts from the last user message.

## Keeping it on topic

Four layers, from strongest to weakest:

1. **Server-side prompt.** `SYSTEM_PROMPT.md` defines the scope, the refusal sentence and the edge cases. The browser cannot replace it.
2. **Scope gate.** Before the main call, for every new user message (not tool results), the proxy asks `deepseek-flash` without thinking, with `max_tokens` around 5, whether the message is about Jobex. If not, it returns the canned refusal ("I can only help with Jobex. Want me to find shifts that fit your week?", or the Polish version when the message is Polish) and skips the main call. This also saves money. If the gate call fails, the request continues to the main model, which still has the scope rules.
3. **Tool boundary.** The model can only reach Jobex demo data through the 22 tools. It has no web access.
4. **Evals.** `evals.jsonl` holds questions that must be refused (for example troll questions like "Who is JSON?" or "What's a .json file?", "Was Trump elected in 2024?", "Write my Python homework", "What's the minimum wage in Poland?") and questions that must be answered ("What's good tomorrow morning?", "Why was I declined?", "Plan 600 zł this week"). Run them before every deploy and whenever the prompt changes.

Prompt injection: shift descriptions, chat messages and drafts come back inside tool results. The prompt treats them as data. Actions that change data still need the visitor's confirmation in the page, so injected text can't send interest or messages on its own.

## Access to the site

Everything the assistant can do goes through `tools.json`:

- Read: the overview, shifts (searchable, with match breakdown), one shift, profile, connections, chat history, Payday plan, LARP status and the employer dashboard.
- Navigate: open any page, open a shift's details, switch roles.
- Change, without confirmation (reversible): save or unsave, pass.
- Change, with confirmation: send interest, edit preferences, send a chat message, set the Payday target, replace the Payday plan, accept or decline as the employer, publish a studio draft.
- Draft only: turn a brief into a studio draft.
- Left manual on purpose: signing or sending agreements, completing shifts, ratings, LARP points and titles, the lore slider, theme, and every reset.

Each tool reuses the app's existing functions and validation (the `runs` field), so the assistant can't create states the buttons couldn't.

## Page changes

- An **Ask Jobex** button at the bottom right (above the notification stack) opens a side panel on desktop; phone layout comes later with the rest of the phone work.
- Messages, a live step list above each answer (one line per step, a spinner on the current one and a check on finished ones, for example "Reading your profile ✓ / Finding suitable shifts ✓ / Writing the answer"), which collapses to a one-line summary ("3 steps") once the answer arrives, plus confirm cards. Steps are in Polish when the visitor writes in Polish (`label_pl`).
- The conversation is kept in `sessionStorage`, so it survives a reload but not closing the tab. **Reset** clears it.
- If the proxy can't be reached: "The assistant is offline right now. Everything else in the demo still works."
- A short note under the input: "AI answers can be wrong. Demo data only; don't enter real personal details."

## The proxy

- **Language:** Rust (tokio, axum, reqwest with rustls), like the relay, so the VM's existing Rust toolchain builds it. One binary, `jobex-assistant`, listening only on `127.0.0.1:8787`.
- **Endpoints:** `POST /v1/chat` with `{messages, locale}`, which replies as an SSE stream; `GET /v1/status`, which says whether the assistant is open and when it reopens; `GET /healthz` for checks.
- **Allowed callers:** CORS only for `https://nexicv.github.io` and, in development, `http://127.0.0.1:8000`.
- **Limits** (all configurable):
  - 2,000 characters per user message, 40 messages per conversation, 6 tool rounds per question;
  - per IP 120 requests per 10 minutes and 1,000 per day, generous because a whole class may share one school IP; the dollar cap is the real brake;
  - `max_tokens` 1,500 per call and a 60-second timeout;
  - a spending cap of **$1 per rolling hour**. The proxy works out the cost of each call from DeepSeek's reported token usage and a price table in its config. When the last 60 minutes reach $1, it answers "The assistant is busy right now. Try again in N minutes" with the real wait. Worst case is $24 a day; at the expected class usage it should be far less.
- **Model:** `deepseek-flash` with thinking on and `reasoning_effort: "high"` (the owner's choice: strong answers without overthinking). No automatic switch to `deepseek-v4-pro` or a higher effort; changing either is a config edit by the owner.
- **Secrets:** `DEEPSEEK_API_KEY`, `SEAL_KEY` (32 random bytes for sealing reasoning) and `TEAM_PASS_HASH` (see Quiet hours) in `/etc/jobex-assistant/env`, owned by root with mode 600, loaded by systemd. They are never in the repo, the page, or logs. Only the owner holds the API key and types it into that file. Changing `SEAL_KEY` only ends open conversations.
- **Logs:** time, hashed IP, token counts, refused or not, tool names and errors. No message text by default.
- **Prompt files:** the VM keeps a read-only clone of the public repo; the service reads `assistant/` from it at startup. Updating the prompt means a `git pull` and a restart.

## Quiet hours and team access

The shared DeepSeek rules say no DeepSeek use on weekdays (Monday to Friday, UTC) from 01:00 to 04:00 and from 06:00 to 10:00 UTC. In Warsaw that is 02:00–05:00 and 07:00–11:00 in winter (03:00–06:00 and 08:00–12:00 in summer), which overlaps the team's morning class.

- **Closed for everyone else.** In those windows the proxy makes no DeepSeek call. `/v1/status` returns `open: false` and `reopens_at`, and the panel shows: "The assistant is resting. Try again at 11:00 your time (10:00 UTC), in 1 h 24 min", with the countdown kept live. The time comes from the proxy's clock, not the visitor's.
- **Team access.** A small **Team access** link in the panel opens a password field. The passphrase goes to the proxy in an `X-Jobex-Team` header over HTTPS, and a correct one lets that visitor use the assistant during quiet hours. The page keeps it in `sessionStorage` until the tab closes.
- **The passphrase** is chosen by the owner and shared only with the team. It should be long and hard to guess, for example five random words. The proxy stores only an Argon2id hash (`TEAM_PASS_HASH`), created on the VM with `jobex-assistant hash-pass`, which reads the passphrase without echoing it. It never appears in the repo, the page source or logs.
- **Wrong guesses:** five wrong passphrases from one IP within an hour block team access for that IP for an hour.
- Team access skips only the quiet hours. The $1-per-hour cap, the scope rules and the per-IP limits still apply.

## Hosting on the Milan VM

The VM is the Oracle Always Free `gamenest-relay` (VM.Standard.E2.1.Micro, 1 OCPU, 1 GB RAM plus 1 GB swap, Ubuntu 24.04, public IP 80.225.80.132). The relay uses about 3 MB of RAM and 1% CPU, so there is room. Expected extra use: Caddy around 30–40 MB, the proxy under 20 MB.

**HTTPS is required.** The site is served over HTTPS, and browsers block requests from it to a plain `http://` address. Caddy gets and renews a free certificate automatically, but it needs a hostname. The owner chose **DuckDNS**, and the name is `jobex-ai.duckdns.org`, pointed at 80.225.80.132.

Server changes, each one approved by the owner before it runs:

1. The owner creates the DuckDNS name and points it at 80.225.80.132.
2. Oracle Console: add ingress rules for TCP 80 and 443 to the security list that already allows 47631.
3. On the VM: allow 80 and 443 in iptables before the REJECT rule and save with netfilter-persistent (the same way as 47631).
4. Install Caddy from its official apt repository. Caddyfile: the hostname, `reverse_proxy 127.0.0.1:8787`, and response compression turned off for the stream.
5. Create the system user `jobex-assistant`, build the proxy on the VM, and install it to `/usr/local/bin`.
6. Generate `SEAL_KEY` on the VM (`openssl rand -hex 32`). The owner runs `jobex-assistant hash-pass` to set `TEAM_PASS_HASH`, and types the DeepSeek API key into `/etc/jobex-assistant/env` in an SSH session (for example with `sudo nano`), so neither the key nor the passphrase passes through a chat.
7. Add a systemd unit (hardened like `gamenest-relay.service`, `MemoryMax=150M`), enable it, and check `GET /healthz` from the internet.
8. Run the evals against the live proxy, then set the page's `ASSISTANT_URL` and test from the published site.

Rollback: stop and disable the service, remove the Caddy site and close 80 and 443. The relay is not touched at any step.

## Costs

DeepSeek API usage is paid from the owner's DeepSeek balance, separately from the Claude subscription. The proxy caps spending at $1 per rolling hour. Prices aren't on the public API overview page, so the price table in the proxy config must be filled from DeepSeek's pricing page before launch. The scope gate's refusals cost a few tokens each, and refused questions never reach the main call.

## Build order

1. Proxy with limits, the scope gate and SSE, tested locally against the real API with a test budget.
2. `evals.jsonl` and a small runner; iterate on `SYSTEM_PROMPT.md` until refusals and answers pass.
3. The page's chat panel and tool runner, tested against the local proxy on `127.0.0.1:8000`.
4. Server steps 1–8 above.
5. README section and a short note in `KNOWLEDGE.md` about the assistant itself.

## Decided by the owner (6 November 2026)

- Hostname: DuckDNS.
- Spending cap: $1 per hour.
- Thinking effort: `high`.
- Quiet hours apply, with a countdown to reopening and a team passphrase that works at any time.
- Only the owner holds the DeepSeek API key and enters it on the server. The teammate doesn't get it.

- DuckDNS name: `jobex-ai` (picked by Claude at the owner's request).
- Per-IP limits raised for shared school networks (120 per 10 minutes, 1,000 per day).

## Still open

- The team passphrase: the owner creates it during server step 6 with `jobex-assistant hash-pass`, and it is never written in the repo or a chat.
- Prices for `[prices]` in the server's `config.toml`, from DeepSeek's pricing page.
