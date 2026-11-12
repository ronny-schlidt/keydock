---
name: keydock-verify
description: Verify KeyDock runtime, UI, dependency or build changes using the project checks and appropriate native evidence. Also use for an explicit KeyDock verification request; routine prose edits need only documentation checks.
---

# Verify KeyDock changes

Resolve the repository root using `git rev-parse --show-toplevel`; paths below are relative to it. Read `AGENTS.md` if not already loaded. Use `docs/testing.md` as the single owner of check commands and current platform scope.

1. Inspect `git status --short --branch`, the relevant diff and untracked files. Distinguish the task's changes from pre-existing work; do not reset, stash or commit unrelated files.
2. Classify the change using `docs/testing.md#automated-checks`. For code/tests/dependencies/build changes, run the required macOS suite (`npm run verify` runs it in one step). For documentation-only verification, check local Markdown references, skill frontmatter, shared symlink targets and `git diff --check`; do not build the app just for prose.
3. Use existing installed dependencies when consistent with lockfiles. Don't dump lockfiles or reinstall packages unless the task requires it. Keep operations sharing generated build output sequential; don't run concurrent Cargo builds against the same target directory.
4. For UI work, inspect the affected screen with synthetic content. For clipboard/focus/shortcut/credential/lifecycle work, select the relevant native macOS scenarios. Browser preview and unit tests do not establish native insertion safety. Do not make real provider calls or use private selected text just to run the local suite.
5. Windows validation is deferred until Ronny requests it. Keep Windows source and configured CI intact; don't trigger a Windows run to finish this workflow. Report it as skipped.
6. Fix failures caused by the task and rerun affected checks. Once relevant inputs are unchanged and checks pass, do not repeat them merely for another summary. Distinguish pre-existing failures from regressions and state any blocked check.
7. Update behavior docs if needed. Record actual native/packaging evidence in `docs/testing.md` with date, OS, what ran and what it proves. Do not rewrite historical evidence as current or log every routine successful lint run.

Report the change, passed/failed/skipped checks, remaining platform/native limits, and uncommitted/unpushed state concisely. Never describe an unrun check as passed.
