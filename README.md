# Jobex

A university prototype for short-term work matching. The existing demo is preserved in `Jobex-prototype.html`, with HTML, CSS, JavaScript, and SVG illustrations in one file.

## Run

Open `Jobex-prototype.html` directly in a current browser. On macOS:

```sh
open Jobex-prototype.html
```

No dependency installation, package manager, build step, account, environment variable, database, or external API is required. There is no package manifest or lockfile. No Node.js or Python runtime is needed to use the app.

For a consistent local HTTP origin, optionally use Python 3 from the repository root:

```sh
python3 -m http.server 8000 --bind 127.0.0.1
```

Then visit http://127.0.0.1:8000/Jobex-prototype.html. Browser storage is separate for each origin; changing how the app is opened may start a separate demo session.

## Current features

- Worker shift discovery, saved shifts, preferences, and availability filters.
- Local text parsing that creates a reviewable shift draft.
- Candidate scoring based on skills, availability, commute, and preferred rate.
- Employer acceptance, local chat, agreement previews, and simulated signing.
- Simulated completion, mutual ratings, and metrics derived from demo actions.

All profiles, shifts, verification badges, commute values, and agreements are illustrative. AI uses local parsing and templates. Chat is simulated by switching roles in the same browser. No real AI service, identity verification, legal signing, payments, or shared backend is connected.

State is stored in the browser's `localStorage` under `jobex-v1`. It is not shared with teammates or written to repository files. Use **Reset demo** to restore the sample state. Avoid entering personal or sensitive information.

## Try the main flow

1. In **I'm looking for work**, express interest in the barista shift.
2. Switch to **I'm hiring**, open the candidates view, and accept the worker.
3. In chat, prepare and send the demo agreement.
4. Switch to the worker, review the agreement, and simulate signing.
5. Simulate completing the shift, then leave feedback from both roles.
6. In the employer workspace, generate a shift draft, review its fields, and publish it locally.

## Checks and limitations

Import verification on 5 November 2026 used headless Google Chrome. The main flow above, shift creation, reload persistence, and a 390 px horizontal overflow check passed with no browser errors or external HTTP requests. JavaScript syntax also passed Node.js `--check`. These are import verification results, not a committed automated test suite.

The original demo has no build, lint, or test commands. No dependency upgrades or application rewrites were made. Sample dates are fixed to 6–8 November 2026; the parser recognizes a limited set of roles, phrases, and dates. Future work should agree on a small task before changing this baseline.

## Team workflow

Read the shared `AGENTS.md` and, for Claude, `CLAUDE.md` before work. Follow the handoff's freshness checks at the start or resumption of work and before pushing. Fetch before editing, preserve uncommitted changes, and use separate worktrees for concurrent agents. Never commit secrets or browser demo data. The initial app import requires review and approval before publication.
