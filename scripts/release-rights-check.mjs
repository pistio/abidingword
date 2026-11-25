#!/usr/bin/env node
/**
 * Release gate: integrity and notice *inventory* for bundled Bible archives.
 * This cannot establish legal ownership; humans must review the source claims.
 * CI checks that no unrecorded source or silently modified ZIP can ship.
 */
import { createHash } from "node:crypto";
import { readdirSync, readFileSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export function auditBundledScripture(root) {
  const dir = join(root, "resources", "scripture");
  const noticesPath = join(root, "docs", "release", "THIRD-PARTY-NOTICES.md");
  const notices = readFileSync(noticesPath, "utf8");
  const ids = readdirSync(dir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name).sort();
  if (!ids.length) throw new Error("No bundled Bible editions found");
  // Verify that the installer contains precisely the reviewed source archive
  // paths and the public rights-notice file, not merely a notice in Git.
  const config = JSON.parse(readFileSync(join(root, "src-tauri", "tauri.conf.json"), "utf8"));
  const mappings = config.bundle?.resources;
  if (!mappings || typeof mappings !== "object" || Array.isArray(mappings)) {
    throw new Error("Tauri resource mappings are missing");
  }
  const noticeSource = "../docs/release/THIRD-PARTY-NOTICES.md";
  if (mappings[noticeSource] !== "licenses/THIRD-PARTY-NOTICES.md") {
    throw new Error("Public notices must travel inside the packaged installer");
  }
  const expectedArchives = new Set(ids.map((id) => `../resources/scripture/${id}/${id}_usfm.zip`));
  const observedArchives = Object.keys(mappings).filter((name) => name.startsWith("../resources/scripture/") && name.endsWith("_usfm.zip"));
  if (observedArchives.length !== expectedArchives.size || observedArchives.some((p) => !expectedArchives.has(p))) {
    throw new Error("Packaged Bible archives differ from reviewed source folders");
  }
  const results = [];
  for (const id of ids) {
    if (!/^[A-Za-z0-9_-]+$/.test(id)) throw new Error(`Unsafe edition identifier ${id}`);
    const home = join(dir, id);
    const infoPath = join(home, "SOURCE.md");
    const archivePath = join(home, `${id}_usfm.zip`);
    if (mappings[`../resources/scripture/${id}/${id}_usfm.zip`] !== `scripture/${id}_usfm.zip`) {
      throw new Error(`${id}: archive not included under its expected installer path`);
    }
    if (!existsSync(infoPath) || !existsSync(archivePath)) throw new Error(`${id}: missing SOURCE.md or recorded USFM ZIP`);
    const md = readFileSync(infoPath, "utf8");
    const hash = md.match(/^\| SHA-256 \| `([0-9a-f]{64})` \|/m)?.[1];
    if (!hash) throw new Error(`${id}: no SHA-256 provenance in SOURCE.md`);
    const bytes = readFileSync(archivePath);
    const actual = createHash("sha256").update(bytes).digest("hex");
    if (hash !== actual) throw new Error(`${id}: SHA-256 differs from SOURCE.md`);
    const rights = md.match(/^\| Licence \| (.*?) \|/m)?.[1];
    if (!rights) throw new Error(`${id}: missing licence statement`);
    const cc = /CC BY 4\.0|Creative Commons Attribution 4\.0/i.test(rights);
    const pd = /public domain/i.test(rights);
    const expired = /economic copyright expired/i.test(rights);
    if (!cc && !pd && !expired) throw new Error(`${id}: source rights need human review`);
    const line = notices.split("\n").find((s) => s.startsWith("| ") && s.includes("(`" + id + "`)"));
    if (!line) throw new Error(`${id}: missing from distribution notices`);
    if (cc && !/CC BY 4\.0/.test(line)) throw new Error(`${id}: CC BY attribution not identified in distribution notices`);
    if (expired && !/attribution|integrity|성명표시권|동일성유지권/i.test(line)) throw new Error(`${id}: integrity and attribution notice missing`);
    const copyright = id === "kor1961" ? "copyright-statement.txt" : "copr.htm";
    if (!existsSync(join(home, copyright))) throw new Error(`${id}: original rights evidence ${copyright} missing`);
    const otherZips = readdirSync(home).filter((p) => p.endsWith(".zip") && p !== `${id}_usfm.zip`);
    if (otherZips.length) throw new Error(`${id}: unrecorded ZIP resources: ${otherZips.join(", ")}`);
    results.push({ id, bytes: bytes.length, sha256: actual, rights });
  }
  return results;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
    const results = auditBundledScripture(root);
    console.log(`Bundled Scripture resources: ${results.length} exact archives match SOURCE.md and the notices inventory.`);
  } catch (error) {
    console.error(`Bundled Bible resource audit FAILED: ${error.message}`);
    process.exitCode = 1;
  }
}
