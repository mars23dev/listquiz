interface SetupQuizProps {
  onBackToHome: () => void;
  onLaunchRole: (path: string) => void;
}

export function SetupQuiz({ onBackToHome }: SetupQuizProps) {
  return (
    <main className="relative z-10 mx-auto w-full max-w-4xl px-6 py-10">
      <button type="button" onClick={onBackToHome} className="mb-6 text-sm text-slate-400 hover:text-cyan-300">
        &larr; Back
      </button>
      <h1 className="font-display text-3xl font-bold">Set Up Quiz</h1>
    </main>
  );
}