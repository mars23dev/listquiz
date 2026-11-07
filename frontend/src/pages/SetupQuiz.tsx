import { useEffect, useMemo, useState } from 'react';
import {
  ApiError,
  createSession,
  fetchDefaultLists,
  fetchFolderLists,
  fetchFolders,
  unlockFolder,
  type FolderSummary,
} from '../api/client';
import type { ListColumnSummary, ListSummary } from '../types/session';

const MAX_TEAMS = 4;
const MIN_ROUNDS = 1;
const MAX_ROUNDS = 10;
const TEAM_COLOURS = ['#06b6d4', '#fb7185', '#facc15', '#a78bfa'] as const;

interface RoundDraft {
  roundNumber: number;
  quizType: 'ORDER' | 'TRIPMINE' | 'CONNECT';
  listId: string;
  columnId: string;
  tripmineSide: 'highest' | 'lowest' | 'random';
  tripmineSlots: 10 | 20;
}

interface SetupQuizProps {
  onBackToHome: () => void;
  onLaunchRole: (path: string) => void;
}

interface SourceList extends ListSummary {
  folderId: string;
  folderName: string;
}

const MODE_MINIMUM: Record<RoundDraft['quizType'], number> = {
  ORDER: 11,
  CONNECT: 10,
  TRIPMINE: 10,
};

function requiredItems(round: Pick<RoundDraft, 'quizType' | 'tripmineSlots'>): number {
  return round.quizType === 'TRIPMINE' ? round.tripmineSlots : MODE_MINIMUM[round.quizType];
}

function modeLabel(round: Pick<RoundDraft, 'quizType' | 'tripmineSlots'>): string {
  if (round.quizType === 'TRIPMINE') return `Tripmine with ${round.tripmineSlots} items`;
  return round.quizType === 'ORDER' ? 'Order' : 'Connect';
}

function bestValueColumn(list: Pick<ListSummary, 'columns'>): ListColumnSummary | undefined {
  let best: ListColumnSummary | undefined;
  for (const column of list.columns) {
    if (!best || (column.valueCount ?? 0) > (best.valueCount ?? 0)) best = column;
  }
  return best;
}

function selectedColumnValueCount(list: ListSummary, round: RoundDraft): number {
  const columnId =
    round.listId === list.id && round.columnId
      ? round.columnId
      : bestValueColumn(list)?.id;
  return list.columns.find((column) => column.id === columnId)?.valueCount ?? 0;
}

function supportsModeMinimum(list: ListSummary, round: RoundDraft): boolean {
  return selectedColumnValueCount(list, round) >= MODE_MINIMUM[round.quizType];
}

function capacityError(round: RoundDraft, list: ListSummary): string | null {
  const required = requiredItems(round);
  const available = selectedColumnValueCount(list, round);
  if (available >= required) return null;
  return `Round ${round.roundNumber}: ${modeLabel(round)} needs at least ${required} items with values in the selected column; the list has ${available}.`;
}

function newRound(
  roundNumber: number,
  lists: Array<Pick<ListSummary, 'id' | 'columns'>>,
): RoundDraft {
  const first =
    lists.find((list) => (bestValueColumn(list)?.valueCount ?? 0) >= MODE_MINIMUM.ORDER) ??
    lists[0];
  return {
    roundNumber,
    quizType: 'ORDER',
    listId: first?.id ?? '',
    columnId: bestValueColumn(first ?? { columns: [] })?.id ?? first?.columns[0]?.id ?? '',
    tripmineSide: 'highest',
    tripmineSlots: 10,
  };
}

