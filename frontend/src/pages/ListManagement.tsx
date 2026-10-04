interface ListManagementProps {
  onBackToHome: () => void;
}

export function ListManagement({ onBackToHome }: ListManagementProps) {
  return (
    <main className="relative z-10 mx-auto w-full max-w-6xl px-6 py-10">
      <button type="button" onClick={onBackToHome} className="mb-6 text-sm text-slate-400 hover:text-cyan-300">
        &larr; Back
      </button>
      <h1 className="font-display text-3xl font-bold">List Management</h1>
    </main>
  );
}