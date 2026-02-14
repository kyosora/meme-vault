import type { CSSProperties } from "react";

import type { TagItem } from "../lib/tauri";

type TagPanelProps = {
  tags: TagItem[];
  onUseTag: (name: string) => void;
  onDeleteTag?: (tagId: number) => Promise<void>;
};

function buildTagStyle(color: string | null): CSSProperties {
  if (!color) {
    return {};
  }

  return {
    borderColor: color,
    backgroundColor: `${color}22`,
  };
}

export function TagPanel({ tags, onUseTag, onDeleteTag }: TagPanelProps) {
  return (
    <section className="rounded-xl border border-slate-700/70 bg-shell-900/80 p-4 shadow-lg shadow-black/20">
      <div className="flex items-center justify-between">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-300">Tags</h2>
        <span className="text-xs text-slate-400">{tags.length}</span>
      </div>

      <div className="mt-3 flex max-h-64 flex-wrap gap-2 overflow-auto pr-1">
        {tags.length === 0 ? (
          <p className="text-xs text-slate-500">尚未建立標籤</p>
        ) : (
          tags.map((tag) => (
            <div
              key={tag.id}
              style={buildTagStyle(tag.color)}
              className="group flex items-center gap-1 rounded-full border border-slate-600/80 bg-shell-800 px-2 py-1"
            >
              <span
                className="inline-block h-2 w-2 rounded-full"
                style={{ backgroundColor: tag.color ?? "#64748b" }}
              />
              <button
                type="button"
                onClick={() => onUseTag(tag.name)}
                className="text-xs text-slate-200 transition hover:text-white"
                title="加入搜尋"
              >
                {tag.name}
              </button>
              <span className="text-[10px] text-slate-500">{tag.imageCount}</span>
              {onDeleteTag ? (
                <button
                  type="button"
                  onClick={() => void onDeleteTag(tag.id)}
                  className="hidden text-[10px] text-rose-300 transition hover:text-rose-200 group-hover:inline"
                  title="刪除標籤"
                >
                  ×
                </button>
              ) : null}
            </div>
          ))
        )}
      </div>
    </section>
  );
}