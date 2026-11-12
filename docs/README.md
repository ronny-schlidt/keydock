# Documentation index

Start with [shared project instructions](../AGENTS.md). Read the following documents only when their topic applies.

| Document | Purpose |
| --- | --- |
| [Product README](../README.md) | Setup, prompts, privacy, platform limits, development and packaging |
| [Repository map and analysis](repository-map.md) | File ownership, request flow, coupled changes and open engineering limits |
| [Architecture](architecture.md) | Design decisions, security boundaries, application identity and future trade-offs |
| [Testing](testing.md) | Check commands, current macOS scope, manual scenarios and dated evidence |
| [AI workflow](ai-workflow.md) | Claude/Codex setup, shared skills, efficient reading and durable learning |
| [Learnings](learnings.md) | Verified pitfalls and reusable time/token savings |

## Assistant entry points and skills

- [AGENTS.md](../AGENTS.md): shared rules and task routing.
- [CLAUDE.md](../CLAUDE.md): imports the shared rules for Claude Code.
- [keydock-verify](../.agents/skills/keydock-verify/SKILL.md): choose and run checks; distinguish UI, native and Windows evidence.
- [keydock-knowledge](../.agents/skills/keydock-knowledge/SKILL.md): keep the project knowledge accurate, compact and shared.

Add new durable documentation here and link it from its topic owner. Keep detailed procedures out of the automatically loaded instruction file.
