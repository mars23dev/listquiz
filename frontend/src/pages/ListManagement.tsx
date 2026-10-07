import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  ApiError,
  createFolder as createFolderRequest,
  createList as createListRequest,
  deleteFolder as deleteFolderRequest,
  deleteList as deleteListRequest,
  exportListCsv,
  fetchFolders,
  fetchFolderLists,
  fetchList,
  importListCsv,
  renameFolder as renameFolderRequest,
  unlockFolder,
  updateList as updateListRequest,
  type FolderSummary,
} from '../api/client';
import { IconPickerModal } from '../components/IconPickerModal';
import { ItemIcon } from '../components/ItemIcon';
import type {
  ImportReport,
  ListDetail,
  ListItemView,
  ListSummary,
} from '../types/session';

interface ListManagementProps {
  onBackToHome: () => void;
}

type Dialog =
  | { kind: 'create' }
  | { kind: 'unlock'; folderId: string }
  | { kind: 'rename'; folderId: string; current: string }
  | { kind: 'delete'; folderId: string; name: string }
  | null;

interface EditorState {
  folderId: string;
  password: string;
  mode: 'create' | 'edit';
  draft: ListDetail;
}

interface CsvState {
  folderId: string;
  password: string;
}

function emptyDraft(folderId: string): ListDetail {
  const now = Date.now();
  return {
    id: '',
    folderId,
    title: '',
    description: '',
    orderDirection: 'low_to_high',
    columns: [{ id: 'value', name: 'Value' }],
    itemCount: 0,
    isDefault: false,
    items: Array.from({ length: 11 }, (_, index) => ({
      id: `tmp-${index}`,
      name: '',
      iconName: null,
      iconImage: null,
      position: index,
      values: { value: { display: '', numeric: 0, wasDefaulted: false } },
    })),
  };
}

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
  const [editor, setEditor] = useState<EditorState | null>(null);
  const [csv, setCsv] = useState<CsvState | null>(null);
  const [busy, setBusy] = useState(false);

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

  function toPayload(draft: ListDetail) {
    return {
      title: draft.title,
      description: draft.description,
      orderDirection: draft.orderDirection,
      columns: draft.columns.map((column) => ({ id: column.id, name: column.name })),
      items: draft.items.map((item) => ({
        id: item.id,
        name: item.name,
        iconName: item.iconName,
        iconImage: item.iconImage,
        position: item.position,
        values: Object.fromEntries(
          Object.entries(item.values).map(([columnId, value]) => [
            columnId,
            {
              display: value.display,
              numeric: Number.isFinite(value.numeric)
                ? value.numeric
                : parseDisplayNumber(value.display),
              wasDefaulted: value.wasDefaulted,
            },
          ]),
        ),
      })),
    };
  }

  async function openEditor(list: ListSummary) {
    if (!activeId) return;
    setBusy(true);
    try {
      const detail = await fetchList(list.id, passwords[activeId]);
      setEditor({
        folderId: activeId,
        password: passwords[activeId] ?? '',
        mode: 'edit',
        draft: detail,
      });
    } catch (cause) {
      setError(messageOf(cause, 'The list could not be opened.'));
    } finally {
      setBusy(false);
    }
  }

  function newList() {
    if (!activeId) return;
    setEditor({
      folderId: activeId,
      password: passwords[activeId] ?? '',
      mode: 'create',
      draft: emptyDraft(activeId),
    });
  }

  async function saveEditor(draft: ListDetail) {
    if (!editor) return;
    setBusy(true);
    try {
      const payload = toPayload(draft);
      if (editor.mode === 'create') {
        await createListRequest(editor.folderId, payload, editor.password);
      } else {
        await updateListRequest(editor.draft.id, payload, editor.password);
      }
      setEditor(null);
      await loadLists();
    } catch (cause) {
      setError(messageOf(cause, 'The list could not be saved.'));
    } finally {
      setBusy(false);
    }
  }

  async function duplicateEditor(draft: ListDetail) {
    if (!editor) return;
    setBusy(true);
    try {
      await createListRequest(
        editor.folderId,
        { ...toPayload(draft), title: `${draft.title} (Copy)` },
        editor.password,
      );
      setEditor(null);
      await loadLists();
    } catch (cause) {
      setError(messageOf(cause, 'The list could not be duplicated.'));
    } finally {
      setBusy(false);
    }
  }

  async function deleteList(list: ListSummary) {
    if (!activeId) return;
    setBusy(true);
    try {
      await deleteListRequest(list.id, passwords[activeId]);
      await loadLists();
    } catch (cause) {
      setError(messageOf(cause, 'The list could not be deleted.'));
    } finally {
      setBusy(false);
    }
  }

  async function duplicateList(list: ListSummary) {
    if (!activeId) return;
    setBusy(true);
    try {
      const detail = await fetchList(list.id, passwords[activeId]);
      await createListRequest(
        activeId,
        { ...toPayload(detail), title: `${detail.title} (Copy)` },
        passwords[activeId] ?? '',
      );
      await loadLists();
    } catch (cause) {
      setError(messageOf(cause, 'The list could not be duplicated.'));
    } finally {
      setBusy(false);
    }
  }

  async function exportList(list: ListSummary) {
    if (!activeId) return;
    try {
      const csvText = await exportListCsv(list.id, passwords[activeId]);
      const blob = new Blob([csvText], { type: 'text/csv' });
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement('a');
      anchor.href = url;
      anchor.download = `${list.title}.csv`;
      anchor.click();
      URL.revokeObjectURL(url);
    } catch (cause) {
      setError(messageOf(cause, 'The list could not be exported.'));
    }
  }

  async function importCsv(title: string, direction: string, body: string): Promise<ImportReport | null> {
    if (!csv) return null;
    try {
      const answer = await importListCsv(
        csv.folderId,
        { title, orderDirection: direction, csv: body },
        csv.password,
      );
      await loadLists();
      return answer.report;
    } catch (cause) {
      setError(messageOf(cause, 'The CSV could not be imported.'));
      return null;
    }
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
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => setDialog({ kind: 'create' })}
            className="rounded-lg bg-cyan-500 px-3.5 py-1.5 text-xs font-bold text-slate-950 hover:bg-cyan-400"
          >
            + New Folder
          </button>
        </div>
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

      {active && isUnlocked && !active.isDefault ? (
        <div className="mb-5 flex flex-wrap gap-2">
          <button
            type="button"
            onClick={newList}
            className="rounded-lg bg-cyan-500 px-3.5 py-1.5 text-xs font-bold text-slate-950 hover:bg-cyan-400"
          >
            Create List
          </button>
          <button
            type="button"
            onClick={() => setCsv({ folderId: active.id, password: activePassword })}
            className="rounded-lg border border-slate-700 bg-slate-900 px-3.5 py-1.5 text-xs font-semibold text-slate-200 hover:bg-slate-800"
          >
            Import CSV
          </button>
        </div>
      ) : null}

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
        <ListGrid
          lists={lists}
          editable={Boolean(active && !active.isDefault)}
          onEdit={openEditor}
          onDuplicate={duplicateList}
          onDelete={deleteList}
          onExport={exportList}
        />
      )}

      {editor ? (
        <ListEditor
          draft={editor.draft}
          mode={editor.mode}
          busy={busy}
          onCancel={() => setEditor(null)}
          onSave={saveEditor}
          onDuplicate={duplicateEditor}
        />
      ) : null}

      {csv ? (
        <CsvImportModal
          onCancel={() => setCsv(null)}
          onSubmit={(title, direction, body) => importCsv(title, direction, body)}
          onDone={() => setCsv(null)}
        />
      ) : null}

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

