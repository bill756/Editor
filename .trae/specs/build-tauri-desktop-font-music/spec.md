# Editor Tauri 桌面化与字体音乐增强 Spec

## Why
当前 `editor.html` 仅能以网页形式运行，无法提供桌面端安装与本地文件能力。用户需要更强的创作自由度，包括局部字体自定义与“图文+音乐”视频导出能力。

## What Changes
- 将现有 `editor.html` 改造为可运行的 Tauri 桌面应用
- 新增本地字体文件导入能力，并支持分别应用到标题、作者、印章、正文
- 新增编辑时插入音乐能力（选择本地音频文件并预览）
- 新增保存为视频能力：将当前画面与所选音乐合成视频文件导出
- 增加必要的错误处理与导出状态反馈（加载失败、导出失败、进度提示）

## Impact
- Affected specs: 桌面应用运行能力、本地资源管理、排版样式系统、媒体导出能力
- Affected code: 前端编辑器页面结构与样式逻辑、Tauri 启动与文件访问桥接、音视频导出流程

## ADDED Requirements
### Requirement: Tauri Desktop Runtime
系统 SHALL 提供桌面端运行能力，使用户可在本地以 Tauri 应用方式启动编辑器并访问本地文件。

#### Scenario: 启动桌面应用
- **WHEN** 用户启动桌面应用
- **THEN** 应显示原编辑器核心功能并保持可用

### Requirement: Per-Section Custom Font
系统 SHALL 支持用户选择本地字体文件，并分别设置标题、作者、印章、正文的字体。

#### Scenario: 分部位字体设置成功
- **WHEN** 用户为任一部位选择有效字体文件并应用
- **THEN** 对应部位文本样式立即更新且不影响其他部位

#### Scenario: 字体文件无效
- **WHEN** 用户选择损坏或不支持的字体文件
- **THEN** 系统提示失败原因并保持原字体设置

### Requirement: Music Insert And Preview
系统 SHALL 支持在编辑过程中选择本地音乐文件并进行播放预览。

#### Scenario: 音乐插入成功
- **WHEN** 用户选择支持的音频文件
- **THEN** 系统加载音频并允许播放/暂停预览

### Requirement: Export Video With Image And Music
系统 SHALL 支持将当前编辑画面与已插入音乐合成为视频文件并保存到本地。

#### Scenario: 视频导出成功
- **WHEN** 用户触发“保存为视频”且画面与音乐均有效
- **THEN** 系统生成视频文件并提示保存位置或成功状态

#### Scenario: 缺少音乐时导出
- **WHEN** 用户未选择音乐即触发“保存为视频”
- **THEN** 系统阻止导出并提示先插入音乐

## MODIFIED Requirements
### Requirement: Save Output Behavior
系统原有保存能力扩展为多模式输出：在保留现有静态输出能力的基础上，新增“视频导出（画面+音乐）”入口与流程。

## REMOVED Requirements
### Requirement: 仅网页运行形态
**Reason**: 需求已升级为桌面端可分发应用，需支持更完整的本地文件与导出能力。  
**Migration**: 将网页静态入口迁移为 Tauri 前端资源入口，原编辑逻辑保留并逐步接入桌面 API。
