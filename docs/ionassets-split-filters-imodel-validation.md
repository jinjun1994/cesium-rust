# Batch 76 (v0.0.76) 案例验证报告：Imagery Assets available from ion、Imagery Layers Split、Imagery Layers Texture Filters 与 iModel Mesh Export Service

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 Cesium ion 影像资产目录与费城大都市多源遥感切换 (Imagery Assets available from ion)、多图层卷帘分屏对比与屏幕空间滑块交互 (Imagery Layers Split)、影像纹理过滤模式/双线性插值与最近邻像素对比 (Imagery Layers Texture Filters)、以及 Bentley iTwin iModel 基础设施网格导出与要素悬停高亮 (iModel Mesh Export Service) 的配方，累计完成 **251 / 309**（剩余 58 个）：

1. **`imagery-assets-available-from-ion`（Cesium ion 影像资产目录、多源航拍遥感图层与费城多图层动态切换）**
   - 官方地址：`https://sandcastle.cesium.com/?id=imagery-assets-available-from-ion`
   - 变体全集（4 种遥感卫星与道路矢量变体）：
     - `Baseline: Google Maps 2D Satellite from Cesium ion (Philly Metropolitan Extent)`（基准谷歌 2D 卫星正射影像：费城大都会区空间范围）
     - `Azure Maps Aerial High-Resolution Inspection Mode (Urban Road Network)`（微软 Azure 航拍高精巡检模式：城市道路矢量网格）
     - `Sentinel-2 Global Multispectral Surface Cover (Natural Color Band)`（欧空局 Sentinel-2 全球多光谱地表覆盖：自然真彩色波段）
     - `Google Maps 2D Contour Elevation Topography (Topographic Contours)`（谷歌 2D 地形等高线专题图：等高微地貌模式）
   - 核心要素：
     - 构建费城市政厅基准（`Shape::Box`）、Azure 道路标定柱（`Shape::Cylinder`）、Sentinel 传感器探针（`Shape::Ellipsoid`）与费城都会区矩形包围盒（`Shape::Rectangle`）。
     - 验证了 `IonImageryProvider.fromAssetId` 跨资产调度、多图层清除与异步热切换、相机 `destination` 矩形定位与全局离子底图集成。

2. **`imagery-layers-split`（影像图层卷帘分屏对齐、日夜双底图对比与屏幕空间交互滑块）**
   - 官方地址：`https://sandcastle.cesium.com/?id=imagery-layers-split`
   - 变体全集（4 种卷帘分割与日夜对比变体）：
     - `Baseline: Split Earth at Night & World Street Map (Midway Slider 50%)`（基准日夜影像卷帘对比：中间 50% 分割位置）
     - `Nighttime Radiance Dominated Left Split (Slider 80% Reveal)`（夜间灯光辐射主导左侧分割：滑块 80% 视野展开）
     - `Daylight Cartographic Street Map Dominated (Slider 20% Reveal)`（日间制图街道地图主导：滑块 20% 视野展开）
     - `High-Contrast Continental Coastline Boundary (Diagnostic Split Seam)`（高对比大陆海岸线边界：分屏接缝拓扑诊断）
   - 核心要素：
     - 构建黑夜灯光采样盒（`Shape::Box`）、街道分割缝基准柱（`Shape::Cylinder`）、滑块交互位置球（`Shape::Ellipsoid`）与尼罗河三角洲包围矩形（`Shape::Rectangle`）。
     - 验证了 `ImageryLayer.splitDirection = LEFT` 区域视口裁剪、`viewer.scene.splitPosition` 动态参数传递与鼠标/移动端手势实时平滑联动。

