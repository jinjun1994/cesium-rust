# Batch 64 (v0.0.64) 案例验证报告：Aerometrex San Francisco、ArcGIS Tiled Elevation Terrain、ArcticDEM 与 Cesium Inspector

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 Aerometrex 旧金山高精倾斜摄影实景三维与微米级街道网格 (Aerometrex San Francisco)、Esri ArcGIS 3D 全球瓦片高程切片地形服务 (ArcGIS Tiled Elevation Terrain)、北极高保真 ArcticDEM 冰川与北美最高峰极地地形 (ArcticDEM)、以及 Cesium Inspector 引擎混合诊断面板与图元深度测试 (Cesium Inspector) 的配方，累计完成 **203 / 309**（剩余 106 个）：

1. **`aerometrex-san-francisco`（Aerometrex 旧金山倾斜摄影实景三维高精模型、街道级增强与地标巡游）**
   - 官方地址：`https://sandcastle.cesium.com/?id=aerometrex-san-francisco`
   - 变体全集（4 种地标巡游与街景增强分辨率变体）：
     - `Baseline: San Francisco Waterfront & Landmarks`（旧金山滨水区与标志性建筑基准全景）
     - `Ferry Building & Embarcadero (~6mm/px)`（渡轮大厦街景增强微米级细节模式：~6mm 像素分辨率）
     - `Pier 39 Marina (2cm/px High Resolution)`（39 号码头游艇港湾高精度网格模式：2cm 像素分辨率）
     - `Financial District Skyline & Lombard Russian Hill`（泛美金字塔金融中心天际线与九曲花街折线透视模式）
   - 核心要素：
     - 构建渡轮大厦钟楼（`Shape::Box`）、39 号码头圆柱（`Shape::Cylinder`）、泛美金字塔高耸塔楼（`Shape::Box`）、九曲花街回折丘陵椭球体（`Shape::Ellipsoid`）与旧金山海湾滨水区覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `maximumScreenSpaceError: 4` 紧密屏幕空间误差控制，结合 `Cartesian3` 与 `HeadingPitchRoll` 的多点航拍相机快速平滑巡航定位。

2. **`arcgis-tiled-elevation-terrain`（Esri ArcGIS 全球瓦片高程影像切片服务与 LERC 压缩地貌解算）**
   - 官方地址：`https://sandcastle.cesium.com/?id=arcgis-tiled-elevation-terrain`
   - 变体全集（4 种峡谷断层高程剖面与采样基准变体）：
     - `Grand Canyon Elevation Profile (Global Terrain3D LERC)`（科罗拉多大峡谷真实地势高程切片基准）
     - `High-Plateau Rim Elevation Benchmarks`（大峡谷南缘高原测绘基准标高点模式）
     - `Inner Gorge Riverbed Contour Verification`（大峡谷底深槽科罗拉多河床水准面验证模式）
     - `All Elevation Grid Ground Control Points`（全高程网格地面控制点高对比校准模式）
   - 核心要素：
     - 构建南缘大地测绘标杆（`Shape::Cylinder`）、马瑟观景点悬崖台地（`Shape::Box`）、大峡谷底水准面基准椭球（`Shape::Ellipsoid`）与局部切片高程覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `ArcGISTiledElevationTerrainProvider.fromUrl` 异步初始化、LERC 压缩高程瓦片空间拓扑与异常回退弹窗控制。

3. **`arcticdem`（极地地理空间中心 ArcticDEM 2米极高精北极立体摄影测绘地形与阿拉斯加巡礼）**
   - 官方地址：`https://sandcastle.cesium.com/?id=arcticdem`
   - 变体全集（4 种极地极高海拔与冰川峡谷变体）：
     - `Baseline: Denali & Alaska Range High Elevation Profile`（迪纳利峰与阿拉斯加山脉极高海拔地势基准）
     - `Denali Summit Focus (Highest Peak 6,190m)`（北美最高峰麦金利峰 6190 米顶峰定位模式）
     - `Anchorage Chugach Mountains & Glacier Trunk`（安克雷奇楚加奇山脉与卡希尔特纳冰川干流模式）
     - `Subarctic Lateral Moraine & Braided Plain`（极地侧碛冰碛碎屑垄与辫状河床平原模式）
   - 核心要素：
     - 构建迪纳利极顶标高圆柱（`Shape::Cylinder`）、大型冰川舌体（`Shape::Box`）、冰碛碎屑岩堆椭球体（`Shape::Ellipsoid`）与极地条带立体摄影覆盖矩形（`Shape::Rectangle`）。
     - 验证了基于 Cesium ion Asset 3956 的 ArcticDEM Release 4 全球切片加载、极地大倾角观察姿态定位与零耗时初始化视角复位。

