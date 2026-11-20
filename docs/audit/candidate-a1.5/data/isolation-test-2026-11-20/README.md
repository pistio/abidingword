# Independent-environment check (20 November 2026)

**Question.** Can a builder or reviewer session reach and write to its own
private repository while the public `pistio/abidingword` repository stays
blocked?

**Answer, as tested: no, not by enforcement.** A fresh session in a
restricted environment cannot open the repository's web page, raw files,
API or zip archive. It **can** read the repository with plain `git`
(`git ls-remote` and `git clone`), as it can any public repository. Writing
to its own repository was not demonstrated. A session started by another
session will not push without a person's go-ahead (below).

Invented files only. No Set H material was selected, downloaded, inspected
or made.

## Set-up

| | |
|---|---|
| Disposable private repository | `pistio/abidingword-seth-isolation-test` (made for this test; a README only; safe to delete) |
| Environment | `Set H isolation test` (`env_01MziRBQfjyurQjdAc63sYsK`), personal: network **Limited**, allowed hosts `www.gutenberg.org` only, package managers **off**, MCP servers **off**, no setup script, no variables, no secrets |
| Sessions | `session_011iiPcVvhdecywvqehnFXRQ` (probes) and `session_011iiPcUd2VF4YHZiXU9gpCL` (write path), each new, each with only the test repository; created by this engineering session, so each has a parent |

## Results

The raw tool outputs were read from the probe session's transcript, not
only from its summary. The refs it read were checked against the real
repository: they match (`HEAD ceba01ff…`).

| # | Probe | Result | Verdict |
|---|---|---|---|
| A1 | Working directory, remotes | only the test repository's checkout; one remote, the test repository | enforced: no project checkout |
| A2 | Any `*abidingword*` path or `CLAUDE.md` elsewhere on disk | none | enforced |
| A3 | `~/.claude/projects` | only the test repository's folder | enforced: no project memory |
| A4 | The session's own context | no earlier message; GitHub access scoped to the test repository only; nothing about the project; **no `mcp__github__` tools** | enforced |
| B1 | `https://github.com/pistio/abidingword` | 403 (proxy) | **blocked** |
| B2 | `raw.githubusercontent.com/…/README.md` | CONNECT 403 | **blocked** |
| B3 | `api.github.com/repos/pistio/abidingword` | 403 | **blocked** |
| B4 | `codeload.github.com/…/zip/HEAD` | 403 | **blocked** |
| B5 | `git ls-remote https://github.com/pistio/abidingword` | exit 0; returned HEAD and a branch | **NOT BLOCKED** |
| B6 | `git clone --depth 1 …/pistio/abidingword` | succeeded (15 top-level entries; deleted unread) | **NOT BLOCKED** |
| B8 | `gh api repos/pistio/abidingword` | "GitHub access to this repository is not enabled for this session" | blocked |
| D1 | `https://github.com/` | 400 | blocked |
| D2 | `https://api.github.com/` | 200 | open (the API root; repository paths outside the session's scope were 403) |
| D3 | `git ls-remote` of another public repository (`octocat/Hello-World`) | returned HEAD | **NOT BLOCKED**: any public repository |
| D4 | `git ls-remote` of a private repository of the same account, not attached | "could not read Username" | **blocked** |
| D5 | `git ls-remote` of the session's own repository | returned HEAD | allowed (expected) |
| E1 | `https://www.gutenberg.org/robots.txt` (the allowed host) | 503 | the site refuses cloud clients: the same `503`, `retry-after: 21600` from the unrestricted engineering session |
| E2 | `https://ebible.org/` (not allowed) | CONNECT 403 | **blocked** |
| E3 | `https://example.com/` (not allowed) | CONNECT 403 | **blocked** |
| E4 | `https://pypi.org/simple/` (package registry) | name not resolved | **blocked** |
| C | Commit and push an invented note to the session's own repository | refused by the session's own permission check ("Sensitive Information Leakage", on a command that also ran network probes); a second fresh session declined to push without a person's go-ahead, because its instructions came from another session | **not demonstrated** |
| G | WebFetch of the repository page and raw file (server-side fetching); `add_repo` for the repository | not run: the sessions declined without a person's go-ahead | **untested** |

## What this means

* **Enforced, demonstrated:** no checkout, memory or context from the
  project; GitHub tools and API scoped to the session's repository; the
  repository's web, raw, API and archive endpoints refused; other private
  repositories refused; hosts outside the allowlist refused; package
  registries off.
* **Not enforced, demonstrated:** `git` reads of **any public repository**,
  including this one. The environment's network allowlist does not govern
  them. While `pistio/abidingword` is public, keeping a builder away from
  its source is **procedure**, checked afterwards in the transcript.
* **Not yet known:** whether WebFetch (which may fetch outside the
  container) or `add_repo` reaches the repository; and the write path to
  the session's own repository. Both sessions wait for a person's message.
* **Operational:** a session started by another session treats its
  instructions as untrusted and will not push or widen its access without
  a person. The builder and the reviewer must therefore be started and
  instructed by the owner. Project Gutenberg refuses requests from this
  cloud (503 with a six-hour retry), so it cannot be a source for cloud
  sessions.

## To finish the check (owner, one message)

Open `https://claude.ai/code/session_011iiPcUd2VF4YHZiXU9gpCL` and reply:

> Confirmed. Run all four steps, including the push to `main` of this test repository.

That session then pushes the invented note (the write path) and runs the
two WebFetch probes and the two read-only diagnostics, and reports them.

## Clean-up after the check

Delete the repository `pistio/abidingword-seth-isolation-test` and the
environment `Set H isolation test`, and archive the two sessions. None
holds anything but invented text.
