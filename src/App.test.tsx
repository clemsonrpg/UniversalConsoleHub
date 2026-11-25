import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from './App';

describe('App foundation smoke test', () => {
  it('renders the Universal Console Hub heading', () => {
    render(<App />);
    expect(screen.getByRole('heading', { name: /Universal Console Hub/i })).toBeInTheDocument();
  });
});
