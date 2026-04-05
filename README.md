# Editor - 古风文字桌面编辑器

一款基于 Tauri 2.x 的桌面应用，用于创作古风图文/视频作品。

## 功能特性

- **文字卡片编辑** - 标题、作者、正文、印章自定义
- **自定义字体** - 标题/作者/正文/印章四个区域支持上传独立字体
- **音乐横幅** - 上传背景音乐，自动识别标题/作者/专辑/封面/歌词
- **歌词字幕** - 支持时间轴歌词，与背景音乐同步滚动
- **视频导出** - 帧动画 + 背景音乐合成 MP4
- **自定义导出目录** - 用户指定导出根目录，文件写入 `{选择目录}/Editor/图片/` 或 `{选择目录}/Editor/视频/`

## 技术栈

- **前端**: Vanilla JS + Canvas 2D（无框架依赖）
- **后端**: Rust + Tauri 2.x
- **音频元数据**: lofty 0.22
- **视频合成**: FFmpeg（需在 PATH 或可执行文件同目录下）

## 开发

```bash
# 安装依赖
npm install

# 开发模式（热重载）
npm run tauri:dev

# 生产构建
npm run tauri:build
```

## 项目结构

```
src-tauri/          Rust 后端
  src/main.rs       Tauri 命令实现
  tauri.conf.json   Tauri 配置
  icons/            应用图标
web/
  editor_app.html   主应用（单文件 ~2400 行）
  index.html        入口重定向
```

## 配置说明

### FFmpeg

应用自动搜索 FFmpeg，优先级：
1. `FFMPEG_PATH` 环境变量
2. 可执行文件同目录
3. 当前工作目录
4. 系统 PATH

### 导出目录

首次使用会导出到 `/桌面/Editor/图片/` 和 `/桌面/Editor/视频/`。用户可在应用内通过"导出设置"面板自定义导出根目录（会在其下创建 `Editor` 子文件夹）。

## 下载安装

Windows 安装包：[Github releases](https://github.com/bill756/Editor/releases/tag/%E5%AE%89%E8%A3%85%E5%8C%85)

## 构建需求

- Node.js
- Rust toolchain
- FFmpeg（用于视频导出）
