import type { TFunction } from "i18next";
import { writeLocal } from "@/lib/storage";
import type { Api } from "@/lib/api";

const LAST_PROJECT_KEY = "studyPane.lastProject";

/** Remembers where the user last worked (a per-device convenience, never study data). */
export function rememberProject(projectId: string): void {
  writeLocal(LAST_PROJECT_KEY, projectId);
}

/** Root-level creation always uses the stable implicit container, never the last named folder. */
export async function ensureStudyProject(api: Api, workspaceId: string, t: TFunction): Promise<string> {
  return (await api.projectRoot(workspaceId, t("studies.projects.defaultTitle"))).id;
}
