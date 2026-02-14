import { convertFileSrc } from "@tauri-apps/api/core";
import { useVirtualizer } from "@tanstack/react-virtual";
import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type MouseEvent,
  type MouseEventHandler,
  type UIEventHandler,
} from "react";

import type { GalleryImage } from "../lib/tauri";

type ImageGridProps = {
  images: GalleryImage[];
  loading: boolean;
  selectedIds: Set<number>;
  onSelect: (image: GalleryImage, index: number, event: MouseEvent<HTMLButtonElement>) => void;
  onOpenPreview: (image: GalleryImage) => void;
  onQuickCopy: (image: GalleryImage) => void;
  onColumnsChange?: (columns: number) => void;
  onReachEnd?: () => void;
};

const CARD_WIDTH = 180;
const CARD_THUMB_HEIGHT = 180;
const CARD_HEIGHT = 240;
const GAP = 12;
const END_REACH_THRESHOLD = 300;

export function ImageGrid({
  images,
  loading,
  selectedIds,
  onSelect,
  onOpenPreview,
  onQuickCopy,
  onColumnsChange,
  onReachEnd,
}: ImageGridProps) {
  const parentRef = useRef<HTMLDivElement | null>(null);
  const [columns, setColumns] = useState(4);

  useEffect(() => {
    const container = parentRef.current;
    if (!container) {
      return;
    }

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      const nextColumns = Math.max(1, Math.floor(entry.contentRect.width / (CARD_WIDTH + GAP)));
      setColumns(nextColumns);
      onColumnsChange?.(nextColumns);
    });

    observer.observe(container);
    return () => observer.disconnect();
  }, [onColumnsChange]);

  const rowCount = useMemo(() => Math.ceil(images.length / columns), [images.length, columns]);

  const virtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => parentRef.current,
    estimateSize: () => CARD_HEIGHT + GAP,
    overscan: 3,
  });

  const virtualRows = virtualizer.getVirtualItems();

  const handleScroll: UIEventHandler<HTMLDivElement> = () => {
    if (!onReachEnd) {
      return;
    }

    const container = parentRef.current;
    if (!container) {
      return;
    }

    const distanceToBottom = container.scrollHeight - container.scrollTop - container.clientHeight;
    if (distanceToBottom < END_REACH_THRESHOLD) {
      onReachEnd();
    }
  };

  if (!loading && images.length === 0) {
    return (
      <section className="flex h-[65vh] items-center justify-center rounded-xl border border-slate-700/70 bg-shell-900/80 p-6 text-center text-sm text-slate-400">
        沒有符合條件的圖片。
      </section>
    );
  }

  return (
    <section className="rounded-xl border border-slate-700/70 bg-shell-900/80 p-3 shadow-lg shadow-black/20">
      <div ref={parentRef} onScroll={handleScroll} className="h-[65vh] overflow-auto rounded-lg bg-shell-950/50">
        <div style={{ height: virtualizer.getTotalSize(), position: "relative" }}>
          {virtualRows.map((virtualRow) => {
            const startIndex = virtualRow.index * columns;
            const rowItems = images.slice(startIndex, startIndex + columns);

            return (
              <div
                key={virtualRow.index}
                className="absolute left-0 top-0 grid w-full px-3"
                style={{
                  transform: `translateY(${virtualRow.start}px)`,
                  gap: GAP,
                  gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
                }}
              >
                {rowItems.map((image, rowIndex) => {
                  const absoluteIndex = startIndex + rowIndex;
                  const isSelected = selectedIds.has(image.id);
                  const thumbSrc = image.thumbFilePath ? convertFileSrc(image.thumbFilePath) : "";

                  const handleClick: MouseEventHandler<HTMLButtonElement> = (event) => {
                    onSelect(image, absoluteIndex, event);
                  };

                  return (
                    <button
                      key={image.id}
                      type="button"
                      onClick={handleClick}
                      onDoubleClick={() => onOpenPreview(image)}
                      onContextMenu={(event) => {
                        event.preventDefault();
                        onQuickCopy(image);
                      }}
                      style={{ height: CARD_HEIGHT }}
                      title="Double click: 預覽 / Right click: 複製到剪貼簿"
                      className={`flex flex-col overflow-hidden rounded-lg border text-left transition ${
                        isSelected
                          ? "border-accent-400 bg-accent-400/10"
                          : "border-slate-700/70 bg-shell-900 hover:border-slate-500"
                      }`}
                    >
                      <div className="relative w-full bg-shell-800" style={{ height: CARD_THUMB_HEIGHT }}>
                        {thumbSrc ? (
                          <img
                            src={thumbSrc}
                            alt={image.filename}
                            loading="lazy"
                            className="h-full w-full object-cover"
                          />
                        ) : (
                          <div className="flex h-full items-center justify-center text-xs text-slate-500">
                            no thumb
                          </div>
                        )}
                      </div>
                      <div className="p-2">
                        <p className="truncate text-xs font-medium text-slate-100">{image.filename}</p>
                        <p className="mt-1 text-[11px] text-slate-400">{Math.round(image.fileSize / 1024)} KB</p>
                      </div>
                    </button>
                  );
                })}
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}