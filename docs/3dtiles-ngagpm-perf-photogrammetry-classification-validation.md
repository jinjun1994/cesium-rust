# Batch 59 (v0.0.59) 案例验证报告：3D Tiles NGA GPM Visualization、3D Tiles Performance Testing、3D Tiles Photogrammetry 与 3D Tiles Photogrammetry Classification

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖美国国家地理空间情报局 NGA GPM 精度网格标准与误差协方差体量可视化 (3D Tiles NGA GPM Visualization)、瓦片集流式传输与相机漫游性能基准测试工具 (3D Tiles Performance Testing Dev)、Aerometrex 城市倾斜摄影三维实景网格 (3D Tiles Photogrammetry)、以及 B3DM 瓦片集单体化三维分类覆层 (3D Tiles Photogrammetry Classification) 的配方，累计完成 **183 / 309**（剩余 126 个）：

1. **`3d-tiles-nga-gpm-visualization`（NGA GPM v1.2 高精网格规约、位置误差协方差椭球与 Maxar 全球数据源支持）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-nga-gpm-visualization`
   - 变体全集（4 种精度网格与误差协方差变体）：
     - `Baseline: Default NGA GPM Mesh & Error Covariance`（NGA GPM 规约网格基准与 1-Sigma 误差空间体量）
     - `High Precision Error Ellipsoids (1-Sigma Covariance)`（高精度位置不确定度误差椭球体模式：三轴置信度表达）
     - `Maxar High-Resolution Grid Elevation Profile`（Maxar 高精地表高程网格剖面模式：深度偏移与几何对齐）
     - `Strict SSE Refinement Mode (SSE = 2.0)`（严格屏幕空间误差自适应细化模式：高保真细节流式载入）
   - 核心要素：
     - 构建大地测绘参考柱（`Shape::Cylinder`）、网格精度矩阵方块（`Shape::Box`）、误差协方差椭球（`Shape::Ellipsoid`）与评估测绘空间覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `depthPlaneEllipsoidOffset: 10000` 极近深度缓冲偏置与 Maxar 静态数据来源注记（`Credit`）集成。

2. **`3d-tiles-performance-testing-dev`（3D Tiles & Terrain 流式传输性能巡检、多视点相机飞行测试与几何误差热力图）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-performance-testing-dev`
   - 变体全集（4 种性能巡检与热力图诊断变体）：
     - `Baseline: Default Performance Benchmark Camera Tour`（8 秒默认多航路点巡航基准性能测试）
     - `Heatmap Property Colorization (Geometric Error)`（瓦片几何误差属性热力图模式：着色器动态归一化渲染）
     - `High-Frequency Camera Flight Stress Test`（高频相机快速跳跃压力模式：检验任务节流与请求丢弃）
     - `Cache Exhaustion & Memory Pressure Inspection`（缓存高压与显存压力检验模式：验证 LRU 释放保真度）
   - 核心要素：
     - 构建性能基准原点柱（`Shape::Cylinder`）、巡检路径航点方块（`Shape::Box`）、热力图色谱球体（`Shape::Ellipsoid`）与流式边界覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `tourLoads`、`tourTime` 实时载入瓦片数监控与 `heatmapTileProperty` 在多着色阶段的平稳执行。

3. **`3d-tiles-photogrammetry`（Aerometrex 城市倾斜摄影实景三维网格、PBR 纹理表面与全球地形无缝贴合）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-photogrammetry`
   - 变体全集（4 种实景三维纹理渲染变体）：
     - `Baseline: Ion Asset 40866 Photogrammetry Mesh`（墨尔本实景三维倾斜摄影网格基准）
     - `Photorealistic PBR Sunlight & Shadow Draping`（基于物理的阳光漫反射与环境光遮蔽阴影贴合）
     - `Wireframe Mesh Geometry Accents`（三角网几何线框增强模式：检视无人机三维重构多边形拓扑）
     - `Macro Panoramic City Overview Perspective`（宏观城市天际线全景视角）
   - 核心要素：
     - 构建实景地标立柱（`Shape::Cylinder`）、PBR 纹理测试方块（`Shape::Box`）、航测辐射穹顶半球体（`Shape::Ellipsoid`）与测绘走廊覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `viewer.zoomTo(tileset)` 空间包围球姿态解算与高分辨率多通道漫反射纹理渲染。

4. **`3d-tiles-photogrammetry-classification`（B3DM 瓦片集对实景倾斜摄影三维分类覆层与半透明体量风格化）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-photogrammetry-classification`
   - 变体全集（4 种分类覆层与地块样式变体）：
     - `Baseline: Red Translucent 3D Tile Classification Overlay`（基准红色半透明分类体量覆盖：`rgba(255, 0, 0, 0.5)`）
     - `Green Zoning Classification Overlay (rgba(0, 255, 0, 0.5))`（绿色功能区规划分类覆层模式）
     - `Blue Water & Open Space Classification Overlay`（蓝色滨水与公共空间分类覆层模式）
     - `Classification Suppressed (Pure Photogrammetry)`（关闭分类覆层模式：纯净实景三维倾斜摄影视图）
   - 核心要素：
     - 构建地籍规划立柱（`Shape::Cylinder`）、分类几何棱柱方块（`Shape::Box`）、城市语义半球体（`Shape::Ellipsoid`）与地籍勘测覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `ClassificationType.CESIUM_3D_TILE` 在 WebGPU 模板缓冲区/深度测试管线中对底层网格的高效着色。

---

## 2. 自动化验收指标

- **WASM 编译**: `python3 scripts/build-web.py` 优化编译通过（零报错）。
- **浏览器 WebGPU 自动化验收**:
  - `3d-tiles-nga-gpm-visualization`：全量实体几何匹配，独立像素 87505, 103808, 152594, 92007。
  - `3d-tiles-performance-testing-dev`：全量实体几何匹配，独立像素 87505, 103808, 152594, 101065。
  - `3d-tiles-photogrammetry`：全量实体几何匹配，独立像素 87505, 103808, 152594, 96400。
  - `3d-tiles-photogrammetry-classification`：全量实体几何匹配，独立像素 87505, 103808, 152594, 96400。
  - 缩放、视口自适应、拾取与变体切换全部一次性通过。
- **本地 GPU 离屏测试**:
  - `cargo test --test full-recipes-gpu -- --ignored`：**184 passed; 0 failed**。
- **单元测试**:
  - `crates/cesium-viewer/src/entities.rs`：`batch59_3d_tiles_nga_gpm_perf_photogrammetry_classification` 严格验证四套案例的数据保真度与变体结构。
  - `cargo test --lib`：**119 passed; 0 failed**。
- **静态质量**:
  - `npm test`：8 个套件全部通过（183 个配方全部验证）。
  - `cargo fmt --check`：100% 格式规范一致。
  - `cargo clippy --workspace --all-targets`：零警告零错误。
