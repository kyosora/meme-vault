import { useState, type DragEventHandler } from "react";

import type { ImportProgress, ImportSummary } from "../lib/tauri";

type ImportDropZoneProps = {
  importing: boolean;
  progress: ImportProgress | null;
  summary: ImportSummary | null;
  onImport: (path: string) => Promise<void>;
};

type TauriFile = File & {
  path?: string;
};

function extractDirectoryPath(filePath: string): string {
  const lastSlash = Math.max(filePath.lastIndexOf("\\"), filePath.lastIndexOf("/"));
  if (lastSlash <= 0) {
    return filePath;
  }
  return filePath.slice(0, lastSlash);
}

export function ImportDropZone({ importing, progress, summary, onImport }: ImportDropZoneProps) {
  const [inputPath, setInputPath] = useState("");
  const [isDragging, setIsDragging] = useState(false);

  const disabled = importing || inputPath.trim().length === 0;

  const progressPercent =
    !progress || progress.total === 0
      ? 0
      : Math.min(100, Math.round((progress.current / progress.total) * 100));

  const handleDrop: DragEventHandler<HTMLDivElement> = (event) => {
    event.preventDefault();
    setIsDragging(false);

    const firstFile = event.dataTransfer.files.item(0) as TauriFile | null;
    if (!firstFile?.path) {
      return;
    }

    setInputPath(extractDirectoryPath(firstFile.path));
  };

  const handleImportClick = async () => {
    const trimmed = inputPath.trim();
    if (!trimmed) {
      return;
    }

    await onImport(trimmed);
  };

  return (
    <section className="rounded-xl border border-slate-700/70 bg-shell-900/80 p-4 shadow-lg shadow-black/20">
      <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-300">Import</h2>
      <div
        onDragOver={(event) => {
          event.preventDefault();
          setIsDragging(true);
        }}
        onDragLeave={() => setIsDragging(false)}
        onDrop={handleDrop}
        className={`mt-3 rounded-lg border border-dashed p-4 transition ${
          isDragging ? "border-accent-400 bg-accent-400/10" : "border-slate-600 bg-shell-800/50"
        }`}
      >
        <p className="text-sm text-slate-200">把資料夾拖進來，或直接貼上資料夾路徑。</p>
        <p className="mt-1 text-xs text-slate-400">支援 png / jpg / jpeg / gif / webp</p>
      </div>

      <div className="mt-3 flex flex-col gap-2 md:flex-row">
        <input
          type="text"
          value={inputPath}
          onChange={(event) => setInputPath(event.target.value)}
          placeholder="例如：D:\\Memes"
          className="w-full rounded-lg border border-slate-600 bg-shell-800 px-3 py-2 font-mono text-sm text-slate-100 outline-none ring-accent-400/70 transition focus:ring"
        />
        <button
          type="button"
          disabled={disabled}
          onClick={() => void handleImportClick()}
          className="rounded-lg border border-accent-400/60 bg-accent-400/10 px-4 py-2 text-sm font-semibold text-accent-400 transition hover:bg-accent-400/20 disabled:cursor-not-allowed disabled:opacity-40"
        >
          {importing ? "Importing..." : "Import folder"}
        </button>
      </div>

      {progress ? (
        <div className="mt-4 space-y-2">
          <div className="h-2 overflow-hidden rounded-full bg-shell-800">
            <div
              className="h-full rounded-full bg-accent-400 transition-all"
              style={{ width: `${progressPercent}%` }}
            />
          </div>
          <p className="text-xs text-slate-300">
            {progress.current}/{progress.total} imported:{progress.imported} skipped:{progress.skipped} failed:{progress.failed}
          </p>
        </div>
      ) : null}

      {summary ? (
        <p className="mt-3 text-xs text-slate-300">
          完成：imported {summary.imported} / skipped {summary.skipped} / failed {summary.failed}
        </p>
      ) : null}
    </section>
  );
}