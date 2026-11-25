import { useEffect } from 'react';

export type HubSectionId =
  | 'home'
  | 'consoles'
  | 'library'
  | 'profile'
  | 'trophies'
  | 'emulators'
  | 'settings';

export const HUB_SECTIONS: HubSectionId[] = [
  'home',
  'consoles',
  'library',
  'profile',
  'trophies',
  'emulators',
  'settings',
];

export function useHubNavigation(
  activeSection: HubSectionId,
  onSelect: (section: HubSectionId) => void
) {
  useEffect(() => {
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
  }, [activeSection, onSelect]);
}
