import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { CapabilityState } from '../types/domain';
import { CapabilityBadge } from './CapabilityBadge';

describe('CapabilityBadge', () => {
  it('renders clear, honest labels for all six canonical capability states', () => {
    const cases: Array<[CapabilityState, RegExp]> = [
      ['FULL_SYSTEM_BOOT', /Boot Completo/i],
      ['ORIGINAL_DASHBOARD', /Dashboard Original/i],
      ['PARTIAL_DASHBOARD', /Dashboard Parcial/i],
      ['DIRECT_GAME_ONLY', /Jogo Direto/i],
      ['EXPERIMENTAL', /Experimental/i],
      ['UNSUPPORTED', /Não Suportado/i],
    ];

    for (const [state, pattern] of cases) {
      const { unmount } = render(<CapabilityBadge state={state} />);
      expect(screen.getByText(pattern)).toBeInTheDocument();
      unmount();
    }
  });
});
