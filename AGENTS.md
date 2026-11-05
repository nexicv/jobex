# Jobex agent instructions

Recorded by Codex 6.1 Sol Medium, 5 November 2026 (Europe/Warsaw), at the project owner's request.

## Scope and current baseline

Jobex is a university project for finding short-term work and filling shifts. A teammate already has a working demo. Preserve that demo and its working behavior. Inspect its actual code, framework, dependencies, and run commands before proposing changes. Do not scaffold a replacement or choose a different stack without the team's agreement.

The owner's local folder is `D:\projects\jobex`. Other contributors use their own local paths. The intended shared repository is `nexicv/jobex`, public. Confirm the actual remote and repository status before depending on it. Read `FRIEND_SETUP.md` for the initial import and collaboration workflow.

## Freshness check for every work session or new task

These rules apply to all assistants, including Codex and Claude, on every contributor's machine.

1. Before editing, inspect `git status --short --branch`, the current branch, and `git remote -v`. Preserve all existing local changes and verify that origin is the team's repository.
2. Run `git fetch origin --prune` every time work starts or resumes. Check both the branch's upstream and `origin/main`. Compare the actual fetched commits, not a previous chat's description.
3. On a clean `main`, use `git pull --ff-only origin main` before creating a feature branch. On a feature branch, compare it with its upstream and with `origin/main`. Incorporate relevant newer commits before editing. Fast-forward where possible. If histories have diverged, inspect and merge deliberately, resolve conflicts, and run appropriate checks. Do not silently continue from an older baseline.
4. If local changes prevent an update, preserve them and explain the situation. Never automatically discard or stash them. Do not use reset --hard, git clean, force-push, or overwrite another person's work to make synchronization succeed.
5. If fetching fails or no remote exists, say so. Do not claim the checkout is current. During initial setup, follow the handoff's bootstrap steps. Later, ask the owner whether to continue offline when freshness cannot be established.
6. Fetch again before pushing and before merging. If new changes affect the work, integrate and check them before proceeding. A pre-push fetch cannot prevent a later race, so handle push rejection by fetching and reconciling, never by force-pushing.
7. In the completion report, name the working branch and starting commit, summarize checks, and distinguish local changes, pushed commits, and merged work. Do not claim another agent read an update without evidence.

## Collaboration

- Use one branch per bounded task. Keep `main` as the shared working baseline.
- Use a separate worktree for each agent that edits concurrently. Never run Codex and Claude as concurrent writers in the same working tree.
- Agree on file ownership before parallel changes. Coordinate shared files such as dependency manifests, lockfiles, database schemas, and configuration.
- Review the actual diff and run the demo's documented checks before delivery. Do not invent passing test or build results.
- Preserve the original demo until the imported copy runs successfully.
- Keep real credentials, API keys, personal data, dependency folders, and generated outputs out of the public repository. Audit the initial import before publishing it.
- Commit, push, merge, publish, or deploy only within the owner's authorization for the current task. An instruction to plan or inspect is not publishing permission.
- Use US English by default. Keep exact code, source quotations, and official names unchanged.
- Local project paths and account setup are machine-specific. Do not copy another person's global assistant configuration, credentials, or personal instructions.

## Current stage

The task is preparing collaboration around an existing demo. Do not start building new app features merely because this instruction file exists. Authentication, repository access, initial publication, and installing project dependencies must be verified on the machine where they run.
