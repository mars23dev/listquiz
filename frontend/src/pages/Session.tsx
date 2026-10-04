import type { UserRole } from '../types/quiz';

interface SessionProps {
  sessionId: string;
  role: UserRole;
  onNavigate: (path: string) => void;
}

export function Session({ sessionId, role }: SessionProps) {
  return (
    <main className="relative z-10 mx-auto w-full max-w-7xl px-6 py-8">
      <p className="font-mono text-sm text-slate-400">
        {sessionId} / {role}
      </p>
    </main>
  );
}