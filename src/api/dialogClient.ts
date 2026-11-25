import { open, type DialogFilter } from '@tauri-apps/plugin-dialog';
import { isTauriDesktopRuntime } from './tauriClient';

export interface PickFileOptions {
  title?: string;
  defaultPath?: string;
  filters?: DialogFilter[];
}

export interface PickDirectoryOptions {
  title?: string;
  defaultPath?: string;
}

export interface HubDialogTransport {
  pickFile(options?: PickFileOptions): Promise<string | null>;
  pickDirectory(options?: PickDirectoryOptions): Promise<string | null>;
}

declare global {
  interface Window {
    __UCH_TEST_DIALOG__?: HubDialogTransport;
  }
}

export async function pickFile(options?: PickFileOptions): Promise<string | null> {
  if (typeof window !== 'undefined' && window.__UCH_TEST_DIALOG__) {
    return window.__UCH_TEST_DIALOG__.pickFile(options);
  }
  if (!isTauriDesktopRuntime()) {
    return null;
  }
  const selected = await open({
    title: options?.title,
    defaultPath: options?.defaultPath,
    filters: options?.filters,
    directory: false,
    multiple: false,
  });
  return typeof selected === 'string' ? selected : null;
}

export async function pickDirectory(options?: PickDirectoryOptions): Promise<string | null> {
  if (typeof window !== 'undefined' && window.__UCH_TEST_DIALOG__) {
    return window.__UCH_TEST_DIALOG__.pickDirectory(options);
  }
  if (!isTauriDesktopRuntime()) {
    return null;
  }
  const selected = await open({
    title: options?.title,
    defaultPath: options?.defaultPath,
    directory: true,
    multiple: false,
  });
  return typeof selected === 'string' ? selected : null;
}
