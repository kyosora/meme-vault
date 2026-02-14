import type { RefObject } from "react";

type SearchBarProps = {
  value: string;
  inputRef?: RefObject<HTMLInputElement | null>;
  onChange: (value: string) => void;
  onClear: () => void;
};

export function SearchBar({ value, inputRef, onChange, onClear }: SearchBarProps) {
  return (
    <section className="rounded-xl border border-slate-700/70 bg-shell-900/80 p-4 shadow-lg shadow-black/20">
      <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-300">Search</h2>
      <input
        ref={inputRef}
        type="text"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder="例如：貓 + 開心 | dog -難過"
        className="mt-3 w-full rounded-lg border border-slate-600 bg-shell-800 px-3 py-2 text-sm text-slate-100 outline-none ring-accent-400/70 transition focus:ring"
      />
      <p className="mt-2 text-xs text-slate-400">空白或 + = AND，| = OR，-tag = 排除</p>
      {value.trim() ? (
        <button
          type="button"
          onClick={onClear}
          className="mt-2 rounded border border-slate-600 px-2 py-1 text-xs text-slate-300 transition hover:border-slate-500"
        >
          清除搜尋
        </button>
      ) : null}
    </section>
  );
}