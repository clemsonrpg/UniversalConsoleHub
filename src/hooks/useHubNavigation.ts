import { useEffect } from 'react';

export type HubSectionId =
  | 'home'
  | 'consoles'
  | 'library'
  | 'profile'
  | 'statistics'
  | 'trophies'
  | 'emulators'
  | 'themes'
  | 'settings';

export const HUB_SECTIONS: HubSectionId[] = [
  'home',
  'consoles',
  'library',
  'profile',
  'statistics',
  'trophies',
  'emulators',
  'themes',
  'settings',
];

export function useHubNavigation(
  activeSection: HubSectionId,
  onSelect: (section: HubSectionId) => void,
  disabled = false
) {
  useEffect(() => {
    if (disabled) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (!event.altKey) return;
      const idx = HUB_SECTIONS.indexOf(activeSection);
      if (event.key === 'ArrowDown' || event.key === 'ArrowRight') {
        event.preventDefault();
        const next = HUB_SECTIONS[(idx + 1) % HUB_SECTIONS.length];
        if (next) onSelect(next);
      } else if (event.key === 'ArrowUp' || event.key === 'ArrowLeft') {
        event.preventDefault();
        const prev = HUB_SECTIONS[(idx - 1 + HUB_SECTIONS.length) % HUB_SECTIONS.length];
        if (prev) onSelect(prev);
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [activeSection, onSelect, disabled]);
}
