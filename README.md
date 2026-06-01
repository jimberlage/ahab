# ahab

Ahab (Aha Butler) is a CLI helper for generating Aha tickets from plain text.  It is opinionated.

Like the AWS CLI, it operates off of profiles.  There is a default profile, and others are configurable.

## Configuration

Configuration is stored in `"${HOME}/.ahab".  Running `ahab configure` will walk you through setup.  At a minimum, you need an API token.

`.ahab/credentials` stores credentials, in TOML format, while `.ahab/config` stores other configuration (like workspaces, teams, etc.) for specific profiles.

## Usage

Running `ahab breakdown` will take a piece of documentation in an Aha page, convert it to markdown, and suggest epics as a series of markdown documents.  It writes documents to `.ahab/sessions/<session_id>`.  It returns the session_id to the user.  It also contains a special file, `.ahab/sessions/<session_id>/metadata.toml`, used to store the Aha page and any other metadata that makes sense.

Running `ahab accept --session <session_id>` will convert the markdown documents in `.ahab/sessions/<session_id>` to epics in Aha, and return links to the epics.  It also references the metadata.toml section to find the parent page, and adds links to the created epics in a comment. 
