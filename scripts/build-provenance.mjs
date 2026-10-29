#!/usr/bin/env node
// Offline input attestation for an approved Windows build. Does not build or publish.
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = process.argv.slice(2);
function value(flag) { const index = args.indexOf(flag); return index < 0 ? undefined : args[index + 1]; }
try {
  const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
  const commit = git("rev-parse", "HEAD");
  const expected = value("--expected-sha");
  if (!expected || !/^[a-f0-9]{40}$/.test(expected) || commit !== expected) throw new Error("checkout does not match the intended full source SHA");
  const dirty = Boolean(git("status", "--porcelain", "--untracked-files=no"));
  if (args.includes("--require-clean") && dirty) throw new Error("tracked source changes are present before the build");
  const catalogue = readFileSync(resolve(root, "src-tauri/src/db/migrations.rs"), "utf8").split("pub const MIGRATIONS")[1]?.split("\n];")[0] ?? "";
  const migrations = [...catalogue.matchAll(/version:\s*(\d+),\s*name:\s*"([^"]+)",\s*sql:\s*include_str!\("\.\.\/\.\.\/migrations\/([^"]+)"\)/g)].map((match) => {
    const sql = readFileSync(resolve(root, "src-tauri/migrations", match[3]));
    return { version: Number(match[1]), name: match[2], file: match[3], sha256: createHash("sha256").update(sql).digest("hex") };
  });
  if (migrations.length !== 26 || migrations.some((m, i) => m.version !== i + 1) || migrations[9].name !== "presentations" || migrations[23].name !== "section_status_and_fields" || migrations[24].name !== "teaching_blocks" || migrations[25].name !== "project_teaching") throw new Error("the Windows source does not contain the compatible 1–24 migration catalogue followed by teaching blocks and project-owned teaching");
  const result = { commit, dirty, schemaVersion: 26, migrations };
  const out = value("--out");
  if (!out) throw new Error("--out is required");
  writeFileSync(out, `${JSON.stringify(result, null, 2)}\n`);
  console.log(`Verified source ${commit}, schema 26, 26 migrations${dirty ? " (local changes present)" : ""}`);
} catch (error) {
  console.error(`build provenance: ${error.message}`);
  process.exitCode = 1;
}
