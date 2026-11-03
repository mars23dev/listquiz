import { describe, expect, it } from 'vitest';
import { isNewerRevision } from '../hooks/useSessionStream';

describe('isNewerRevision', () => {
  it('accepts a revision the client has not seen', () => {
    expect(isNewerRevision(42, 43)).toBe(true);
  });

  it('skips a heartbeat that repeats the current revision', () => {
    expect(isNewerRevision(43, 43)).toBe(false);
  });

  it('skips a stale revision', () => {
    expect(isNewerRevision(43, 42)).toBe(false);
  });
});