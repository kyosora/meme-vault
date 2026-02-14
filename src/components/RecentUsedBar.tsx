import { convertFileSrc } from "@tauri-apps/api/core";

import type { GalleryImage } from "../lib/tauri";

type RecentUsedBarProps = {
  items: GalleryImage[];
  onOpen: (image: GalleryImage) => void;
  onQuickCopy: (image: GalleryImage) => void;
};

export function RecentUsedBar({ items, onOpen, onQuickCopy }: RecentUsedBarProps) {
  return (
    <section className="rounded-xl border border-slate-700/70 bg-shell-900/80 p-4 shadow-lg shadow-black/20">
      <div className="flex items-center justify-between">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-300">最近使用</h2>
        <span className="text-xs text-slate-500">{items.length}</span>
      </div>

      {items.length === 0 ? (
        <p className="mt-2 text-xs text-slate-500">尚未有最近使用紀錄。</p>
      ) : (
        <div className="mt-3 flex gap-2 overflow-auto pb-1">
          {items.map((image) => {
            const thumbSrc = image.thumbFilePath ? convertFileSrc(image.thumbFilePath) : "";

            return (
              <button
                key={`recent-${image.id}`}
                type="button"
                onClick={() => onOpen(image)}
                onContextMenu={(event) => {
                  event.preventDefault();
                  onQuickCopy(image);
                }}
                title={image.filename}
                className="h-16 w-16 flex-none overflow-hidden rounded border border-slate-700 transition hover:border-accent-400"
              >
                {thumbSrc ? (
                  <img src={thumbSrc} alt={image.filename} className="h-full w-full object-cover" />
                ) : (
                  <span className="text-[10px] text-slate-500">no</span>
                )}
              </button>
            );
          })}
        </div>
      )}
    </section>
  );
}