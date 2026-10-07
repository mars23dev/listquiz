import { useEffect, useMemo, useState } from 'react';
import { ApiError, createSession, fetchDefaultLists } from '../api/client';
import type { ListSummary } from '../types/session';

const MAX_TEAMS = 4;
const MIN_ROUNDS = 1;
const MAX_ROUNDS = 10;
const TEAM_COLOURS = ['#06b6d4', '#fb7185', '#facc15', '#a78bfa'] as const;

interface RoundDraft {
  roundNumber: number;
  quizType: 'ORDER' | 'TRIPMINE';
  listId: string;
  columnId: string;
  tripmineSide: 'highest' | 'lowest';
}

interface SetupQuizProps {
  onBackToHome: () => void;
  onLaunchRole: (path: string) => void;
}

function newRound(roundNumber: number, lists: ListSummary[]): RoundDraft {
  const first = lists[0];
  return {
    roundNumber,
    quizType: 'ORDER',
    listId: first?.id ?? '',
    columnId: first?.columns[0]?.id ?? '',
    tripmineSide: 'highest',
  };
}

export function SetupQuiz({ onBackToHome, onLaunchRole }: SetupQuizProps) {
  const [step, setStep] = useState<1 | 2 | 3>(1);
  const [gmPassword, setGmPassword] = useState('');
  const [teamCount, setTeamCount] = useState(2);
  const [teamNames, setTeamNames] = useState<string[]>(['', '']);
  const [lists, setLists] = useState<ListSummary[]>([]);
  const [rounds, setRounds] = useState<RoundDraft[]>([]);
  const [copied, setCopied] = useState<string | null>(null);
  const [created, setCreated] = useState<{ sessionId: string; links: CreatedLinks } | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchDefaultLists()
      .then((answer) => {
        setLists(answer.lists);
        setRounds((current) =>
          current.length === 0 ? [newRound(1, answer.lists)] : current,
        );
      })
      .catch((cause: unknown) => {
        setLists([]);
        setRounds((current) =>
          current.length === 0 ? [newRound(1, [])] : current,
        );
        setError(cause instanceof Error ? cause.message : 'Lists could not be loaded.');
      });
  }, []);

  const teams = useMemo(
    () =>
      Array.from({ length: teamCount }, (_, index) => ({
        name: teamNames[index]?.trim() || `Team ${index + 1}`,
        primaryHex: TEAM_COLOURS[index % TEAM_COLOURS.length],
      })),
    [teamCount, teamNames],
  );

  function changeTeamCount(next: number) {
    setTeamCount(next);
    setTeamNames((current) => {
      const names = [...current];
      while (names.length < next) names.push('');
      return names.slice(0, next);
    });
  }

  function updateRound(index: number, patch: Partial<RoundDraft>) {
    setRounds((current) =>
      current.map((round, position) =>
        position === index ? { ...round, ...patch } : round,
      ),
    );
  }

  function pickList(index: number, listId: string) {
    const list = lists.find((candidate) => candidate.id === listId);
    updateRound(index, { listId, columnId: list?.columns[0]?.id ?? '' });
  }

  function addRound() {
    if (rounds.length >= MAX_ROUNDS) return;
    setRounds((current) => [...current, newRound(current.length + 1, lists)]);
  }

  function removeRound(index: number) {
    setRounds((current) =>
      current.length <= MIN_ROUNDS
        ? current
        : current
            .filter((_, position) => position !== index)
            .map((round, position) => ({ ...round, roundNumber: position + 1 })),
    );
  }

  function validateStepOne(): string | null {
    if (!gmPassword.trim()) return 'A gamemaster password is required.';
    return null;
  }

  function validateStepTwo(): string | null {
    if (rounds.length < MIN_ROUNDS) return 'Add at least one round.';
    const incomplete = rounds.find((round) => !round.listId || !round.columnId);
    if (incomplete) {
      return `Round ${incomplete.roundNumber} needs a list and a column.`;
    }
    return null;
  }

  async function createSessionAndShowLinks() {
    setBusy(true);
    setError(null);
    try {
      const response = await createSession({
        gmPassword,
        teamCount,
        teams,
        roundCount: rounds.length,
        rounds: rounds.map((round) => ({
          roundNumber: round.roundNumber,
          quizType: round.quizType,
          listId: round.listId,
          columnId: round.columnId,
          ...(round.quizType === 'TRIPMINE'
            ? { tripmineSide: round.tripmineSide }
            : {}),
        })),
      });
      setCreated({
        sessionId: response.sessionId,
        links: response.links,
      });
      setStep(3);
    } catch (cause) {
      setError(
        cause instanceof ApiError
          ? cause.message
          : 'The session could not be created.',
      );
    } finally {
      setBusy(false);
    }
  }

  async function copyLink(key: string, path: string) {
    const absolute = `${window.location.origin}${path}`;
    try {
      await navigator.clipboard.writeText(absolute);
      setCopied(key);
      window.setTimeout(() => setCopied(null), 1500);
    } catch {
      setError('The link could not be copied.');
    }
  }

  function submitStepOne() {
    const problem = validateStepOne();
    setError(problem);
    if (!problem) setStep(2);
  }

  function submitStepTwo() {
    const problem = validateStepTwo();
    setError(problem);
    if (problem) return;
    void createSessionAndShowLinks();
  }

  return (
    <main className="relative z-10 mx-auto w-full max-w-4xl px-6 py-10">
      <button
        type="button"
        onClick={onBackToHome}
        className="mb-6 text-sm text-slate-400 hover:text-cyan-300"
      >
        &larr; Back
      </button>

      <ol className="mb-8 flex items-center gap-3 text-xs font-mono uppercase">
        {['Participants & Password', 'Round Configuration', 'Session Links'].map(
          (label, index) => {
            const number = index + 1;
            const active = step === number;
            return (
              <li key={label} className="flex items-center gap-3">
                <span
                  aria-current={active ? 'step' : undefined}
                  className={
                    active
                      ? 'rounded-full bg-cyan-500 px-3 py-1 font-bold text-slate-950'
                      : 'rounded-full border border-slate-700 px-3 py-1 text-slate-400'
                  }
                >
                  Step {number}: {label}
                </span>
                {number < 3 ? <span className="text-slate-700">&rarr;</span> : null}
              </li>
            );
          },
        )}
      </ol>

      {error ? (
        <p role="alert" className="mb-6 rounded-lg border border-rose-500/40 bg-rose-500/10 px-4 py-3 text-sm text-rose-200">
          {error}
        </p>
      ) : null}

      {step === 1 ? (
        <section className="rounded-2xl border border-slate-800 bg-slate-900/90 p-6 sm:p-8">
          <h1 className="font-display text-3xl font-bold text-white">
            Step 1: Participants &amp; Password
          </h1>
          <p className="mt-1 text-sm text-slate-400">
            Configure session security and participant team names (1 to 4 teams).
          </p>

          <label className="mt-6 block text-sm font-semibold text-slate-200" htmlFor="gm-password">
            Moderator password
          </label>
          <input
            id="gm-password"
            type="password"
            value={gmPassword}
            onChange={(event) => setGmPassword(event.target.value)}
            placeholder="Enter the moderator password"
            className="mt-2 w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-2 text-slate-100"
          />
          <p className="mt-1 text-xs text-slate-500">Required. Players never see it.</p>

          <fieldset className="mt-6">
            <legend className="text-sm font-semibold text-slate-200">Teams</legend>
            <div className="mt-2 flex flex-wrap gap-2">
              {Array.from({ length: MAX_TEAMS }, (_, index) => index + 1).map((count) => (
                <button
                  key={count}
                  type="button"
                  aria-pressed={teamCount === count}
                  onClick={() => changeTeamCount(count)}
                  className={
                    teamCount === count
                      ? 'rounded-lg bg-cyan-500 px-4 py-2 text-sm font-bold text-slate-950'
                      : 'rounded-lg border border-slate-700 px-4 py-2 text-sm font-semibold text-slate-300'
                  }
                >
                  {count} {count === 1 ? 'Team' : 'Teams'}
                </button>
              ))}
            </div>
          </fieldset>

          <div className="mt-6 grid gap-3 sm:grid-cols-2">
            {teams.map((team, index) => (
              <label key={index} className="block text-sm">
                <span className="flex items-center gap-2 font-semibold text-slate-200">
                  <span
                    aria-hidden="true"
                    className="inline-block h-3 w-3 rounded-full"
                    style={{ backgroundColor: team.primaryHex }}
                  />
                  Player {index + 1}
                </span>
                <input
                  value={teamNames[index] ?? ''}
                  onChange={(event) =>
                    setTeamNames((current) => {
                      const names = [...current];
                      names[index] = event.target.value;
                      return names;
                    })
                  }
                  placeholder={`Team ${index + 1}`}
                  className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2"
                />
              </label>
            ))}
          </div>

          <div className="mt-8 flex justify-end">
            <button
              type="button"
              onClick={submitStepOne}
              className="rounded-xl bg-cyan-500 px-6 py-3 font-bold text-slate-950 hover:bg-cyan-400"
            >
              Continue
            </button>
          </div>
        </section>
      ) : null}

      {step === 2 ? (
        <section className="rounded-2xl border border-slate-800 bg-slate-900/90 p-6 sm:p-8">
          <h1 className="font-display text-3xl font-bold text-white">
            Step 2: Round Configuration
          </h1>
          <p className="mt-1 text-sm text-slate-400">
            {rounds.length} of {MAX_ROUNDS} rounds. Every round names the list and column it
            plays from.
          </p>

          <ol className="mt-6 space-y-4">
            {rounds.map((round, index) => {
              const list = lists.find((candidate) => candidate.id === round.listId);
              return (
                <li key={round.roundNumber} className="rounded-xl border border-slate-800 bg-slate-950 p-4">
                  <div className="flex items-center justify-between">
                    <h2 className="font-mono text-sm font-bold uppercase text-cyan-300">
                      Round {round.roundNumber}
                    </h2>
                    <button
                      type="button"
                      onClick={() => removeRound(index)}
                      disabled={rounds.length <= MIN_ROUNDS}
                      className="text-xs font-semibold text-rose-300 disabled:opacity-30"
                    >
                      Remove
                    </button>
                  </div>

                  <div className="mt-3 grid gap-3 sm:grid-cols-4">
                    <label className="text-xs font-semibold text-slate-300">
                      Quiz type
                      <select
                        value={round.quizType}
                        onChange={(event) =>
                          updateRound(index, {
                            quizType: event.target.value as 'ORDER' | 'TRIPMINE',
                          })
                        }
                        className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2"
                      >
                        <option value="ORDER">Order</option>
                        <option value="TRIPMINE">Tripmine</option>
                      </select>
                    </label>

                    <label className="text-xs font-semibold text-slate-300">
                      List
                      <select
                        value={round.listId}
                        onChange={(event) => pickList(index, event.target.value)}
                        className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2"
                      >
                        <option value="">Select a list</option>
                        {lists.map((candidate) => (
                          <option key={candidate.id} value={candidate.id}>
                            {candidate.title} ({candidate.itemCount} items)
                          </option>
                        ))}
                      </select>
                    </label>

                    {list && list.columns.length > 1 ? (
                      <label className="text-xs font-semibold text-slate-300">
                        Value Column
                        <select
                          value={round.columnId}
                          onChange={(event) => updateRound(index, { columnId: event.target.value })}
                          className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2"
                        >
                          <option value="">Select a column</option>
                          {(list?.columns ?? []).map((column) => (
                            <option key={column.id} value={column.id}>
                              {column.name}
                            </option>
                          ))}
                        </select>
                      </label>
                    ) : null}

                    <label className="text-xs font-semibold text-slate-300">
                      Tripmine side
                      <select
                        value={round.tripmineSide}
                        disabled={round.quizType !== 'TRIPMINE'}
                        onChange={(event) =>
                          updateRound(index, {
                            tripmineSide: event.target.value as 'highest' | 'lowest',
                          })
                        }
                        className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2"
                      >
                        <option value="highest">Highest</option>
                        <option value="lowest">Lowest</option>
                      </select>
                    </label>
                  </div>
                </li>
              );
            })}
          </ol>

          <div className="mt-4 flex flex-wrap items-center justify-between gap-3">
            <button
              type="button"
              onClick={addRound}
              disabled={rounds.length >= MAX_ROUNDS}
              className="rounded-lg border border-slate-700 px-4 py-2 text-sm font-semibold text-slate-200 disabled:opacity-30"
            >
              Add round
            </button>
            <div className="flex gap-2">
              <button
                type="button"
                onClick={() => setStep(1)}
                className="rounded-lg border border-slate-700 px-5 py-2.5 text-sm font-semibold text-slate-300"
              >
                Back
              </button>
              <button
                type="button"
                onClick={submitStepTwo}
                disabled={busy}
                className="rounded-xl bg-cyan-500 px-6 py-3 font-bold text-slate-950 hover:bg-cyan-400 disabled:opacity-50"
              >
                {busy ? 'Creating...' : 'Create session'}
              </button>
            </div>
          </div>
        </section>
      ) : null}

      {step === 3 && created ? (
        <section className="rounded-2xl border border-slate-800 bg-slate-900/90 p-6 sm:p-8">
          <h2 className="text-center font-display text-2xl font-black uppercase text-white">
            Session Ready: <span className="text-cyan-400">{created.sessionId}</span>
          </h2>
          <p className="mt-1 text-center text-xs text-slate-400">
            Share links with each participant, moderator, or open stage view. All views are
            live synchronized in real-time.
          </p>

          <div className="mt-6 space-y-3">
            <LinkRow
              key="gamemaster"
              path={created.links.gamemaster}
              title="Gamemaster / Moderator"
              badge="HOST"
              tone="amber"
              copied={copied === 'gamemaster'}
              onCopy={() => void copyLink('gamemaster', created.links.gamemaster)}
              onLaunch={() => onLaunchRole(created.links.gamemaster)}
              launchLabel="Launch GM"
            />

            {created.links.players.map((link) => (
              <LinkRow
                key={link.role}
                path={link.path}
                title={`Player ${link.role.slice(1)}`}
                badge={link.hasTeam ? undefined : 'NO TEAM'}
                tone={link.hasTeam ? 'slate' : 'muted'}
                copied={copied === link.role}
                onCopy={() => void copyLink(link.role, link.path)}
                onLaunch={() => onLaunchRole(link.path)}
                launchLabel={`Launch ${link.role.toUpperCase()}`}
              />
            ))}

            <LinkRow
              key="spectator"
              path={created.links.spectator}
              title="Live Stage / Spectator View"
              badge="PUBLIC"
              tone="cyan"
              copied={copied === 'spectator'}
              onCopy={() => void copyLink('spectator', created.links.spectator)}
              onLaunch={() => onLaunchRole(created.links.spectator)}
              launchLabel="Launch View"
            />
          </div>

          <div className="mt-8 flex justify-center border-t border-slate-800 pt-6">
            <button
              type="button"
              onClick={() => onLaunchRole(created.links.gamemaster)}
              className="w-full rounded-xl bg-gradient-to-r from-cyan-500 to-amber-400 px-8 py-3 text-sm font-black uppercase tracking-wider text-slate-950"
            >
              Start Game as Gamemaster
            </button>
          </div>
        </section>
      ) : null}
    </main>
  );
}

