import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useMemo, useRef, useState, type MouseEvent } from "react";

import { BatchTagger } from "./components/BatchTagger";
import { ImageGrid } from "./components/ImageGrid";
import { ImagePreview } from "./components/ImagePreview";
import { ImportDropZone } from "./components/ImportDropZone";
import { RecentUsedBar } from "./components/RecentUsedBar";
import { SearchBar } from "./components/SearchBar";
import { TagInput } from "./components/TagInput";
import { TagPanel } from "./components/TagPanel";
import {
  addTagsToImages,
  copyImageToClipboard,
  deleteTag,
  getAppInfo,
  getImagePath,
  getImages,
  getImageTags,
  getRecentImages,
  getTags,
  healthCheck,
  importFolder,
  markImageUsed,
  removeTagsFromImages,
  type AppInfo,
  type GalleryImage,
  type HealthStatus,
  type ImportProgress,
  type ImportSummary,
  type TagItem,
} from "./lib/tauri";

const PAGE_SIZE = 120;
const SEARCH_DEBOUNCE_MS = 180;

type PreviewState = {
  filename: string;
  filepath: string;
};

function parseTagNames(input: string): string[] {
  return input
    .split(/[\s,]+/)
    .map((item) => item.trim())
    .filter((item, index, array) => item.length > 0 && array.indexOf(item) === index);
}

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  const tag = target.tagName.toLowerCase();
  return tag === "input" || tag === "textarea" || target.isContentEditable;
}

