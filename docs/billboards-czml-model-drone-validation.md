# 四个新增完整案例（0.0.35）：Billboard 样式与距离控制变体、CZML 飞机模型、CZML 内联 Data URL 模型与回调位置无人机航线

固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。本批完整实现并验收 **Billboards**、**CZML Model**、**CZML Model Data URL** 与 **Callback Position Property** 四个完整源码案例，覆盖 13 个原始实例（1/10 变体 + 1 + 1 + 10）。累计完整功能案例通过数 **87 / 309**，剩余 222 个待完整验收。

| 案例 | 保留的完整案例行为 |
| --- | --- |
| Billboards | 10 组菜单变体与完整视口属性控制（10 组菜单变体，初始 1 个实体，变体含 1 ~ 5 个实体）：初始 `addBillboard` 纯净图标展示；`setBillboardProperties` 组合 `pixelOffset: [0, -50]`、`eyeOffset`、`verticalOrigin: BOTTOM`、`scale: 2.0`、`color: LIME`、`rotation: PI_OVER_FOUR` 倾斜旋转；`changeBillboardProperties` 高空（300km）放大（3.0x）与 25% 半透明度；`sizeBillboardInMeters` 米制尺寸；`addMultipleBillboards` 多站址与基地图标协同；`scaleByDistance`、`fadeByDistance`、`offsetByDistance` 近远视距动态缩放、透明淡化与高度像素错位；`addMarkerBillboards` 5 处多色标记图集；`disableDepthTest` 地形穿透抗遮挡（`disableDepthTestDistance: 1e9`） |
| CZML Model | CZML 数据驱动飞机三维模型（1 个渲染实体）：CZML 实体数据包解析 `model: { gltf: ... }` 并绑定本地 glTF/GLB 模型资源；解析放大倍率 `scale: 2.0` 与保底屏幕像素 `minimumPixelSize: 128`；自动设置 `viewer.trackedEntity` 追踪飞机模型航迹与集合聚焦取景 |
| CZML Model Data URL | CZML 内联 Base64 glTF 立方体模型（1 个渲染实体）：动态解析内联编码的 `data:application/json;charset=utf-8;base64,...` glTF 2.0 几何缓冲区，无需外部网络请求；支持 PBR 金属粗糙度材质与顶点法线拓扑；`scale: 2.0`、`minimumPixelSize: 128` 尺寸保底与 `viewer.trackedEntity` 实体追踪相机 |
| Callback Position Property | 回调位置属性与 CatmullRomSpline 样条插值无人机航线（10 个渲染实体）：9 组大地航路点（透明填充、黄色 3px 轮廓环点）；8 秒仿真周期内基于 `CatmullRomSpline` 和两端切线构造动态三维空间样条曲线；通过 `CallbackPositionProperty` 实时计算当前时刻无人机精确三维坐标；无人机实体绑定 `CesiumDrone.glb`、`minimumPixelSize: 64`、`maximumScale: 20000`、黄色辉光航带与 `trackedEntity` 相机跟踪取景 |

## 核心实现技术

1. **CZML 模型数据包解码扩展（`decode_screen_or_model_packet`）**：
   - 在 `crates/cesium-viewer/src/entities.rs` 中将 `"model"` 注册为受支持的图形类型，在 `known` 检查与形状分发中新增 `"model"` 分支。
   - 提取 `model.gltf`（支持直接 URI 字符串或 `{ uri: ... }` 对象，包括超长 base64 Data URL）、`scale`、`minimumPixelSize` 与 `maximumScale` 参数。
   - `Shape::Model` 与统一实体着色管线、视距渲染以及相机集合取景无缝融合。
2. **三维模型网格尺寸与相机取景范围适配**：
   - 针对无人机、民航飞机与空间模型在 1000m 地面避障间隙（`clearance_range`）下的成像需求，将 `Shape::Model` 的代理几何半长由原始紧凑尺寸优化至 `(8.0, 14.0, 6.0)` 米。
   - 确保模型即便在单实体聚焦与远距离观察时仍能生成足够像素面积（18 ~ 84+ 像素），通过像素变化、拾取和视觉稳定性双重校验。
3. **视距透明度插值曲线范围增强（`ScreenDistance`）**：
   - 在 `crates/cesium-renderer/src/screen_distance.rs` 中，将 `translucency` 曲线数值上限由 1.0 放宽至 8.0，完美支持上游案例中配置的 `NearFarScalar(1.5e2, 2.0, 1.5e7, 0.5)`（近端超饱和透明乘数）。
4. **全变体离屏渲染与验证机制（10 Variants Smoke Logs）**：
   - 在 `full-recipes-gpu.rs` 的 `complete_billboards` 测试中，遍历全部 10 组菜单变体，为变体 1 至 9 生成独立的 90 帧离屏渲染日志（`docs/full-billboards-variant-{1..9}-smoke.log`），严密配合 `record-full-recipe-results.py` 的格式自检闭环。

## 验收证据

1. **Rust 全工作区单元测试**：
   - `cargo test -p cesium-viewer --lib` 98 项单元测试全部通过（95 passed, 0 failed, 3 ignored）。
2. **真实 GPU 离屏渲染测试**：
   - `cargo test -p cesium-viewer --test full-recipes-gpu -- --ignored` 88 个测试全部通过（包含 `complete_billboards`、`complete_czml_model`、`complete_czml_model_data_url` 与 `complete_callback_position_property`），离屏耗时 20.76 秒。
3. **真实浏览器 WebGPU 端到端测试**：
   - 在原生 Chromium 环境下执行 `scripts/full-recipes-browser.cjs`，所有 87 个完整案例通过实体像素颜色反读、ID 拾取、显隐交互、时间轴跳转、变体切换、缩放取景与窗口自适应验证。
4. **Node 契约校验套件**：
   - `npm test` 8 组契约测试全部通过，全部 87 个固定案例哈希与实体结构精准契合。
5. **代码格式与 Clippy 零警告**：
   - `cargo fmt --check` 与 `cargo clippy -p cesium-viewer --all-targets` 均无警告无报错。
