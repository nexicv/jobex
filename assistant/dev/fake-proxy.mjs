#!/usr/bin/env node
// LARPd assistant fake proxy. Created by (DeepSeek v4.1 Flash Max 1017), 2026-11-06.
// Speaks the same SSE contract as assistant/proxy/README.md so the page can be tried without
// the real proxy or a DeepSeek key. No dependencies, Node 20+.
//
//   node assistant/dev/fake-proxy.mjs [port] [--closed]
//
// Scenarios are chosen by a keyword in the last user message:
//   find -> status steps + search_shifts tool call, then an answer quoting the count
//   interest -> express_interest call with a real shift id from the earlier search (or 1)
//   weather -> refused
//   closed  -> closed event with reopens_at 90 minutes ahead
//   busy    -> busy
//   error   -> error
//   anything else -> a short streamed answer in chunks

import { createServer } from 'node:http';

const argv = process.argv.slice(2);
const closedMode = argv.includes('--closed');
const portArg = argv.find((a) => /^[0-9]+$/.test(a));
const PORT = Number(portArg || process.env.PORT || 8788);
const TEAM_PASS = 'test-pass';
const REOPEN_MS = 90 * 60 * 1000;
const ALLOWED_ORIGINS = new Set(['http://127.0.0.1:8765', 'http://127.0.0.1:8000']);

function log(line) {
  console.log(new Date().toISOString() + ' ' + line);
}

function setCors(req, res) {
  const origin = req.headers.origin;
  if (origin && ALLOWED_ORIGINS.has(origin)) res.setHeader('Access-Control-Allow-Origin', origin);
  res.setHeader('Vary', 'Origin');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type, x-jobex-team');
  res.setHeader('Access-Control-Max-Age', '3600');
}

function readJson(req) {
  return new Promise((resolve) => {
    let raw = '';
    req.on('data', (chunk) => {
      raw += chunk;
      if (raw.length > 262144) req.destroy();
    });
    req.on('end', () => {
      try { resolve(JSON.parse(raw)); } catch { resolve(null); }
    });
    req.on('error', () => resolve(null));
  });
}

function bad(res, message) {
  const body = JSON.stringify({ error: message });
  res.writeHead(400, { 'Content-Type': 'application/json; charset=utf-8' });
  res.end(body);
}

function startStream(res) {
  res.writeHead(200, {
    'Content-Type': 'text/event-stream; charset=utf-8',
    'Cache-Control': 'no-cache, no-transform',
    Connection: 'keep-alive',
    'X-Accel-Buffering': 'no',
  });
  res.write(': fake stream\n\n');
}

function send(res, event, data) {
  res.write('event: ' + event + '\ndata: ' + JSON.stringify(data) + '\n\n');
}

