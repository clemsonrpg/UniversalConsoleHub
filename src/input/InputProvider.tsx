import { getCurrentWindow } from '@tauri-apps/api/window';
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from 'react';
import { isTauriDesktopRuntime } from '../api/tauriClient';
import type {
  ConnectedGamepadInfo,
  DisplayMode,
  GamepadActionEvent,
  GamepadGlyphFamily,
  GamepadNavAction,
} from '../types/domain';
import { globalFocusManager, setGamepadFocusVisual } from './focusManager';
import { BrowserGamepadPoller } from './gamepadEngine';
import {
  DEFAULT_GAMEPAD_CONFIG,
  getButtonGlyph,
  parseGamepadConfig,
  type GamepadConfig,
} from './types';

export interface InputContextValue {
  displayMode: DisplayMode;
  setDisplayMode: (mode: DisplayMode) => Promise<void>;
  toggleDisplayMode: () => Promise<void>;
  gamepadConfig: GamepadConfig;
  updateGamepadConfig: (patch: Partial<GamepadConfig>) => Promise<void>;
  resetGamepadConfig: () => Promise<void>;
  connectedGamepads: ConnectedGamepadInfo[];
  inputSource: 'gamepad' | 'keyboard' | 'mouse';
  activeGlyphFamily: GamepadGlyphFamily | 'keyboard';
  lastAction: GamepadActionEvent | null;
  quickMenuOpen: boolean;
  setQuickMenuOpen: (open: boolean) => void;
  glyphFor: (action: GamepadNavAction) => string;
  dispatchAction: (action: GamepadNavAction, source?: 'gamepad' | 'keyboard') => void;
}

const InputContext = createContext<InputContextValue | null>(null);

export interface InputProviderProps {
  settings: Record<string, string>;
  suspended: boolean;
  nativeGamepads: ConnectedGamepadInfo[];
  lastNativeAction: GamepadActionEvent | null;
  onUpdateSetting: (key: string, value: string) => Promise<void>;
  onNavigateBack: () => boolean;
  onCycleSection: (delta: number) => void;
  children: ReactNode;
}

function isEditableTarget(el: EventTarget | null): boolean {
  if (!(el instanceof HTMLElement)) return false;
  if (el.isContentEditable) return true;
  if (el.tagName === 'TEXTAREA') return true;
  if (el.tagName === 'INPUT') {
    const type = (el as HTMLInputElement).type;
    return !['button', 'submit', 'reset', 'checkbox', 'radio', 'range'].includes(type);
  }
  return false;
}

