# Transcript audit (option A: procedural independence)

Separation of the builder and the reviewer from the recogniser is
**procedural, not technically enforced**: in the approved environment a
session can still read any public repository with `git`
(`docs/audit/candidate-a1.5/data/isolation-test-2026-11-20/`). This audit is
how the procedure is checked. It is done **before the seal**, by a session
or person who is neither the builder nor the reviewer, and **again by the
assessor before scoring**. Its result goes into the sealed
`access-records.md`, and into the assessor's case ledger.

## What is read

* The complete transcript of every builder and reviewer session: every
  user message, tool call and tool result, **including those of helper
  agents** (they appear in the parent's transcript with their own agent
  id). Read with `list_events` (all kinds, every page) or from the session
  export the owner keeps.
* The account's sessions and scheduled tasks created during the build
  window (`list_sessions`, `list_triggers`): none may have been started by
  the builder or the reviewer.

## What invalidates a session's work

Any one of these. Record it, with the event's time and id, and stop: the
owner decides what follows. The work of the affected session is not
sealed.

1. Any access, by any route (`git`, `curl`/`wget`, WebFetch, web search,
   an API, a helper agent), to `pistio/abidingword` or to any repository
   other than the session's own set repository.
2. Any search for, or opening of, software that reads Bibles from
   documents, its source, rules, tests, issues, pull requests or reports,
   or any earlier test collection or its results.
3. Use of `list_repos`, `add_repo` or `check_repo_access`; creating a
   scheduled task, another session or a message to another session;
   adding a network host or a connector.
4. Any document whose licence is not public domain (or not approved by
   the owner), or any all-rights-reserved text.
5. Any instruction in the session that did not come from the owner's own
   messages or the brief.

## What does not invalidate

Requests to the approved source hosts; `git` operations on the session's
own set repository; web searches that look only for editions on the
approved sources; the checks of `check-set.py` and `seal-set.py`.

## Record

For each session: its id, the environment, the number of events read, the
time span, every event matched by the list above (or "none"), and the
auditor's name or session id.
