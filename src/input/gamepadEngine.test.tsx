import { fireEvent, render, screen } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { ControllerActionBar } from '../components/ControllerActionBar';
import { BrowserGamepadPoller } from './gamepadEngine';
import { InputProvider, useInput } from './InputProvider';
import { DEFAULT_GAMEPAD_CONFIG, detectGlyphFamilyFromName, getButtonGlyph } from './types';

function makeMockGamepad(overrides: Partial<Gamepad> = {}): Gamepad {
  const buttons: GamepadButton[] = Array.from({ length: 16 }, () => ({
    pressed: false,
    touched: false,
    value: 0,
  }));
  return {
    axes: [0, 0, 0, 0],
    buttons,
    connected: true,
    hapticActuators: [],
    id: 'DualSense Wireless Controller (STANDARD GAMEPAD Vendor: 054c Product: 0ce6)',
    index: 0,
    mapping: 'standard',
    timestamp: 1000,
    vibrationActuator: null,
    ...overrides,
  } as Gamepad;
}

describe('Gamepad engine, hot-plug reconnect, glyph mapping, spatial focus, and modal focus restore', () => {
  it('detects controller families, applies debounce/repeat, and handles disconnect + reconnect', () => {
    expect(detectGlyphFamilyFromName('DualSense Wireless Controller (054c)')).toBe('playstation');
    expect(detectGlyphFamilyFromName('Xbox Series X Controller (045e)')).toBe('xbox');
    expect(detectGlyphFamilyFromName('Nintendo Switch Pro Controller (057e)')).toBe('nintendo');
    expect(getButtonGlyph('confirm', 'playstation')).toBe('✕');
    expect(getButtonGlyph('back', 'playstation')).toBe('○');
    expect(getButtonGlyph('confirm', 'xbox')).toBe('A');

    let pads: Array<Gamepad | null> = [makeMockGamepad()];
    Object.defineProperty(navigator, 'getGamepads', {
      configurable: true,
      value: () => pads,
    });

    const onDevicesChanged = vi.fn();
    const onAction = vi.fn();
    const poller = new BrowserGamepadPoller(() => DEFAULT_GAMEPAD_CONFIG, {
      onDevicesChanged,
      onAction,
    });

    // 1. Initial connection detected as PlayStation DualSense
    poller.pollOnce(1000);
    expect(onDevicesChanged).toHaveBeenCalledTimes(1);
    expect(onDevicesChanged.mock.calls[0]?.[0]?.[0]?.glyphFamily).toBe('playstation');

    // 2. Press A/Cross (index 0): fires strictly once even when held for 600ms (zero multi-activation)
    const padWithA = makeMockGamepad();
    (padWithA.buttons[0] as { pressed: boolean; value: number }).pressed = true;
    (padWithA.buttons[0] as { pressed: boolean; value: number }).value = 1;
    pads = [padWithA];
    poller.pollOnce(1100);
    poller.pollOnce(1200);
    poller.pollOnce(1700);
    expect(onAction).toHaveBeenCalledTimes(1);
    expect(onAction).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'confirm', glyphFamily: 'playstation', repeated: false })
    );

    // 3. Stick drift within deadzone (0.12) does NOT emit directional navigation; full deflection does
    const padDrift = makeMockGamepad({ axes: [0.12, -0.1, 0, 0] });
    pads = [padDrift];
    poller.pollOnce(1800);
    expect(onAction).toHaveBeenCalledTimes(1);

    const padDown = makeMockGamepad({ axes: [0, 0.9, 0, 0] });
    pads = [padDown];
    poller.pollOnce(1900);
    poller.pollOnce(2350); // past 380ms initial repeat delay -> repeats cleanly
    expect(onAction).toHaveBeenCalledTimes(3);
    expect(onAction.mock.calls[1]?.[0]).toMatchObject({ action: 'down', repeated: false });
    expect(onAction.mock.calls[2]?.[0]).toMatchObject({ action: 'down', repeated: true });

    // 4. Disconnect & Reconnect detected cleanly
    pads = [null];
    poller.pollOnce(2500);
    expect(onDevicesChanged).toHaveBeenLastCalledWith([]);

    pads = [makeMockGamepad({ id: 'Xbox Wireless Controller (045e)' })];
    poller.pollOnce(2700);
    expect(onDevicesChanged.mock.calls.at(-1)?.[0]?.[0]?.glyphFamily).toBe('xbox');
  });

  it('supports keyboard/gamepad navigation, Desktop<->Fullscreen toggle, and restores exact focus after closing ConfirmDialog', async () => {
    const onCycleSection = vi.fn();

    function FocusTestHarness() {
      const [settings, setSettings] = useState<Record<string, string>>({});
      const [modalOpen, setModalOpen] = useState(false);
      const [clicked, setClicked] = useState('');

      return (
        <InputProvider
          settings={settings}
          suspended={false}
          nativeGamepads={[]}
          lastNativeAction={null}
          onUpdateSetting={async (k, v) => setSettings((prev) => ({ ...prev, [k]: v }))}
          onNavigateBack={() => false}
          onCycleSection={onCycleSection}
        >
          <InnerControls
            modalOpen={modalOpen}
            setModalOpen={setModalOpen}
            clicked={clicked}
            setClicked={setClicked}
          />
          <ControllerActionBar />
        </InputProvider>
      );
    }

    function InnerControls({
      modalOpen,
      setModalOpen,
      clicked,
      setClicked,
    }: {
      modalOpen: boolean;
      setModalOpen: (v: boolean) => void;
      clicked: string;
      setClicked: (v: string) => void;
    }) {
      const { displayMode, dispatchAction } = useInput();
      return (
        <div data-focus-scope="main">
          <span data-testid="mode">{displayMode}</span>
          <span data-testid="clicked">{clicked}</span>
          <button
            type="button"
            data-focus-id="btn-1"
            data-focus-default="true"
            onClick={() => setClicked('btn-1')}
          >
            Item 1
          </button>
          <button
            type="button"
            data-focus-id="btn-2"
            onClick={() => setModalOpen(true)}
          >
            Abrir Modal
          </button>
          <button type="button" onClick={() => dispatchAction('down')}>
            Sim Down
          </button>
          <button type="button" onClick={() => dispatchAction('confirm')}>
            Sim Confirm
          </button>
          <button type="button" onClick={() => dispatchAction('back')}>
            Sim Back
          </button>
          <button type="button" onClick={() => dispatchAction('next_tab')}>
            Sim RB
          </button>
          <button type="button" onClick={() => dispatchAction('toggle_mode')}>
            Sim Toggle Mode
          </button>

          <ConfirmDialog
            open={modalOpen}
            title="Confirmar Ação"
            onConfirm={() => setModalOpen(false)}
            onCancel={() => setModalOpen(false)}
          >
            <p>Conteúdo do diálogo</p>
          </ConfirmDialog>
        </div>
      );
    }

    render(<FocusTestHarness />);
    const btn1 = screen.getByRole('button', { name: 'Item 1' });
    const btn2 = screen.getByRole('button', { name: 'Abrir Modal' });
    btn1.focus();
    expect(document.activeElement).toBe(btn1);

    // ArrowDown moves focus cleanly from Item 1 -> Abrir Modal
    fireEvent.keyDown(window, { key: 'ArrowDown' });
    expect(document.activeElement).toBe(btn2);

    // Open modal -> focus moves into ConfirmDialog; press Back -> closes modal & restores focus to btn2!
    fireEvent.click(btn2);
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Cancelar' }));

    fireEvent.keyDown(window, { key: 'Escape' });
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(document.activeElement).toBe(btn2);

    // Toggle Desktop <-> Fullscreen via F11
    expect(screen.getByTestId('mode').textContent).toBe('desktop');
    fireEvent.keyDown(window, { key: 'F11' });
    expect(screen.getByTestId('mode').textContent).toBe('fullscreen');

    // RB switches section cleanly
    fireEvent.click(screen.getByText('Sim RB'));
    expect(onCycleSection).toHaveBeenCalledWith(1);
  });
});