function ListGrid({
  lists,
  editable,
  onEdit,
  onDuplicate,
  onDelete,
  onExport,
}: {
  lists: ListSummary[];
  editable: boolean;
  onEdit: (list: ListSummary) => void;
  onDuplicate: (list: ListSummary) => void;
  onDelete: (list: ListSummary) => void;
  onExport: (list: ListSummary) => void;
}) {
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
          {editable && !list.isDefault ? (
            <div className="mt-3 flex flex-wrap gap-2">
              <button
                type="button"
                onClick={() => onEdit(list)}
                className="rounded-lg border border-slate-700 px-2.5 py-1 text-xs font-semibold text-slate-200 hover:border-cyan-400"
              >
                Edit
              </button>
              <button
                type="button"
                onClick={() => onDuplicate(list)}
                className="rounded-lg border border-slate-700 px-2.5 py-1 text-xs font-semibold text-slate-200 hover:border-cyan-400"
              >
                Duplicate List
              </button>
              <button
                type="button"
                onClick={() => onExport(list)}
                className="rounded-lg border border-slate-700 px-2.5 py-1 text-xs font-semibold text-slate-200 hover:border-cyan-400"
              >
                Export CSV
              </button>
              <button
                type="button"
                onClick={() => onDelete(list)}
                className="rounded-lg border border-rose-500/50 px-2.5 py-1 text-xs font-semibold text-rose-300 hover:border-rose-400"
              >
                Delete List
              </button>
            </div>
          ) : null}
        </article>
      ))}
    </section>
  );
}

