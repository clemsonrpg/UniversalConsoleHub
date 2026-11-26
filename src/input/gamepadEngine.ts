import type { ConnectedGamepadInfo, GamepadActionEvent, GamepadNavAction } from '../types/domain';
import {
  detectGlyphFamilyFromName,
  STANDARD_BUTTON_INDEX,
  type GamepadConfig,
  type StandardButtonSlot,
} from './types';

export function applyScaledRadialDeadzone(
  x: number,
  y: number,
  deadzone: number
): [number, number] {
  const dz = Math.max(0.05, Math.min(0.75, deadzone));
  const mag = Math.hypot(x, y);
  if (mag < dz || mag === 0) return [0, 0];
  const scaled = Math.max(0, Math.min(1, (mag - dz) / (1 - dz)));
  return [(x / mag) * scaled, (y / mag) * scaled];
}

export class ActionRepeatGate {
  private readonly states = new Map<GamepadNavAction, { firstAt: number; lastFiredAt: number }>();

  update(
    action: GamepadNavAction,
    pressed: boolean,
    repeatable: boolean,
    nowMs: number,
    delayMs: number,
    intervalMs: number
  ): { fire: boolean; repeated: boolean } {
    if (!pressed) {
      this.states.delete(action);
      return { fire: false, repeated: false };
    }
    const existing = this.states.get(action);
    if (!existing) {
      this.states.set(action, { firstAt: nowMs, lastFiredAt: nowMs });
      return { fire: true, repeated: false };
    }
    if (!repeatable) {
      return { fire: false, repeated: false };
    }
    if (nowMs - existing.firstAt >= delayMs && nowMs - existing.lastFiredAt >= intervalMs) {
      existing.lastFiredAt = nowMs;
      return { fire: true, repeated: true };
    }
    return { fire: false, repeated: false };
  }

  clear(): void {
    this.states.clear();
  }
}

export interface BrowserGamepadPollerCallbacks {
  onDevicesChanged: (devices: ConnectedGamepadInfo[]) => void;
  onAction: (event: GamepadActionEvent) => void;
}

const REPEATABLE_ACTIONS = new Set<GamepadNavAction>([
  'up',
  'down',
  'left',
  'right',
  'prev_tab',
  'next_tab',
  'page_prev',
  'page_next',
]);

/**
 * Unified W3C Gamepad API poller (supports DirectInput/DualSense/DualShock/Switch Pro in WebView2
 * plus deterministic automated testing in Vitest/Playwright). Deduplicates cleanly with native XInput.
 */
export class BrowserGamepadPoller {
  private readonly gates = new Map<number, ActionRepeatGate>();
  private lastDeviceSignature = '';
  private rafId: number | null = null;
  private suspended = false;
  private drainUntilMs = 0;

  constructor(
    private getConfig: () => GamepadConfig,
    private readonly callbacks: BrowserGamepadPollerCallbacks
  ) {}

  setSuspended(suspended: boolean): void {
    if (this.suspended && !suspended) {
      // 350ms input drain on resume after emulator exit
      this.drainUntilMs = performance.now() + 350;
    }
    this.suspended = suspended;
    if (suspended) {
      for (const gate of this.gates.values()) gate.clear();
    }
  }

  start(): () => void {
    if (typeof window === 'undefined') return () => {};
    const onConnect = () => this.pollOnce(performance.now());
    const onDisconnect = () => this.pollOnce(performance.now());
    window.addEventListener('gamepadconnected', onConnect);
    window.addEventListener('gamepaddisconnected', onDisconnect);

    const tick = (now: number) => {
      this.pollOnce(now);
      this.rafId = window.requestAnimationFrame(tick);
    };
    this.rafId = window.requestAnimationFrame(tick);

    return () => {
      window.removeEventListener('gamepadconnected', onConnect);
      window.removeEventListener('gamepaddisconnected', onDisconnect);
      if (this.rafId !== null) {
        window.cancelAnimationFrame(this.rafId);
        this.rafId = null;
      }
      this.gates.clear();
    };
  }

  pollOnce(nowMs: number): void {
    if (typeof navigator === 'undefined' || typeof navigator.getGamepads !== 'function') return;
    const config = this.getConfig();
    if (!config.enableBrowserFallback) return;

    const rawPads = navigator.getGamepads();
    const devices: ConnectedGamepadInfo[] = [];

    for (let i = 0; i < rawPads.length; i += 1) {
      const pad = rawPads[i];
      if (!pad || !pad.connected) {
        this.gates.delete(i);
        continue;
      }
      const detectedFamily = detectGlyphFamilyFromName(pad.id);
      const glyphFamily =
        config.glyphOverride === 'auto' ? detectedFamily : config.glyphOverride;
      devices.push({
        slot: pad.index,
        name: pad.id || `Gamepad #${pad.index + 1}`,
        backend: 'browser_gamepad_api',
        glyphFamily,
        connected: true,
      });

      if (this.suspended || nowMs < this.drainUntilMs) {
        this.gates.get(pad.index)?.clear();
        continue;
      }

      let gate = this.gates.get(pad.index);
      if (!gate) {
        gate = new ActionRepeatGate();
        this.gates.set(pad.index, gate);
      }

      const [sx, sy] = applyScaledRadialDeadzone(
        pad.axes[0] ?? 0,
        pad.axes[1] ?? 0,
        config.deadzone
      );

      const isPressed = (idx: number) => {
        const btn = pad.buttons[idx];
        return Boolean(btn && (btn.pressed || btn.value > 0.55));
      };

      const pressedActions = new Set<GamepadNavAction>();
      if (isPressed(12) || sy < -0.55) pressedActions.add('up');
      if (isPressed(13) || sy > 0.55) pressedActions.add('down');
      if (isPressed(14) || sx < -0.55) pressedActions.add('left');
      if (isPressed(15) || sx > 0.55) pressedActions.add('right');

      for (const slot of Object.keys(STANDARD_BUTTON_INDEX) as StandardButtonSlot[]) {
        if (isPressed(STANDARD_BUTTON_INDEX[slot])) {
          pressedActions.add(config.buttonMap[slot]);
        }
      }

      const allPossible: GamepadNavAction[] = [
        'up',
        'down',
        'left',
        'right',
        'confirm',
        'back',
        'secondary',
        'context_menu',
        'prev_tab',
        'next_tab',
        'page_prev',
        'page_next',
        'quick_menu',
        'toggle_mode',
      ];
      for (const action of allPossible) {
        const res = gate.update(
          action,
          pressedActions.has(action),
          REPEATABLE_ACTIONS.has(action),
          nowMs,
          config.repeatDelayMs,
          config.repeatIntervalMs
        );
        if (res.fire) {
          this.callbacks.onAction({
            slot: pad.index,
            action,
            glyphFamily,
            repeated: res.repeated,
          });
        }
      }
    }

    const sig = JSON.stringify(devices.map((d) => `${d.slot}:${d.name}:${d.glyphFamily}`));
    if (sig !== this.lastDeviceSignature) {
      this.lastDeviceSignature = sig;
      this.callbacks.onDevicesChanged(devices);
    }
  }
}
