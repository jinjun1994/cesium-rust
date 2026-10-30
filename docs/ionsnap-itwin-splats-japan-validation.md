# Batch 77 (v0.0.77) 案例验证报告：Ion Snapping - Dev、iTwin Feature Service、iTwin Gaussian Splats 与 Japan Buildings

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 Cesium ion 服务端几何对齐吸附 (Ion Snapping - Dev)、Bentley iTwin 地理要素服务与现实数据矢量化渲染分类 (iTwin Feature Service)、Bentley 3D 高斯泼溅真实感模型加载与全球地形对齐 (iTwin Gaussian Splats)、以及日本全境 3D 建筑物大数据集与动态相机光源跟踪 (Japan Buildings) 的配方，累计完成 **255 / 309**（剩余 54 个）：

1. **`ion-snapping-dev`（Cesium ion 服务端几何对齐吸附、最近/关键点/中心多模式吸附与交互容差孔径）**
   - 官方地址：`https://sandcastle.cesium.com/?id=ion-snapping-dev`
   - 变体全集（4 种几何对齐吸附模式变体）：
     - `Baseline: Ion BIM/CAD Snapping Service (Nearest Point Mode)`（基准 Ion BIM/CAD 服务端几何吸附：最近点对齐模式）
     - `Keypoints Snapping Mode (Edge / Vertex / Corner Snapping Target)`（关键点吸附模式：边线、顶点、拐角几何目标）
     - `Center Snapping Mode (Face / Volume Geometric Centroid Snap)`（中心吸附模式：表面与体几何形心对齐）
     - `High Tolerance Aperture Mode (Expanded 32px Snap Search Volume)`（高容差搜索孔径模式：扩大 32 像素吸附体）
   - 核心要素：
     - 构建 BIM 建筑结构标定盒（`Shape::Box`）、屏幕空间吸附孔径柱（`Shape::Cylinder`）、捕捉命中测针球（`Shape::Ellipsoid`）与建筑场地包围矩形（`Shape::Rectangle`）。
     - 验证了 `IonSnapService.fromAssetId(5161569)` 服务端几何拓扑提取、`IonSnapMode`（NEAREST / KEYPOINTS / CENTER）枚举调度、`DEFAULT_SNAP_APERTURE` 可调容差与实时捕捉点高亮绘制。

2. **`itwin-feature-service`（Bentley iTwin 地理要素服务、点线面多尺度现实数据分类渲染与图层控制台）**
   - 官方地址：`https://sandcastle.cesium.com/?id=itwin-feature-service`
   - 变体全集（4 种基础设施地理空间要素变体）：
     - `Baseline: Bentley iTwin Feature Service (Points, Lines, Areas & Mesh)`（基准 Bentley iTwin 要素服务：点、线、面多源要素及网格）
     - `Point Features Isolation Mode (Trees, Lamps, Traffic Signals)`（点状基础设施隔离模式：树木、路灯、交通信号灯）
     - `Linear Infrastructure Markings Mode (Contours, Lane Markings, Kerbs)`（线性工程标线模式：等高线、车道线、路缘石）
     - `Birdseye Viewpoint Aerial Survey Extent (Cadastral Zoned Boundaries)`（鸟瞰航拍地籍多边形区域：分级面状分区边界）
   - 核心要素：
     - 构建现实网格包围盒（`Shape::Box`）、路灯要素立柱（`Shape::Cylinder`）、交通信号灯测针球（`Shape::Ellipsoid`）与要素调查包围矩形（`Shape::Rectangle`）。
     - 验证了 `ITwinData.loadGeospatialFeatures` 多 collection 批量加载、Maki 图标 PinBuilder 标记生成、多类别图层可见性开关以及全局鸟瞰/点云相机飞行动画。

3. **`itwin-gaussian-splats`（Bentley iTwin 3D 高斯泼溅真实感模型、模型矩阵高程微调与全球地形结合）**
   - 官方地址：`https://sandcastle.cesium.com/?id=itwin-gaussian-splats`
   - 变体全集（4 种 3D 高斯泼溅检视与对齐变体）：
     - `Baseline: 3D Gaussian Splats Reality Data on Cesium World Terrain`（基准 3D 高斯泼溅真实数据：结合 Cesium 全球地形）
     - `Dam Crest Close Inspection Mode (Heading Pitch Range Zoom)`（大坝坝顶近景巡检模式：HeadingPitchRange 定向视角）
     - `Terrain Subsurface Matrix Offset Calibration (Z-Offset -8m Mode)`（地形基底矩阵偏移校准：Z 轴下移 8 米精确贴合）
     - `Survey Footprint Cadastral Boundary Mode (High-Density Splat Area)`（勘测外包范围边界模式：高密度高斯泼溅核覆盖区）
   - 核心要素：
     - 构建水坝高斯泼溅锚定盒（`Shape::Box`）、模型矩阵地形对齐基准柱（`Shape::Cylinder`）、高斯协方差核测针球（`Shape::Ellipsoid`）与水利工程包围矩形（`Shape::Rectangle`）。
     - 验证了 `ITwinData.createTilesetForRealityDataId` 3D Gaussian Splats 瓦片集创建、`Matrix4.fromTranslation` 几何位姿对齐、`Terrain.fromWorldTerrain` 地形贴合与大坝局部相机 `zoomTo` 取景。

4. **`japan-buildings`（日本全境 3D 建筑物大数据集、局部地形加载、多地标漫游与动态相机光源）**
   - 官方地址：`https://sandcastle.cesium.com/?id=japan-buildings`
   - 变体全集（4 种日本城市地标与动态照明变体）：
     - `Baseline: Japan 3D Buildings & Regional Terrain (Tokyo Tower Landmark)`（基准日本 3D 建筑大数据集与区域地形：东京塔地标）
     - `Kyoto Nijo Castle Cultural Heritage Landmark Viewpoint`（京都二条城文化遗产地标视角：古都建筑与庭院）
     - `Camera Dynamic Directional Lighting Highlighting Building Facades`（相机动态平行光源：高亮建筑立面反射纹理）
     - `Mt. Fuji - Gotemba Regional Topography Elevation Mode`（富士山-御殿场地势高程模式：山地建筑与地形复合）
   - 核心要素：
     - 构建东京塔地标建筑盒（`Shape::Box`）、相机平行光标定柱（`Shape::Cylinder`）、京都二条城测针球（`Shape::Ellipsoid`）与日本核心区包围矩形（`Shape::Rectangle`）。
     - 验证了 日本全境 3D 建筑瓦片集（Ion 2602291）与区域地形（Ion 2767062）双图层加载、`DirectionalLight` 与 `scene.camera.directionWC` 每帧动态更新、多城市/名胜相机快速导引交互。

---

## 2. 覆盖与测试指标

- **累计完整移植案例数**：255 / 309
- **本批新增测试案例**：4 个
- **Rust 单元测试**：137 个 passed (`entities::tests::batch77_ionsnap_itwin_splats_japan`)
- **Native GPU 测试**：256 passed (`tests/full-recipes-gpu.rs`，包含 255 个完整案例 + 1 个宽折线材质测试)
- **WebGPU 浏览器测试**：255 passed (`docs/full-recipes-browser.json`)
- **Node.js 测试套件**：8 个测试套件全绿（`controls`、`sdk`、`ui`、`terrain`、`sandcastle`、`images`、`camera`、`recipes`）
- **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets -- -D warnings` 零警告通过
