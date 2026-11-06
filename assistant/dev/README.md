# assistant/dev

Fake proxy for trying the Ask LARPd panel without the real server or a DeepSeek key. Created by (DeepSeek v4.1 Flash Max 1017), 2026-11-06.

1. Run `node assistant/dev/fake-proxy.mjs` (default port 8788; add a port or `--closed` to fake quiet hours).
2. Serve the repo root on the port the page expects for CORS: `python -m http.server 8765` (it also allows 127.0.0.1:8000).
3. Open `http://127.0.0.1:8765/Jobex-prototype.html?assistant=http://127.0.0.1:8788`, then ask "find shifts", "interest", "weather", "closed", "busy" or "error" and use Team access with "test-pass".
