# Jobex: setup handoff for a teammate and their Codex

Prepared by Codex 6.1 Sol Medium, 5 November 2026 (Europe/Warsaw).

## Start here

Give this file to your Codex with this message:

> Read this handoff and follow it to prepare my existing Jobex demo for shared development. Inspect the demo before changing anything. Preserve my files and identify the real framework, dependencies, and run commands. Set up my own GitHub access and a local clone of the team's repository. Read the shared AGENTS.md rules and make Claude follow them too. Check for new GitHub pushes every time work starts or resumes, and before pushing. Prepare and verify the initial import. Show me what will become public before the first commit and push, then wait for my approval to publish. Do not scaffold a replacement app or start unrelated features.

This file explains the initial setup and the later daily workflow. Commands are PowerShell examples. Replace placeholders before running them and stop when a required value is unknown.

## Agreed project details

| Item | Decision |
| --- | --- |
| Project | Jobex |
| Purpose | A university app for matching people with short-term work and businesses with available workers |
| Existing baseline | A teammate already has a working demo |
| GitHub owner | `nexicv`, display name Nexic |
| Repository name | `jobex` |
| Visibility | Public |
| Intended URL | `https://github.com/nexicv/jobex` |
| Owner's local folder | `D:\projects\jobex` |
| Friend's local folder | Choose a suitable location on the friend's computer |
| Framework and runtime | Inspect the existing demo before deciding |
| Shared integration branch | `main` |

Updated 5 November 2026: the public repository `https://github.com/nexicv/jobex` exists. These shared setup documents establish the initial `main` baseline. The owner's local checkout is `D:\projects\jobex`. A write-access invitation was sent to `olafcodemowicz`; acceptance must be verified. The demo import remains pending. Fetch and verify current state before acting.

Earlier project material used both Jobex and JobSwipe. Use Jobex in new documentation. Check with the team before renaming anything inside the working demo.

## 1. Inspect the existing demo first

Ask the friend for its folder if it is not already open. Work only in that project and its agreed clone.

Record:

- Where the working demo lives and whether it already uses Git
- Its current branch, remotes, and uncommitted files, if applicable
- Framework, runtime version, package manager, and lockfile
- Actual install, development, build, lint, and test commands, where available
- Any database, external API, account, environment variable, or local service it needs
- Which features work today and a short sequence that demonstrates them
- Which actions use mock data and which contact a real service

Read package manifests, lockfiles, README files, and example environment files. Do not print secrets from actual environment files. Do not run install scripts or other project commands before inspecting what they execute.

Do not delete, relocate, rewrite, or reinitialize the original demo. If it has history worth preserving, discuss a history-preserving import with the team before taking a snapshot. If it has an unrelated origin, do not replace that origin automatically.

## 2. Verify the friend's tools and identity

Git, GitHub, and GitHub CLI are different things. Git manages local history. GitHub hosts the shared repository. `gh` is the GitHub CLI. GitHub Desktop is optional.

Check the installed tools:

```powershell
git --version
gh --version
```

If either command is missing, install the official tool or finish the installation already in progress, then reopen the terminal. Do not install a second competing copy unnecessarily.

Sign in with the friend's own GitHub account:

```powershell
gh auth status
```

If needed, start browser authentication:

```powershell
gh auth login --hostname github.com --git-protocol https --web
gh auth setup-git
gh api user --jq .login
```

The human completes the browser sign-in. Never borrow Nexic's token or share credentials through chat or the repository. Never print `gh auth token` or collect authentication files in a handoff.

Check commit identity inside the clone when it exists:

```powershell
git config --get user.name
git config --get user.email
```

Use the friend's chosen name and verified or GitHub-provided noreply email. Prefer repository-local settings rather than changing global settings for unrelated projects:

```powershell
git config --local user.name "FRIEND_CHOSEN_NAME"
git config --local user.email "FRIEND_VERIFIED_OR_NOREPLY_EMAIL"
```

Git commit identity does not grant repository access. GitHub authentication does not automatically select the desired commit email.

## 3. Owner creates the shared repository and grants write access

The owner has created the public repository `nexicv/jobex` and is publishing the shared setup documents on `main`. Verify it exists and reuse its history when importing the demo.

The following command documents the creation step, which is already complete. Do not run it again:

```powershell
gh repo create nexicv/jobex --public --description "Jobex university project: short-term work matching"
```

The owner has invited `olafcodemowicz` with write access. The friend must accept the invitation while signed into that account. Public visibility allows reading, but does not give the friend write access by itself.

Verify the actual repository and effective access:

```powershell
gh repo view nexicv/jobex --json nameWithOwner,url,visibility,viewerPermission,defaultBranchRef
```

