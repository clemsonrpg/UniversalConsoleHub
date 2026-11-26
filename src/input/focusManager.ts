export type SpatialDirection = 'up' | 'down' | 'left' | 'right';

const FOCUSABLE_SELECTOR = [
  'button:not(:disabled)',
  'a[href]',
  'input:not(:disabled):not([type="hidden"])',
  'select:not(:disabled)',
  'textarea:not(:disabled)',
  '[tabindex]:not([tabindex="-1"])',
  '[data-focusable="true"]:not([aria-disabled="true"])',
].join(', ');

function isElementVisible(el: HTMLElement): boolean {
  if (el.hidden || el.getAttribute('aria-hidden') === 'true') return false;
  if (el.closest('[hidden], [aria-hidden="true"]')) return false;
  const style = window.getComputedStyle(el);
  if (style.display === 'none' || style.visibility === 'hidden') return false;
  return true;
}

export function getActiveModalRoot(): HTMLElement | null {
  const modals = Array.from(
    document.querySelectorAll<HTMLElement>('[data-focus-scope="modal"], [role="dialog"]')
  ).filter(isElementVisible);
  return modals.length > 0 ? modals[modals.length - 1] ?? null : null;
}

export function collectFocusables(root: ParentNode): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)).filter(
    (el) => isElementVisible(el) && !el.hasAttribute('disabled')
  );
}

export function setGamepadFocusVisual(target: HTMLElement | null): void {
  const prev = document.querySelectorAll<HTMLElement>('[data-gamepad-focused="true"]');
  for (const el of prev) {
    if (el !== target) {
      el.removeAttribute('data-gamepad-focused');
    }
  }
  if (target) {
    target.setAttribute('data-gamepad-focused', 'true');
  }
}

export function focusElementSafely(el: HTMLElement | null, markGamepad = true): boolean {
  if (!el || !document.contains(el) || !isElementVisible(el)) return false;
  el.focus({ preventScroll: true });
  if (markGamepad) {
    setGamepadFocusVisual(el);
  }
  if (typeof el.scrollIntoView === 'function') {
    el.scrollIntoView({ block: 'nearest', inline: 'nearest', behavior: 'smooth' });
  }
  return true;
}

function hasUsableLayoutRects(elements: HTMLElement[]): boolean {
  return elements.some((el) => {
    const r = el.getBoundingClientRect();
    return r.width > 0 || r.height > 0;
  });
}

function pickGeometricCandidate(
  current: HTMLElement,
  candidates: HTMLElement[],
  direction: SpatialDirection
): HTMLElement | null {
  const curRect = current.getBoundingClientRect();
  const curCx = curRect.left + curRect.width / 2;
  const curCy = curRect.top + curRect.height / 2;

  let best: HTMLElement | null = null;
  let bestScore = Number.POSITIVE_INFINITY;

  for (const cand of candidates) {
    if (cand === current) continue;
    const rect = cand.getBoundingClientRect();
    if (rect.width === 0 && rect.height === 0) continue;

    const cx = rect.left + rect.width / 2;
    const cy = rect.top + rect.height / 2;
    const dx = cx - curCx;
    const dy = cy - curCy;

    let primary = 0;
    let perp = 0;
    if (direction === 'right') {
      if (dx <= 4) continue;
      primary = dx;
      perp = Math.abs(dy);
    } else if (direction === 'left') {
      if (dx >= -4) continue;
      primary = -dx;
      perp = Math.abs(dy);
    } else if (direction === 'down') {
      if (dy <= 4) continue;
      primary = dy;
      perp = Math.abs(dx);
    } else {
      if (dy >= -4) continue;
      primary = -dy;
      perp = Math.abs(dx);
    }

    // Strongly favor aligned items along the axis while remaining reachable across uneven cards
    const score = primary + perp * 2.35;
    if (score < bestScore) {
      bestScore = score;
      best = cand;
    }
  }

  return best;
}

export class SpatialFocusManager {
  private readonly modalReturnStack: Array<{ element: HTMLElement | null; focusId: string | null }> =
    [];
  private readonly sectionLastFocusId = new Map<string, string>();

  rememberSectionFocus(sectionKey: string): void {
    const active = document.activeElement as HTMLElement | null;
    const id = active?.getAttribute('data-focus-id');
    if (id) {
      this.sectionLastFocusId.set(sectionKey, id);
    }
  }

  focusSectionEntry(sectionKey: string, markGamepad = false): boolean {
    const mainRoot =
      document.querySelector<HTMLElement>('[data-focus-scope="main"]') ?? document.body;
    const rememberedId = this.sectionLastFocusId.get(sectionKey);
    if (rememberedId) {
      const remembered = mainRoot.querySelector<HTMLElement>(
        `[data-focus-id="${CSS.escape(rememberedId)}"]`
      );
      if (focusElementSafely(remembered, markGamepad)) return true;
    }
    const preferred = mainRoot.querySelector<HTMLElement>('[data-focus-default="true"]');
    if (focusElementSafely(preferred, markGamepad)) return true;
    const first = collectFocusables(mainRoot)[0] ?? null;
    return focusElementSafely(first, markGamepad);
  }

