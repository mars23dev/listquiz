import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  ApiError,
  createFolder as createFolderRequest,
  deleteFolder as deleteFolderRequest,
  fetchFolders,
  fetchFolderLists,
  renameFolder as renameFolderRequest,
  unlockFolder,
  type FolderSummary,
} from '../api/client';
import type { ListSummary } from '../types/session';

interface ListManagementProps {
  onBackToHome: () => void;
}

type Dialog =
  | { kind: 'create' }
  | { kind: 'unlock'; folderId: string }
  | { kind: 'rename'; folderId: string; current: string }
  | { kind: 'delete'; folderId: string; name: string }
  | null;

/**
 * Folder passwords live in this component's memory and nowhere else. There is no
 * token and no cookie (research D-006): a reload locks every protected folder
 * again, which is the behaviour the gamemaster is told to expect.
 */
export function ListManagement({ onBackToHome }: ListManagementProps) {
  const [folders, setFolders] = useState<FolderSummary[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [passwords, setPasswords] = useState<Record<string, string>>({});
  const [lists, setLists] = useState<ListSummary[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [dialog, setDialog] = useState<Dialog>(null);

  const active = useMemo(
    () => folders.find((folder) => folder.id === activeId) ?? null,
    [folders, activeId],
  );
  const activePassword = activeId ? (passwords[activeId] ?? '') : '';
  // The default folder is open from the start; every other folder needs its
  // password once per page load.
  const isUnlocked = Boolean(active) && (active?.isDefault || activePassword !== '');

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const loaded = await fetchFolders();
        // A payload that is not a folder array is a broken answer, not an empty
        // account; show the error banner instead of crashing the page.
        if (!Array.isArray(loaded)) {
          throw new TypeError('The folder list response was not an array.');
        }
        if (cancelled) return;
        setFolders(loaded);
        // The default folder comes first and never needs a password (FR-050).
        setActiveId((current) => current ?? loaded[0]?.id ?? null);
      } catch (cause) {
        if (!cancelled) setError(messageOf(cause, 'Folders could not be loaded.'));
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  // A locked folder never has its lists requested, so no list name can leak into
  // the DOM or the network log (FR-047, SC-009).
  const loadLists = useCallback(async () => {
    if (!activeId) return;
    try {
      const answer = await fetchFolderLists(activeId, passwords[activeId]);
      setLists(answer.lists);
      setError(null);
    } catch (cause) {
      setLists([]);
      setError(messageOf(cause, 'Lists could not be loaded.'));
    }
  }, [activeId, passwords]);

  useEffect(() => {
    void loadLists();
  }, [loadLists]);

  function select(folder: FolderSummary) {
    setActiveId(folder.id);
    setLists([]);
    setError(null);
  }

  function rememberPassword(folderId: string, password: string) {
    setPasswords((current) => ({ ...current, [folderId]: password }));
  }

  function forgetPassword(folderId: string) {
    setPasswords((current) => {
      const next = { ...current };
      delete next[folderId];
      return next;
    });
  }

  async function afterChange(message?: string) {
    setDialog(null);
    const loaded = await fetchFolders();
    setFolders(loaded);
    if (message) setError(message);
  }

  if (loading) {
    return (
      <main className="relative z-10 mx-auto w-full max-w-6xl px-6 py-10">
        <p className="font-mono text-sm text-slate-400">Loading folders...</p>
      </main>
    );
  }

  return (
    <main className="relative z-10 mx-auto w-full max-w-6xl px-6 py-10">
      <div className="mb-6 flex flex-wrap items-center justify-between gap-4 border-b border-slate-800 pb-4">
        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={onBackToHome}
            className="rounded-lg border border-slate-700 bg-slate-900 px-3 py-1.5 text-xs font-semibold text-slate-300 hover:bg-slate-800"
          >
            &larr; Home
          </button>
          <div>
            <h1 className="font-display text-2xl font-black uppercase tracking-tight text-white">
              List Management
            </h1>
            <p className="text-xs text-slate-400">
              {lists.length} lists available for ListQuiz rounds
            </p>
          </div>
        </div>
        <button
          type="button"
          onClick={() => setDialog({ kind: 'create' })}
          className="rounded-lg bg-cyan-500 px-3.5 py-1.5 text-xs font-bold text-slate-950 hover:bg-cyan-400"
        >
          + New Folder
        </button>
      </div>

      <FolderBar
        folders={folders}
        activeId={activeId}
        passwords={passwords}
        onSelect={select}
        onUnlock={(folderId) => setDialog({ kind: 'unlock', folderId })}
        onRename={(folder) =>
          setDialog({ kind: 'rename', folderId: folder.id, current: folder.name })
        }
        onDelete={(folder) =>
          setDialog({ kind: 'delete', folderId: folder.id, name: folder.name })
        }
      />

      {error ? (
        <p
          role="alert"
          className="mt-4 rounded-lg border border-rose-500/40 bg-rose-500/10 px-4 py-3 text-sm text-rose-200"
        >
          {error}
        </p>
      ) : null}

      {active && !isUnlocked ? (
        <LockedFolderPrompt
          name={active.name}
          onUnlock={() => setDialog({ kind: 'unlock', folderId: active.id })}
        />
      ) : (
        <ListGrid lists={lists} />
      )}

      {dialog?.kind === 'create' ? (
        <CreateFolderDialog
          onCancel={() => setDialog(null)}
          onSubmit={async (name, password) => {
            try {
              await createFolderRequest(name, password);
              await afterChange();
            } catch (cause) {
              setError(messageOf(cause, 'The folder could not be created.'));
              setDialog(null);
            }
          }}
        />
      ) : null}

      {dialog?.kind === 'unlock' ? (
        <UnlockFolderDialog
          onCancel={() => setDialog(null)}
          onSubmit={async (password) => {
            try {
              await unlockFolder(dialog.folderId, password);
              rememberPassword(dialog.folderId, password);
              setActiveId(dialog.folderId);
              setDialog(null);
            } catch (cause) {
              setError(messageOf(cause, 'Incorrect password!'));
              setDialog(null);
            }
          }}
        />
      ) : null}

      {dialog?.kind === 'rename' ? (
        <RenameFolderDialog
          current={dialog.current}
          onCancel={() => setDialog(null)}
          onSubmit={async (name) => {
            try {
              await renameFolderRequest(dialog.folderId, name, passwords[dialog.folderId] ?? '');
              await afterChange();
            } catch (cause) {
              setError(messageOf(cause, 'The folder could not be renamed.'));
              setDialog(null);
            }
          }}
        />
      ) : null}

      {dialog?.kind === 'delete' ? (
        <DeleteFolderDialog
          name={dialog.name}
          listCount={lists.length}
          onCancel={() => setDialog(null)}
          onSubmit={async (confirmPassword) => {
            const folderId = dialog.folderId;
            try {
              await deleteFolderRequest(
                folderId,
                confirmPassword,
                passwords[folderId] ?? '',
              );
              forgetPassword(folderId);
              if (folderId === activeId) setActiveId('default');
              await afterChange();
            } catch (cause) {
              setError(messageOf(cause, 'The folder could not be deleted.'));
              setDialog(null);
            }
          }}
        />
      ) : null}
    </main>
  );
}

function FolderBar({
  folders,
  activeId,
  passwords,
  onSelect,
  onUnlock,
  onRename,
  onDelete,
}: {
  folders: FolderSummary[];
  activeId: string | null;
  passwords: Record<string, string>;
  onSelect: (folder: FolderSummary) => void;
  onUnlock: (folderId: string) => void;
  onRename: (folder: FolderSummary) => void;
  onDelete: (folder: FolderSummary) => void;
}) {
  return (
    <nav
      aria-label="Folders"
      className="mb-5 flex flex-wrap items-center gap-2 rounded-xl border border-slate-800 bg-slate-900/60 p-3"
    >
      {folders.map((folder) => {
        const unlocked = folder.isDefault || passwords[folder.id] !== undefined;
        const selected = folder.id === activeId;
        return (
          <div key={folder.id} className="flex items-center gap-1">
            <button
              type="button"
              aria-current={selected ? 'true' : undefined}
              onClick={() => (unlocked ? onSelect(folder) : onUnlock(folder.id))}
              className={[
                'rounded-lg border px-3 py-1.5 text-xs font-semibold',
                selected
                  ? 'border-cyan-500 bg-cyan-500/15 text-cyan-200'
                  : 'border-slate-700 bg-slate-950 text-slate-300 hover:border-slate-500',
              ].join(' ')}
            >
              {unlocked ? '' : '\u{1F512} '}
              {folder.name}
              {folder.isDefault ? ' (Default)' : ''}
            </button>
            {unlocked && !folder.isDefault && selected ? (
              <>
                <button
                  type="button"
                  onClick={() => onRename(folder)}
                  className="rounded-lg border border-slate-700 px-2 py-1.5 text-xs text-slate-300 hover:border-cyan-400"
                >
                  Rename
                </button>
                <button
                  type="button"
                  onClick={() => onDelete(folder)}
                  className="rounded-lg border border-rose-500/50 px-2 py-1.5 text-xs text-rose-300 hover:border-rose-400"
                >
                  Delete
                </button>
              </>
            ) : null}
          </div>
        );
      })}
    </nav>
  );
}

function LockedFolderPrompt({ name, onUnlock }: { name: string; onUnlock: () => void }) {
  return (
    <section className="rounded-xl border border-slate-800 bg-slate-900/90 p-6">
      <h2 className="font-display text-lg font-bold text-white">{name} is locked</h2>
      <p className="mt-1 text-sm text-slate-400">
        Its lists stay hidden until the folder password is entered.
      </p>
      <button
        type="button"
        onClick={onUnlock}
        className="mt-4 rounded-lg bg-cyan-500 px-4 py-2 text-sm font-bold text-slate-950 hover:bg-cyan-400"
      >
        Unlock
      </button>
    </section>
  );
}

function ListGrid({ lists }: { lists: ListSummary[] }) {
  if (lists.length === 0) {
    return (
      <p className="rounded-xl border border-slate-800 bg-slate-900/60 px-4 py-6 text-sm text-slate-400">
        This folder holds no lists yet.
      </p>
    );
  }
  return (
    <section className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
      {lists.map((list) => (
        <article
          key={list.id}
          className="flex flex-col justify-between rounded-xl border border-slate-800 bg-slate-900/90 p-4"
        >
          <div>
            <div className="mb-2 flex items-center justify-between gap-2">
              <span className="rounded border border-slate-700 bg-slate-800 px-2 py-0.5 font-mono text-[10px] font-bold uppercase text-cyan-300">
                {list.orderDirection === 'high_to_low' ? 'High → Low' : 'Low → High'}
              </span>
              {list.isDefault ? (
                <span className="rounded border border-slate-700 px-2 py-0.5 font-mono text-[10px] uppercase text-slate-400">
                  Read-only
                </span>
              ) : null}
            </div>
            <h3 className="text-sm font-bold text-white">{list.title}</h3>
            {list.description ? (
              <p className="mt-1 text-xs text-slate-400">{list.description}</p>
            ) : null}
          </div>
          <p className="mt-3 font-mono text-xs text-slate-500">
            {list.itemCount} items / {list.columns.length} value column
            {list.columns.length === 1 ? '' : 's'}
          </p>
        </article>
      ))}
    </section>
  );
}

function DialogShell({
  title,
  children,
  onCancel,
}: {
  title: string;
  children: React.ReactNode;
  onCancel: () => void;
}) {
  return (
    <div className="fixed inset-0 z-30 flex items-center justify-center bg-slate-950/80 p-4">
      <div
        role="dialog"
        aria-label={title}
        className="w-full max-w-md rounded-2xl border border-slate-800 bg-slate-900 p-6 shadow-2xl"
      >
        <h2 className="font-display text-lg font-bold text-white">{title}</h2>
        <div className="mt-4">{children}</div>
        <button
          type="button"
          onClick={onCancel}
          className="mt-4 w-full rounded-lg border border-slate-700 px-4 py-2 text-sm font-semibold text-slate-300 hover:border-slate-500"
        >
          Cancel
        </button>
      </div>
    </div>
  );
}

function CreateFolderDialog({
  onCancel,
  onSubmit,
}: {
  onCancel: () => void;
  onSubmit: (name: string, password: string) => Promise<void>;
}) {
  const [name, setName] = useState('');
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  return (
    <DialogShell title="New Folder" onCancel={onCancel}>
      <p className="mb-3 text-xs text-slate-400">
        A folder password is required; it protects the lists inside it.
      </p>
      <label className="block text-sm font-semibold text-slate-200" htmlFor="folder-name">
        Folder name
      </label>
      <input
        id="folder-name"
        value={name}
        onChange={(event) => setName(event.target.value)}
        className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-slate-100"
      />
      <label
        className="mt-3 block text-sm font-semibold text-slate-200"
        htmlFor="folder-password"
      >
        Folder password
      </label>
      <input
        id="folder-password"
        type="password"
        value={password}
        onChange={(event) => setPassword(event.target.value)}
        className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-slate-100"
      />
      <button
        type="button"
        disabled={busy || !name.trim() || !password.trim()}
        onClick={async () => {
          setBusy(true);
          await onSubmit(name, password);
          setBusy(false);
        }}
        className="mt-4 w-full rounded-lg bg-cyan-500 px-4 py-2 text-sm font-bold text-slate-950 hover:bg-cyan-400 disabled:opacity-50"
      >
        Create Folder
      </button>
    </DialogShell>
  );
}

function UnlockFolderDialog({
  onCancel,
  onSubmit,
}: {
  onCancel: () => void;
  onSubmit: (password: string) => Promise<void>;
}) {
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  return (
    <DialogShell title="Unlock Folder" onCancel={onCancel}>
      <label className="block text-sm font-semibold text-slate-200" htmlFor="unlock-password">
        Folder password
      </label>
      <input
        id="unlock-password"
        type="password"
        value={password}
        onChange={(event) => setPassword(event.target.value)}
        className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-slate-100"
      />
      <button
        type="button"
        disabled={busy || !password}
        onClick={async () => {
          setBusy(true);
          await onSubmit(password);
          setBusy(false);
        }}
        className="mt-4 w-full rounded-lg bg-cyan-500 px-4 py-2 text-sm font-bold text-slate-950 hover:bg-cyan-400 disabled:opacity-50"
      >
        Unlock
      </button>
    </DialogShell>
  );
}

function RenameFolderDialog({
  current,
  onCancel,
  onSubmit,
}: {
  current: string;
  onCancel: () => void;
  onSubmit: (name: string) => Promise<void>;
}) {
  const [name, setName] = useState(current);
  const [busy, setBusy] = useState(false);
  return (
    <DialogShell title="Rename Folder" onCancel={onCancel}>
      <label className="block text-sm font-semibold text-slate-200" htmlFor="rename-folder">
        Folder name
      </label>
      <input
        id="rename-folder"
        value={name}
        onChange={(event) => setName(event.target.value)}
        className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-slate-100"
      />
      <button
        type="button"
        disabled={busy || !name.trim()}
        onClick={async () => {
          setBusy(true);
          await onSubmit(name);
          setBusy(false);
        }}
        className="mt-4 w-full rounded-lg bg-cyan-500 px-4 py-2 text-sm font-bold text-slate-950 hover:bg-cyan-400 disabled:opacity-50"
      >
        Rename Folder
      </button>
    </DialogShell>
  );
}

function DeleteFolderDialog({
  name,
  listCount,
  onCancel,
  onSubmit,
}: {
  name: string;
  listCount: number;
  onCancel: () => void;
  onSubmit: (confirmPassword: string) => Promise<void>;
}) {
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  return (
    <DialogShell title={`Delete ${name}`} onCancel={onCancel}>
      <p className="text-sm text-slate-300">
        This permanently deletes <strong>{name}</strong> and the {listCount} list
        {listCount === 1 ? '' : 's'} inside it. Other folders are untouched.
      </p>
      <label className="mt-4 block text-sm font-semibold text-slate-200" htmlFor="confirm-password">
        Re-enter the folder password to confirm
      </label>
      <input
        id="confirm-password"
        type="password"
        value={password}
        onChange={(event) => setPassword(event.target.value)}
        className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-slate-100"
      />
      <button
        type="button"
        disabled={busy || !password}
        onClick={async () => {
          setBusy(true);
          await onSubmit(password);
          setBusy(false);
        }}
        className="mt-4 w-full rounded-lg bg-rose-500 px-4 py-2 text-sm font-bold text-slate-950 hover:bg-rose-400 disabled:opacity-50"
      >
        Delete Folder
      </button>
    </DialogShell>
  );
}

function messageOf(cause: unknown, fallback: string): string {
  if (cause instanceof ApiError) return cause.message;
  return fallback;
}