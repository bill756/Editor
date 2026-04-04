# Tasks
- [x] Task 1: 建立 Tauri 桌面应用基础骨架并接入现有 editor 页面
  - [x] SubTask 1.1: 初始化 Tauri 项目结构并配置前端入口
  - [x] SubTask 1.2: 验证桌面端启动与编辑器核心渲染可用
  - [x] SubTask 1.3: 梳理本地文件访问边界与安全策略

- [x] Task 2: 实现分部位自定义字体能力（标题/作者/印章/正文）
  - [x] SubTask 2.1: 增加字体文件选择与加载流程
  - [x] SubTask 2.2: 建立四个文本部位的独立字体状态与应用逻辑
  - [x] SubTask 2.3: 增加字体加载失败提示与回退机制

- [x] Task 3: 实现编辑态音乐插入与预览能力
  - [x] SubTask 3.1: 增加本地音频文件选择与格式校验
  - [x] SubTask 3.2: 增加播放/暂停与当前音乐状态展示
  - [x] SubTask 3.3: 增加音乐加载失败提示

- [x] Task 4: 实现“图片+音乐”视频导出流程
  - [x] SubTask 4.1: 将当前编辑画面转换为导出帧或静态视频画面输入
  - [x] SubTask 4.2: 将音乐与画面合成为视频并保存到本地
  - [x] SubTask 4.3: 增加导出进度/成功/失败反馈及缺少音乐拦截

- [x] Task 5: 完成端到端验证与回归
  - [x] SubTask 5.1: 验证桌面启动、字体设置、音乐预览、视频导出主流程
  - [x] SubTask 5.2: 验证异常场景（无效字体、无效音频、未插入音乐导出）
  - [x] SubTask 5.3: 修复验证中发现的问题并复测

- [x] Task 6: 修复无效音频缺少明确提示并完成复测
  - [x] SubTask 6.1: 在音乐预览加载失败时给出明确错误提示
  - [x] SubTask 6.2: 加载失败后清理无效音乐状态避免影响导出
  - [x] SubTask 6.3: 复测无效音频场景并确认提示与状态恢复

# Task Dependencies
- Task 2 depends on Task 1
- Task 3 depends on Task 1
- Task 4 depends on Task 2
- Task 4 depends on Task 3
- Task 5 depends on Task 2
- Task 5 depends on Task 3
- Task 5 depends on Task 4