  pushModalFocus(modalRoot: HTMLElement | null): void {
    const active = document.activeElement as HTMLElement | null;
    this.modalReturnStack.push({
      element: active,
      focusId: active?.getAttribute('data-focus-id') ?? null,
    });
    if (!modalRoot) return;
    const preferred = modalRoot.querySelector<HTMLElement>('[data-focus-default="true"]');
    if (focusElementSafely(preferred, true)) return;
    const first = collectFocusables(modalRoot)[0] ?? null;
    focusElementSafely(first, true);
  }

  popModalFocus(): void {
    const prev = this.modalReturnStack.pop();
    if (!prev) return;
    if (prev.element && focusElementSafely(prev.element, true)) return;
    if (prev.focusId) {
      const byId = document.querySelector<HTMLElement>(
        `[data-focus-id="${CSS.escape(prev.focusId)}"]`
      );
      if (focusElementSafely(byId, true)) return;
    }
    const mainRoot =
      document.querySelector<HTMLElement>('[data-focus-scope="main"]') ?? document.body;
    focusElementSafely(collectFocusables(mainRoot)[0] ?? null, true);
  }

  move(direction: SpatialDirection): HTMLElement | null {
    const modalRoot = getActiveModalRoot();
    const searchRoot = modalRoot ?? document.body;
    const allFocusables = collectFocusables(searchRoot);
    if (allFocusables.length === 0) return null;

    const current = document.activeElement as HTMLElement | null;
    if (!current || !searchRoot.contains(current) || !allFocusables.includes(current)) {
      const target =
        searchRoot.querySelector<HTMLElement>('[data-focus-default="true"]') ?? allFocusables[0]!;
      focusElementSafely(target, true);
      return target;
    }

    // Geometric 2D spatial navigation when browser layout rects exist
    if (hasUsableLayoutRects(allFocusables)) {
      const best = pickGeometricCandidate(current, allFocusables, direction);
      if (best) {
        focusElementSafely(best, true);
        return best;
      }
      // If inside modal and at boundary, wrap cleanly
      if (modalRoot) {
        const idx = allFocusables.indexOf(current);
        const delta = direction === 'down' || direction === 'right' ? 1 : -1;
        const next = allFocusables[(idx + delta + allFocusables.length) % allFocusables.length]!;
        focusElementSafely(next, true);
        return next;
      }
      return current;
    }

    // Deterministic fallback for headless/jsdom environments with zero-sized bounding boxes
    const idx = allFocusables.indexOf(current);
    const delta = direction === 'down' || direction === 'right' ? 1 : -1;
    const nextIdx = (idx + delta + allFocusables.length) % allFocusables.length;
    const next = allFocusables[nextIdx]!;
    focusElementSafely(next, true);
    return next;
  }

  jumpPage(deltaItems: number): HTMLElement | null {
    const modalRoot = getActiveModalRoot();
    const root =
      modalRoot ??
      document.querySelector<HTMLElement>('[data-focus-scope="main"]') ??
      document.body;
    const list = collectFocusables(root);
    if (list.length === 0) return null;
    const current = document.activeElement as HTMLElement | null;
    const idx = current ? list.indexOf(current) : 0;
    const nextIdx = Math.max(0, Math.min(list.length - 1, (idx < 0 ? 0 : idx) + deltaItems));
    const target = list[nextIdx]!;
    focusElementSafely(target, true);
    return target;
  }

  activateFocused(): boolean {
    const active = document.activeElement as HTMLElement | null;
    if (!active || active === document.body) {
      const mainRoot =
        document.querySelector<HTMLElement>('[data-focus-scope="main"]') ?? document.body;
      const fallback = collectFocusables(mainRoot)[0] ?? null;
      if (fallback) {
        focusElementSafely(fallback, true);
        fallback.click();
        return true;
      }
      return false;
    }
    active.setAttribute('data-gamepad-pressed', 'true');
    window.setTimeout(() => active.removeAttribute('data-gamepad-pressed'), 120);
    if (active instanceof HTMLInputElement && active.type === 'checkbox') {
      active.click();
      return true;
    }
    if (active instanceof HTMLSelectElement) {
      const next = (active.selectedIndex + 1) % Math.max(1, active.options.length);
      active.selectedIndex = next;
      active.dispatchEvent(new Event('change', { bubbles: true }));
      return true;
    }
    active.click();
    return true;
  }
}

export const globalFocusManager = new SpatialFocusManager();