4. **`cesium-inspector`（Cesium Inspector 调试混入、地形深度遮挡、几何体图元与广告牌集合）**
   - 官方地址：`https://sandcastle.cesium.com/?id=cesium-inspector`
   - 变体全集（4 种图元结构与深度调试变体）：
     - `Inspector Diagnostics: Wireframe & Primitives`（检视器诊断基准：多几何图元综合诊断）
     - `Inspect Translucent Box Primitive Geometry`（半透明红色 BoxGeometry 封闭外观图元检视）
     - `Inspect Elevated Rectangle Extruded Surface`（高空 100km 蓝色拉伸矩形几何体图元检视）
     - `Inspect Billboard Overlay & Depth Buffer`（广告牌图层深度缓冲遮挡与地形碰撞检视）
   - 核心要素：
     - 构建半透明红色封闭图元方块（`Shape::Box`）、高空蓝色拉伸图元矩形（`Shape::Rectangle`）、Cesium Logo 广告牌诊断椭球（`Shape::Ellipsoid`）与地表深度测试基准圆柱（`Shape::Cylinder`）。
     - 验证了 `viewer.extend(viewerCesiumInspectorMixin)`、`scene.globe.depthTestAgainstTerrain = true` 地形深度测试、以及多图元实例（`GeometryInstance` 与 `PerInstanceColorAppearance`）在 WebGPU 渲染管线中的稳定协同。

---

## 2. 跨平台与四层验收指标

| 案例 ID | 上游源码 SHA-256 | 实体数 | 变体数 | 原生 GPU (Metal/Vulkan) | 真实浏览器 WebGPU | 像素拾取判定 |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| `aerometrex-san-francisco` | `07083a059d09c6ebf2aa06399ba09257d079eb73d6eb2a853ecad7e30d4a9ec0` | 5 | 4 | `passed (90f)` | `passed` | 5/5 地标构件与海湾边界全拾取 |
| `arcgis-tiled-elevation-terrain` | `c547a6db0024f2b189ff7056094ea6be0481ecff699ef7fa1d92a0e44f80879d` | 4 | 4 | `passed (90f)` | `passed` | 4/4 测绘标杆与高程边界全拾取 |
| `arcticdem` | `4de76cc09594aa0cfdc37bc865d1d6a36f6d0a7905ee71d182270921df67140e` | 4 | 4 | `passed (90f)` | `passed` | 4/4 顶峰冰川与条带范围全拾取 |
| `cesium-inspector` | `f867a46ba62b9a101f375c3f3bc6e8b2bf3eeaa38699b646c0d8df3027b6c507` | 4 | 4 | `passed (90f)` | `passed` | 4/4 调试图元与深度标杆全拾取 |

---

## 3. 自动化测试与工程一致性保障

- **单元测试**：`cargo test --lib batch64` 通过（覆盖 4 种地标与高精地形/图元结构体解码与变体契约）。
- **GPU 离屏测试**：`cargo test --test full-recipes-gpu -- --ignored` 执行 204 项测试全部通过（0 失败，覆盖 90 帧状态留痕与相机投影计算）。
- **浏览器 WebGPU**：通过 Playwright 在 Chromium 环境中执行无头 WebGPU 渲染测试，全部 4 项案例通过像素差异检验（`samples.count > 8`）与鼠标拾取测试。
- **静态质量保障**：`npm test` 8 个测试套件全绿，`cargo fmt --check` 严格格式化，`cargo clippy --workspace --all-targets` 0 警告通过。
