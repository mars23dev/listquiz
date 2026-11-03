import { useCallback, useEffect, useRef, useState } from 'react';
import { ApiError, fetchSessionState, sessionEventsUrl } from '../api/client';
import type { SessionState } from '../types/session';
import type { UserRole } from '../types/quiz';

type ConnectionStatus = 'connecting' | 'live' | 'reconnecting' | 'failed';

export interface UseSessionStream {
  state: SessionState | null;
  status: ConnectionStatus;
  error: string | null;
  /** The API error code behind `error`, e.g. `not_found` for an unknown session. */
  errorCode: ApiError['code'] | null;
  /** Re-read state on demand, e.g. after the gamemaster acts. */
  refresh: () => Promise<void>;
}

const BACKOFF_MS = [1000, 2000, 5000, 10000];

/** Parse a stream frame body; a frame we cannot read is treated as "no news". */
function readRevision(raw: string): number | null {
  try {
    const parsed = JSON.parse(raw) as { revision?: unknown };
    return typeof parsed.revision === 'number' ? parsed.revision : null;
  } catch {
    return null;
  }
}

/**
 * Subscribe to a session's live updates.
 *
 * The stream carries only a revision number, so every visible value comes from
 * `GET /state` re-read for this role: the room and the gamemaster can never
 * disagree, and a reconnect cannot replay a scoring sound (contracts/realtime.md).
 * There is no polling fallback by design; a dropped stream retries with backoff and
 * shows one reconnecting notice.
 */
export function useSessionStream(sessionId: string, role: UserRole): UseSessionStream {
  const [state, setState] = useState<SessionState | null>(null);
  const [status, setStatus] = useState<ConnectionStatus>('connecting');
  const [error, setError] = useState<string | null>(null);
  const [errorCode, setErrorCode] = useState<ApiError['code'] | null>(null);

  // Refs keep the EventSource callbacks reading current values without
  // re-subscribing on every state change.
  const revisionRef = useRef(-1);
  const attemptRef = useRef(0);
  const sourceRef = useRef<EventSource | null>(null);
  const timerRef = useRef<number | null>(null);
  const mountedRef = useRef(true);

  const applyState = useCallback((next: SessionState) => {
    revisionRef.current = next.revision;
    setState(next);
    setError(null);
    setErrorCode(null);
  }, []);

  const refresh = useCallback(async () => {
    try {
      applyState(await fetchSessionState(sessionId, role));
    } catch (err) {
      if (mountedRef.current) {
        setError(err instanceof ApiError ? err.message : 'Could not load the session.');
        setErrorCode(err instanceof ApiError ? err.code : null);
      }
    }
  }, [applyState, role, sessionId]);

  useEffect(() => {
    mountedRef.current = true;
    let disposed = false;

    const scheduleReconnect = () => {
      if (disposed) return;
      const index = Math.min(attemptRef.current, BACKOFF_MS.length - 1);
      attemptRef.current += 1;
      setStatus('reconnecting');
      timerRef.current = window.setTimeout(open, BACKOFF_MS[index]);
    };

    const open = () => {
      if (disposed) return;
      setStatus(attemptRef.current === 0 ? 'connecting' : 'reconnecting');
      const source = new EventSource(sessionEventsUrl(sessionId, role));
      sourceRef.current = source;

      // Both frames carry a revision. A heartbeat repeats the current one, so the
      // comparison is what keeps an idle stream from refetching every 15 seconds
      // (contracts/realtime.md).
      const onSignal = (event: Event) => {
        const incoming = readRevision((event as MessageEvent<string>).data ?? '');
        if (incoming === null || isNewerRevision(revisionRef.current, incoming)) {
          void refresh();
        }
      };

      source.addEventListener('state', onSignal);
      source.addEventListener('heartbeat', onSignal);
      source.onopen = () => {
        attemptRef.current = 0;
        setStatus('live');
      };
      source.onerror = () => {
        source.close();
        sourceRef.current = null;
        scheduleReconnect();
      };
    };

    void refresh().then(open);

    return () => {
      disposed = true;
      mountedRef.current = false;
      if (timerRef.current !== null) window.clearTimeout(timerRef.current);
      sourceRef.current?.close();
      sourceRef.current = null;
    };
  }, [refresh, role, sessionId]);

  return { state, status, error, errorCode, refresh };
}

/**
 * True when a newer revision has arrived than the one already rendered. Used to
 * skip a refetch when a heartbeat carries nothing new.
 */
export function isNewerRevision(current: number, incoming: number): boolean {
  return incoming > current;
}