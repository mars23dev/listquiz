// Provenance: the view shell is adapted from the reference frontend's App.tsx. That draft is
// a read-only reference (constitution v2.1.1) and is never imported or read at
// build or run time; this file is the only copy.
//
// Corrections against the draft, all required by specs/001-listquiz-game:
//   - hash routing replaced with history-based path routing, because the backend
//     owns the routes and redirects /{sessionId} to /{sessionId}/p1
//     (contracts/rest-api.md, FR-012)
//   - localStorage and BroadcastChannel state replaced by the API client and the
//     SSE subscription (FR-060, FR-061)
//   - the Quick Start Sample action removed: the product ships no session list
//     and no demo generator (clarification 5)
//   - the four page components moved from components/ to pages/
import { useCallback, useEffect, useState } from 'react';
import { StageLighting } from './components/StageLighting';
import { Landing } from './pages/Landing';
import { ListManagement } from './pages/ListManagement';
import { SetupQuiz } from './pages/SetupQuiz';
import { Session } from './pages/Session';
import type { UserRole } from './types/quiz';

type Route =
  | { view: 'landing' }
  | { view: 'lists' }
  | { view: 'setup' }
  | { view: 'session'; sessionId: string; role: UserRole };

const ROLES: UserRole[] = ['gamemaster', 'p1', 'p2', 'p3', 'p4', 'view'];

function parseRoute(pathname: string): Route {
  const parts = pathname.replace(/^\/+|\/+$/g, '').split('/');

  if (parts.length === 0 || parts[0] === '') return { view: 'landing' };
  if (parts.length === 1) {
    if (parts[0] === 'lists') return { view: 'lists' };
    if (parts[0] === 'setup') return { view: 'setup' };
  }
  if (parts.length === 2) {
    const role = parts[1] as UserRole;
    if (ROLES.includes(role)) return { view: 'session', sessionId: parts[0], role };
  }
  // An unknown session still renders the session view; the state read answers
  // not_found and the view shows the not-found message (FR-013).
  if (parts.length >= 1 && parts[0] !== 'lists' && parts[0] !== 'setup') {
    return { view: 'session', sessionId: parts[0], role: 'p1' };
  }
  return { view: 'landing' };
}

export default function App() {
  const [route, setRoute] = useState<Route>(() => parseRoute(window.location.pathname));

  useEffect(() => {
    const onPopState = () => setRoute(parseRoute(window.location.pathname));
    window.addEventListener('popstate', onPopState);
    return () => window.removeEventListener('popstate', onPopState);
  }, []);

  const navigate = useCallback((to: string) => {
    window.history.pushState({}, '', to);
    setRoute(parseRoute(to));
  }, []);

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex flex-col justify-between overflow-x-hidden selection:bg-cyan-500/30">
      <StageLighting />

      {route.view === 'landing' && (
        <Landing
          onNavigateToLists={() => navigate('/lists')}
          onNavigateToSetup={() => navigate('/setup')}
        />
      )}

      {route.view === 'lists' && <ListManagement onBackToHome={() => navigate('/')} />}

      {route.view === 'setup' && (
        <SetupQuiz onBackToHome={() => navigate('/')} onLaunchRole={navigate} />
      )}

      {route.view === 'session' && (
        <Session
          sessionId={route.sessionId}
          role={route.role}
          onNavigate={navigate}
        />
      )}
    </div>
  );
}