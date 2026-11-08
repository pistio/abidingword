import { afterEach, expect, it } from "vitest";
import i18n from "@/i18n";
import { imageAttribution } from "@/features/teaching/imageAttribution";

afterEach(async () => { await i18n.changeLanguage("en"); });
it.each(["en", "de", "ko", "fr", "es", "pt"])("keeps exact supplied attribution in %s and composes only generated credits", async language => {
  await i18n.changeLanguage(language);
  const image = { attribution: "Author, licence, via Wikimedia Commons", attributionProvided: false, author: "via Wikimedia Commons — Jane 한글", licence: "CC BY-SA 4.0" };
  expect(imageAttribution(i18n.t, image)).toBe("via Wikimedia Commons — Jane 한글 · CC BY-SA 4.0 · Wikimedia Commons");
  const exact = { ...image, attributionProvided: true, attribution: "Exact licence credit via a private archive — 한글" };
  expect(imageAttribution(i18n.t, exact)).toBe(exact.attribution);
  const legacy = { attribution: exact.attribution, author: image.author, licence: image.licence };
  expect(imageAttribution(i18n.t, legacy)).toBe(exact.attribution);
});
