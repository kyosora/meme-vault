import { useMemo, useState, type CSSProperties } from "react";

import type { TagItem } from "../lib/tauri";

type TagInputProps = {
  selectedCount: number;
  currentTags: TagItem[];
  allTags: TagItem[];
  onAddTags: (value: string) => Promise<void>;
  onRemoveTag: (name: string) => Promise<void>;
};

function buildTagStyle(color: string | null): CSSProperties {
  if (!color) {
    return {};
  }

  return {
    borderColor: color,
    backgroundColor: `${color}1f`,
  };
}

export function TagInput({
  selectedCount,
  currentTags,
  allTags,
  onAddTags,
  onRemoveTag,
}: TagInputProps) {
  const [value, setValue] = useState("");

  const suggestions = useMemo(() => {
    const keyword = value.trim().toLowerCase();
    if (!keyword) {
      return allTags.slice(0, 10);
    }

    return allTags
      .filter((tag) => tag.name.toLowerCase().includes(keyword))
      .slice(0, 10);
  }, [allTags, value]);

  const submit = async () => {
    const trimmed = value.trim();
    if (!trimmed || selectedCount === 0) {
      return;
    }

    await onAddTags(trimmed);
    setValue("");
  };

  return (
    <section className="rounded-xl border border-slate-700/70 bg-shell-900/80 p-4 shadow-lg shadow-black/20">
      <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-300">Tag Input</h2>
      <p className="mt-1 text-xs text-slate-400">選取 {selectedCount} 張。可輸入多個標籤，以空白或逗號分隔。</p>

      <div className="mt-3 flex gap-2">
        <input
          type="text"
          value={value}
          onChange={(event) => setValue(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              void submit();
            }

            if (event.key === "Backspace" && !value.trim() && currentTags.length > 0 && selectedCount > 0) {
              const lastTag = currentTags[currentTags.length - 1];
              void onRemoveTag(lastTag.name);
            }
          }}
          placeholder="例如：reaction cat"
          className="w-full rounded-lg border border-slate-600 bg-shell-800 px-3 py-2 text-sm text-slate-100 outline-none ring-accent-400/70 transition focus:ring"
          disabled={selectedCount === 0}
        />
        <button
          type="button"
          onClick={() => void submit()}
          disabled={selectedCount === 0 || !value.trim()}
          className="rounded-lg border border-accent-400/60 bg-accent-400/10 px-3 py-2 text-sm font-semibold text-accent-400 transition hover:bg-accent-400/20 disabled:cursor-not-allowed disabled:opacity-40"
        >
          加上
        </button>
      </div>

      <div className="mt-3 flex flex-wrap gap-2">
        {currentTags.map((tag) => (
          <button
            key={tag.id}
            type="button"
            onClick={() => void onRemoveTag(tag.name)}
            disabled={selectedCount === 0}
            style={buildTagStyle(tag.color)}
            className="rounded-full border border-slate-600 px-2 py-1 text-xs text-slate-200 transition hover:border-rose-400 hover:text-rose-200 disabled:opacity-40"
            title="移除這個標籤"
          >
            {tag.name}
          </button>
        ))}
      </div>

      <div className="mt-3 flex flex-wrap gap-2">
        {suggestions.map((tag) => (
          <button
            key={`suggestion-${tag.id}`}
            type="button"
            onClick={() => setValue(tag.name)}
            style={buildTagStyle(tag.color)}
            className="rounded-full border border-slate-700 px-2 py-1 text-[11px] text-slate-300 transition hover:border-slate-500"
          >
            {tag.name}
          </button>
        ))}
      </div>
    </section>
  );
}