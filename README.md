# 🎨 Meme Vault

個人迷因圖片管理工具，支援標籤系統與快速複製。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8DB)
![React](https://img.shields.io/badge/React-19-61DAFB)
![TypeScript](https://img.shields.io/badge/TypeScript-5.8-3178C6)
![Rust](https://img.shields.io/badge/Rust-Latest-000000)

## ✨ 核心功能

- 📁 **圖片匯入與管理** - 支援 PNG/JPG/JPEG/GIF/WebP
- 🏷️ **標籤系統** - 新增、移除、批次操作
- 🔍 **全文搜尋** - 支援 AND/OR/排除語法（`cat + happy` 或 `dog -sad`）
- ⚡ **虛擬滾動** - 高效能載入大量圖片
- 📋 **快速複製** - 右鍵或 `Ctrl+C` 直接複製到剪貼簿
- ⌨️ **鍵盤導航** - 方向鍵、Enter 預覽、`Ctrl+K` 搜尋
- 🕒 **最近使用** - 自動追蹤使用記錄

## 🚀 技術棧

### Frontend
- **React 19** - UI 框架
- **TypeScript** - 型別安全
- **Tailwind CSS** - 樣式系統
- **@tanstack/react-virtual** - 虛擬滾動

### Backend
- **Tauri 2** - 桌面應用框架
- **Rust** - 後端邏輯
- **SQLite (rusqlite)** - 本地資料庫
- **image** - 圖片處理與縮圖生成

## 📦 安裝與執行

### 前置需求

- **Node.js** 18+ 或 **pnpm** 8+
- **Rust** 1.70+（含 `cargo`）

### 開發模式

```bash
# 安裝依賴
pnpm install

# 啟動開發伺服器
pnpm tauri dev
```

### 建置生產版本

```bash
# 建置應用程式
pnpm tauri build
```

建置完成後，可執行檔位於 `src-tauri/target/release/`。

## 🎯 使用說明

### 匯入圖片

1. 將包含圖片的資料夾拖曳至「Import」區域
2. 或直接貼上資料夾路徑（例如：`D:\Memes`）
3. 點擊「Import folder」開始匯入

### 標籤管理

- **新增標籤**：選擇圖片後，在 Tag Input 區域輸入標籤名稱（支援空白或逗號分隔多個標籤）
- **移除標籤**：點擊已加上的標籤即可移除
- **批次操作**：使用 Batch Tagger 對多張圖片同時加上或移除標籤

### 搜尋語法

- `cat` - 搜尋包含「cat」標籤的圖片
- `cat + happy` 或 `cat happy` - 搜尋同時包含「cat」和「happy」的圖片（AND）
- `cat | dog` - 搜尋包含「cat」或「dog」的圖片（OR）
- `-sad` - 排除包含「sad」標籤的圖片

### 鍵盤快捷鍵

| 快捷鍵 | 功能 |
|--------|------|
| `方向鍵` | 移動選取 |
| `Enter` | 開啟預覽 |
| `Ctrl/Cmd + C` | 複製圖片到剪貼簿 |
| `Ctrl/Cmd + K` | 聚焦搜尋框 |
| `Esc`（預覽時） | 關閉預覽 |
| `右鍵` | 快速複製到剪貼簿 |

## 📁 專案結構

```
meme-vault/
├── src/                    # React 前端程式碼
│   ├── components/         # UI 元件
│   │   ├── BatchTagger.tsx
│   │   ├── ImageGrid.tsx
│   │   ├── ImagePreview.tsx
│   │   ├── ImportDropZone.tsx
│   │   ├── RecentUsedBar.tsx
│   │   ├── SearchBar.tsx
│   │   ├── TagInput.tsx
│   │   └── TagPanel.tsx
│   ├── lib/
│   │   └── tauri.ts        # Tauri API 封裝
│   ├── styles/
│   │   └── globals.css
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/              # Rust 後端程式碼
│   ├── src/
│   │   ├── clipboard.rs    # 剪貼簿操作
│   │   ├── commands.rs     # Tauri 指令
│   │   ├── db.rs           # 資料庫初始化
│   │   ├── error.rs        # 錯誤處理
│   │   ├── image.rs        # 圖片操作
│   │   ├── models.rs       # 資料模型
│   │   ├── scanner.rs      # 圖片掃描器
│   │   ├── search.rs       # 搜尋引擎
│   │   ├── tags.rs         # 標籤管理
│   │   └── thumbnail.rs    # 縮圖生成
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── README.md
```

## 🛠️ 資料庫結構

應用程式使用 SQLite 儲存資料，資料庫檔案位於：
- **Windows**: `%APPDATA%\com.kyosora.meme-vault\meme-vault.db`
- **macOS**: `~/Library/Application Support/com.kyosora.meme-vault/meme-vault.db`
- **Linux**: `~/.local/share/com.kyosora.meme-vault/meme-vault.db`

### 主要資料表

- `images` - 圖片資訊（檔名、路徑、尺寸、MIME 類型、匯入時間、最後使用時間）
- `tags` - 標籤資訊（名稱、顏色、父標籤 ID）
- `image_tags` - 圖片與標籤的多對多關聯

## 📝 待辦事項

- [ ] 標籤階層管理（父子標籤）
- [ ] 標籤顏色自訂
- [ ] 圖片編輯功能（裁切、旋轉）
- [ ] 匯出選取的圖片
- [ ] 雲端同步支援
- [ ] 多語言支援

## 🤝 貢獻

歡迎提交 Issue 或 Pull Request！

## 📄 授權

本專案採用 [MIT License](LICENSE)。

---

**開發者**: [Kyosora](https://github.com/kyosora)
**技術支援**: Claude Code (ENI)
