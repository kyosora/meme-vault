import { invoke } from "@tauri-apps/api/core";

export type AppInfo = {
  appDataDir: string;
  databasePath: string;
};

export type HealthStatus = {
  ok: boolean;
  imageCount: number;
  tagCount: number;
};

export type GalleryImage = {
  id: number;
  filename: string;
  filepath: string;
  thumbPath: string | null;
  thumbFilePath: string | null;
  width: number | null;
  height: number | null;
  fileSize: number;
  mimeType: string;
  importedAt: string;
  lastUsedAt: string | null;
};

export type ImagesPage = {
  items: GalleryImage[];
  total: number;
  page: number;
  limit: number;
};

export type ImportProgress = {
  current: number;
  total: number;
  imported: number;
  skipped: number;
  failed: number;
  currentPath: string | null;
};

export type ImportSummary = {
  imported: number;
  skipped: number;
  failed: number;
  total: number;
};

export type TagItem = {
  id: number;
  name: string;
  parentId: number | null;
  color: string | null;
  usageCount: number;
  imageCount: number;
};

export type TagMutationResult = {
  selectedImages: number;
  resolvedTags: number;
  linkChanges: number;
};

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("get_app_info");
}

export async function healthCheck(): Promise<HealthStatus> {
  return invoke<HealthStatus>("health_check");
}

export async function importFolder(path: string): Promise<ImportSummary> {
  return invoke<ImportSummary>("import_folder", { path });
}

export async function getImages(
  page: number,
  limit: number,
  search: string,
): Promise<ImagesPage> {
  return invoke<ImagesPage>("get_images", { page, limit, search });
}

export async function getRecentImages(limit = 20): Promise<GalleryImage[]> {
  return invoke<GalleryImage[]>("get_recent_images", { limit });
}

export async function getImagePath(imageId: number): Promise<string> {
  return invoke<string>("get_image_path", { imageId });
}

export async function markImageUsed(imageId: number): Promise<void> {
  await invoke<void>("mark_image_used", { imageId });
}

export async function copyImageToClipboard(imageId: number): Promise<void> {
  await invoke<void>("copy_image_to_clipboard", { imageId });
}

export async function getTags(query = "", limit = 80): Promise<TagItem[]> {
  return invoke<TagItem[]>("get_tags", { query, limit });
}

export async function getImageTags(imageId: number): Promise<TagItem[]> {
  return invoke<TagItem[]>("get_image_tags", { imageId });
}

export async function createTag(
  name: string,
  color?: string,
  parentId?: number,
): Promise<TagItem> {
  return invoke<TagItem>("create_tag", {
    name,
    color: color ?? null,
    parentId: parentId ?? null,
  });
}

export async function deleteTag(tagId: number): Promise<void> {
  await invoke<void>("delete_tag", { tagId });
}

export async function addTagsToImages(
  imageIds: number[],
  tagNames: string[],
): Promise<TagMutationResult> {
  return invoke<TagMutationResult>("add_tags_to_images", { imageIds, tagNames });
}

export async function removeTagsFromImages(
  imageIds: number[],
  tagNames: string[],
): Promise<TagMutationResult> {
  return invoke<TagMutationResult>("remove_tags_from_images", {
    imageIds,
    tagNames,
  });
}