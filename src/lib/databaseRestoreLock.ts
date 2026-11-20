/** Blocks stale writes after a database-replacement operation begins.
 * Scoped to the running webview; restored data is reloaded before unlocking.
 */
let restoring = false;

export function beginDatabaseRestore() {
  restoring = true;
  document.getElementById("root")?.setAttribute("inert", "");
}

export function endDatabaseRestore() {
  restoring = false;
  document.getElementById("root")?.removeAttribute("inert");
}

export function databaseRestoreActive() {
  return restoring;
}
