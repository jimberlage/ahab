# ahab

Read the README first for an idea of the intention for the project, CLI configuration layout, and commands.

## Architecture

This is a Rust CLI tool for interacting with the [Aha API](https://www.aha.io/api).  Aha is a project management tool.

It reads documents, and writes back epics (not tasks, yet.)  It also interfaces with the OpenRouter API to do some of the commands, like `ahab breakdown`.  Here is the OpenRouter API (it does not have a Rust SDK yet): https://openrouter.ai/docs/quickstart#using-the-openrouter-api

The Aha token, the OpenRouter API key, and the default model to use should all be configurable in `ahab configure`, and can all be set on a per-profile basis.

## Prompting

When breaking down documents into epics, the tool should prioritize technical detail.  The use cases, while detailed, are there to indicate intent.  The epics are meant to be authoritative on how the features are actually completed on a technical level.  Multiple features may be included in a single epic.

## Agent System

Ahab uses OpenCode agents to provide an interactive workflow for creating and refining epics:

### Critic Agent (Primary)
The `critic` agent is the main interface when using `ahab critique`. It:
- Reviews and discusses created epics with the user
- Modifies epics in response to user feedback and concerns
- Validates epic quality and provides suggestions
- Pushes finished epics to Aha using the `ahab push` command via MCP
- Delegates to the `breakdown` subagent when creating new epics from pages

### Breakdown Agent (Subagent)
The `breakdown` agent is invoked by the critic when new epics need to be created. It:
- Analyzes `*.page.md` files in the session directory
- Breaks pages down into technical epics as `*.epic.md` files
- Prioritizes technical detail and implementation authority
- Groups related features into single epics where appropriate

### Workflow
1. Run `ahab convert` to fetch Aha pages into a session as `*.page.md` files
2. Run `ahab critique --session <id>` to start an interactive OpenCode session
3. The critic agent can create epics by delegating to @breakdown
4. Discuss and refine epics with the critic agent
5. When satisfied, ask the critic to push epics to Aha
6. The critic uses the MCP server to run `ahab push`

Both agents are automatically configured in each session's `.opencode/agents/` directory.

## Testing

Write unit tests, and attempt to mock the APIs where possible (we do not want to clog up a real Aha instance with test tasks.)