export function SetupQuiz({ onBackToHome, onLaunchRole }: SetupQuizProps) {
  const [step, setStep] = useState<1 | 2 | 3>(1);
  const [gmPassword, setGmPassword] = useState('');
  const [teamCount, setTeamCount] = useState(2);
  const [teamNames, setTeamNames] = useState<string[]>(['', '']);
  const [lists, setLists] = useState<ListSummary[]>([]);
  const [folders, setFolders] = useState<FolderSummary[]>([]);
  const [folderLists, setFolderLists] = useState<Record<string, ListSummary[]>>({});
  const [folderPasswords, setFolderPasswords] = useState<Record<string, string>>({});
  const [unlockTarget, setUnlockTarget] = useState<string | null>(null);
  const [unlockPassword, setUnlockPassword] = useState('');
  const [unlocking, setUnlocking] = useState<string | null>(null);
  const [rounds, setRounds] = useState<RoundDraft[]>([]);
  const [copied, setCopied] = useState<string | null>(null);
  const [created, setCreated] = useState<{ sessionId: string; links: CreatedLinks } | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const answer = await fetchDefaultLists();
        if (cancelled) return;
        setLists(answer.lists);
        setRounds((current) =>
          current.length === 0 ? [newRound(1, answer.lists)] : current,
        );
      } catch (cause: unknown) {
        if (cancelled) return;
        setLists([]);
        setRounds((current) =>
          current.length === 0 ? [newRound(1, [])] : current,
        );
        setError(cause instanceof Error ? cause.message : 'Lists could not be loaded.');
      }

      try {
        const answer = await fetchFolders();
        if (!cancelled) setFolders(answer);
      } catch (cause: unknown) {
        if (!cancelled) {
          setError(cause instanceof Error ? cause.message : 'Folders could not be loaded.');
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const availableLists = useMemo<SourceList[]>(() => {
    const defaultFolderName =
      folders.find((folder) => folder.isDefault)?.name ?? 'Default Lists';
    const defaults = lists.map((list) => ({
      ...list,
      folderId: 'default',
      folderName: defaultFolderName,
    }));
    const custom = Object.entries(folderLists).flatMap(([folderId, folderItems]) => {
      const folderName =
        folders.find((folder) => folder.id === folderId)?.name ?? folderId;
      return folderItems.map((list) => ({ ...list, folderId, folderName }));
    });
    return [...defaults, ...custom];
  }, [folders, folderLists, lists]);

  const capacityErrors = useMemo(
    () =>
      rounds.map((round) => {
        const list = availableLists.find((candidate) => candidate.id === round.listId);
        return list ? capacityError(round, list) : null;
      }),
    [rounds, availableLists],
  );

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
    setError(null);
    setRounds((current) =>
      current.map((round, position) =>
        position === index ? { ...round, ...patch } : round,
      ),
    );
  }

  function pickList(index: number, listId: string) {
    const list = availableLists.find((candidate) => candidate.id === listId);
    const round = rounds[index];
    const minimum = round ? MODE_MINIMUM[round.quizType] : MODE_MINIMUM.ORDER;
    const column =
      list?.columns.find((candidate) => (candidate.valueCount ?? 0) >= minimum) ??
      list?.columns[0];
    updateRound(index, { listId, columnId: column?.id ?? '' });
  }

  function addRound() {
    if (rounds.length >= MAX_ROUNDS) return;
    setRounds((current) => [...current, newRound(current.length + 1, availableLists)]);
  }

  function chooseFolder(folder: FolderSummary) {
    if (folder.isDefault || folderPasswords[folder.id]) return;
    setUnlockTarget(folder.id);
    setUnlockPassword('');
    setError(null);
  }

  async function submitFolderUnlock(folder: FolderSummary) {
    if (!unlockPassword.trim()) return;
    setUnlocking(folder.id);
    setError(null);
    try {
      await unlockFolder(folder.id, unlockPassword);
      const answer = await fetchFolderLists(folder.id, unlockPassword);
      setFolderPasswords((current) => ({ ...current, [folder.id]: unlockPassword }));
      setFolderLists((current) => ({ ...current, [folder.id]: answer.lists }));
      setUnlockTarget(null);
      setUnlockPassword('');
    } catch (cause) {
      setFolderPasswords((current) => {
        const next = { ...current };
        delete next[folder.id];
        return next;
      });
      setFolderLists((current) => {
        const next = { ...current };
        delete next[folder.id];
        return next;
      });
      setUnlockPassword('');
      setError(
        cause instanceof ApiError
          ? cause.message
          : 'The folder could not be unlocked.',
      );
    } finally {
      setUnlocking(null);
    }
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
    for (const round of rounds) {
      const list = availableLists.find((candidate) => candidate.id === round.listId);
      if (!list) return `Round ${round.roundNumber} needs a list and a column.`;
      if (!list.columns.some((column) => column.id === round.columnId)) {
        return `Round ${round.roundNumber} needs a valid value column.`;
      }
      const problem = capacityError(round, list);
      if (problem) return problem;
      if (!availableLists.some((candidate) => supportsModeMinimum(candidate, round))) {
        return `Round ${round.roundNumber}: no list can support ${modeLabel(round)}.`;
      }
    }
    return null;
  }

  async function createSessionAndShowLinks() {
    setBusy(true);
    setError(null);
    try {
      const selectedFolderPasswords: Record<string, string> = {};
      for (const round of rounds) {
        const list = availableLists.find((candidate) => candidate.id === round.listId);
        if (list && !list.isDefault && folderPasswords[list.folderId]) {
          selectedFolderPasswords[list.folderId] = folderPasswords[list.folderId];
        }
      }

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
            ? { tripmineSide: round.tripmineSide, tripmineSlots: round.tripmineSlots }
            : {}),
        })),
        ...(Object.keys(selectedFolderPasswords).length > 0
          ? { folderPasswords: selectedFolderPasswords }
          : {}),
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

          {folders.length > 0 ? (
            <section
              aria-label="List folders"
              className="mt-4 rounded-xl border border-slate-800 bg-slate-950 p-4"
            >
              <h2 className="text-xs font-bold uppercase tracking-wide text-slate-300">
                Source Folders
              </h2>
              <p className="mt-1 text-xs text-slate-500">
                Default lists are always available. Unlock a protected folder to use its
                lists in this setup.
              </p>
              <div className="mt-3 flex flex-wrap items-start gap-3">
                {folders.map((folder) => {
                  const unlocked = folder.isDefault || Boolean(folderPasswords[folder.id]);
                  const isTarget = unlockTarget === folder.id;
                  return (
                    <div key={folder.id} className="flex flex-col gap-2">
                      <button
                        type="button"
                        aria-expanded={isTarget}
                        onClick={() => chooseFolder(folder)}
                        disabled={unlocked}
                        className={[
                          'rounded-lg border px-3 py-1.5 text-xs font-semibold',
                          unlocked
                            ? 'border-emerald-500/40 bg-emerald-500/10 text-emerald-200'
                            : isTarget
                              ? 'border-cyan-500 bg-cyan-500/15 text-cyan-200'
                              : 'border-slate-700 bg-slate-900 text-slate-300 hover:border-slate-500',
                        ].join(' ')}
                      >
                        {folder.isDefault ? '' : unlocked ? 'Unlocked: ' : '\u{1F512} '}
                        {folder.name}
                      </button>
                      {isTarget && !unlocked ? (
                        <form
                          className="flex items-center gap-2"
                          onSubmit={(event) => {
                            event.preventDefault();
                            void submitFolderUnlock(folder);
                          }}
                        >
                          <label
                            htmlFor={`folder-password-${folder.id}`}
                            className="sr-only"
                          >
                            Password for {folder.name}
                          </label>
                          <input
                            id={`folder-password-${folder.id}`}
                            type="password"
                            value={unlockPassword}
                            onChange={(event) => setUnlockPassword(event.target.value)}
                            placeholder={`${folder.name} password`}
                            autoFocus
                            className="w-40 rounded-lg border border-slate-700 bg-slate-900 px-2 py-1.5 text-xs text-slate-100"
                          />
                          <button
                            type="submit"
                            disabled={unlocking === folder.id || !unlockPassword.trim()}
                            className="rounded-lg bg-cyan-500 px-3 py-1.5 text-xs font-bold text-slate-950 hover:bg-cyan-400 disabled:opacity-50"
                          >
                            {unlocking === folder.id ? 'Unlocking...' : 'Unlock'}
                          </button>
                        </form>
                      ) : null}
                    </div>
                  );
                })}
              </div>
            </section>
          ) : null}

          <ol className="mt-6 space-y-4">
            {rounds.map((round, index) => {
              const list = availableLists.find((candidate) => candidate.id === round.listId);
              const options = availableLists.filter(
                (candidate) =>
                  supportsModeMinimum(candidate, round) || candidate.id === round.listId,
              );
              const noCompatibleList = !availableLists.some((candidate) =>
                supportsModeMinimum(candidate, round),
              );
              const roundError = capacityErrors[index];
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
                            quizType: event.target.value as 'ORDER' | 'TRIPMINE' | 'CONNECT',
                          })
                        }
                        className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2"
                      >
                          <option value="ORDER">Order</option>
                          <option value="TRIPMINE">Tripmine</option>
                          <option value="CONNECT">Connect</option>
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
                        {options.map((candidate) => (
                          <option key={candidate.id} value={candidate.id}>
                            {candidate.isDefault
                              ? `${candidate.title} (${candidate.itemCount} items)`
                              : `${candidate.folderName} / ${candidate.title} (${candidate.itemCount} items)`}
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

                    {round.quizType === 'TRIPMINE' ? (
                      <label className="text-xs font-semibold text-slate-300">
                        Tripmine side
                        <select
                          value={round.tripmineSide}
                          onChange={(event) =>
                            updateRound(index, {
                              tripmineSide: event.target.value as 'highest' | 'lowest' | 'random',
                            })
                          }
                          className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2"
                        >
                          <option value="highest">Highest</option>
                          <option value="lowest">Lowest</option>
                          <option value="random">Random (Hard)</option>
                        </select>
                      </label>
                    ) : null}

                    {round.quizType === 'TRIPMINE' ? (
                      <label className="text-xs font-semibold text-slate-300">
                        Tripmine items
                        <select
                          value={round.tripmineSlots}
                          onChange={(event) =>
                            updateRound(index, {
                              tripmineSlots: Number(event.target.value) as 10 | 20,
                            })
                          }
                          className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2"
                        >
                          <option value={10}>10 items</option>
                          <option value={20}>20 items</option>
                        </select>
                      </label>
                    ) : null}
                  </div>

                  {noCompatibleList ? (
                    <p className="mt-3 text-xs font-semibold text-amber-200">
                      No list can support {modeLabel(round)}. Choose another mode or add a larger
                      list.
                    </p>
                  ) : null}

                  {roundError ? (
                    <p
                      role="alert"
                      className="mt-3 rounded-lg border border-rose-500/40 bg-rose-500/10 px-3 py-2 text-xs text-rose-200"
                    >
                      {roundError}
                    </p>
                  ) : null}
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