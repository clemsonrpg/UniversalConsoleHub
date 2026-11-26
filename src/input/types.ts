import type { GamepadGlyphFamily, GamepadNavAction } from '../types/domain';

export type StandardButtonSlot =
  | 'south'
  | 'east'
  | 'west'
  | 'north'
  | 'lb'
  | 'rb'
  | 'lt'
  | 'rt'
  | 'back'
  | 'start';

export interface GamepadConfig {
  glyphOverride: 'auto' | GamepadGlyphFamily;
  deadzone: number;
  repeatDelayMs: number;
  repeatIntervalMs: number;
  enableBrowserFallback: boolean;
  buttonMap: Record<StandardButtonSlot, GamepadNavAction>;
}

export const DEFAULT_GAMEPAD_CONFIG: GamepadConfig = {
  glyphOverride: 'auto',
  deadzone: 0.24,
  repeatDelayMs: 380,
  repeatIntervalMs: 115,
  enableBrowserFallback: true,
  buttonMap: {
    south: 'confirm',
    east: 'back',
    west: 'secondary',
    north: 'context_menu',
    lb: 'prev_tab',
    rb: 'next_tab',
    lt: 'page_prev',
    rt: 'page_next',
    back: 'toggle_mode',
    start: 'quick_menu',
  },
};

export const STANDARD_BUTTON_INDEX: Record<StandardButtonSlot, number> = {
  south: 0,
  east: 1,
  west: 2,
  north: 3,
  lb: 4,
  rb: 5,
  lt: 6,
  rt: 7,
  back: 8,
  start: 9,
};

export const ACTION_LABELS: Record<GamepadNavAction, string> = {
  up: 'Navegar acima',
  down: 'Navegar abaixo',
  left: 'Navegar esquerda',
  right: 'Navegar direita',
  confirm: 'Confirmar',
  back: 'Voltar / Cancelar',
  secondary: 'Ação secundária',
  context_menu: 'Menu de contexto',
  prev_tab: 'Aba / Categoria anterior',
  next_tab: 'Próxima aba / categoria',
  page_prev: 'Página anterior',
  page_next: 'Próxima página',
  quick_menu: 'Menu rápido do Hub',
  toggle_mode: 'Alternar Desktop / Fullscreen',
};

export function parseGamepadConfig(raw: string | undefined): GamepadConfig {
  if (!raw) return DEFAULT_GAMEPAD_CONFIG;
  try {
    const parsed = JSON.parse(raw) as Partial<GamepadConfig>;
    return {
      glyphOverride: parsed.glyphOverride ?? DEFAULT_GAMEPAD_CONFIG.glyphOverride,
      deadzone:
        typeof parsed.deadzone === 'number'
          ? Math.max(0.1, Math.min(0.55, parsed.deadzone))
          : DEFAULT_GAMEPAD_CONFIG.deadzone,
      repeatDelayMs:
        typeof parsed.repeatDelayMs === 'number'
          ? Math.max(180, Math.min(700, parsed.repeatDelayMs))
          : DEFAULT_GAMEPAD_CONFIG.repeatDelayMs,
      repeatIntervalMs:
        typeof parsed.repeatIntervalMs === 'number'
          ? Math.max(60, Math.min(300, parsed.repeatIntervalMs))
          : DEFAULT_GAMEPAD_CONFIG.repeatIntervalMs,
      enableBrowserFallback:
        typeof parsed.enableBrowserFallback === 'boolean'
          ? parsed.enableBrowserFallback
          : DEFAULT_GAMEPAD_CONFIG.enableBrowserFallback,
      buttonMap: {
        ...DEFAULT_GAMEPAD_CONFIG.buttonMap,
        ...(parsed.buttonMap ?? {}),
      },
    };
  } catch {
    return DEFAULT_GAMEPAD_CONFIG;
  }
}

export function detectGlyphFamilyFromName(rawName: string): GamepadGlyphFamily {
  const lower = rawName.toLowerCase();
  if (
    lower.includes('045e') ||
    lower.includes('xbox') ||
    lower.includes('xinput') ||
    lower.includes('microsoft')
  ) {
    return 'xbox';
  }
  if (
    lower.includes('054c') ||
    lower.includes('dualsense') ||
    lower.includes('dualshock') ||
    lower.includes('playstation') ||
    lower.includes('ps4') ||
    lower.includes('ps5') ||
    lower.includes('wireless controller')
  ) {
    return 'playstation';
  }
  if (
    lower.includes('057e') ||
    lower.includes('nintendo') ||
    lower.includes('switch') ||
    lower.includes('joy-con') ||
    lower.includes('pro controller')
  ) {
    return 'nintendo';
  }
  return 'generic';
}

export function getButtonGlyph(
  action: GamepadNavAction,
  family: GamepadGlyphFamily | 'keyboard',
  buttonMap: Record<StandardButtonSlot, GamepadNavAction> = DEFAULT_GAMEPAD_CONFIG.buttonMap
): string {
  if (family === 'keyboard') {
    const kb: Record<GamepadNavAction, string> = {
      up: '↑',
      down: '↓',
      left: '←',
      right: '→',
      confirm: 'Enter',
      back: 'Esc',
      secondary: 'F',
      context_menu: 'M',
      prev_tab: 'Alt+←',
      next_tab: 'Alt+→',
      page_prev: 'PgUp',
      page_next: 'PgDn',
      quick_menu: 'F2',
      toggle_mode: 'F11',
    };
    return kb[action];
  }

  const slot = (Object.keys(buttonMap) as StandardButtonSlot[]).find(
    (s) => buttonMap[s] === action
  );
  if (!slot) {
    return action === 'up' || action === 'down' || action === 'left' || action === 'right'
      ? 'D-Pad'
      : '•';
  }

  const table: Record<GamepadGlyphFamily, Record<StandardButtonSlot, string>> = {
    xbox: {
      south: 'A',
      east: 'B',
      west: 'X',
      north: 'Y',
      lb: 'LB',
      rb: 'RB',
      lt: 'LT',
      rt: 'RT',
      back: 'View',
      start: 'Menu',
    },
    playstation: {
      south: '✕',
      east: '○',
      west: '□',
      north: '△',
      lb: 'L1',
      rb: 'R1',
      lt: 'L2',
      rt: 'R2',
      back: 'Create',
      start: 'Options',
    },
    nintendo: {
      south: 'B',
      east: 'A',
      west: 'Y',
      north: 'X',
      lb: 'L',
      rb: 'R',
      lt: 'ZL',
      rt: 'ZR',
      back: '−',
      start: '+',
    },
    generic: {
      south: '1',
      east: '2',
      west: '3',
      north: '4',
      lb: 'L1',
      rb: 'R1',
      lt: 'L2',
      rt: 'R2',
      back: 'Sel',
      start: 'Start',
    },
  };
  return table[family][slot];
}
