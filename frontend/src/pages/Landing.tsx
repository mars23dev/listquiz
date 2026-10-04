interface LandingProps {
  onNavigateToLists: () => void;
  onNavigateToSetup: () => void;
}

export function Landing({ onNavigateToLists, onNavigateToSetup }: LandingProps) {
  return (
    <main className="relative z-10 flex flex-col items-center justify-center gap-6 min-h-screen px-6">
      <h1 className="font-display text-5xl font-extrabold tracking-tight">
        <span className="text-cyan-400">LIST</span>
        <span className="text-slate-100">QUIZ</span>
      </h1>
      <div className="flex flex-wrap gap-4 justify-center">
        <button
          type="button"
          onClick={onNavigateToSetup}
          className="rounded-xl bg-cyan-500 px-6 py-3 font-bold text-slate-950 hover:bg-cyan-400"
        >
          Host a Quiz
        </button>
        <button
          type="button"
          onClick={onNavigateToLists}
          className="rounded-xl border border-slate-700 px-6 py-3 font-bold hover:border-cyan-400"
        >
          Manage Lists
        </button>
      </div>
    </main>
  );
}