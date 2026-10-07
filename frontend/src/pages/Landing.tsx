interface LandingProps {
  onNavigateToLists: () => void;
  onNavigateToSetup: () => void;
  onNavigateToQuick: (gameType: 'ORDER' | 'TRIPMINE') => void;
}

export function Landing({ onNavigateToLists, onNavigateToSetup, onNavigateToQuick }: LandingProps) {
  return (
    <main className="relative z-10 mx-auto flex min-h-screen w-full max-w-3xl flex-col items-center justify-center gap-10 px-6 py-14">
      <header className="mb-2 text-center">
        <p className="mb-4 inline-flex items-center gap-2 rounded-full border border-cyan-500/30 bg-cyan-950/60 px-3 py-1 font-mono text-xs font-semibold uppercase tracking-wider text-cyan-300">
          Interactive Broadcast Quiz Platform
        </p>
        <h1 className="font-display text-4xl font-black uppercase leading-none tracking-tight text-white sm:text-6xl">
          <span className="text-cyan-400">LIST</span>
          <span className="text-slate-100">QUIZ</span>
        </h1>
        <p className="mx-auto mt-3 max-w-md text-sm text-slate-400 sm:text-base">
          Create custom timeline lists, configure multiplayer game shows, and host live
          turn-based Order challenges.
        </p>
      </header>

      <div className="grid w-full grid-cols-1 gap-6 md:grid-cols-2">
        <button
          type="button"
          onClick={onNavigateToLists}
          className="group relative flex flex-col justify-between rounded-2xl border-2 border-slate-700/80 bg-slate-900/90 p-6 text-left shadow-2xl transition hover:border-cyan-400/80 sm:p-8"
        >
          <span className="absolute right-4 top-4 font-mono text-xs font-bold text-slate-500 group-hover:text-cyan-400">
            01
          </span>
          <span>
            <span className="mb-6 flex h-14 w-14 items-center justify-center rounded-xl border border-cyan-500/30 bg-cyan-500/10 text-cyan-400">
              01
            </span>
            <span className="block font-display text-2xl font-black tracking-wide text-white group-hover:text-cyan-300">
              List Management
            </span>
            <span className="mt-2 block text-xs leading-relaxed text-slate-400 sm:text-sm">
              Create, edit, and import quiz lists from CSV. Reorder items via
              drag-and-drop, add icons, and configure multiple value columns.
            </span>
          </span>
          <span className="mt-6 flex items-center gap-2 border-t border-slate-800/80 pt-4 text-xs font-bold text-cyan-400">
            Manage Lists
          </span>
        </button>

        <button
          type="button"
          onClick={onNavigateToSetup}
          className="group relative flex flex-col justify-between rounded-2xl border-2 border-slate-700/80 bg-slate-900/90 p-6 text-left shadow-2xl transition hover:border-amber-400/80 sm:p-8"
        >
          <span className="absolute right-4 top-4 font-mono text-xs font-bold text-slate-500 group-hover:text-amber-400">
            02
          </span>
          <span>
            <span className="mb-6 flex h-14 w-14 items-center justify-center rounded-xl border border-amber-500/30 bg-amber-500/10 text-amber-400">
              02
            </span>
            <span className="block font-display text-2xl font-black tracking-wide text-white group-hover:text-amber-300">
              Setup ListQuiz
            </span>
            <span className="mt-2 block text-xs leading-relaxed text-slate-400 sm:text-sm">
              Choose teams, configure the rounds, and launch a live session with a link
              for every player.
            </span>
          </span>
          <span className="mt-6 flex items-center gap-2 border-t border-slate-800/80 pt-4 text-xs font-bold text-amber-400">
            Host a Quiz
          </span>
        </button>
      </div>

      <div className="w-full rounded-2xl border-2 border-emerald-500/40 bg-emerald-950/30 p-5">
        <h2 className="font-display text-lg font-black uppercase tracking-wide text-emerald-300">
          Quick Game
        </h2>
        <p className="mt-1 text-xs text-slate-400 sm:text-sm">
          Pick a game type and play immediately — one team, no host controls.
        </p>
        <div className="mt-4 flex flex-wrap gap-3">
          <button
            type="button"
            onClick={() => onNavigateToQuick('ORDER')}
            className="rounded-lg border border-emerald-400/60 bg-emerald-500/10 px-4 py-2 text-sm font-bold text-emerald-200 hover:bg-emerald-500/20 focus:outline-none focus-visible:ring-2 focus-visible:ring-emerald-300"
          >
            Order
          </button>
          <button
            type="button"
            onClick={() => onNavigateToQuick('TRIPMINE')}
            className="rounded-lg border border-emerald-400/60 bg-emerald-500/10 px-4 py-2 text-sm font-bold text-emerald-200 hover:bg-emerald-500/20 focus:outline-none focus-visible:ring-2 focus-visible:ring-emerald-300"
          >
            Tripmine
          </button>
        </div>
      </div>
    </main>
  );
}