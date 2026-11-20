import { createHash } from "node:crypto";
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
// @ts-expect-error -- Node script is intentionally plain JavaScript.
import { auditBundledScripture } from "../../scripts/release-rights-check.mjs";

const folders: string[] = [];
function fixture({ licence = "Public Domain", withNotice = true }: { licence?: string; withNotice?: boolean } = {}) {
  const root = mkdtempSync(join(tmpdir(), "aw-release-rights-"));
  folders.push(root);
  const home = join(root, "resources", "scripture", "invented");
  const notices = join(root, "docs", "release");
  mkdirSync(home, { recursive: true });
  mkdirSync(notices, { recursive: true });
  mkdirSync(join(root, "src-tauri"), { recursive: true });
  writeFileSync(join(root, "src-tauri", "tauri.conf.json"), JSON.stringify({
    bundle: { resources: {
      "../docs/release/THIRD-PARTY-NOTICES.md": "licenses/THIRD-PARTY-NOTICES.md",
      "../resources/scripture/invented/invented_usfm.zip": "scripture/invented_usfm.zip",
    } },
  }));
  const archive = Buffer.from("This is invented test content.");
  const sha = createHash("sha256").update(archive).digest("hex");
  writeFileSync(join(home, "invented_usfm.zip"), archive);
  writeFileSync(join(home, "copr.htm"), "An invented public-domain source for a test.");
  writeFileSync(join(home, "SOURCE.md"), `| SHA-256 | \`${sha}\` |\n| Licence | ${licence} |\n`);
  writeFileSync(join(notices, "THIRD-PARTY-NOTICES.md"), withNotice ? "| Test (\`invented\`) | Public Domain |" : "# No notices");
  return { root, home, sha };
}
afterEach(() => { for (const p of folders.splice(0)) rmSync(p, { recursive: true, force: true }); });

describe("distribution integrity checks for bundled Scripture", () => {
  it("accepts a recorded archive with matching rights evidence", () => {
    const { root, sha } = fixture();
    expect(auditBundledScripture(root)).toEqual([expect.objectContaining({ id: "invented", sha256: sha })]);
  });
  it("refuses a silently changed source", () => {
    const { root, home } = fixture();
    writeFileSync(join(home, "invented_usfm.zip"), "changed");
    expect(() => auditBundledScripture(root)).toThrow(/SHA-256 differs/);
  });
  it("refuses a resource missing from public distribution notices", () => {
    const { root } = fixture({ withNotice: false });
    expect(() => auditBundledScripture(root)).toThrow(/missing from distribution notices/);
  });
  it("refuses a CC BY text without its credit in the notice", () => {
    const { root } = fixture({ licence: "CC BY 4.0" });
    expect(() => auditBundledScripture(root)).toThrow(/CC BY attribution/);
  });
  it("refuses an archive missing from the installer resources", () => {
    const { root } = fixture();
    const config = join(root, "src-tauri", "tauri.conf.json");
    const wrong = JSON.parse(readFileSync(config, "utf8"));
    delete wrong.bundle.resources["../resources/scripture/invented/invented_usfm.zip"];
    writeFileSync(config, JSON.stringify(wrong));
    expect(() => auditBundledScripture(root)).toThrow(/Packaged Bible archives differ/);
  });
  it("refuses a build that omits its public attribution notice", () => {
    const { root } = fixture();
    const config = join(root, "src-tauri", "tauri.conf.json");
    const wrong = JSON.parse(readFileSync(config, "utf8"));
    delete wrong.bundle.resources["../docs/release/THIRD-PARTY-NOTICES.md"];
    writeFileSync(config, JSON.stringify(wrong));
    expect(() => auditBundledScripture(root)).toThrow(/notices must travel/);
  });
  it("refuses an unrecorded source archive or missing rights", () => {
    const { root, home } = fixture();
    writeFileSync(join(home, "unrecorded.zip"), "stray");
    expect(() => auditBundledScripture(root)).toThrow(/unrecorded ZIP/);
    writeFileSync(join(home, "SOURCE.md"), "| Licence | private |");
    expect(() => auditBundledScripture(root)).toThrow(/no SHA-256 provenance/);
  });
});