type CreatedLinks = NonNullable<
  Awaited<ReturnType<typeof createSession>>['links']
>;

interface LinkRowProps {
  path: string;
  title: string;
  badge?: string;
  tone: 'amber' | 'cyan' | 'slate' | 'muted';
  copied: boolean;
  onCopy: () => void;
  onLaunch: () => void;
  launchLabel: string;
}

const TONE_CLASSES: Record<LinkRowProps['tone'], string> = {
  amber: 'border-amber-500/40',
  cyan: 'border-cyan-500/30',
  slate: 'border-slate-800',
  muted: 'border-slate-800 opacity-60',
};

function LinkRow({
  path,
  title,
  badge,
  tone,
  copied,
  onCopy,
  onLaunch,
  launchLabel,
}: LinkRowProps) {
  return (
    <div className={`flex flex-col gap-3 rounded-xl border bg-slate-950 p-3.5 sm:flex-row sm:items-center sm:justify-between ${TONE_CLASSES[tone]}`}>
      <div className="min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-xs font-bold text-white">{title}</span>
          {badge ? (
            <span className="rounded bg-slate-800 px-1.5 py-0.5 font-mono text-[10px] text-slate-300">
              {badge}
            </span>
          ) : null}
        </div>
        <span className="block truncate font-mono text-[11px] text-slate-400">{path}</span>
      </div>
      <div className="flex w-full items-center gap-2 sm:w-auto">
        <button
          type="button"
          onClick={onCopy}
          className="flex-1 rounded-lg bg-slate-800 px-3 py-1.5 text-xs font-semibold text-slate-200 hover:bg-slate-700 sm:flex-none"
        >
          {copied ? 'Copied' : 'Copy Link'}
        </button>
        <button
          type="button"
          onClick={onLaunch}
          className="flex-1 rounded-lg bg-slate-700 px-3 py-1.5 text-xs font-semibold text-white hover:bg-slate-600 sm:flex-none"
        >
          {launchLabel}
        </button>
      </div>
    </div>
  );
}