If it does not exist or access is insufficient, report the precise blocker and wait for the account owner. Do not create a second repository or fork as a silent workaround. The friend's Codex must not try to act as Nexic.

## 4. Prepare a separate clone for the import

Choose the friend's clone location. It does not need to be `D:\projects\jobex` and it must not overwrite the working demo.

Example after setting a real local parent folder:

```powershell
$jobexParent = 'C:\Projects'
New-Item -ItemType Directory -Force -Path $jobexParent | Out-Null
Set-Location -LiteralPath $jobexParent
gh repo clone nexicv/jobex jobex
Set-Location -LiteralPath (Join-Path $jobexParent 'jobex')
git remote -v
git status --short --branch
git fetch origin --prune
```

Before cloning, check whether the destination already exists. Reuse it only after checking its contents and remotes. Do not delete a folder because clone says it is not empty.

The clone should contain the setup documents on `main`. If it looks empty, fetch and check the repository's current state before continuing. Do not initialize an unrelated history.

If the repository already has commits by the time you get here, fetch and inspect them. Switch to its existing main branch and fast-forward it before importing anything. Coordinate with the owner rather than replacing existing history.

## 5. Import the demo without publishing private files

Inspect the original demo and copy only the files needed to reproduce it into the new clone. Keep the original demo untouched until the clone works.

Include source, required public assets, dependency manifests, lockfiles, configuration safe to publish, and useful existing project documentation. Exclude the original `.git` folder so unrelated history and remotes do not get copied by accident.

Create a stack-appropriate `.gitignore` before staging files. Keep out:

- Real `.env` files and local overrides, credentials, tokens, private keys, and service-account files
- Dependency and build folders such as `node_modules`, `.next`, `dist`, or virtual environments, where applicable
- Local databases, uploads, logs, caches, and test accounts containing real personal data
- Unrelated screenshots, classmate information, and machine-specific assistant configuration

Use a safe `.env.example` with placeholders for required variables. If the app has client code, inspect what gets bundled into it. A key in browser code or a variable designated as public is visible to users even if it came from an ignored file.

Check third-party assets and dependencies for permission to publish them. Flag uncertain items to the team. If a real secret already entered Git history, removing the current file is insufficient: report it and rotate the secret before publication.

Keep the shared `AGENTS.md`, `CLAUDE.md`, and this handoff from the cloned repository. Preserve useful existing demo instructions and reconcile conflicts rather than overwriting them wholesale. Create an `import/existing-demo` branch from the latest `origin/main` for the import.

Create or update the app README from observed facts:

- What works in the current demo
- Supported runtime and package-manager versions
- Exact install and run commands from the actual project
- Required environment variables and how to obtain personal development access
- Available checks and any known failures
- Whether data is local, shared, or mocked

Do not invent a framework, install command, database, or deployment service. Keep existing lockfiles and avoid unnecessary dependency upgrades during import.

## 6. Verify the imported copy

Install the existing dependencies with the project's package manager and lockfile. Run the documented app and repeat the original demo's main user flow. Check the build and existing tests or lint commands where they exist. Record missing checks honestly rather than adding tests merely to claim coverage.

Confirm that setup does not depend on an absolute path on the original machine. Confirm that ignored environment files have safe setup instructions. If the original demo runs but the clone does not, fix or document the difference before calling the import ready.

## 7. Review the public demo import

Before staging, inspect what is untracked and ignored:

```powershell
git status --short
git status --ignored --short
```

Stage only reviewed paths. Do not blindly use `git add .` on an unaudited demo folder. Then inspect exactly what the commit would contain:

```powershell
git diff --cached --stat
git diff --cached --check
git diff --cached
```

Review filenames as well as contents. Automated secret checks can supplement this review, but do not prove the import is safe.

Show the friend the intended public contents, check results, current branch, and destination repository. Obtain their approval to publish the import. With that approval and collaborator access, the import branch can use:

```powershell
git branch --show-current
git commit -m "Import existing Jobex demo"
git fetch origin --prune
git push -u origin import/existing-demo
```

The push example assumes the verified local branch is `import/existing-demo`. Reconcile any newer main commits and open a pull request against `main` using the workflow in section 10. The owner reviews the import before merging. Never force-push over their work.

Verify the result:

```powershell
gh repo view nexicv/jobex --json url,visibility,defaultBranchRef
git status --short --branch
git log -1 --oneline
git ls-remote origin refs/heads/main
```

Report the repository URL and published commit SHA. Do not call local staged files or a successful commit a successful push.

## 8. Mandatory freshness check for every later task

This applies equally to the owner's Codex and the friend's Codex and Claude. Do it at the start of each session, when returning after a break, and before taking a new task, even if the previous chat said everything was current.

Before editing:

