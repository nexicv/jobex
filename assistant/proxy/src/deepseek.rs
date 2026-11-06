//! Minimal DeepSeek chat-completions client: one streaming call with tool-call assembly,
//! and one short non-streaming call for the scope gate.

use anyhow::{Result, anyhow, bail};
use futures_util::StreamExt;
use serde_json::{Value, json};
use std::time::Duration;

use crate::config::Prices;

#[derive(Debug, Default, Clone)]
pub struct Usage {
    pub cache_hit: u64,
    pub cache_miss: u64,
    pub completion: u64,
}

impl Usage {
    fn from_json(v: &Value) -> Self {
        let n = |k: &str| v.get(k).and_then(Value::as_u64).unwrap_or(0);
        let prompt = n("prompt_tokens");
        let hit = n("prompt_cache_hit_tokens");
        let miss = v.get("prompt_cache_miss_tokens").and_then(Value::as_u64).unwrap_or(prompt.saturating_sub(hit));
        Self { cache_hit: hit, cache_miss: miss, completion: n("completion_tokens") }
    }

    pub fn usd(&self, p: &Prices) -> f64 {
        (self.cache_hit as f64 * p.input_cache_hit_per_m + self.cache_miss as f64 * p.input_cache_miss_per_m + self.completion as f64 * p.output_per_m)
            / 1_000_000.0
    }

    /// Without a usage report, assume the worst for the cap: the whole budget of output tokens.
    pub fn pessimistic(max_tokens: u32) -> Self {
        Self { cache_hit: 0, cache_miss: 8_000, completion: max_tokens as u64 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Default)]
pub struct Final {
    pub content: String,
    pub reasoning: String,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Option<Usage>,
}

pub enum Delta<'a> {
    Reasoning(&'a str),
    Content(&'a str),
    ToolStarted(&'a str),
}

/// What went wrong upstream, phrased for a visitor.
#[derive(Debug)]
pub struct UpstreamError {
    pub status: Option<u16>,
    pub detail: String,
}

impl std::fmt::Display for UpstreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DeepSeek error {:?}: {}", self.status, self.detail)
    }
}
impl std::error::Error for UpstreamError {}

pub struct Client {
    http: reqwest::Client,
    url: String,
    key: String,
}

impl Client {
    pub fn new(base_url: &str, key: &str, timeout_secs: u64) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .connect_timeout(Duration::from_secs(10))
            .user_agent("jobex-assistant/0.1")
            .build()?;
        Ok(Self { http, url: format!("{}/chat/completions", base_url.trim_end_matches('/')), key: key.to_string() })
    }

    async fn post(&self, body: &Value) -> Result<reqwest::Response> {
        let resp =
            self.http.post(&self.url).bearer_auth(&self.key).json(body).send().await.map_err(|e| UpstreamError { status: None, detail: e.to_string() })?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(UpstreamError { status: Some(status), detail: text.chars().take(400).collect() }.into());
        }
        Ok(resp)
    }

    /// Short classification call. Returns the reply text and usage.
    pub async fn complete(&self, body: &Value) -> Result<(String, Usage)> {
        let v: Value = self.post(body).await?.json().await?;
        let text = v.pointer("/choices/0/message/content").and_then(Value::as_str).unwrap_or("").to_string();
        Ok((text, v.get("usage").map(Usage::from_json).unwrap_or_default()))
    }

    /// Streaming call. `on` sees reasoning and content as they arrive and each tool call when it starts.
    pub async fn stream(&self, body: &Value, mut on: impl FnMut(Delta<'_>)) -> Result<Final> {
        let resp = self.post(body).await?;
        let mut bytes = resp.bytes_stream();
        let mut buf: Vec<u8> = Vec::new();
        let mut out = Final::default();
        let mut calls: Vec<ToolCall> = Vec::new();
        let mut done = false;
        while let Some(chunk) = bytes.next().await {
            let chunk = chunk.map_err(|e| UpstreamError { status: None, detail: e.to_string() })?;
            buf.extend_from_slice(&chunk);
            while let Some(pos) = buf.iter().position(|b| *b == b'\n') {
                let line: Vec<u8> = buf.drain(..=pos).collect();
                let line = String::from_utf8_lossy(&line);
                let line = line.trim();
                let Some(data) = line.strip_prefix("data:") else { continue };
                let data = data.trim();
                if data == "[DONE]" {
                    done = true;
                    break;
                }
                let v: Value = serde_json::from_str(data).map_err(|e| anyhow!("bad stream chunk: {e}"))?;
                if let Some(u) = v.get("usage").filter(|u| !u.is_null()) {
                    out.usage = Some(Usage::from_json(u));
                }
                let Some(delta) = v.pointer("/choices/0/delta") else { continue };
                if let Some(r) = delta.get("reasoning_content").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                    out.reasoning.push_str(r);
                    on(Delta::Reasoning(r));
                }
                if let Some(c) = delta.get("content").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                    out.content.push_str(c);
                    on(Delta::Content(c));
                }
                if let Some(tcs) = delta.get("tool_calls").and_then(Value::as_array) {
                    for tc in tcs {
                        let idx = tc.get("index").and_then(Value::as_u64).unwrap_or(calls.len() as u64) as usize;
                        while calls.len() <= idx {
                            calls.push(ToolCall { id: String::new(), name: String::new(), arguments: String::new() });
                        }
                        let slot = &mut calls[idx];
                        if let Some(id) = tc.get("id").and_then(Value::as_str) {
                            slot.id.push_str(id);
                        }
                        if let Some(name) = tc.pointer("/function/name").and_then(Value::as_str) {
                            let first = slot.name.is_empty();
                            slot.name.push_str(name);
                            if first && !slot.name.is_empty() {
                                on(Delta::ToolStarted(&slot.name.clone()));
                            }
                        }
                        if let Some(args) = tc.pointer("/function/arguments").and_then(Value::as_str) {
                            slot.arguments.push_str(args);
                        }
                    }
                }
            }
            if done {
                break;
            }
        }
        if !done && out.content.is_empty() && calls.is_empty() {
            bail!(UpstreamError { status: None, detail: "the stream ended early".into() });
        }
        out.tool_calls = calls.into_iter().filter(|c| !c.name.is_empty()).collect();
        Ok(out)
    }
}

/// The assistant message to keep in the conversation, in DeepSeek's format.
pub fn assistant_message(f: &Final, reasoning_field: Value) -> Value {
    let mut m = json!({ "role": "assistant", "content": f.content });
    if !f.tool_calls.is_empty() {
        m["tool_calls"] = Value::Array(
            f.tool_calls.iter().map(|c| json!({ "id": c.id, "type": "function", "function": { "name": c.name, "arguments": c.arguments } })).collect(),
        );
    }
    if !reasoning_field.is_null() {
        m["sealed_reasoning"] = reasoning_field;
    }
    m
}
