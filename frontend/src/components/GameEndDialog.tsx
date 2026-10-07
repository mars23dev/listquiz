import { useEffect, useRef } from 'react';

export interface GameEndTeam {
  id: string;
  name: string;
  score: number;
  primaryHex: string;
}

interface GameEndDialogProps {
  teams: GameEndTeam[];
  /** Called when the dialog is dismissed; the caller deletes the finished session. */
  onDismiss: () => void;
}

/**
 * The end-of-game pop-up (feature 005, US3): every team ranked by points, highest
 * first. Hearts are deliberately omitted — they reset each round and mean nothing at
 * the end. Focus moves into the dialog and it is dismissible by keyboard.
 */
export function GameEndDialog({ teams, onDismiss }: GameEndDialogProps) {
  const dismissRef = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    dismissRef.current?.focus();
  }, []);

  const ranked = [...teams].sort((first, second) => second.score - first.score);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/80 p-4">
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="game-end-title"
        className="w-full max-w-lg rounded-2xl border border-amber-400/60 bg-slate-900 p-6 shadow-2xl"
      >
        <h2
          id="game-end-title"
          className="font-display text-2xl font-black uppercase tracking-wide text-amber-300"
        >
          Final Ranking
        </h2>
        <ol className="mt-4 space-y-2">
          {ranked.map((team, index) => (
            <li
              key={team.id}
              className="flex items-center justify-between rounded-lg border bg-slate-950 px-3 py-2"
              style={{ borderColor: team.primaryHex }}
            >
              <span className="font-bold text-white">
                #{index + 1} {team.name}
              </span>
              <span className="font-mono font-bold text-amber-200">{team.score} pts</span>
            </li>
          ))}
        </ol>
        <button
          ref={dismissRef}
          type="button"
          onClick={onDismiss}
          className="mt-5 w-full rounded-lg bg-amber-400 px-4 py-2 font-bold text-slate-950 hover:bg-amber-300 focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-200"
        >
          Back to home
        </button>
      </div>
    </div>
  );
}
