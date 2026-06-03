# ahab

Ahab (Aha Butler) is a CLI helper for generating Aha tickets from plain text.  It is opinionated.

Like the AWS CLI, it operates off of profiles.  There is a default profile, and others are configurable.

## Configuration

Configuration is stored in `"${HOME}/.ahab"`.  Running `ahab configure` will walk you through setup.  At a minimum, you need an API token.

`.ahab/credentials` stores credentials, in TOML format, while `.ahab/config` stores other configuration (like workspaces, teams, etc.) for specific profiles.

## Install

On macOS:

```zsh
brew tap jimberlage/ahab https://github.com/jimberlage/ahab
brew install ahab
```

## Usage

### Converting Pages

Running `ahab convert` takes one or more Aha pages (either as full URLs like `https://apexlabs.aha.io/pages/VAFM-N-91` or as slugs like `VAFM-N-91`) and converts them to markdown files in a session directory. The pages are stored as `*.page.md` files in `.ahab/sessions/<session_id>`. You can specify a session ID to add pages to an existing session, or let it create a new one.

**Key behaviors:**
- **Recursive child fetching**: If a page has child pages, they will be automatically fetched and converted as well. This is useful for pages with hierarchical structure like documentation trees.
- **Always overwrites**: Running convert on the same pages will always fetch the latest content from Aha and overwrite existing `.page.md` files. This ensures you always have the most up-to-date content.

Example:
```sh
# Convert one or more pages (automatically includes child pages)
ahab convert https://apexlabs.aha.io/pages/VAFM-N-91 VAFM-N-92

# Add to an existing session
ahab convert --session <session_id> VAFM-N-93

# Fetch latest changes for a page (overwrites existing)
ahab convert --session <session_id> VAFM-N-91
```

### Creating Epics

After converting pages, you can manually create epic markdown files as `*.epic.md` in the session directory. Each epic should follow the format:

```markdown
# Epic Title

## Description

Epic description here...

## Acceptance Criteria

- Criterion 1
- Criterion 2

## Technical Notes

Technical implementation details...
```

### Uploading Epics

Running `ahab push --session <session_id>` will convert the `*.epic.md` files in `.ahab/sessions/<session_id>` to epics in Aha, and return links to the created epics. 

**Key behaviors:**
- **Skips existing epics**: The command maintains a manifest of uploaded epics, so running push multiple times will only upload new epics that haven't been created yet.
- **Source page comments**: The command references the metadata.toml to find the parent page and adds links to the created epics in a comment. 

## Release new version

Run the release script:

```sh
./scripts/release.sh 1.0.1
```

Then, go to the releases, find the SHA256 values, and add them into [`Formula/ahab.rb`](Formula/ahab.rb).
