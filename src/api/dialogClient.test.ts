import { afterEach, describe, expect, it, vi } from 'vitest';
import { pickDirectory, pickFile, type HubDialogTransport } from './dialogClient';

declare global {
  interface Window {
    __UCH_TEST_DIALOG__?: HubDialogTransport;
  }
}

describe('dialogClient native file and directory pickers', () => {
  afterEach(() => {
    delete window.__UCH_TEST_DIALOG__;
  });

  it('uses injected test dialog transport for file and directory selection in UI tests', async () => {
    const pickFileMock = vi.fn().mockResolvedValue('D:\\Emulators\\RPCS3\\rpcs3.exe');
    const pickDirMock = vi.fn().mockResolvedValue('D:\\Jogos PS3');

    window.__UCH_TEST_DIALOG__ = {
      pickFile: pickFileMock,
      pickDirectory: pickDirMock,
    };

    const exe = await pickFile({
      title: 'Selecionar RPCS3',
      filters: [{ name: 'Executáveis', extensions: ['exe'] }],
    });
    const dir = await pickDirectory({ title: 'Selecionar pasta PS3' });

    expect(exe).toBe('D:\\Emulators\\RPCS3\\rpcs3.exe');
    expect(dir).toBe('D:\\Jogos PS3');
    expect(pickFileMock).toHaveBeenCalledTimes(1);
    expect(pickDirMock).toHaveBeenCalledTimes(1);
  });

  it('returns null gracefully when called outside Tauri runtime without test transport', async () => {
    await expect(pickFile({ title: 'Teste' })).resolves.toBeNull();
    await expect(pickDirectory({ title: 'Teste' })).resolves.toBeNull();
  });
});
