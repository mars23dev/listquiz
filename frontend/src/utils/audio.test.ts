import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

interface FakeContext {
  state: string;
  currentTime: number;
  destination: object;
  resume: ReturnType<typeof vi.fn>;
  createOscillator: ReturnType<typeof vi.fn>;
  createGain: ReturnType<typeof vi.fn>;
}

function fakeContext(state = 'suspended') {
  const oscillator = {
    type: '',
    frequency: {
      setValueAtTime: vi.fn(),
      exponentialRampToValueAtTime: vi.fn(),
      linearRampToValueAtTime: vi.fn(),
    },
    connect: vi.fn(),
    start: vi.fn(),
    stop: vi.fn(),
  };
  const gain = {
    gain: {
      setValueAtTime: vi.fn(),
      exponentialRampToValueAtTime: vi.fn(),
      linearRampToValueAtTime: vi.fn(),
    },
    connect: vi.fn(),
  };
  const context: FakeContext = {
    state,
    currentTime: 0,
    destination: {},
    resume: vi.fn(async () => {}),
    createOscillator: vi.fn(() => oscillator),
    createGain: vi.fn(() => gain),
  };
  return { context, oscillator };
}

describe('audio', () => {
  beforeEach(() => {
    vi.resetModules();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('unlockAudio resumes a suspended context', async () => {
    const { context } = fakeContext('suspended');
    vi.stubGlobal('AudioContext', vi.fn(() => context));
    const audio = await import('./audio');

    audio.unlockAudio();

    expect(context.resume).toHaveBeenCalledTimes(1);
  });

  it('plays a cue once the context is available', async () => {
    const { context } = fakeContext('running');
    vi.stubGlobal('AudioContext', vi.fn(() => context));
    const audio = await import('./audio');

    audio.playSnapSound();

    expect(context.createOscillator).toHaveBeenCalledTimes(1);
  });

  it('stays silent while the session is muted', async () => {
    const { context } = fakeContext('running');
    vi.stubGlobal('AudioContext', vi.fn(() => context));
    const audio = await import('./audio');

    audio.setSoundMuted(true);
    audio.playSnapSound();
    audio.playFailureSound();

    expect(context.createOscillator).not.toHaveBeenCalled();
  });
});
