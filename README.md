# LARPd

A university prototype for short-term work matching, now branded as LARPd. The existing demo is preserved in `Jobex-prototype.html`, with HTML, CSS, JavaScript, and SVG illustrations in one file.

## Run

Open `Jobex-prototype.html` directly in a current browser. On macOS:

```sh
open Jobex-prototype.html
```

No dependency installation, package manager, build step, account, environment variable, database, or external API is required. There is no package manifest or lockfile. No Node.js or Python runtime is needed to use the app. The page loads its fonts (Archivo, Geist Mono and Inter) from Google Fonts; offline, it falls back to system fonts.

For a consistent local HTTP origin, optionally use Python 3 from the repository root:

```sh
python3 -m http.server 8000 --bind 127.0.0.1
```

Then visit http://127.0.0.1:8000/Jobex-prototype.html. Browser storage is separate for each origin; changing how the app is opened may start a separate demo session.

## Current features

- Light (default) and dark themes with a blue accent and condensed display type, used across every screen. Switch with the sun/moon button in the header; the choice is remembered in this browser (key `jobex-theme`) and Reset does not change it.
- A four-step intro on first visit: welcome, confirm your details, confirm your availability, see how many shifts fit. The demo profile is prefilled, so each step only needs a confirm. **Skip intro** closes it.
- On screens 1000 px and wider, Discover is a full-screen deck: one large shift card with the next two stacked behind. Drag the card, use the buttons, or press ← (pass), → (I'm interested) and ↑ (details). Search, sort, the lore slider and the availability switch are under **Refine**. Narrower screens keep the earlier Discover layout for now.
- Notifications at the bottom right when a company accepts or declines your interest, with a button to open the chat or find similar shifts.
- Worker shift discovery, saved shifts, preferences, and availability filters.
- DeepSeek extraction for editable shift drafts, with manual entry and optional past-posting suggestions.
- An editable CV builder with Classic and Skills first print layouts.
- Candidate scoring based on skills, availability, commute, and preferred rate.
- Employer acceptance, local chat, agreement previews, and simulated signing.
- Simulated completion, mutual ratings, and metrics derived from demo actions.

All profiles, shifts, verification badges, commute values, and agreements are illustrative. Worker and employer chat is simulated by switching roles in the same browser. On [larpd.work](https://larpd.work), Ask LARPd sends assistant messages and submitted draft briefs or CV notes to DeepSeek via the LARPd API. Past postings used to fill shift blanks and names entered on the CV review form stay in the browser. Use sample data only.

State is stored in the browser's `localStorage` under `jobex-v1`. It is not shared with teammates or written to repository files. Use **Reset** in the header, **Reset demo** at the bottom of the page, or **Reset everything** in the LARP demo admin panel to clear all stored demo data and reload the sample state. Avoid entering personal or sensitive information.

Each reload opens a blank assistant chat. Chat history retains up to 12 conversations in the current tab so you can reopen one and continue. If you reload mid-reply, reopen the chat to continue from the last completed turn.

Open Ask LARPd from the Worker or Hiring navigation. Discover's Find shifts that fit button opens the same conversation. When a reply includes search results, it shows up to three links in one collapsed list.

## Try the main flow

1. In **I'm looking for work**, express interest in the barista shift.
2. Switch to **I'm hiring**, open the candidates view, and accept or decline the worker. If you stay in the worker view instead, the company answers by itself after about 5–8 seconds: it accepts when you share at least one required skill and declines otherwise.
3. In chat, prepare and send the demo agreement.
4. Switch to the worker, review the agreement, and simulate signing.
5. Simulate completing the shift, then leave feedback from both roles.
6. In Shift studio, extract a draft from a brief or enter it manually. Select and confirm a previous posting if you want it to fill blank fields, then review and publish the draft locally.

## Checks and limitations

Import verification on 5 November 2026 used headless Google Chrome. The main flow above, shift creation, reload persistence, and a 390 px horizontal overflow check passed with no browser errors or external HTTP requests. JavaScript syntax also passed Node.js `--check`. These are import verification results, not a committed automated test suite.

The intro, deck and notification changes were checked manually on 6 November 2026 in Chromium at 1440 × 900 over a local HTTP server: the full intro, dragging and arrow-key decisions, undo, Refine, automatic accept, employer decline and the delayed notification on switching back to the worker, with no console errors. Phone layouts of the new screens have not been designed yet.

The original demo has no build, lint, or test commands. No dependency upgrades or application rewrites were made. Sample shift dates are relative (today, tomorrow, and the day after), so the demo does not go stale; the parser still recognizes a limited set of roles, phrases, and dates. Future work should agree on a small task before changing this baseline.

## Hosting

GitHub Pages serves this demo from the `main` branch and the repository root (`/`). The site entry is `index.html`, which forwards to `Jobex-prototype.html`. Demo state is stored in each browser under `localStorage` (`jobex-v1`). Ask LARPd on the published site uses the proxy at `https://api.larpd.work`. For local assistant testing, use the [fake backend](assistant/dev/README.md).

## Team workflow

Read the shared `AGENTS.md` and, for Claude, `CLAUDE.md` before work. Follow its freshness checks at the start or resumption of work and before pushing. Fetch before editing, preserve uncommitted changes, and use separate worktrees for concurrent agents. Never commit secrets or browser demo data.
