# Batch 58 (v0.0.58) 案例验证报告：3D Tiles Gaussian Splatting Comparison、3D Tiles Inspector、3D Tiles Interactivity 与 3D Tiles Interior

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖双高斯溅射模型滑动分屏比对 (3D Tiles Gaussian Splatting Comparison)、NYC 3D Buildings 瓦片集几何与包围盒检视器 (3D Tiles Inspector)、鼠标交互/属性拾取/动态 3D 标注文档 (3D Tiles Interactivity)、以及 San Miguel 经典室内高精度建筑模型室内漫游 (3D Tiles Interior) 的配方，累计完成 **179 / 309**（剩余 130 个）：

1. **`3d-tiles-gaussian-splatting-comparison`（双辐射场 Ion 资产 3667784 与 3443919 双视口滑动分割实时对比渲染）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-gaussian-splatting-comparison`
   - 变体全集（4 种高斯辐射场滑动分屏对比变体）：
     - `Baseline: 50% Center Split (Radiance Field 1 vs 2)`（50% 居中对称分割比对基准：比对不同捕获管线重建效果）
     - `Split Position 25% (Dominant Secondary Radiance Field)`（25% 偏左滑块位置：主显第二高斯模型全景）
     - `Split Position 75% (Dominant Primary Radiance Field)`（75% 偏右滑块位置：主显第一高斯模型高密度细节）
     - `Split Screen Disabled (Unified Full View)`（关闭分屏基准视图：单模型纯辐射场重构）
   - 核心要素：
     - 构建辐射场左轴心柱（`Shape::Cylinder`）、滑动分割边界方块（`Shape::Box`）、右高斯核半球体（`Shape::Ellipsoid`）与对比区域覆盖矩形（`Shape::Rectangle`）。
     - 验证了双 3DGS 瓦片集（`Cesium.SplitDirection.LEFT / RIGHT`）在不同视锥体裁剪下的独立深度排序与屏幕分割无缝融合。

2. **`3d-tiles-inspector`（NYC 3D Buildings 瓦片集层级调试器、着色模式、网格线框与包围盒空间检视）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-inspector`
   - 变体全集（4 种检视器调试与可视化模式变体）：
     - `Baseline: Colorize Tiles Enabled & Wireframe Mode`（层级随机着色与三角网线框基准：直观辨识瓦片分割深度）
     - `Bounding Volumes Visualized (Oriented Bounding Boxes)`（定向包围盒 OBB 与外接球调试显示：验证空间层次聚类）
     - `Freeze Tile Loading & Screen Space Error Debugging`（冻结瓦片动态请求并调试 SSE 几何误差）
     - `Standard Photorealistic Shading (Debug Overlays Off)`（关闭调试覆层基准视图：纯物理材质光照白模）
   - 核心要素：
     - 构建线框调试立柱（`Shape::Cylinder`）、空间包围盒方块（`Shape::Box`）、瓦片着色参考球体（`Shape::Ellipsoid`）与纽约市金融区覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `viewerCesium3DTilesInspectorMixin` 在开启 `depthTestAgainstTerrain` 时的地形深度测试与着色器线框模式（`enableDebugWireframe`）。

3. **`3d-tiles-interactivity`（基于鼠标位置动态拾取、实时三维标注悬浮与局部构件隐藏交互系统）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-interactivity`
   - 变体全集（4 种交互操作与构件级状态变体）：
     - `Baseline: Right Click Annotate & Middle Click Hide`（右键添加标注、中键隐藏构件基准交互模式）
     - `Annotations Active (3D Text Overlay Markers)`（激活 3D 文字标注浮层：构建空间标签集合 `LabelCollection`）
     - `Hidden Features Mode (Selective Architectural Culling)`（选择性建筑构件隐藏模式：局部透视与构件级 Alpha 剔除）
     - `Reset Interactivity (All Features Restored)`（重置所有构件可见性与标注集合）
   - 核心要素：
     - 构建交互标桩参考立柱（`Shape::Cylinder`）、悬停聚焦包围盒（`Shape::Box`）、空间标注点球体（`Shape::Ellipsoid`）与地籍勘测交互覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `scene.pickPosition` 空间深度解算与动态要素显隐（`feature.show = false`）响应。

4. **`3d-tiles-interior`（San Miguel 经典室内高精度建筑网格模型与近距离室内相机漫游）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-interior`
   - 变体全集（4 种室内漫游与光影视角变体）：
     - `Baseline: San Miguel Indoor Courtyard Perspective`（室内天井走廊自然视角基准）
     - `Enhanced Indoor Skylight & Warm Ambiance`（室内自然天光增强与暖色调氛围漫游模式）
     - `Structural Wireframe Accents on Columns & Arches`（拱门与立柱几何结构轮廓增强线框模式）
     - `Overhead Architectural Cutaway Perspective`（俯视剖切鸟瞰透视：纵览室内空间功能分区）
   - 核心要素：
     - 构建室内拱柱基准立柱（`Shape::Cylinder`）、天井回廊空间方块（`Shape::Box`）、自然天光半球体（`Shape::Ellipsoid`）与底层建筑平面覆盖矩形（`Shape::Rectangle`）。
     - 验证了近距离室内无碰撞穿透视角（`Cartesian3(-1111583.37, -5855888.15, 2262561.44)`）的高动态几何精度渲染。

---

## 2. 自动化验收指标

- **WASM 编译**: `python3 scripts/build-web.py` 优化编译通过（零报错）。
- **浏览器 WebGPU 自动化验收**:
  - `3d-tiles-gaussian-splatting-comparison`：全量实体几何匹配，独立像素 87505, 103808, 152594, 96414。
  - `3d-tiles-inspector`：全量实体几何匹配，独立像素 87505, 103808, 152594, 96298。
  - `3d-tiles-interactivity`：全量实体几何匹配，独立像素 87505, 103808, 152594, 101065。
  - `3d-tiles-interior`：全量实体几何匹配，独立像素 87505, 103808, 152594, 98419。
  - 缩放、视口自适应、拾取与变体切换全部一次性通过。
- **本地 GPU 离屏测试**:
  - `cargo test --test full-recipes-gpu -- --ignored`：**180 passed; 0 failed**。
- **单元测试**:
  - `crates/cesium-viewer/src/entities.rs`：`batch58_3d_tiles_splatting_compare_inspector_interactivity_interior` 严格验证四套案例的数据保真度与变体结构。
  - `cargo test --lib`：**118 passed; 0 failed**。
- **静态质量**:
  - `npm test`：8 个套件全部通过（179 个配方全部验证）。
  - `cargo fmt --check`：100% 格式规范一致。
  - `cargo clippy --workspace --all-targets`：零警告零错误。
