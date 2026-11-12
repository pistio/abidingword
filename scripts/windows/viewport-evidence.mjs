// Geometry policy only. Synthetic unit/browser results are never Windows acceptance.
export function domSnapshot() {
  const rect = node => {
    if (!node) return null;
    const r = node.getBoundingClientRect();
    return { x: r.x, y: r.y, width: r.width, height: r.height, bottom: r.bottom, right: r.right };
  };
  const panes = Object.fromEntries(['scripture', 'study', 'sources'].map(name => {
    const node = document.querySelector(`#pane-${name}`);
    const scrollRoots = node ? [...node.querySelectorAll('*')].filter(el => /^(auto|scroll)$/.test(getComputedStyle(el).overflowY) && el.clientHeight > 0).map(el => ({ rect: rect(el), clientHeight: el.clientHeight, scrollHeight: el.scrollHeight, scrollTop: el.scrollTop })) : [];
    return [name, { rect: rect(node), scrollRoots }];
  }));
  const active = document.activeElement;
  return {
    innerWidth, innerHeight, devicePixelRatio,
    documentHeight: document.documentElement.clientHeight,
    visualViewport: window.visualViewport ? { width: visualViewport.width, height: visualViewport.height, offsetLeft: visualViewport.offsetLeft, offsetTop: visualViewport.offsetTop, scale: visualViewport.scale } : null,
    shell: rect(document.querySelector('[data-testid="app-shell"]')),
    workspace: rect(document.querySelector('[data-workspace-layout]')),
    panes,
    focus: { tag: active?.tagName, selectionStart: active?.selectionStart ?? null, selectionEnd: active?.selectionEnd ?? null }
  };
}

export function compareGeometry(native, sample, dom, tolerance = 4) {
  const errors = [];
  const equal = (a, b) => Number.isFinite(a) && Number.isFinite(b) && Math.abs(a - b) <= tolerance;
  const main = native.Main;
  const controller = sample?.controller;
  if (!main || !controller?.rect || !sample.stableClient) return { passed: false, diagnosis: 'missing-or-unstable-native-measurement', errors: ['Require independent Win32 client, stable Tauri sample and actual COM controller bounds.'] };
  const [left, top, right, bottom] = controller.rect;
  const host = [main, ...(native.Descendants ?? [])].find(r => r.Hwnd === controller.parentHwnd);
  if (!host) errors.push('Controller parent HWND not present in independent Win32 capture.');
  const controllerMatches = equal(right - left, main.ClientWidth) && equal(bottom - top, main.ClientHeight);
  if (!controllerMatches) errors.push('Actual controller size differs from native client.');
  if (!equal(sample.client[0], main.ClientWidth) || !equal(sample.client[1], main.ClientHeight)) errors.push('Tauri sample and fresh Win32 client differ (possibly stale sample).');
  if (!equal(sample.webview[0], right - left) || !equal(sample.webview[1], bottom - top)) errors.push('Tauri size and actual controller size differ.');
  const viewportMatches = equal(dom.innerWidth * dom.devicePixelRatio, main.ClientWidth) && equal(dom.innerHeight * dom.devicePixelRatio, main.ClientHeight);
  if (!viewportMatches) errors.push('Browser CSS viewport × DPR differs from native physical client.');
  if (Math.abs(dom.devicePixelRatio - main.Dpi / 96) > 0.02) errors.push('DPR differs from actual Windows DPI; do not substitute emulated DPI/zoom evidence.');
  if (host && (!equal(host.ClientScreenX + left, main.ClientScreenX) || !equal(host.ClientScreenY + top, main.ClientScreenY))) errors.push('Controller host/origin is offset from native client.');
  if (host && (left < -tolerance || top < -tolerance || right > host.ClientWidth + tolerance || bottom > host.ClientHeight + tolerance)) errors.push('Controller extends outside its native host client (child HWND clipping).');
  const originY = (host?.ClientScreenY ?? main.ClientScreenY) + top;
  const expectedBottom = main.ClientScreenY + main.ClientHeight;
  for (const [name, rect] of Object.entries({ shell: dom.shell, workspace: dom.workspace, ...Object.fromEntries(Object.entries(dom.panes).map(([k, v]) => [k, v.rect])) })) {
    if (!rect || !equal(originY + rect.bottom * dom.devicePixelRatio, expectedBottom)) errors.push(`${name} bottom does not reach actual native client bottom.`);
  }
  const diagnosis = !controllerMatches ? 'native-controller-mismatch' : !viewportMatches ? 'controller-matches-investigate-browser-viewport' : errors.length ? 'investigate-DOM-or-host-offset' : 'geometry-matches-repaint-still-requires-native-screenshot-review';
  return { passed: errors.length === 0, diagnosis, errors, nearInitial800Logical: Math.abs(sample.webview[1] / (sample.scale || 1) - 800) <= 4 };
}