export function InputProvider({
  settings,
  suspended,
  nativeGamepads,
  lastNativeAction,
  onUpdateSetting,
  onNavigateBack,
  onCycleSection,
  children,
}: InputProviderProps) {
  const persistedMode: DisplayMode =
    settings['ui.display_mode'] === 'fullscreen' ? 'fullscreen' : 'desktop';
  const [displayMode, setDisplayModeState] = useState<DisplayMode>(persistedMode);
  const [browserGamepads, setBrowserGamepads] = useState<ConnectedGamepadInfo[]>([]);
  const [inputSource, setInputSource] = useState<'gamepad' | 'keyboard' | 'mouse'>('keyboard');
  const [lastAction, setLastAction] = useState<GamepadActionEvent | null>(null);
  const [quickMenuOpen, setQuickMenuOpen] = useState(false);

  const gamepadConfig = useMemo(
    () => parseGamepadConfig(settings['gamepad.config_json']),
    [settings]
  );
  const configRef = useRef(gamepadConfig);
  configRef.current = gamepadConfig;

  const lastActionDedupRef = useRef<{ action: GamepadNavAction; at: number } | null>(null);

  useEffect(() => {
    setDisplayModeState(persistedMode);
  }, [persistedMode]);

  const applyWindowFullscreen = useCallback(async (mode: DisplayMode) => {
    if (!isTauriDesktopRuntime()) return;
    try {
      await getCurrentWindow().setFullscreen(mode === 'fullscreen');
    } catch {
      // Ignore if window fullscreen is unavailable in current test host
    }
  }, []);

  const setDisplayMode = useCallback(
    async (mode: DisplayMode) => {
      setDisplayModeState(mode);
      await applyWindowFullscreen(mode);
      await onUpdateSetting('ui.display_mode', mode);
    },
    [applyWindowFullscreen, onUpdateSetting]
  );

  const toggleDisplayMode = useCallback(async () => {
    const next: DisplayMode = displayMode === 'fullscreen' ? 'desktop' : 'fullscreen';
    await setDisplayMode(next);
  }, [displayMode, setDisplayMode]);

  const updateGamepadConfig = useCallback(
    async (patch: Partial<GamepadConfig>) => {
      const next: GamepadConfig = {
        ...configRef.current,
        ...patch,
        buttonMap: {
          ...configRef.current.buttonMap,
          ...(patch.buttonMap ?? {}),
        },
      };
      await onUpdateSetting('gamepad.config_json', JSON.stringify(next));
    },
    [onUpdateSetting]
  );

  const resetGamepadConfig = useCallback(async () => {
    await onUpdateSetting('gamepad.config_json', JSON.stringify(DEFAULT_GAMEPAD_CONFIG));
  }, [onUpdateSetting]);

  const connectedGamepads = useMemo(() => {
    const map = new Map<string, ConnectedGamepadInfo>();
    for (const dev of nativeGamepads) {
      map.set(`native-${dev.slot}`, {
        ...dev,
        glyphFamily:
          gamepadConfig.glyphOverride === 'auto' ? dev.glyphFamily : gamepadConfig.glyphOverride,
      });
    }
    for (const dev of browserGamepads) {
      if (!nativeGamepads.some((n) => n.slot === dev.slot)) {
        map.set(`browser-${dev.slot}`, dev);
      }
    }
    return Array.from(map.values());
  }, [nativeGamepads, browserGamepads, gamepadConfig.glyphOverride]);

  const activeGlyphFamily: GamepadGlyphFamily | 'keyboard' = useMemo(() => {
    if (gamepadConfig.glyphOverride !== 'auto' && inputSource === 'gamepad') {
      return gamepadConfig.glyphOverride;
    }
    if (inputSource === 'gamepad' && connectedGamepads[0]) {
      return connectedGamepads[0].glyphFamily;
    }
    if (connectedGamepads[0]) {
      return connectedGamepads[0].glyphFamily;
    }
    return 'keyboard';
  }, [gamepadConfig.glyphOverride, inputSource, connectedGamepads]);

  const cyclePageTabsOrSections = useCallback(
    (delta: number) => {
      const tabs = Array.from(
        document.querySelectorAll<HTMLElement>('[data-focus-scope="main"] [role="tab"]')
      ).filter((el) => !el.hidden);
      if (tabs.length > 1) {
        const selectedIdx = tabs.findIndex((t) => t.getAttribute('aria-selected') === 'true');
        const nextIdx = (selectedIdx + delta + tabs.length) % tabs.length;
        const target = tabs[nextIdx];
        if (target) {
          target.click();
          target.focus({ preventScroll: true });
          setGamepadFocusVisual(target);
          return;
        }
      }
      onCycleSection(delta);
    },
    [onCycleSection]
  );

  const dispatchAction = useCallback(
    (action: GamepadNavAction, source: 'gamepad' | 'keyboard' = 'gamepad') => {
      if (suspended) return;

      const now = performance.now();
      const prev = lastActionDedupRef.current;
      if (source === 'gamepad' && prev && prev.action === action && now - prev.at < 55) {
        return;
      }
      lastActionDedupRef.current = { action, at: now };

      setInputSource(source);
      setLastAction({
        slot: 0,
        action,
        glyphFamily: activeGlyphFamily === 'keyboard' ? 'xbox' : activeGlyphFamily,
        repeated: false,
      });

      switch (action) {
        case 'up':
        case 'down':
        case 'left':
        case 'right':
          globalFocusManager.move(action);
          break;
        case 'confirm':
          globalFocusManager.activateFocused();
          break;
        case 'back': {
          if (quickMenuOpen) {
            setQuickMenuOpen(false);
            break;
          }
          const cancelBtn = document.querySelector<HTMLElement>(
            '[role="dialog"] [data-dialog-cancel="true"]'
          );
          if (cancelBtn) {
            cancelBtn.click();
            break;
          }
          if (!onNavigateBack()) {
            const navActive = document.querySelector<HTMLElement>(
              '[data-focus-scope="nav"] [aria-current="page"], [data-focus-scope="nav"] button'
            );
            if (navActive) {
              navActive.focus({ preventScroll: true });
              setGamepadFocusVisual(navActive);
            }
          }
          break;
        }
        case 'secondary': {
          const active = document.activeElement as HTMLElement | null;
          const card = active?.closest<HTMLElement>('[data-focus-card="true"]');
          const secBtn =
            card?.querySelector<HTMLElement>('[data-gamepad-secondary="true"]') ??
            document.querySelector<HTMLElement>(
              '[data-focus-scope="main"] [data-gamepad-secondary="true"]'
            );
          if (secBtn) secBtn.click();
          break;
        }
        case 'context_menu':
        case 'quick_menu':
          setQuickMenuOpen((prevOpen) => !prevOpen);
          break;
        case 'prev_tab':
          cyclePageTabsOrSections(-1);
          break;
        case 'next_tab':
          cyclePageTabsOrSections(1);
          break;
        case 'page_prev':
          globalFocusManager.jumpPage(-5);
          break;
        case 'page_next':
          globalFocusManager.jumpPage(5);
          break;
        case 'toggle_mode':
          void toggleDisplayMode();
          break;
      }
    },
    [
      suspended,
      activeGlyphFamily,
      quickMenuOpen,
      onNavigateBack,
      cyclePageTabsOrSections,
      toggleDisplayMode,
    ]
  );

  // Forward native Rust XInput events into unified dispatcher
  useEffect(() => {
    if (!lastNativeAction) return;
    dispatchAction(lastNativeAction.action, 'gamepad');
  }, [lastNativeAction, dispatchAction]);

  // Browser W3C Gamepad API poller (DualSense/DirectInput/Switch Pro & automated tests)
  const pollerRef = useRef<BrowserGamepadPoller | null>(null);
  useEffect(() => {
    const poller = new BrowserGamepadPoller(() => configRef.current, {
      onDevicesChanged: setBrowserGamepads,
      onAction: (evt) => dispatchAction(evt.action, 'gamepad'),
    });
    poller.setSuspended(suspended);
    pollerRef.current = poller;
    return poller.start();
  }, [dispatchAction, suspended]);

  useEffect(() => {
    pollerRef.current?.setSuspended(suspended);
  }, [suspended]);

  // Track mouse/keyboard activity & global keyboard controller-equivalent shortcuts
  useEffect(() => {
    const onPointerDown = () => {
      setInputSource('mouse');
      setGamepadFocusVisual(null);
    };

    const onKeyDown = (event: KeyboardEvent) => {
      if (suspended) return;
      setInputSource('keyboard');

      if (event.key === 'F11') {
        event.preventDefault();
        void toggleDisplayMode();
        return;
      }
      if (event.key === 'F2') {
        event.preventDefault();
        setQuickMenuOpen((prev) => !prev);
        return;
      }

      if (isEditableTarget(event.target)) {
        if (event.key === 'Escape') {
          (event.target as HTMLElement).blur();
        }
        return;
      }

      // Preserve Alt+Arrow section switching from Phase 1-4
      if (event.altKey) {
        if (event.key === 'ArrowDown' || event.key === 'ArrowRight') {
          event.preventDefault();
          onCycleSection(1);
        } else if (event.key === 'ArrowUp' || event.key === 'ArrowLeft') {
          event.preventDefault();
          onCycleSection(-1);
        }
        return;
      }

      if (event.key === 'Escape') {
        if (quickMenuOpen) {
          event.preventDefault();
          setQuickMenuOpen(false);
        } else if (!document.querySelector('[role="dialog"]')) {
          if (onNavigateBack()) {
            event.preventDefault();
          }
        }
        return;
      }

      if (event.key === 'ArrowUp') {
        event.preventDefault();
        globalFocusManager.move('up');
      } else if (event.key === 'ArrowDown') {
        event.preventDefault();
        globalFocusManager.move('down');
      } else if (event.key === 'ArrowLeft') {
        event.preventDefault();
        globalFocusManager.move('left');
      } else if (event.key === 'ArrowRight') {
        event.preventDefault();
        globalFocusManager.move('right');
      } else if (event.key === 'PageUp') {
        event.preventDefault();
        globalFocusManager.jumpPage(-5);
      } else if (event.key === 'PageDown') {
        event.preventDefault();
        globalFocusManager.jumpPage(5);
      }
    };

    window.addEventListener('pointerdown', onPointerDown, { passive: true });
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('pointerdown', onPointerDown);
      window.removeEventListener('keydown', onKeyDown);
    };
  }, [suspended, quickMenuOpen, toggleDisplayMode, onCycleSection, onNavigateBack]);

  const glyphFor = useCallback(
    (action: GamepadNavAction) =>
      getButtonGlyph(action, activeGlyphFamily, gamepadConfig.buttonMap),
    [activeGlyphFamily, gamepadConfig.buttonMap]
  );

  const value = useMemo<InputContextValue>(
    () => ({
      displayMode,
      setDisplayMode,
      toggleDisplayMode,
      gamepadConfig,
      updateGamepadConfig,
      resetGamepadConfig,
      connectedGamepads,
      inputSource,
      activeGlyphFamily,
      lastAction,
      quickMenuOpen,
      setQuickMenuOpen,
      glyphFor,
      dispatchAction,
    }),
    [
      displayMode,
      setDisplayMode,
      toggleDisplayMode,
      gamepadConfig,
      updateGamepadConfig,
      resetGamepadConfig,
      connectedGamepads,
      inputSource,
      activeGlyphFamily,
      lastAction,
      quickMenuOpen,
      glyphFor,
      dispatchAction,
    ]
  );

  return <InputContext.Provider value={value}>{children}</InputContext.Provider>;
}

export function useOptionalInput(): InputContextValue | null {
  return useContext(InputContext);
}

export function useInput(): InputContextValue {
  const ctx = useContext(InputContext);
  if (!ctx) {
    throw new Error('useInput must be used within an InputProvider');
  }
  return ctx;
}
