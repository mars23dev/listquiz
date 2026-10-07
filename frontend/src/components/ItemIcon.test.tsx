import { describe, expect, it } from 'vitest';
import { render } from '@testing-library/react';
import { ItemIcon } from './ItemIcon';

describe('ItemIcon', () => {
  it('renders a country flag for a flag: name (feature 014)', () => {
    const { container } = render(<ItemIcon name="flag:de" />);

    const flag = container.querySelector('.fi');
    expect(flag).not.toBeNull();
    expect(flag?.classList.contains('fi-de')).toBe(true);
    // A valid flag must not fall back to the generic icon.
    expect(container.querySelector('.lucide-circle-question-mark')).toBeNull();
  });

  it('still renders a lucide icon for a known name', () => {
    const { container } = render(<ItemIcon name="Film" />);

    expect(container.querySelector('.lucide-film')).not.toBeNull();
    expect(container.querySelector('.fi')).toBeNull();
  });

  it('falls back to HelpCircle for an unknown name', () => {
    const { container } = render(<ItemIcon name="NotAnIcon" />);

    expect(container.querySelector('.lucide-circle-question-mark')).not.toBeNull();
    expect(container.querySelector('.fi')).toBeNull();
  });
});
