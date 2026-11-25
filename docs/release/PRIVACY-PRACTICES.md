# Abiding Word — Privacy and local-data practices (release draft)

**Owner review required before public distribution.** This document describes the behaviour implemented in the current local-first desktop application, not a final GDPR policy or a contractual promise by an unidentified publisher. A publication contact, distribution jurisdiction, exact version and effective date must be supplied by the owner.

## Where your work lives

Abiding Word's **Bible Study and Teaching writing, quotations, notes, annotations, projects, source indexing, reading positions, presentations, templates and settings** are held locally in its per-user SQLite database and managed source folders. The regular `.awbackup` format is a private copy of the database and imported source files, including third-party books and other material whose licences may prohibit redistribution. Do not upload or share it as a publication. The optional daily recovery snapshots contain the SQLite database **only**, not the imported source/image files; a snapshot does not replace a full backup.

Abiding Word does not require an online account for core Bible study, source reading or writing. These core features can operate offline, subject to any document files and language resources the user has actually installed. The programme does not encrypt the SQLite database or all backup files itself: secure local accounts, device encryption and sensible backup handling remain necessary. API keys are stored through the operating system's credential store, not ordinary settings or the SQLite content database.

## When a network connection is used

| Activity | What the app may transmit | User control |
|---|---|---|
| **Optional AI writing help or research** | The selected request, its explicitly previewed context from Scripture/notes/sources, model and the configured provider endpoint; authentication as required by the provider | Disabled by default; user configures provider and requests help. The assistant gives suggestions rather than silently editing the user's original writing. |
| **Optional web search and image services** | Search terms, query text or chosen image-description terms, and provider credentials where required | Run only when the relevant optional service is enabled/invoked. Imported image content can leave the provider's service if the user requests it; rights and attribution must still be checked. |
| **Optional remote-compatible or local-model provider** | The same prepared request, to the exact base URL set in Settings | A local-looking provider name alone is not proof of locality: a configured remote address sends data to that remote host. Users must check their configured endpoint. |
| **Development/test updater** | Requests for update metadata and signed update packages from the configured update endpoint | Only available in updater-configured builds; isolated unsigned acceptance installers do not register the updater. Ordinary core work is not dependent on update availability. |
| **External links** | Whatever a website normally receives when the user opens that website in their system browser | Opening the link is a separate user action. |

The single HTTP transport does not follow redirects (to prevent forwarding an authentication header to another host). For request failures, the application logs the hostname and a URL-redacted network error, not the AI request body, key or full URL. This is an engineering design property, not an independent audit of every third-party dependency. Individual AI, search, update and publisher services have **their own data-processing, retention and jurisdiction policies**; Abiding Word cannot control their server-side practices.

## Private material and exports

Source files the user imports are their own local managed copies. A citation link, direct passage recommendation or export must not be treated as permission to redistribute a copyrighted book. Authors decide whether to include excerpts in handouts or other publication; Abiding Word must clearly distinguish audience-facing material from private leader notes. These separation guarantees remain part of the Windows release acceptance gates and must be checked before public release.

AI outputs are suggestions. The user remains responsible for reviewing theological claims, verbatim Bible quotation accuracy, factual citations, copyright and rights before distributing a sermon, article, handout or presentation.

## Recovery and deletion

Users can export complete private backups from **Settings → Data and backup**. On restore, the application verifies the backup first and makes a safety copy of the existing database before replacing it. Deleting or archiving sources uses the application's library controls; permanent deletion can leave citation tombstone metadata so existing studies still explain the source originally cited. Ordinary logs, safety backups, separately exported work and OS backups are not necessarily removed by deleting a library record.

For a full local erasure a user must also manage their own separately created backups, exported files and operating-system storage. An offline application cannot delete copies uploaded by users to third-party providers.

## Required pre-publication sign-off

Before a stable public release, the publisher must supply: responsible entity and privacy contact; precise version, supported jurisdictions, telemetry and diagnostics choices (if any), support/log-handling practices, dependency/security audit, third-party processors and linked policies, rights complaint/contact route, and effective date. A final privacy notice must be checked against the actual signed distribution build. **This draft is not a declaration that release conditions are already met.**
