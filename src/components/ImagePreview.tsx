import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect } from "react";

type ImagePreviewProps = {
  filename: string;
  filepath: string;
  onClose: () => void;
};

export function ImagePreview({ filename, filepath, onClose }: ImagePreviewProps) {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onClose]);

  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/80 p-6" onClick={onClose}>
      <div
        className="max-h-[90vh] w-full max-w-5xl overflow-hidden rounded-xl border border-slate-700 bg-shell-900 shadow-2xl"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="flex items-center justify-between border-b border-slate-700 px-4 py-3">
          <h2 className="truncate text-sm font-semibold text-slate-200">{filename}</h2>
          <button
            type="button"
            onClick={onClose}
            className="rounded border border-slate-600 px-2 py-1 text-xs text-slate-300 transition hover:border-slate-500 hover:text-white"
          >
            ESC
          </button>
        </header>
        <div className="flex max-h-[80vh] items-center justify-center overflow-auto bg-black/30 p-3">
          <img src={convertFileSrc(filepath)} alt={filename} className="max-h-[78vh] w-auto object-contain" />
        </div>
      </div>
    </div>
  );
}