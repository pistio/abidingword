# Independent-environment check (20 November 2026)

**Question.** Can a builder or reviewer session reach and write to its own
private repository while the public `pistio/abidingword` repository stays
blocked?

**Answer, as tested: write yes, block no.**
* **Write: yes.** With the owner's go-ahead, a fresh session committed and
  pushed an invented file to its own private repository (`f50b267`,
  corroborated on GitHub).
* **Block: not by enforcement.** A fresh session in a restricted
  environment cannot open the repository's web page, raw files, API or
  zip archive, and server-side page fetches are blocked too. It **can**
  read the repository with plain `git` (`git ls-remote` and `git clone`),
  as it can any public repository, and web search works. The owner chose
  procedural independence with a full transcript audit (option A,
  `SET-H-APPROVAL.md`); nothing here is described as technically
  enforced.

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
| C | Commit and push an invented note to the session's own repository | first session: refused by its own permission check (the instruction came from another session, in a command that also ran network probes). Second session, after the owner's message: `isolation-test/invented-note.txt` committed and pushed, `e5179d1..f50b267 HEAD -> main`; on GitHub `main` is `f50b267` holding only the README and that note | **allowed (demonstrated)** |
| G1, G2 | WebFetch (through a helper agent) of the repository page and its raw README | `getaddrinfo ENOTFOUND github.com` / `raw.githubusercontent.com`: page fetches follow the environment's allowlist | **blocked** |
| G5 | How git reaches GitHub | through the session's HTTPS proxy (`HTTPS_PROXY`, `http.proxyauthmethod=basic`), SSH URLs rewritten to HTTPS | explains B5, B6, D3 |
| G6 | Gutenberg's 503 | the proxy connected (`200 Connection Established`), then the site answered `503`, `retry-after: 21600` | the site, not the environment |
| — | Does a helper agent's activity reach the transcript? | yes: the WebFetch helper's prompt, tool calls and results appear in the parent session's events with the agent's id | **auditable** |

## Readiness probes in the approved environment

`Collection H` (`env_01Kus7n5JxhUcNsGy6fHog2r`): network **Limited**, allowed
host `ebible.org` only, package managers off, MCP servers off, no setup
script, variables or secrets; repository `pistio/collection-h` (private).
Read-only probe sessions `session_011iiPcSZEDbzb7FEUTXiBo7` and
`session_011iiPcWVox1cFwyXNzRqa8y`; nothing created, committed or pushed by
them.

| Probe | Result | Route |
|---|---|---|
| Workspace and context | only `pistio/collection-h`; no `CLAUDE.md`; no earlier message; GitHub access scoped to `pistio/collection-h` | as tested above |
| `https://ebible.org/` | 200 | **open** (the approved host) |
| `www.gutenberg.org`, `example.com` | CONNECT 403 | **blocked** |
| `github.com/pistio/abidingword`, `api.github.com/repos/pistio/abidingword` | 403, 403 | **blocked** |
| `git ls-remote https://github.com/pistio/abidingword HEAD` | exit 0, `ceba01ff62c1` | **open** (not enforced) |
| WebSearch (a neutral query) | 17 results | **open**: search runs outside the network allowlist; every query is in the transcript |
| Repository and scheduling tools (`list_repos`, `add_repo`, `check_repo_access`, scheduled tasks) | not exposed to the session: two probes found none, loaded or deferred (the session's metadata still lists them) | **not available** as tested; the audit still checks for any use |

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
* **Open but visible:** web search, and `git` reads of public
  repositories. Every tool call, including a helper agent's, is in the
  session's transcript, which is what the audit reads
  (`docs/evaluation/transcript-audit.md`).
* **Operational:** a session started by another session treats its
  instructions as untrusted and will not push or widen its access without
  a person. The builder and the reviewer must therefore be started and
  instructed by the owner. Project Gutenberg refuses requests from this
  cloud (503 with a six-hour retry), so it cannot be a source for cloud
  sessions.

The owner's go-ahead was given in `session_011iiPcUd2VF4YHZiXU9gpCL` on
20 November 2026 (06:24 UTC); its results are in the table above.

## Clean-up after the check

Delete the repository `pistio/abidingword-seth-isolation-test` and the
environment `Set H isolation test`, and archive the two sessions. None
holds anything but invented text.
