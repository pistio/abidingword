import { expect, it } from "vitest";
// @ts-expect-error -- test-only Node measurement helper, not application code
import { compareGeometry } from "../../scripts/windows/viewport-evidence.mjs";

function capture(dpi = 96) {
  const scale = dpi / 96;
  const rect = { bottom: 1000 / scale };
  return {
    native: { Main: { Hwnd: "1", Dpi: dpi, ClientWidth: 1600, ClientHeight: 1000, ClientScreenX: 10, ClientScreenY: 30 }, Descendants: [{ Hwnd: "2", ClientWidth: 1600, ClientHeight: 1000, ClientScreenX: 10, ClientScreenY: 30 }] },
    sample: { stableClient: true, client: [1600, 1000], webview: [1600, 1000], controller: { rect: [0, 0, 1600, 1000], parentHwnd: "2" }, scale },
    dom: { innerWidth: 1600 / scale, innerHeight: 1000 / scale, devicePixelRatio: scale, shell: rect, workspace: rect, panes: { scripture: { rect }, study: { rect }, sources: { rect } } },
  };
}
it.each([96, 120, 144, 192])("converts actual %i DPI and independent client origin without inventing native evidence", dpi => {
  const c = capture(dpi);
  expect(compareGeometry(c.native, c.sample, c.dom)).toMatchObject({ passed: true, diagnosis: "geometry-matches-repaint-still-requires-native-screenshot-review" });
});
it("rejects the initial-800-height controller even when its own DOM fills perfectly", () => {
  const c = capture();
  c.sample.webview[1] = 800; c.sample.controller.rect[3] = 800; c.dom.innerHeight = 800;
  Object.assign(c.dom, { shell: { bottom: 800 }, workspace: { bottom: 800 } });
  expect(compareGeometry(c.native, c.sample, c.dom)).toMatchObject({ passed: false, diagnosis: "native-controller-mismatch", nearInitial800Logical: true });
});
it("routes a matching native/controller size with short browser viewport to the browser fault path", () => {
  const c = capture(); c.dom.innerHeight = 800;
  expect(compareGeometry(c.native, c.sample, c.dom)).toMatchObject({ passed: false, diagnosis: "controller-matches-investigate-browser-viewport" });
});
it("rejects a clipped pane and a displaced controller host even if all reported sizes match", () => {
  const c = capture(); c.dom.panes.study.rect = { bottom: 807 };
  expect(compareGeometry(c.native, c.sample, c.dom).passed).toBe(false);
  c.dom.panes.study.rect = c.dom.shell; c.native.Descendants[0]!.ClientHeight = 800;
  expect(compareGeometry(c.native, c.sample, c.dom).errors).toContain("Controller extends outside its native host client (child HWND clipping).");
  c.native.Descendants[0]!.ClientHeight = 1000; c.native.Descendants[0]!.ClientScreenY = 223;
  expect(compareGeometry(c.native, c.sample, c.dom).errors).toContain("Controller host/origin is offset from native client.");
});
it("does not treat a deliberately stacked narrow workspace as three-pane Windows acceptance", () => {
  const c = capture();
  (c.dom as typeof c.dom & { layoutMode: string }).layoutMode = "stacked";
  expect(compareGeometry(c.native, c.sample, c.dom)).toMatchObject({ passed: false });
  expect(compareGeometry(c.native, c.sample, c.dom).errors).toContain(
    "Stacked/narrow workspace: wide three-pane native geometry was not exercised.",
  );
});

it("fails closed without a stable COM measurement or with emulated/nonmatching DPI", () => {
  const c = capture();
  expect(compareGeometry(c.native, null, c.dom).passed).toBe(false);
  c.dom.devicePixelRatio = 1.5;
  expect(compareGeometry(c.native, c.sample, c.dom).passed).toBe(false);
});
