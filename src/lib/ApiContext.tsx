import { createContext, useContext, useMemo, type ReactNode } from "react";
import type { Api } from "./api";
import { guardApiDuringDatabaseRestore } from "./guardedApi";

const ApiContext = createContext<Api | null>(null);

export function ApiProvider({ api, children }: { api: Api; children: ReactNode }) {
  const protectedApi = useMemo(() => guardApiDuringDatabaseRestore(api), [api]);
  return <ApiContext.Provider value={protectedApi}>{children}</ApiContext.Provider>;
}

export function useApi(): Api {
  const api = useContext(ApiContext);
  if (!api) throw new Error("useApi must be used inside <ApiProvider>");
  return api;
}
