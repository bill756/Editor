# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a **Tauri 2.x desktop application** for creating Chinese poetry/font music compositions with video export capabilities. The app provides a traditional Chinese aesthetic editor where users can compose poetry with custom fonts, insert music, and export the result as a video.

## Build Commands

```bash
npm run tauri:dev      # Start development mode with hot reload
npm run tauri:build    # Build production release
```

**Requirements:**
- Node.js (for npm)
- Rust toolchain (for Tauri backend)
- FFmpeg in PATH or bundled (for media export)

**Debugging:** Set `app.windows[0].devtools: true` in [tauri.conf.json](src-tauri/tauri.conf.json) to enable webview DevTools (Tauri 2.x requirement).

## Architecture

### Frontend
- Single HTML file: [web/editor_app.html](web/editor_app.html) (~2400 lines)
- Vanilla JS/CSS with no build step for the frontend
- Chinese fonts: Ma Shan Zheng (titles), Noto Serif SC (body)
- Theme colors: `--ink`, `--paper`, `--seal-red` (traditional Chinese aesthetic)

### Backend (Rust)
- [src-tauri/src/main.rs](src-tauri/src/main.rs) - Tauri command handlers
- [src-tauri/Cargo.toml](src-tauri/Cargo.toml) - Dependencies: tauri 2, base64 0.22, dirs 6, serde 1
- Two Tauri commands exposed:
  - `inspect_audio_tags` - extracts title/artist/album/duration/cover from audio via ffprobe
  - `export_video_ffmpeg` - combines PNG frame + audio into MP4 via ffmpeg

### Tauri Configuration
- [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json)
  - Frontend served from `../web` directory
  - Window: 1280x820, min 960x640
  - FFmpeg bundled at `src-tauri/target/release/ffmpeg.exe`
  - **Note:** `bundle.resources` contains an absolute path that must be updated for different machines

### Entry Points
- `index.html` → redirects to `web/editor_app.html`
- `editor.html` → redirects to `web/editor_app.html`
- Both exist for backwards compatibility with different hosting setups

## FFmpeg Integration

The app searches for FFmpeg in this order:
1. `FFMPEG_PATH` environment variable
2. Directory containing the running executable
3. Current working directory
4. `src-tauri/target/release/`
5. System PATH (falls back to `ffmpeg` command)

Similarly for ffprobe (used in `inspect_audio_tags`).

## Key Implementation Notes

- Audio metadata extraction writes temp files to `std::env::temp_dir()` with prefix `poem_audio_meta_`
- Video export writes temp files to `std::env::temp_dir()` with prefix `poem_video_`
- Output videos are saved to Downloads or Desktop directory
- Export filename format: `古诗视频_{sanitized_title}_{timestamp}.mp4`
- Video codec tries `aac` first, falls back to `libmp3lame`
