import type { TFunction } from "i18next";
import type { FoundImage } from "@/types/teaching";

/** Preserve supplied licence wording; compose only credits the app generated itself. */
export function imageAttribution(t: TFunction, image: Pick<FoundImage, "attribution" | "attributionProvided" | "author" | "licence">): string {
  if (image.attributionProvided !== false) return image.attribution;
  return [image.author, image.licence, "Wikimedia Commons"].filter(Boolean).join(t("common.listSeparator"));
}