function endStream(res) {
  res.end();
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function teamFrom(req) {
  const raw = req.headers['x-jobex-team'];
  if (raw === undefined) return null;
  return String(raw).trim() === TEAM_PASS;
}

function statusPayload(req) {
  const team = teamFrom(req);
  const open = !closedMode || team === true;
  return {
    open,
    quiet_hours: closedMode,
    reopens_at: closedMode ? new Date(Date.now() + REOPEN_MS).toISOString() : null,
    now: new Date().toISOString(),
    team,
    team_configured: true,
  };
}

function sealed() {
  return 'fake-sealed-' + Math.random().toString(36).slice(2) + Math.random().toString(36).slice(2);
}

function assistantText(content) {
  return { role: 'assistant', content, sealed_reasoning: sealed() };
}

function assistantCalls(calls) {
  return {
    role: 'assistant',
    content: '',
    tool_calls: calls.map((c) => ({ id: c.id, type: 'function', function: { name: c.name, arguments: c.arguments } })),
    sealed_reasoning: sealed(),
  };
}

function lastUserText(messages) {
  for (let i = messages.length - 1; i >= 0; i -= 1) {
    const m = messages[i];
    if (m && m.role === 'user' && typeof m.content === 'string') return m.content;
  }
  return '';
}

function scenarioFor(text) {
  const t = String(text).toLowerCase();
  if (t.includes('closed')) return 'closed';
  if (t.includes('busy')) return 'busy';
  if (t.includes('error')) return 'error';
  if (t.includes('weather')) return 'weather';
  if (t.includes('interest')) return 'interest';
  if (t.includes('find')) return 'find';
  return 'text';
}

function earlierTool(messages, needle) {
  for (let i = messages.length - 1; i >= 0; i -= 1) {
    const m = messages[i];
    if (m && m.role === 'tool' && typeof m.content === 'string' && m.content.includes(needle)) return m.content;
  }
  return null;
}

async function sendChunks(res, text) {
  const size = 24;
  for (let i = 0; i < text.length; i += size) {
    send(res, 'content', { text: text.slice(i, i + size) });
    await sleep(25);
  }
}

async function handleFind(res, messages, continuation) {
  if (!continuation) {
    send(res, 'status', { label: 'Working out what you need' });
    await sleep(40);
    send(res, 'status', { label: 'Finding suitable shifts', tool: 'search_shifts' });
    await sleep(40);
    const args = JSON.stringify({ limit: 8, sort: 'match' });
    const calls = [{ id: 'call_search_1', name: 'search_shifts', arguments: args, label: 'Finding suitable shifts' }];
    send(res, 'tool_calls', { calls, message: assistantCalls(calls) });
    endStream(res);
    return;
  }
  let count = 0;
  let best = 'the first one';
  const raw = earlierTool(messages, '"shifts"');
  if (raw) {
    try {
      const data = JSON.parse(raw);
      count = Array.isArray(data.shifts) ? data.shifts.length : 0;
      if (data.shifts && data.shifts[0] && data.shifts[0].title) best = data.shifts[0].title;
      if (typeof data.error === 'string') {
        send(res, 'status', { label: 'Writing the answer' });
        await sendChunks(res, 'The search failed: ' + data.error);
        send(res, 'done', { message: assistantText('The search failed: ' + data.error) });
        endStream(res);
        return;
      }
    } catch {
      count = 0;
    }
  }
  const answer = count
    ? 'I found ' + count + ' shifts that fit your week. The best match is ' + best + '.'
    : 'No shifts came back for that search. Try widening your availability or rate.';
  send(res, 'status', { label: 'Writing the answer' });
  await sleep(30);
  await sendChunks(res, answer);
  send(res, 'done', { message: assistantText(answer) });
  endStream(res);
}

async function handleInterest(res, messages, continuation) {
  if (!continuation) {
    let shiftId = 1;
    const raw = earlierTool(messages, '"shifts"');
    if (raw) {
      try {
        const data = JSON.parse(raw);
        if (Array.isArray(data.shifts) && data.shifts[0] && typeof data.shifts[0].id === 'number') shiftId = data.shifts[0].id;
      } catch {
        shiftId = 1;
      }
    }
    send(res, 'status', { label: 'Working out what you need' });
    await sleep(40);
    send(res, 'status', { label: 'Sending your interest', tool: 'express_interest' });
    await sleep(40);
    const args = JSON.stringify({ shift_id: shiftId });
    const calls = [{ id: 'call_interest_1', name: 'express_interest', arguments: args, label: 'Sending your interest' }];
    send(res, 'tool_calls', { calls, message: assistantCalls(calls) });
    endStream(res);
    return;
  }
  let answer = 'Interest sent. The company usually replies in a few seconds.';
  const raw = earlierTool(messages, '"shift_id"');
  if (raw) {
    try {
      const data = JSON.parse(raw);
      if (data.cancelled) answer = 'No problem, I did not send it.';
      else if (typeof data.error === 'string') answer = 'That did not go through: ' + data.error;
      else if (data.title) answer = 'Interest sent for ' + data.title + '. The company usually replies in a few seconds.';
    } catch {
      answer = 'Interest sent.';
    }
  }
  send(res, 'status', { label: 'Writing the answer' });
  await sleep(30);
  await sendChunks(res, answer);
  send(res, 'done', { message: assistantText(answer) });
  endStream(res);
}

async function handleChat(req, res) {
  const body = await readJson(req);
  if (!body || !Array.isArray(body.messages) || body.messages.length === 0) {
    bad(res, 'No messages.');
    return;
  }
  for (const m of body.messages) {
    const role = m && m.role;
    if (role !== 'user' && role !== 'assistant' && role !== 'tool') {
      bad(res, 'Unknown message role.');
      return;
    }
    if (role === 'assistant' && (typeof m.sealed_reasoning !== 'string' || m.sealed_reasoning.length === 0)) {
      bad(res, 'An assistant message lost its sealed_reasoning. Send assistant messages back exactly as received.');
      return;
    }
  }
  const last = body.messages[body.messages.length - 1];
  if (last.role === 'system' || last.role === 'developer') {
    bad(res, 'The last message must be a question or a tool result.');
    return;
  }

  const team = teamFrom(req);
  if (closedMode && team !== true) {
    const at = new Date(Date.now() + REOPEN_MS).toISOString();
    startStream(res);
    send(res, 'closed', {
      text: 'The assistant is resting. Try again at ' + new Date(at).toUTCString() + '.',
      reopens_at: at,
      team_rejected: team === false,
    });
    endStream(res);
    return;
  }

  const text = lastUserText(body.messages);
  const scenario = scenarioFor(text);
  const locale = typeof body.locale === 'string' ? body.locale : 'en';
  const continuation = last.role === 'tool';
  log('chat scenario=' + scenario + (continuation ? ' (continuation)' : '') + ' locale=' + locale + ' messages=' + body.messages.length);

  if (scenario === 'closed') {
    const at = new Date(Date.now() + REOPEN_MS).toISOString();
    startStream(res);
    send(res, 'closed', { text: 'The assistant is resting right now.', reopens_at: at, team_rejected: false });
    endStream(res);
    return;
  }
  if (scenario === 'busy') {
    startStream(res);
    send(res, 'busy', { text: 'The assistant is busy right now. Try again in 3 min.', retry_after_s: 180 });
    endStream(res);
    return;
  }
  if (scenario === 'error') {
    startStream(res);
    send(res, 'error', { text: 'The assistant is offline right now. Everything else in the demo still works.' });
    endStream(res);
    return;
  }
  if (scenario === 'weather') {
    startStream(res);
    const refusedText = locale === 'pl'
      ? 'Mogę pomóc tylko w sprawach LARPd. Chcesz, żebym znalazł zmiany pasujące do Twojego tygodnia?'
      : 'I can only help with LARPd. Want me to find shifts that fit your week?';
    send(res, 'refused', { text: refusedText, message: assistantText(refusedText) });
    endStream(res);
    return;
  }
  if (scenario === 'find') {
    startStream(res);
    await handleFind(res, body.messages, continuation);
    return;
  }
  if (scenario === 'interest') {
    startStream(res);
    await handleInterest(res, body.messages, continuation);
    return;
  }

  const answer = locale === 'pl'
    ? 'To jest fałszywy serwer testowy. Umie szukać zmian, wysyłać zainteresowanie i czytać Twój plan. Napisz "find", żeby sprawdzić wyszukiwanie.'
    : 'This is the **fake** test proxy. I can search shifts, send interest and read your plan. Ask me to find shifts to try the tools.';
  startStream(res);
  send(res, 'status', { label: locale === 'pl' ? 'Zastanawiam się, czego potrzebujesz' : 'Working out what you need' });
  await sleep(40);
  send(res, 'status', { label: locale === 'pl' ? 'Piszę odpowiedź' : 'Writing the answer' });
  await sleep(30);
  await sendChunks(res, answer);
  send(res, 'done', { message: assistantText(answer) });
  endStream(res);
}

const server = createServer((req, res) => {
  setCors(req, res);
  const url = new URL(req.url, 'http://127.0.0.1');
  if (req.method === 'OPTIONS') {
    res.writeHead(204);
    res.end();
    return;
  }
  if (req.method === 'GET' && url.pathname === '/healthz') {
    res.writeHead(200, { 'Content-Type': 'text/plain; charset=utf-8' });
    res.end('ok');
    return;
  }
  if (req.method === 'GET' && url.pathname === '/v1/status') {
    res.writeHead(200, { 'Content-Type': 'application/json; charset=utf-8', 'Cache-Control': 'no-store' });
    res.end(JSON.stringify(statusPayload(req)));
    return;
  }
  if (req.method === 'POST' && url.pathname === '/v1/chat') {
    handleChat(req, res).catch((err) => {
      log('chat failed: ' + (err && err.message ? err.message : String(err)));
      if (!res.headersSent) bad(res, 'The fake proxy failed.');
      else endStream(res);
    });
    return;
  }
  res.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' });
  res.end('not found');
});

server.listen(PORT, '127.0.0.1', () => {
  log('fake proxy on http://127.0.0.1:' + PORT + (closedMode ? ' (quiet hours)' : '') + '; team passphrase "' + TEAM_PASS + '"');
});
