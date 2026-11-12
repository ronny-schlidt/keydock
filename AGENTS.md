# KeyDock — shared project instructions

Canonical instructions for Claude Code and ChatGPT/Codex working in this repository.
`CLAUDE.md` imports this file; edit shared rules here only.

## Read only what the task needs

Tauri 2 + Rust backend, Svelte 5 + TypeScript frontend. macOS and Windows are product targets.
**Current work scope (Ronny, 2026-11-12): macOS first; skip Windows builds, tests and manual checks until requested.**
Preserve Windows code and existing CI; report Windows validation as skipped, never passed.

| Task | Read when relevant |
| --- | --- |
| Product behavior, setup, local build | [README](README.md) |
| Find the implementation; project analysis and change coupling | [Repository map](docs/repository-map.md) |
| Native boundaries and design decisions | [Architecture](docs/architecture.md) |
| Choose checks, desktop verification, existing evidence | [Testing](docs/testing.md) |
| Assistant setup, shared skills, efficient work, learning procedure | [AI workflow](docs/ai-workflow.md) |
| Known pitfalls and verified time/token savings | [Learnings](docs/learnings.md) — relevant entries only |
| Complete documentation directory | [Documentation index](docs/README.md) |

## Keep these boundaries

- Rust owns credentials, OpenRouter requests, global shortcut registration and text replacement. Never return a stored key through IPC, log it, export it or use it in browser preview.
- OpenRouter's endpoint is fixed. Imported prompts are text instructions, never permission to execute code or change destinations. Settings use local JSON; secrets use OS credentials. No backend server/database.
- Preserve user clipboard changes. Changed focus/selection or uncertain insertion means review the complete output; empty/partial responses must never be pasted. Native copy/paste is not atomic; do not claim replacement safety without actual desktop evidence.
- No telemetry, background clipboard monitoring or key logging. Observe platform input only during an explicit shortcut operation.
- Preserve `com.ronnyschlidt.promptkey` as app identifier and credential namespace unless a migration is explicitly requested.
- Keep `package-lock.json` and `src-tauri/Cargo.lock` tracked. Never commit keys, personal prompt exports, build outputs, caches or screenshots containing user content. Never add AI attribution.

## Work efficiently and verify

- Start with `git status --short --branch`. Preserve others' changes; don't stash/reset them. Commit and push only when Ronny asks, then include only this task's changes. Report uncommitted/unpushed work.
- Follow the relevant links, then search specific symbols with `rg -n` and read bounded sections. Do not reload the whole repository or every skill/document for each task.
- Use [keydock-verify](.agents/skills/keydock-verify/SKILL.md) for code/build changes or verification requests. Required check commands and scope live in [Testing](docs/testing.md). Docs-only work needs link/skill/diff checks, not application builds.
- Inspect UI changes visually. Browser preview establishes UI evidence only. Reuse successful checks while the relevant files remain unchanged; rerun affected checks after fixes.
- Keep behavior documentation aligned with code and actual platform evidence. Use [keydock-knowledge](.agents/skills/keydock-knowledge/SKILL.md) when new durable knowledge or documentation changes need reconciling.
- Before finishing, persist any newly verified pitfall, corrected assumption or concrete token/time saving in its owning doc/skill; use [Learnings](docs/learnings.md) for cross-cutting findings. Merge duplicates and remove stale advice. Do not append a session diary or invent a saving percentage.
- App icons come from `app-icon.svg`: `npm run tauri icon -- app-icon.svg --output src-tauri/icons`. Tray sources are `tray-icon.svg` → `tray.png` and `tray-done.svg` → `tray-done.png`, rendered at 44 px RGBA with resvg (see [Learnings](docs/learnings.md#render-tray-icons-with-resvg--2026-11-12)). Exclude generated iOS/Android icon directories.
