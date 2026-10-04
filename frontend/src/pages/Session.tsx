import { useEffect, useState } from 'react';
import { ApiError, unlockSession } from '../api/client';
import { useSessionStream } from '../hooks/useSessionStream';
import type { UserRole } from '../types/quiz';

interface SessionProps {
  sessionId: string;
  role: UserRole;
  onNavigate: (path: string) => void;
}

const WRONG_PASSWORD_COPY = 'Incorrect password!';

export function Session({ sessionId, role, onNavigate }: SessionProps) {
  const [password, setPassword] = useState('');
  const [unlocked, setUnlocked] = useState(false);
  const [unlocking, setUnlocking] = useState(false);
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const needsPassword = role === 'gamemaster' && !unlocked;
  const { state, status, error: streamError, refresh } = useSessionStream(sessionId, role);

  useEffect(() => {
    if (needsPassword) return;
    void refresh();
  }, [needsPassword, refresh]);

  async function submitUnlock(event: React.FormEvent) {
    event.preventDefault();
    setUnlocking(true);
    try {
      await unlockSession(sessionId, password);
      setUnlocked(true);
      setPasswordError(null);
    } catch (cause) {
      setPasswordError(
        cause instanceof ApiError && cause.isWrongPassword
          ? WRONG_PASSWORD_COPY
          : WRONG_PASSWORD_COPY,
      );
    } finally {
      setUnlocking(false);
    }
  }

  if (needsPassword) {
    return (
      <main className="relative z-10 mx-auto flex min-h-screen w-full max-w-md flex-col justify-center px-6 py-12">
        <form
          onSubmit={submitUnlock}
          className="rounded-2xl border border-slate-800 bg-slate-900/90 p-6 shadow-2xl sm:p-8"
        >
          <h1 className="font-display text-2xl font-bold text-white">Gamemaster Access</h1>
          <p className="mt-1 text-sm text-slate-400">
            Enter the moderator password to unlock host controls.
          </p>

          <label className="mt-6 block text-sm font-semibold text-slate-200" htmlFor="gm-unlock">
            Moderator password
          </label>
          <input
            id="gm-unlock"
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            placeholder="Enter password..."
            className="mt-2 w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-2 text-slate-100"
          />

          {passwordError ? (
            <p role="alert" className="mt-3 text-sm font-semibold text-rose-400">
              {passwordError}
            </p>
          ) : null}

          <div className="mt-6 flex gap-2">
            <button
              type="button"
              onClick={() => onNavigate(`/${sessionId}/view`)}
              className="flex-1 rounded-lg border border-slate-700 px-4 py-2.5 text-sm font-semibold text-slate-300 hover:border-cyan-400"
            >
              Switch to View
            </button>
            <button
              type="submit"
              disabled={unlocking || !password}
              className="flex-1 rounded-lg bg-cyan-500 px-4 py-2.5 text-sm font-bold text-slate-950 hover:bg-cyan-400 disabled:opacity-50"
            >
              Unlock
            </button>
          </div>
        </form>
      </main>
    );
  }

  return (
    <main className="relative z-10 mx-auto w-full max-w-7xl px-6 py-8">
      <p className="font-mono text-sm text-slate-400">
        {sessionId} / {role}
      </p>

      {streamError ? (
        <p role="alert" className="mt-4 rounded-lg border border-rose-500/40 bg-rose-500/10 px-4 py-3 text-sm text-rose-200">
          {streamError}
        </p>
      ) : null}

      {state ? (
        <section className="mt-6 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          <p className="col-span-full font-mono text-xs text-slate-500">
            revision {state.revision} / {status} / {state.status} / round{' '}
            {state.currentRound} of {state.roundCount}
          </p>
          {state.teams.map((team) => (
            <article
              key={team.id}
              className="rounded-xl border border-slate-800 bg-slate-900/90 p-4"
              style={{ borderColor: team.primaryHex }}
            >
              <h2 className="text-sm font-bold text-white">{team.name}</h2>
              <p className="mt-1 font-mono text-xs text-slate-400">
                {team.score} pts / {team.hearts} hearts
              </p>
              {team.onTurn ? (
                <p className="mt-2 font-mono text-[10px] uppercase text-cyan-300">
                  On turn
                </p>
              ) : null}
            </article>
          ))}
        </section>
      ) : null}
    </main>
  );
}