import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it } from "vitest";

it("attests the exact local source and complete compatible migration catalogue", () => {
  const dir = mkdtempSync(join(tmpdir(), "aw-build-proof-"));
  try {
    const commit = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
    const out = join(dir, "provenance.json");
    const run = spawnSync(process.execPath, ["scripts/build-provenance.mjs", "--expected-sha", commit, "--out", out], { encoding: "utf8" });
    expect(run.status, run.stderr).toBe(0);
    const proof = JSON.parse(readFileSync(out, "utf8"));
    expect(proof.commit).toBe(commit);
    expect(proof.schemaVersion).toBe(24);
    expect(proof.migrations.map((m: { version: number }) => m.version)).toEqual(Array.from({ length: 24 }, (_, i) => i + 1));
    expect(proof.migrations[9].file).toBe("0010_presentations.sql");
    expect(proof.migrations[23].file).toBe("0024_section_status_and_fields.sql");
    expect(proof.migrations.every((m: { sha256: string }) => /^[a-f0-9]{64}$/.test(m.sha256))).toBe(true);
    const wrong = spawnSync(process.execPath, ["scripts/build-provenance.mjs", "--expected-sha", "0".repeat(40), "--out", out], { encoding: "utf8" });
    expect(wrong.status).toBe(1);
    expect(wrong.stderr).toContain("does not match");
    const publish = readFileSync(".github/workflows/publish-windows-test-update.yml", "utf8");
    expect(publish).toContain("ref: ${{ needs.gate.outputs.sha }}");
    expect(publish).toContain('node scripts/build-provenance.mjs --expected-sha "$env:EXPECTED_SOURCE_SHA" --require-clean');
  } finally { rmSync(dir, { recursive: true, force: true }); }
});