```powershell
git status --short --branch
git branch --show-current
git remote -v
git fetch origin --prune
git branch -vv
git log -1 --oneline origin/main
git log --oneline HEAD..origin/main
git log --oneline origin/main..HEAD
```

Check the current branch's upstream too. `HEAD..origin/main` shows main commits missing locally, while `origin/main..HEAD` shows local commits absent from main. Feature commits absent from main are normal, but missing main commits need reconciliation before new work.

For a clean main branch:

```powershell
git switch main
git pull --ff-only origin main
git switch -c feature/SHORT-TASK-NAME
```

Choose a unique task branch, for example `feature/shift-card-layout`. If the branch already exists, inspect and reuse it deliberately rather than recreating it.

On an existing feature branch, check its remote counterpart if it has one. Fast-forward from that branch when possible. Incorporate new main commits with a deliberate merge when needed. Resolve conflicts and run the affected checks before continuing.

If uncommitted work prevents updating, preserve it and coordinate. Do not silently stash, discard, reset, or overwrite it. If fetch fails, explain that freshness is unverified and ask the owner whether to work offline.

## 9. Coordinate two people and multiple agents

Use one branch per task. When Codex and Claude work concurrently on one machine, give each a separate Git worktree and separate file ownership. A branch alone does not isolate two agents editing the same working folder.

Example, after fetching and confirming the new path and branch do not exist:

```powershell
git worktree add -b feature/worker-profile ..\jobex-worker-profile origin/main
```

Use the repository root as the working directory when creating that worktree. Install dependencies and prepare ignored local configuration within the worktree as the actual project requires.

Keep assignments bounded. One person might handle the employer form while another handles worker browsing, but agree on shared types and interfaces first. A schema, lockfile, manifest, or shared configuration needs an identified writer. Do not let multiple agents edit them concurrently without coordination.

Read `AGENTS.md` at the start of each project chat. Explicitly tell Claude to read `CLAUDE.md` and `AGENTS.md` as part of its task. These files belong in Git so both machines receive the same project rules. Keep private personal instructions outside the repository.

## 10. Push a task and request review

When authorized to publish a finished task, inspect the diff and run the project's checks. Fetch again before pushing:

```powershell
git fetch origin --prune
git status --short --branch
git diff --check
```

Review newer upstream and main commits and integrate changes that affect the work. After committing reviewed paths, push the actual branch:

```powershell
$jobexBranch = git branch --show-current
git push -u origin $jobexBranch
gh pr create --base main --head $jobexBranch --title "DESCRIBE THE CHANGE" --body-file 'PATH_TO_PR_BODY.md'
```

Use a real prepared PR description file with the behavior change, checks, and any limitations. Do not run placeholder commands literally.

The other teammate reviews the pull request. Fetch and check again before merging, resolve conflicts, and rerun affected checks. Merge only with the team's authorization. Never force-push main or use a push rejection as a reason to overwrite the remote.

## 11. Prepare the owner's local copy after the initial import

The owner's destination is already `D:\projects\jobex`, on `main`, with `https://github.com/nexicv/jobex.git` as `origin`. It tracks the shared setup baseline. Do not clone or initialize again.

After the demo reaches main, the owner's Codex should inspect the folder and remotes again, fetch, and use `git pull --ff-only origin main` from a clean main checkout. Preserve any local changes and resolve conflicts deliberately rather than overwriting work.

Once the owner has the shared main checked out, follow the actual demo README to install and run it. Keep the local path exactly as requested. Verify the same published commit works on both machines.

## 12. Setup completion report

The friend's Codex should return:

- GitHub username used and confirmed repository access
- Actual local clone path and remote URL
- Current branch, commit SHA, and result of the latest fetch
- Original demo location and whether it remains intact
- Detected framework, runtime, package manager, and commands
- Environment setup requirements, without secret values
- Demo, build, and check results with any failures
- Whether files are only local, committed, pushed, or merged
- The next small task proposed for the team

Before a production deployment, real payments, or real contracting, agree on a separate scope. The original charter's AI matching, contract drafts, time tracking, payouts, and commission ideas are proposed features, not proof that the current demo implements them.

## Official command references

- [Git installation](https://git-scm.com/downloads)
- [GitHub CLI installation](https://cli.github.com/)
- [GitHub CLI authentication](https://cli.github.com/manual/gh_auth_login)
- [GitHub CLI Git authentication setup](https://cli.github.com/manual/gh_auth_setup-git)
- [Create a repository](https://cli.github.com/manual/gh_repo_create)
- [Clone a repository](https://cli.github.com/manual/gh_repo_clone)
- [Create a pull request](https://cli.github.com/manual/gh_pr_create)
- [Git pull and fast-forward behavior](https://git-scm.com/docs/git-pull)
- [Git worktrees](https://git-scm.com/docs/git-worktree)