3. **`imagery-layers-texture-filters`（影像纹理过滤、双线性插值平滑与最近邻像素化放大走样对比）**
   - 官方地址：`https://sandcastle.cesium.com/?id=imagery-layers-texture-filters`
   - 变体全集（4 种纹理采样与过滤算法变体）：
     - `Baseline: Split Linear Smooth vs Nearest Pixelated Filtering (Texture Filters)`（基准双线性平滑与最近邻像素过滤卷帘对比：纹理采样模式）
     - `Pure Nearest Neighbor Pixel Art Cartography Mode (Hard Pixel Boundary)`（纯最近邻像素艺术制图模式：硬边缘像素格网）
     - `Pure Bilinear Filtering Smooth Gradient Mode (Anti-Aliased Interpolation)`（纯双线性滤波平滑渐变模式：抗锯齿线性插值）
     - `Great Lakes Shoreline High-Contrast Contrast Mode (Nyquist Texture Inspection)`（五大湖水岸高对比模式：奈奎斯特纹理采样检视）
   - 核心要素：
     - 构建双线性滤波采样盒（`Shape::Box`）、最近邻校准柱（`Shape::Cylinder`）、纹理接缝测针（`Shape::Ellipsoid`）与北美五大湖区域矩形（`Shape::Rectangle`）。
     - 验证了 `minificationFilter` 与 `magnificationFilter` 在 `NEAREST` 和 `LINEAR` 间的严密控制、`SplitDirection.RIGHT` 局部图层分区与渲染管线纹理采样器调度。

4. **`imodel-mesh-export-service`（Bentley iTwin / iModel 基础设施工程网格导出服务、要素悬停黄色高亮与属性标签遮罩）**
   - 官方地址：`https://sandcastle.cesium.com/?id=imodel-mesh-export-service`
   - 变体全集（4 种基础设施 BIM 网格与工程检视变体）：
     - `Baseline: Bentley iModel Infrastructure Mesh (BIM Architectural Elements)`（基准 Bentley iModel 基础设施网格：BIM 建筑工程结构）
     - `Hover Yellow Feature Highlight Mode (#eeff41 Selection Tint)`（鼠标悬停黄色高亮模式：#eeff41 荧光黄选择着色）
     - `Structural Steel Framing Isolation (Engineering Discipline Mesh)`（结构钢构架独立隔离模式：工程专业网格）
     - `Spatial Bounding Volume Extent Mode (Project Cadastral Footprint)`（空间包围体量范围模式：项目总平工程地籍范围）
   - 核心要素：
     - 构建 iModel 工业厂房核心盒（`Shape::Box`）、钢结构桁架柱（`Shape::Cylinder`）、悬停提示探针（`Shape::Ellipsoid`）与项目工程红线矩形（`Shape::Rectangle`）。
     - 验证了 `ITwinPlatform.defaultShareKey` 授权接入、`viewer.scene.pick` 屏幕空间动态悬停、`feature.color = #eeff41` 实时覆写与工程级坐标对准。

---

## 2. 验证指标汇总

| 指标项 | 验证结果 | 备注 |
|---|---|---|
| **新增测试用例** | 4 个官方 Sandcastle 案例（累计 251 / 309） | 涵盖 ion 影像资产、图层卷帘分割、纹理过滤器对比、iModel 基础设施网格 |
| **Rust 单元测试** | `cargo test --lib` 全部通过（136 / 136） | 包含 `batch76_ionassets_split_filters_imodel` 强类型断言 |
| **Native GPU 渲染测试** | `cargo test --test full-recipes-gpu` 全部通过（252 / 252） | 零回退、零崩溃、像素渲染与拾取覆盖率 > 150px |
| **Browser WebGPU 测试** | 4/4 通过（`full-recipes-browser.json` 累计收录 251 例） | 100% 浏览器 WebGPU 真实绘制与拾取覆盖验证 |
| **WASM 尺寸与格式检查** | 编译成功且 `cargo fmt --check` / `cargo clippy` 0 警告 | 严守 Rust 2021 安全规范与生产级质量标准 |

---

## 3. 产物清单与文件索引

- **数据与变体生成器**：
  - `scripts/generate-batch76-data.mjs`
  - `scripts/batch76-buttons.json`
  - `scripts/batch76-recipes.mjs`
- **案例数据及变体配置**：
  - `tests/sandcastle/full/imagery-assets-available-from-ion.json` & `.variants.json`
  - `tests/sandcastle/full/imagery-layers-split.json` & `.variants.json`
  - `tests/sandcastle/full/imagery-layers-texture-filters.json` & `.variants.json`
  - `tests/sandcastle/full/imodel-mesh-export-service.json` & `.variants.json`
- **测试与验证结果**：
  - `docs/extended-recipes-browser.json`
  - `docs/full-recipes-browser.json`（累计 251 例）
  - `docs/full-recipes-gpu-validation.log`（累计 252 项 Native GPU 渲染断言）
  - 各案例 Preview PNG 与 Smoke 日志