const MIN_ITEMS = 11;
const MAX_ITEMS = 100;

function parseDisplayNumber(display: string): number {
  const cleaned = display.replace(/[^0-9.\-]/g, '');
  const value = Number.parseFloat(cleaned);
  return Number.isFinite(value) ? value : 0;
}

function slugify(name: string): string {
  const slug = name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '');
  return slug || 'value';
}

function ListEditor({
  draft,
  mode,
  busy,
  onCancel,
  onSave,
  onDuplicate,
}: {
  draft: ListDetail;
  mode: 'create' | 'edit';
  busy: boolean;
  onCancel: () => void;
  onSave: (draft: ListDetail) => Promise<void>;
  onDuplicate: (draft: ListDetail) => Promise<void>;
}) {
  const [title, setTitle] = useState(draft.title);
  const [direction, setDirection] = useState(draft.orderDirection);
  const [columns, setColumns] = useState(draft.columns);
  const [items, setItems] = useState(draft.items);
  const [dragIndex, setDragIndex] = useState<number | null>(null);
  const [iconTarget, setIconTarget] = useState<number | null>(null);

  const problem = !title.trim()
    ? 'A list title is required.'
    : items.length < MIN_ITEMS || items.length > MAX_ITEMS
      ? `A list needs between ${MIN_ITEMS} and ${MAX_ITEMS} items; this one has ${items.length}.`
      : null;

  function current(): ListDetail {
    return {
      ...draft,
      title: title.trim(),
      orderDirection: direction,
      columns,
      items,
      itemCount: items.length,
    };
  }

  function updateItem(index: number, patch: Partial<ListItemView>) {
    setItems((current) =>
      current.map((item, position) =>
        position === index ? { ...item, ...patch } : item,
      ),
    );
  }

  function updateValue(index: number, columnId: string, display: string) {
    setItems((current) =>
      current.map((item, position) =>
        position === index
          ? {
              ...item,
              values: {
                ...item.values,
                [columnId]: {
                  display,
                  numeric: parseDisplayNumber(display),
                  wasDefaulted: false,
                },
              },
            }
          : item,
      ),
    );
  }

  function addRow() {
    setItems((current) => [
      ...current,
      {
        id: `tmp-${current.length}-${Date.now()}`,
        name: '',
        iconName: null,
        iconImage: null,
        position: current.length,
        values: Object.fromEntries(
          columns.map((column) => [
            column.id,
            { display: '', numeric: 0, wasDefaulted: false },
          ]),
        ),
      },
    ]);
  }

  function removeRow(index: number) {
    setItems((current) =>
      current
        .filter((_, position) => position !== index)
        .map((item, position) => ({ ...item, position })),
    );
  }

  function addColumn() {
    const id = `column_${columns.length}`;
    setColumns((current) => [...current, { id, name: 'Value' }]);
    setItems((current) =>
      current.map((item) => ({
        ...item,
        values: {
          ...item.values,
          [id]: { display: '', numeric: 0, wasDefaulted: false },
        },
      })),
    );
  }

  function renameColumn(index: number, name: string) {
    setColumns((current) =>
      current.map((column, position) => (position === index ? { ...column, name } : column)),
    );
  }

  function removeColumn(index: number) {
    if (columns.length <= 1) return;
    const removed = columns[index];
    setColumns((current) => current.filter((_, position) => position !== index));
    setItems((current) =>
      current.map((item) => {
        const values = { ...item.values };
        delete values[removed.id];
        return { ...item, values };
      }),
    );
  }

  function reorder(from: number, to: number) {
    if (from === to) return;
    setItems((current) => {
      const next = [...current];
      const [moved] = next.splice(from, 1);
      next.splice(to, 0, moved);
      return next.map((item, position) => ({ ...item, position }));
    });
  }

  return (
    <div className="fixed inset-0 z-40 flex items-start justify-center overflow-auto bg-slate-950/85 p-4">
      <div
        role="dialog"
        aria-label="List Form & Table Editor"
        className="w-full max-w-4xl rounded-2xl border border-slate-800 bg-slate-900 p-6 shadow-2xl"
      >
        <h2 className="font-display text-lg font-bold text-white">List Form &amp; Table Editor</h2>
        <p className="mt-1 text-xs text-amber-300">
          The data is not protected; do not add sensitive data.
        </p>

        <div className="mt-4 grid gap-4 sm:grid-cols-2">
          <label className="block text-sm font-semibold text-slate-200">
            List Title (Mandatory)
            <input
              value={title}
              onChange={(event) => setTitle(event.target.value)}
              className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-slate-100"
            />
          </label>
          <label className="block text-sm font-semibold text-slate-200">
            Order Direction
            <select
              value={direction}
              onChange={(event) =>
                setDirection(event.target.value as ListDetail['orderDirection'])
              }
              className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-slate-100"
            >
              <option value="low_to_high">Low to High (Ascending / Earliest to Latest)</option>
              <option value="high_to_low">High to Low (Descending / Highest to Lowest)</option>
            </select>
          </label>
        </div>

        <div className="mt-5">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-bold text-white">Table Columns</h3>
            <button
              type="button"
              onClick={addColumn}
              className="rounded-lg border border-slate-700 px-3 py-1 text-xs font-semibold text-slate-200"
            >
              Add Value Column
            </button>
          </div>
          <div className="mt-2 flex flex-wrap gap-2">
            {columns.map((column, index) => (
              <div key={column.id} className="flex items-center gap-1">
                <input
                  value={column.name}
                  aria-label={`Column ${index + 1} name`}
                  onChange={(event) => renameColumn(index, event.target.value)}
                  className="w-32 rounded-lg border border-slate-700 bg-slate-950 px-2 py-1 text-xs text-slate-100"
                />
                {columns.length > 1 ? (
                  <button
                    type="button"
                    onClick={() => removeColumn(index)}
                    aria-label={`Remove column ${index + 1}`}
                    className="text-xs text-rose-300"
                  >
                    ×
                  </button>
                ) : null}
              </div>
            ))}
          </div>
        </div>

        <p className="mt-4 text-xs text-slate-400">
          Drag the grip icon on the left to rearrange rows
        </p>
        <div className="mt-2 overflow-x-auto">
          <table className="w-auto border-collapse text-sm">
            <thead>
              <tr className="text-left text-xs uppercase text-slate-400">
                <th className="p-1" />
                <th className="p-1">Name</th>
                {columns.map((column) => (
                  <th key={column.id} className="p-1">
                    {column.name}
                  </th>
                ))}
                <th className="p-1" />
              </tr>
            </thead>
            <tbody>
              {items.map((item, index) => (
                <tr
                  key={item.id}
                  draggable
                  onDragStart={() => setDragIndex(index)}
                  onDragOver={(event) => event.preventDefault()}
                  onDrop={() => {
                    if (dragIndex !== null) reorder(dragIndex, index);
                    setDragIndex(null);
                  }}
                  className="border-t border-slate-800"
                >
                  <td className="p-1">
                    <button
                      type="button"
                      aria-label={`Reorder row ${index + 1}`}
                      className="cursor-grab text-slate-500"
                    >
                      ⠿
                    </button>
                  </td>
                  <td className="p-1">
                    <div className="flex items-center gap-1">
                      <button
                        type="button"
                        onClick={() => setIconTarget(index)}
                        aria-label={`Choose icon for row ${index + 1}`}
                        className="flex h-8 w-8 items-center justify-center rounded border border-slate-700 text-slate-300 hover:border-cyan-400"
                      >
                        <ItemIcon
                          name={item.iconName ?? undefined}
                          customIconUrl={item.iconImage ?? undefined}
                          className="h-4 w-4"
                        />
                      </button>
                      <input
                        value={item.name}
                        aria-label={`Row ${index + 1} name`}
                        onChange={(event) => updateItem(index, { name: event.target.value })}
                        className="w-40 rounded border border-slate-700 bg-slate-950 px-2 py-1 text-slate-100"
                      />
                    </div>
                  </td>
                  {columns.map((column) => (
                    <td key={column.id} className="p-1">
                      <input
                        value={item.values[column.id]?.display ?? ''}
                        aria-label={`Row ${index + 1} ${column.name}`}
                        onChange={(event) => updateValue(index, column.id, event.target.value)}
                        className="w-28 rounded border border-slate-700 bg-slate-950 px-2 py-1 text-slate-100"
                      />
                    </td>
                  ))}
                  <td className="p-1">
                    <button
                      type="button"
                      onClick={() => removeRow(index)}
                      aria-label={`Remove row ${index + 1}`}
                      className="text-xs text-rose-300"
                    >
                      ×
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <div className="mt-3 flex items-center justify-between">
          <button
            type="button"
            onClick={addRow}
            className="rounded-lg border border-slate-700 px-3 py-1.5 text-xs font-semibold text-slate-200"
          >
            Add Row
          </button>
          <span className="font-mono text-xs text-slate-400">
            {items.length} / {MAX_ITEMS} items
          </span>
        </div>

        {problem ? (
          <p role="alert" className="mt-3 text-xs font-semibold text-rose-300">
            {problem}
          </p>
        ) : null}

        <div className="mt-5 flex flex-wrap justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            className="rounded-lg border border-slate-700 px-4 py-2 text-sm font-semibold text-slate-300"
          >
            Cancel
          </button>
          {mode === 'edit' ? (
            <button
              type="button"
              onClick={() => void onDuplicate(current())}
              className="rounded-lg border border-slate-700 px-4 py-2 text-sm font-semibold text-slate-200"
            >
              Duplicate List
            </button>
          ) : null}
          <button
            type="button"
            disabled={busy || problem !== null}
            onClick={() => void onSave(current())}
            className="rounded-lg bg-cyan-500 px-4 py-2 text-sm font-bold text-slate-950 hover:bg-cyan-400 disabled:opacity-50"
          >
            Save List
          </button>
        </div>

        <IconPickerModal
          isOpen={iconTarget !== null}
          currentIcon={iconTarget !== null ? items[iconTarget]?.iconName ?? undefined : undefined}
          onClose={() => setIconTarget(null)}
          onSelectIcon={(iconName) => {
            if (iconTarget !== null) {
              updateItem(iconTarget, { iconName, iconImage: null });
            }
            setIconTarget(null);
          }}
        />
      </div>
    </div>
  );
}

function CsvImportModal({
  onCancel,
  onSubmit,
  onDone,
}: {
  onCancel: () => void;
  onSubmit: (title: string, direction: string, body: string) => Promise<ImportReport | null>;
  onDone: () => void;
}) {
  const [title, setTitle] = useState('Imported List');
  const [direction, setDirection] = useState('low_to_high');
  const [body, setBody] = useState('');
  const [busy, setBusy] = useState(false);
  const [report, setReport] = useState<ImportReport | null>(null);
  const [problem, setProblem] = useState<string | null>(null);

  function readFile(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => setBody(String(reader.result ?? ''));
    reader.readAsText(file);
  }

  async function submit() {
    setBusy(true);
    setProblem(null);
    const result = await onSubmit(title, direction, body);
    setBusy(false);
    if (result) {
      setReport(result);
    } else {
      setProblem('The CSV could not be imported.');
    }
  }

  return (
    <div className="fixed inset-0 z-40 flex items-start justify-center overflow-auto bg-slate-950/85 p-4">
      <div
        role="dialog"
        aria-label="Import List from CSV"
        className="w-full max-w-xl rounded-2xl border border-slate-800 bg-slate-900 p-6 shadow-2xl"
      >
        <h2 className="font-display text-lg font-bold text-white">Import List from CSV</h2>
        <p className="mt-1 text-xs text-slate-400">
          The data is not protected; do not add sensitive data.
        </p>

        <label className="mt-4 block text-sm font-semibold text-slate-200">
          List title
          <input
            value={title}
            onChange={(event) => setTitle(event.target.value)}
            className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-slate-100"
          />
        </label>
        <label className="mt-3 block text-sm font-semibold text-slate-200">
          Order direction
          <select
            value={direction}
            onChange={(event) => setDirection(event.target.value)}
            className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-slate-100"
          >
            <option value="low_to_high">Low to High (Ascending / Earliest to Latest)</option>
            <option value="high_to_low">High to Low (Descending / Highest to Lowest)</option>
          </select>
        </label>

        <p className="mt-4 text-xs font-semibold text-slate-300">
          Upload .csv File OR Paste CSV Below
        </p>
        <input type="file" accept=".csv,text/csv" onChange={readFile} className="mt-1 text-xs" />
        <textarea
          value={body}
          onChange={(event) => setBody(event.target.value)}
          rows={6}
          placeholder={'Icon,Name,Year,Value2\nRocket,Apollo 11,1969,1'}
          className="mt-2 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 font-mono text-xs text-slate-100"
        />

        {problem ? (
          <p role="alert" className="mt-3 text-xs font-semibold text-rose-300">
            {problem}
          </p>
        ) : null}

        {report ? (
          <div className="mt-3 rounded-lg border border-slate-700 bg-slate-950 p-3 text-xs text-slate-300">
            <p>
              Imported {report.rowCount} rows.
              {report.defaultedRows.length > 0
                ? ` ${report.defaultedRows.length} value(s) defaulted.`
                : ''}
            </p>
            {report.warnings.map((warning) => (
              <p key={warning} className="mt-1 text-amber-300">
                {warning}
              </p>
            ))}
          </div>
        ) : null}

        <div className="mt-5 flex justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            className="rounded-lg border border-slate-700 px-4 py-2 text-sm font-semibold text-slate-300"
          >
            Cancel
          </button>
          {report ? (
            <button
              type="button"
              onClick={onDone}
              className="rounded-lg bg-cyan-500 px-4 py-2 text-sm font-bold text-slate-950 hover:bg-cyan-400"
            >
              Done
            </button>
          ) : (
            <button
              type="button"
              disabled={busy || !title.trim() || !body.trim()}
              onClick={() => void submit()}
              className="rounded-lg bg-cyan-500 px-4 py-2 text-sm font-bold text-slate-950 hover:bg-cyan-400 disabled:opacity-50"
            >
              Import List
            </button>
          )}
        </div>
      </div>
    </div>
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