# ahab

Read the README first for an idea of the intention for the project, CLI configuration layout, and commands.

## Architecture

This is a Rust CLI tool for interacting with the [Aha API](https://www.aha.io/api).  Aha is a project management tool.

It reads documents, and writes back epics (not tasks, yet.)  It also interfaces with the OpenRouter API to do some of the commands, like `ahab breakdown`.  Here is the OpenRouter API (it does not have a Rust SDK yet): https://openrouter.ai/docs/quickstart#using-the-openrouter-api

The Aha token, the OpenRouter API key, and the default model to use should all be configurable in `ahab configure`, and can all be set on a per-profile basis.

## Prompting

When breaking down documents into epics, the tool should prioritize technical detail.  The use cases, while detailed, are there to indicate intent.  The epics are meant to be authoritative on how the features are actually completed on a technical level.  Multiple features may be included in a single epic.

## Testing

Write unit tests, and attempt to mock the APIs where possible (we do not want to clog up a real Aha instance with test tasks.)
