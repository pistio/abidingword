/** Keep a focused section's heading below the sticky step controls at any pane width. */
export function revealStudySection(sectionId: string) {
  const section = document.getElementById(`section-anchor-${sectionId}`);
  const heading = document.getElementById(`section-${sectionId}`);
  const controls = section?.closest("article")?.querySelector<HTMLElement>("[data-study-steps]");
  if (section) {
    section.style.scrollMarginTop = `${(controls?.getBoundingClientRect().height ?? 44) + 12}px`;
    section.scrollIntoView?.({ block: "start" });
  }
  heading?.focus({ preventScroll: true });
}
