# Jobex knowledge base for the assistant

Written by Claude Opus 5.5, 6 November 2026, from the code in `Jobex-prototype.html` on branch `feature/onboarding-deck`. The proxy sends this file to the model with every request. Keep it factual and update it whenever the app changes. When this file and the live data from a tool disagree, the tool result wins.

## What Jobex is

Jobex is a university prototype for short-term work. People who want a few hours of work find single shifts that fit their skills, days and commute. Businesses post shifts and pick workers. It is a demo: every company, shift, profile, badge, commute time and agreement is fictional. There are no real jobs, payments, identity checks or legal contracts.

The whole app is one web page. Each visitor's data lives only in their own browser (`localStorage`, key `jobex-v1`). Nothing is shared between visitors, and there is no account or server database. The assistant is the only part that talks to a server.

## The two roles

The header switch has two roles. Both are played by the same person in the same browser to try the full flow.

- **I'm looking for work** (worker): the demo worker is Alex Kowalski, a student in Warsaw.
- **I'm hiring** (employer): the demo employer is MOKO Coffee.

## Worker pages

- **Discover**: the shifts that fit you. On screens 1000 px and wider it is a full-screen deck with one large card and the next two stacked behind it. Drag the card right or press → for **I'm interested**, drag left or press ← to **Pass**, press ↑ for **Details**. **Undo pass** brings back the last shift you passed. The bookmark saves a shift. Filters: All shifts, Hospitality, Events, Retail, Saved. **Refine** holds search, sort (best match, highest pay, shortest commute), the lore slider and the availability switch. On narrower screens Discover is a scrolling page with the same actions.
- **Connections**: every shift you showed interest in, with its status and a button for the next step.
- **My shifts**: shifts you signed for and completed.
- **Payday Lab**: plan a week of shifts against an earnings target.
- **My profile**: name, skills, availability, minimum rate and longest commute, plus an Achievements tab with earned titles.
- **LARP**: the playful "LARP club" with points, unlocks, a dream goal and shareable story cards.

## Employer pages

- **Overview**: your posted shifts and their status.
- **AI shift studio**: type one sentence about the help you need and the studio fills in a shift draft (title, company, area, rate, date, times, skills, category). Review it, then publish it to the demo. The studio uses local text rules, not a real AI model, and recognizes a limited set of roles (café and barista, events and crew, retail and shop work), phrases and dates.
- **Candidates**: workers interested in your shifts. **Accept & open chat** or **Decline**. Also shows a ranked shortlist of three fictional profiles to illustrate scoring.
- **Insights**: demo metrics such as time to fill, interest-to-match rate and completion rate.

## How a shift moves

| Status | Shown as | What happens next |
|---|---|---|
| `applied` | Interest sent | The company reviews it. |
| `matched` | It's a match | Worker and company can chat. The employer can send the demo agreement. |
| `contract` | Contract ready | The worker reviews the agreement and simulates signing. |
| `signed` | Shift confirmed | After the shift, either side can mark it completed. |
| `completed` | Shift completed | Both sides leave a 1–5 rating. The worker earns 100 LARP points. |
| `declined` | Not this time | The company chose someone else. Similar shifts stay in Discover. |

If the worker stays in the worker view after showing interest, the company answers by itself after about 5–8 seconds: it accepts when the worker has at least one of the shift's required skills and declines otherwise. If the user switches to **I'm hiring** first, they decide as the employer instead. A notification appears at the bottom right when a company accepts or declines.

Signing is simulated: the agreement is a preview, nothing is legally binding, and no money moves.

## Matching

Each shift gets a demo match score out of 100:

- skills: up to 45 points, in proportion to how many required skills the worker has;
- availability: 25 points when the shift's day is one of the worker's days and its hours are inside the worker's hours;
- commute: 20 points when the commute is within the worker's longest commute;
- pay: 10 points when the hourly rate is at least the worker's minimum rate.

A shift appears in Discover only when it is **eligible**: it fits the worker's days and hours, commute and minimum rate, does not overlap a signed shift, the worker's availability switch is on, and its LARP tier is unlocked. Skills affect the score but not eligibility.

Pay is always **gross** in złoty (zł). Estimated gross pay for a shift is the hourly rate times its hours. Commute times are illustrative.

## Payday Lab

Set an earnings target (100–10,000 zł) and an hours budget (4–40 hours). Add shifts by hand or use **Suggest my shift plan**, which picks higher-paying, shorter-commute shifts until the target is met. A shift can't join the plan when its LARP tier is locked, it is already in Connections, it is outside your preferences, it is within 45 minutes of a signed shift or another planned shift, or it would exceed the hours budget. The plan is a draft only; it does not send interest.

## LARP club

- Points: +100 for each completed shift. Points are not money and give no discounts.
- Unlocks: some demo shifts are locked until 200, 500 or 1,000 points (for example "Gallery opening host").
- Dream goal: Rolex Daytona (150,000 zł demo target) or Porsche 911 (650,000 zł demo target). Shift cards show how many hours at that rate the goal would take. These are illustrative budgets, not real prices.
- Lore slider ("Reality ↔ delusion"): changes how shift titles and captions are phrased. Below 34 is "Reality check" (plain titles), 34–66 "A little spin", 67 and above "Full main character". The real role is always shown too.
- Titles: completing certain kinds of shifts earns joke profile titles, for example "Junior Vice President of Espresso" for a barista shift. One title can be shown on the profile.

## Intro, themes and reset

- First visit shows a four-step intro: welcome, confirm your details, confirm your availability, see how many shifts fit. **Skip intro** closes it. **I'm hiring instead** jumps to the employer view.
- Light theme is the default. The sun/moon button in the header switches to dark; the choice is remembered in this browser.
- **Reset** (header), **Reset demo** (page footer) or **Reset everything** (LARP admin panel) clears all demo data and shows the intro again.

## What the demo does not do

- No real employers, jobs, payments, payroll, taxes, insurance, identity verification or legal contracts.
- No messages leave the browser; chat is simulated by switching roles.
- No accounts, no shared data between visitors, no mobile app.
- The AI shift studio is rule-based. The assistant (this chat) is the only real AI feature.

## Glossary

- **Shift**: one block of paid work on one day.
- **Interest**: the worker's request for a shift (the heart, → key, or **I'm interested**).
- **Connection**: a shift the worker showed interest in, at any status.
- **Gross**: before tax and deductions.
- **Demo verified**: a fictional badge; no real check happened.
