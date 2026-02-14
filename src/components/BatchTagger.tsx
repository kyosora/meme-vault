import { useState } from "react";

type BatchTaggerProps = {
  selectedCount: number;
  onAddTags: (value: string) => Promise<void>;
  onRemoveTags: (value: string) => Promise<void>;
};

export function BatchTagger({ selectedCount, onAddTags, onRemoveTags }: BatchTaggerProps) {
  const [addValue, setAddValue] = useState("");
  const [removeValue, setRemoveValue] = useState("");

  return (
    <section className="rounded-xl border border-slate-700/70 bg-shell-900/80 p-4 shadow-lg shadow-black/20">
      <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-300">Batch Tagger</h2>
      <p className="mt-1 text-xs text-slate-400">目前選取 {selectedCount} 張。支援空白或逗號分隔多個標籤。</p>

      <div className="mt-3 space-y-2">
        <div className="flex gap-2">
          <input
            type="text"
            value={addValue}
            onChange={(event) => setAddValue(event.target.value)}
            placeholder="批次加標籤"
            disabled={selectedCount === 0}
            className="w-full rounded-lg border border-slate-600 bg-shell-800 px-3 py-2 text-sm text-slate-100 outline-none ring-accent-400/70 transition focus:ring"
          />
          <button
            type="button"
            onClick={() => {
              if (!addValue.trim()) {
                return;
              }
              void onAddTags(addValue);
              setAddValue("");
            }}
            disabled={selectedCount === 0 || !addValue.trim()}
            className="rounded-lg border border-accent-400/60 bg-accent-400/10 px-3 py-2 text-sm font-semibold text-accent-400 transition hover:bg-accent-400/20 disabled:opacity-40"
          >
            加上
          </button>
        </div>

        <div className="flex gap-2">
          <input
            type="text"
            value={removeValue}
            onChange={(event) => setRemoveValue(event.target.value)}
            placeholder="批次移除標籤"
            disabled={selectedCount === 0}
            className="w-full rounded-lg border border-slate-600 bg-shell-800 px-3 py-2 text-sm text-slate-100 outline-none ring-rose-400/70 transition focus:ring"
          />
          <button
            type="button"
            onClick={() => {
              if (!removeValue.trim()) {
                return;
              }
              void onRemoveTags(removeValue);
              setRemoveValue("");
            }}
            disabled={selectedCount === 0 || !removeValue.trim()}
            className="rounded-lg border border-rose-400/60 bg-rose-400/10 px-3 py-2 text-sm font-semibold text-rose-300 transition hover:bg-rose-400/20 disabled:opacity-40"
          >
            移除
          </button>
        </div>
      </div>
    </section>
  );
}