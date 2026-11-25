import type { Api } from "./api";
import {
  assertDatabaseReplacementAllowed, assertDatabaseWritesAllowed,
  observeDatabaseOperation,
} from "./databaseRestoreLock";

const guarded = new WeakMap<Api, Api>();
let apiIdentity = 0;
const replacement = new Set<keyof Api>(["backupRestore", "dailySnapshotRestore"]);

/**
 * Mutation names identify persistent save failures, not the write gate: ALL
 * API requests pass that gate because seemingly read-only queries can refresh
 * native indexes. Events/dialogs do not hold the database and need not drain.
 */
function failureKeys(method: string, args: unknown[], identity: number): string[] {
  if (!/(Update|Save|Set|Rename)/.test(method)) return [];
  if (/^(materialSave|presentationImageSave|libraryPdfStructureDiagnosticSave)/.test(method)) return [];
  const input = args[0];
  if (input && typeof input === "object" && !Array.isArray(input)) {
    const record = input as Record<string, unknown>;
    const entity = record.id ?? record.sourceId ?? record.preparationId ?? record.findingId ?? "settings";
    if (entity !== undefined && /(Update|Save|Set|Rename)/.test(method)) {
      return Object.keys(record).filter(key => key !== "id" && key !== "sourceId" && record[key] !== undefined).sort()
        .map(field => `${identity}:${method}:${String(entity)}:${field}`);
    }
  }
  // Scalar setters identify the target, never its new value (or a secret).
  return [`${identity}:${method}:${String(input)}`];
}

export function guardApiDuringDatabaseRestore(api: Api): Api {
  const existing = guarded.get(api);
  if (existing) return existing;
  const identity = ++apiIdentity;
  const functions = new Map<PropertyKey, { original: unknown; invoke: (...args: unknown[]) => Promise<unknown> }>();
  const proxy = new Proxy(api, {
    get(target, property, receiver) {
      const original = Reflect.get(target, property, receiver) as unknown;
      if (typeof original !== "function" || typeof property !== "string") return original;
      const cached = functions.get(property);
      if (cached?.original === original) return cached.invoke;
      const invoke = (...args: unknown[]): Promise<unknown> => {
        try {
          if (replacement.has(property as keyof Api)) {
            assertDatabaseReplacementAllowed();
            return Promise.resolve(Reflect.apply(original, target, args));
          }
          // Error reporting remains available when the restored workspace is locked.
          if (property === "logError") return Promise.resolve(Reflect.apply(original, target, args));
          assertDatabaseWritesAllowed();
          const result = Promise.resolve(Reflect.apply(original, target, args));
          if (/^(choose|on[A-Z]|openExternalLink)/.test(property)) return result;
          observeDatabaseOperation(result);
          for (const key of failureKeys(property, args, identity)) observeDatabaseOperation(result, key);
          return result;
        } catch (error) {
          return Promise.reject(error);
        }
      };
      functions.set(property, { original, invoke });
      return invoke;
    },
  });
  guarded.set(api, proxy);
  return proxy;
}