function App() {
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [healthStatus, setHealthStatus] = useState<HealthStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  const [images, setImages] = useState<GalleryImage[]>([]);
  const [recentImages, setRecentImages] = useState<GalleryImage[]>([]);
  const [totalImages, setTotalImages] = useState(0);
  const [currentPage, setCurrentPage] = useState(0);
  const [loadingImages, setLoadingImages] = useState(false);

  const [importing, setImporting] = useState(false);
  const [importProgress, setImportProgress] = useState<ImportProgress | null>(null);
  const [importSummary, setImportSummary] = useState<ImportSummary | null>(null);

  const [tags, setTags] = useState<TagItem[]>([]);
  const [selectedImageTags, setSelectedImageTags] = useState<TagItem[]>([]);
  const [selectedImageIds, setSelectedImageIds] = useState<number[]>([]);
  const [selectionAnchor, setSelectionAnchor] = useState<number | null>(null);
  const [gridColumns, setGridColumns] = useState(4);

  const [searchInput, setSearchInput] = useState("");
  const [searchQuery, setSearchQuery] = useState("");

  const [preview, setPreview] = useState<PreviewState | null>(null);
  const [copyMessage, setCopyMessage] = useState<string | null>(null);

  const loadingImagesRef = useRef(false);
  const searchInputRef = useRef<HTMLInputElement | null>(null);

  const selectedIdsSet = useMemo(() => new Set(selectedImageIds), [selectedImageIds]);
  const canLoadMore = useMemo(() => images.length < totalImages, [images.length, totalImages]);
  const firstSelectedId = selectedImageIds[0] ?? null;

  const loadBootstrapState = useCallback(async (): Promise<void> => {
    try {
      const [info, health] = await Promise.all([getAppInfo(), healthCheck()]);
      setAppInfo(info);
      setHealthStatus(health);
      setError(null);
    } catch (unknownError) {
      const message = unknownError instanceof Error ? unknownError.message : "Failed to fetch app info";
      setError(message);
    }
  }, []);

  const loadTags = useCallback(async (query = ""): Promise<void> => {
    try {
      const tagList = await getTags(query, 120);
      setTags(tagList);
      setError(null);
    } catch (unknownError) {
      const message = unknownError instanceof Error ? unknownError.message : "Failed to load tags";
      setError(message);
    }
  }, []);

  const loadRecent = useCallback(async (): Promise<void> => {
    try {
      const list = await getRecentImages(16);
      setRecentImages(list);
      setError(null);
    } catch (unknownError) {
      const message = unknownError instanceof Error ? unknownError.message : "Failed to load recent images";
      setError(message);
    }
  }, []);

  const loadImagesPage = useCallback(async (page: number, replace: boolean, query: string): Promise<void> => {
    if (loadingImagesRef.current) {
      return;
    }

    loadingImagesRef.current = true;
    setLoadingImages(true);

    try {
      const response = await getImages(page, PAGE_SIZE, query);
      setTotalImages(response.total);
      setCurrentPage(response.page);
      setImages((previous) => (replace ? response.items : [...previous, ...response.items]));
      setError(null);
    } catch (unknownError) {
      const message = unknownError instanceof Error ? unknownError.message : "Failed to fetch images";
      setError(message);
    } finally {
      loadingImagesRef.current = false;
      setLoadingImages(false);
    }
  }, []);

  const refreshSelectedImageTags = useCallback(async (): Promise<void> => {
    if (selectedImageIds.length !== 1) {
      setSelectedImageTags([]);
      return;
    }

    try {
      const tagsForImage = await getImageTags(selectedImageIds[0]);
      setSelectedImageTags(tagsForImage);
      setError(null);
    } catch (unknownError) {
      const message = unknownError instanceof Error ? unknownError.message : "Failed to load image tags";
      setError(message);
    }
  }, [selectedImageIds]);

  const loadMore = useCallback(() => {
    if (!canLoadMore || loadingImagesRef.current) {
      return;
    }

    void loadImagesPage(currentPage + 1, false, searchQuery);
  }, [canLoadMore, currentPage, loadImagesPage, searchQuery]);

  const handleQuickCopy = useCallback(
    async (image: GalleryImage): Promise<void> => {
      try {
        await copyImageToClipboard(image.id);
        setCopyMessage(`已複製：${image.filename}`);
        await Promise.all([loadRecent(), loadBootstrapState()]);
        setError(null);
      } catch (unknownError) {
        const message = unknownError instanceof Error ? unknownError.message : "Failed to copy image";
        setError(message);
      }
    },
    [loadBootstrapState, loadRecent],
  );

  const handleImport = useCallback(
    async (path: string): Promise<void> => {
      setImporting(true);
      setImportSummary(null);
      setImportProgress(null);

      try {
        const result = await importFolder(path);
        setImportSummary(result);
        await Promise.all([
          loadImagesPage(1, true, searchQuery),
          loadBootstrapState(),
          loadTags(),
          refreshSelectedImageTags(),
        ]);
      } catch (unknownError) {
        const message = unknownError instanceof Error ? unknownError.message : "Failed to import folder";
        setError(message);
      } finally {
        setImporting(false);
      }
    },
    [loadBootstrapState, loadImagesPage, loadTags, refreshSelectedImageTags, searchQuery],
  );

  const handleGridSelect = useCallback(
    (image: GalleryImage, index: number, event: MouseEvent<HTMLButtonElement>) => {
      const toggleMode = event.ctrlKey || event.metaKey;

      if (event.shiftKey && selectionAnchor !== null) {
        const start = Math.min(selectionAnchor, index);
        const end = Math.max(selectionAnchor, index);
        const rangeIds = images.slice(start, end + 1).map((item) => item.id);

        if (toggleMode) {
          setSelectedImageIds((previous) => Array.from(new Set([...previous, ...rangeIds])));
        } else {
          setSelectedImageIds(rangeIds);
        }

        return;
      }

      if (toggleMode) {
        setSelectedImageIds((previous) => {
          if (previous.includes(image.id)) {
            return previous.filter((id) => id !== image.id);
          }

          return [...previous, image.id];
        });
        setSelectionAnchor(index);
        return;
      }

      setSelectedImageIds([image.id]);
      setSelectionAnchor(index);
    },
    [images, selectionAnchor],
  );

  const handleOpenPreview = useCallback(
    async (image: GalleryImage): Promise<void> => {
      try {
        const filepath = await getImagePath(image.id);
        setPreview({ filename: image.filename, filepath });
        setSelectedImageIds([image.id]);
        await Promise.all([markImageUsed(image.id), loadRecent(), loadBootstrapState()]);
        setError(null);
      } catch (unknownError) {
        const message = unknownError instanceof Error ? unknownError.message : "Failed to open image preview";
        setError(message);
      }
    },
    [loadBootstrapState, loadRecent],
  );

  const handleAddTags = useCallback(
    async (value: string): Promise<void> => {
      const parsed = parseTagNames(value);
      if (parsed.length === 0 || selectedImageIds.length === 0) {
        return;
      }

      try {
        await addTagsToImages(selectedImageIds, parsed);
        await Promise.all([loadTags(), refreshSelectedImageTags()]);
      } catch (unknownError) {
        const message = unknownError instanceof Error ? unknownError.message : "Failed to add tags";
        setError(message);
      }
    },
    [loadTags, refreshSelectedImageTags, selectedImageIds],
  );

  const handleRemoveTags = useCallback(
    async (value: string): Promise<void> => {
      const parsed = parseTagNames(value);
      if (parsed.length === 0 || selectedImageIds.length === 0) {
        return;
      }

      try {
        await removeTagsFromImages(selectedImageIds, parsed);
        await Promise.all([loadTags(), refreshSelectedImageTags()]);
      } catch (unknownError) {
        const message = unknownError instanceof Error ? unknownError.message : "Failed to remove tags";
        setError(message);
      }
    },
    [loadTags, refreshSelectedImageTags, selectedImageIds],
  );

  const handleDeleteTag = useCallback(
    async (tagId: number): Promise<void> => {
      try {
        await deleteTag(tagId);
        await Promise.all([loadTags(), refreshSelectedImageTags(), loadBootstrapState()]);
      } catch (unknownError) {
        const message = unknownError instanceof Error ? unknownError.message : "Failed to delete tag";
        setError(message);
      }
    },
    [loadBootstrapState, loadTags, refreshSelectedImageTags],
  );

  useEffect(() => {
    void loadBootstrapState();
    void loadTags();
    void loadRecent();
  }, [loadBootstrapState, loadTags, loadRecent]);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setSearchQuery(searchInput.trim());
    }, SEARCH_DEBOUNCE_MS);

    return () => window.clearTimeout(timer);
  }, [searchInput]);

  useEffect(() => {
    setSelectedImageIds([]);
    setSelectionAnchor(null);
    setSelectedImageTags([]);
    void loadImagesPage(1, true, searchQuery);
  }, [loadImagesPage, searchQuery]);

  useEffect(() => {
    let isMounted = true;
    const unlistenPromise = listen<ImportProgress>("import-progress", (event) => {
      if (isMounted) {
        setImportProgress(event.payload);
      }
    });

    return () => {
      isMounted = false;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    let isMounted = true;
    const unlistenPromise = listen("focus-search-request", () => {
      if (isMounted) {
        searchInputRef.current?.focus();
      }
    });

    return () => {
      isMounted = false;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    void refreshSelectedImageTags();
  }, [refreshSelectedImageTags]);

  useEffect(() => {
    if (!copyMessage) {
      return;
    }

    const timer = window.setTimeout(() => setCopyMessage(null), 1400);
    return () => window.clearTimeout(timer);
  }, [copyMessage]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (isTypingTarget(event.target)) {
        return;
      }

      if (event.key === "k" && (event.ctrlKey || event.metaKey)) {
        event.preventDefault();
        searchInputRef.current?.focus();
        return;
      }

      if (images.length === 0) {
        return;
      }

      const selectedId = firstSelectedId;
      const currentIndex = selectedId ? images.findIndex((item) => item.id === selectedId) : -1;
      const fallbackIndex = currentIndex >= 0 ? currentIndex : 0;

      let nextIndex: number | null = null;
      if (event.key === "ArrowRight") {
        nextIndex = fallbackIndex + 1;
      } else if (event.key === "ArrowLeft") {
        nextIndex = fallbackIndex - 1;
      } else if (event.key === "ArrowDown") {
        nextIndex = fallbackIndex + Math.max(gridColumns, 1);
      } else if (event.key === "ArrowUp") {
        nextIndex = fallbackIndex - Math.max(gridColumns, 1);
      }

      if (nextIndex !== null) {
        event.preventDefault();
        const clamped = Math.min(Math.max(nextIndex, 0), images.length - 1);
        const nextId = images[clamped]?.id;

        if (nextId) {
          setSelectedImageIds([nextId]);
          setSelectionAnchor(clamped);
        }

        if (canLoadMore && clamped >= images.length - Math.max(gridColumns, 1)) {
          loadMore();
        }

        return;
      }

      if (event.key === "Enter" && selectedId) {
        event.preventDefault();
        const selected = images.find((item) => item.id === selectedId);
        if (selected) {
          void handleOpenPreview(selected);
        }
        return;
      }

      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "c" && selectedId) {
        event.preventDefault();
        const selected = images.find((item) => item.id === selectedId);
        if (selected) {
          void handleQuickCopy(selected);
        }
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [canLoadMore, firstSelectedId, gridColumns, handleOpenPreview, handleQuickCopy, images, loadMore]);

  return (
    <main className="mx-auto flex min-h-screen w-full max-w-[1600px] flex-col gap-4 px-5 py-5">
      <header className="rounded-xl border border-slate-700/70 bg-shell-900/80 p-4 shadow-lg shadow-black/20">
        <p className="text-xs font-semibold uppercase tracking-wide text-accent-400">Meme Vault / Phase 3</p>
        <h1 className="mt-1 text-2xl font-semibold text-slate-100">快速複製與鍵盤工作流</h1>
        <div className="mt-3 grid gap-2 text-xs text-slate-300 md:grid-cols-3">
          <p>database: {appInfo?.databasePath ?? "loading"}</p>
          <p>app data: {appInfo?.appDataDir ?? "loading"}</p>
          <p>
            rows: {healthStatus?.imageCount ?? "-"} images / {healthStatus?.tagCount ?? "-"} tags
          </p>
        </div>
      </header>

      {copyMessage ? (
        <section className="rounded-xl border border-emerald-700/70 bg-emerald-950/40 p-2 text-sm text-emerald-200">
          {copyMessage}
        </section>
      ) : null}

      {error ? (
        <section className="rounded-xl border border-rose-700/70 bg-rose-950/40 p-3 text-sm text-rose-200">{error}</section>
      ) : null}

      <div className="grid gap-4 xl:grid-cols-[360px,1fr]">
        <aside className="space-y-4">
          <ImportDropZone
            importing={importing}
            progress={importProgress}
            summary={importSummary}
            onImport={handleImport}
          />

          <SearchBar
            value={searchInput}
            inputRef={searchInputRef}
            onChange={setSearchInput}
            onClear={() => setSearchInput("")}
          />

          <RecentUsedBar
            items={recentImages}
            onOpen={(image) => void handleOpenPreview(image)}
            onQuickCopy={(image) => void handleQuickCopy(image)}
          />

          <TagInput
            selectedCount={selectedImageIds.length}
            currentTags={selectedImageTags}
            allTags={tags}
            onAddTags={handleAddTags}
            onRemoveTag={(name) => handleRemoveTags(name)}
          />

          <BatchTagger
            selectedCount={selectedImageIds.length}
            onAddTags={handleAddTags}
            onRemoveTags={handleRemoveTags}
          />

          <TagPanel
            tags={tags}
            onUseTag={(name) => {
              setSearchInput((previous) => (previous.trim().length > 0 ? `${previous.trim()} + ${name}` : name));
            }}
            onDeleteTag={handleDeleteTag}
          />
        </aside>

        <section className="space-y-3">
          <section className="flex items-center justify-between text-xs text-slate-300">
            <span>
              loaded {images.length} / total {totalImages} / selected {selectedImageIds.length}
            </span>
            <div className="flex items-center gap-2 text-[11px] text-slate-400">
              <span>Arrow: 移動</span>
              <span>Enter: 預覽</span>
              <span>Ctrl/Cmd+C: 複製</span>
              <span>Ctrl/Cmd+K: 搜尋</span>
              {canLoadMore ? (
                <button
                  type="button"
                  onClick={loadMore}
                  disabled={loadingImages}
                  className="rounded border border-slate-600 px-3 py-1 text-xs transition hover:border-slate-500 disabled:opacity-50"
                >
                  {loadingImages ? "Loading..." : "Load more"}
                </button>
              ) : null}
            </div>
          </section>

          <ImageGrid
            images={images}
            loading={loadingImages}
            selectedIds={selectedIdsSet}
            onSelect={handleGridSelect}
            onOpenPreview={(image) => void handleOpenPreview(image)}
            onQuickCopy={(image) => void handleQuickCopy(image)}
            onColumnsChange={setGridColumns}
            onReachEnd={canLoadMore ? loadMore : undefined}
          />
        </section>
      </div>

      {preview ? (
        <ImagePreview filename={preview.filename} filepath={preview.filepath} onClose={() => setPreview(null)} />
      ) : null}
    </main>
  );
}

export default App;
