import '@testing-library/jest-dom/vitest';

class StubEventSource {
  onopen: ((event: Event) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;

  addEventListener(): void {}
  removeEventListener(): void {}
  close(): void {}
}

Object.defineProperty(globalThis, 'EventSource', {
  value: StubEventSource,
  configurable: true,
});