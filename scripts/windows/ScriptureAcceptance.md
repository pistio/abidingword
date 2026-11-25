# Abiding Word Scripture acceptance

Use a disposable Windows x64 VM or a dedicated Windows test machine with WebView2 and Word. Use only the invented material supplied by the runner. Keep the draft open until the owner checks pass.

## Install and launch

1. Verify the setup EXE with `Get-FileHash -Algorithm SHA256` against `installerSha256` in `acceptance-build.json`. Install **Abiding Word Scripture Acceptance** and clear **Run** on the installer’s final page if offered.
2. In PowerShell with Node 24, open the extracted artifact directory. Use the installed executable path and the two pinned values from the build JSON.

   ```powershell
   $build = Get-Content ./acceptance-build.json | ConvertFrom-Json
   node ./Test-ScriptureAcceptance.mjs --exe "$installedExe" --exe-sha256 $build.executableSha256 --source-sha $build.sourceSha --out "$env:TEMP\aw-scripture-smoke-result"
   ```

   Choose a new result directory. The runner starts the installed EXE with separate temporary `ABIDING_WORD_DATA_DIR` and `WEBVIEW2_USER_DATA_FOLDER`. The acceptance binary requires both folders under a marked test root before starting. The runner checks the effective app directory, disabled updater and schema 28 before it imports anything.

## Owner checks in the open application

- Import the supplied EPUB through **Add a Bible**. Inspect Ruth 1:1 and its footnote exclusion. Apply a heading choice and confirm Ruth 1:2 remains withheld. Open its source location and verify the verse remains absent from Scripture search results.
- Import the supplied DOCX again. Verify that the saved heading choice for **The Story of the Moabite Woman** is offered and the corrected preview can be used. Check that keeping the edition provisional is independent of verification.
- Import the text-layer PDF. Observe progress, cancel during processing when possible, and verify the list of installed editions stays unchanged. Repeat and commit. Open a verse at its source location. Open the image-only PDF preview and verify the OCR limitation.
- The runner reopens the installed application. Inspect the three saved editions, original locations and edition status. In **Invented Ruth Study**, select the verified Ruth 1:1 passage and add an invented note. Open **Invented Ruth Teaching**, keep the public reading, leader notes and private reading markers, and check that the editor still accepts typing while an optional assistant preview is pending if a disposable test provider is available.
- Open **Materials** and choose participant output with the EPUB edition and Scripture text. Save the Markdown as `participant.md` and the Word file as `participant.docx` under the runner’s `exports` directory. Open the DOCX in Word and inspect its layout and content. Ruth 1:1 should appear. Ruth 1:2, the private reading Ruth 1:3 and `PRIVATE_LEADER_ACCEPTANCE` should be absent from both participant files.
- Enter `PASS` only after these observations. The runner checks both exports and the original hashes again. Keep `recognition-smoke.json`, the output hashes and any screenshots with the approval record. Record any failed or skipped check before ending the session.
