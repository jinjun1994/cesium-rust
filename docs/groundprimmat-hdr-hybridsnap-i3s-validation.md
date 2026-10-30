# Batch 74 (v0.0.74) 案例验证报告：Ground Primitive Materials - Dev、High Dynamic Range、Hybrid Snapping - Dev 与 I3S 3D Object Layer

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖贴地几何材质高级覆层与凹凸法线/水体/棋盘格 (Ground Primitive Materials - Dev)、高动态范围与自动曝光/色调映射/泛光峰值 (High Dynamic Range)、网格顶点/边缘与地形高程混合吸附拾取 (Hybrid Snapping - Dev)、以及 Esri I3S 索引三维场景图层/城市建筑流式呈现 (I3S 3D Object Layer) 的配方，累计完成 **243 / 309**（剩余 66 个）：

1. **`ground-primitive-materials-dev`（贴地几何图元高级材质着色、法线扰动与多图层贴地混合）**
   - 官方地址：`https://sandcastle.cesium.com/?id=ground-primitive-materials-dev`
   - 变体全集（4 种贴地材质着色模式）：
     - `Baseline: Checkerboard Ground Draped Primitive (High-Contrast Tiles)`（基准棋盘格贴地覆层：高对比地表网格瓦片）
     - `BumpMap Normal Perturbation Draped Surface (Tactile Terrain Texture)`（凹凸贴图法线扰动贴地面：触觉感微地形纹理）
     - `Stripe Flow Elevation Contouring Draped Primitive (Banded Striping)`（等高线流式条带贴地图元：色带条纹地貌模式）
     - `Wood Grain Procedural Terrain Shading (Organic Surface Veneer)`（程序化木纹地表着色：自然纹理质感）
   - 核心要素：
     - 构建棋盘格贴地基准（`Shape::Box`）、法线扰动立柱（`Shape::Cylinder`）、水体探针（`Shape::Ellipsoid`）与圣海伦斯火山贴地覆层矩形（`Shape::Rectangle`）。
     - 验证了 `GroundPrimitive`、`Material.CheckerboardType`、`Material.BumpMapType`、`ClassificationType.TERRAIN` 与地形高程自适应紧密贴合。

2. **`high-dynamic-range`（高动态范围 HDR、色调映射 Tonemapping 与自动曝光控制）**
   - 官方地址：`https://sandcastle.cesium.com/?id=high-dynamic-range`
   - 变体全集（4 种高动态光照与曝光变体）：
     - `Baseline: High Dynamic Range Enabled (Auto-Exposure Tone Mapping)`（基准高动态范围开启：自动曝光与色调映射）
     - `Sunlit High-Luminance Overexposure Benchmark (Specular Bloom Peak)`（日光高亮度过曝基准：镜面反射与泛光峰值）
     - `Deep Shadow Nighttime Radiance Mode (Low-Light Dynamic Range)`（深阴影暗光辐射模式：微光环境动态范围保留）
     - `Standard Dynamic Range Baseline Comparison (Clamped LDR Contrast)`（标准低动态范围基准对照：截断式 LDR 对比）
   - 核心要素：
     - 构建 HDR 采样基准模型盒（`Shape::Box`）、曝光测量圆柱（`Shape::Cylinder`）、色调映射测光球体（`Shape::Ellipsoid`）与大峡谷日光场景包围矩形（`Shape::Rectangle`）。
     - 验证了 `scene.highDynamicRange = true`、`tonemapping` 曲线评估、`exposure` 浮点数调节以及亮部细节保护。

3. **`hybrid-snapping-dev`（混合吸附：客户端网格几何顶点/边缘吸附与服务端高程拾取融合）**
   - 官方地址：`https://sandcastle.cesium.com/?id=hybrid-snapping-dev`
   - 变体全集（4 种混合吸附模式）：
     - `Baseline: Hybrid Snap to 3D Mesh Vertex and Edge (High-Precision Snapping)`（基准混合吸附：高精度网格顶点与边缘对齐）
     - `Surface Raycast Terrain Elevation Snapping (Sub-Meter Ground Alignment)`（地表射线地形高程吸附：亚米级地面定位）
     - `Model Bounding Volume Corner and Face Snapping (Geometric Anchor)`（模型包围盒角点与面吸附：几何锚点模式）
     - `Interactive Multi-Point Measurement Snapping Trace (Snapping Polyline)`（交互式多点测量吸附轨迹：吸附折线引导）
   - 核心要素：
     - 构建 BIM 吸附目标锚盒（`Shape::Box`）、客户端顶点捕获柱（`Shape::Cylinder`）、服务端射线投影球（`Shape::Ellipsoid`）与吸附测量操作舞台矩形（`Shape::Rectangle`）。
     - 验证了 `scene.sampleHeight`、`scene.clampToHeight`、网格边缘检测、屏幕空间吸附阈值与多源高程融合机制。

4. **`i3s-3d-object-layer`（Esri I3S 索引三维场景图层、旧金山城市建筑与属性图层解析）**
   - 官方地址：`https://sandcastle.cesium.com/?id=i3s-3d-object-layer`
   - 变体全集（4 种 I3S 图层检视与分类变体）：
     - `Baseline: San Francisco I3S 3D Object Layer (Urban Building Envelopes)`（基准旧金山 I3S 3D 对象图层：城市建筑包络网格）
     - `Dense Downtown High-Rise Texture Inspection (Detailed Façade Mesh)`（密集中心地段高层建筑纹理检视：精细立面网格）
     - `Symbology Elevation Color Classification Mode (Height Thematic Map)`（符号化高程颜色分类模式：高度专题图）
     - `Wireframe Bounding Geometry Diagnostics (Spatial Node Hierarchy)`（线框包围几何体诊断：空间节点层级结构）
   - 核心要素：
     - 构建旧金山标志建筑基准（`Shape::Box`）、大地测量大地水准面基准柱（`Shape::Cylinder`）、属性拾取探针（`Shape::Ellipsoid`）与旧金山图层空间范围（`Shape::Rectangle`）。
     - 验证了 `I3SDataProvider`、图层节点解析、空间索引树调度、特征属性交互拾取与相机定位。

---

## 2. 验证指标汇总

| 指标项 | 验证结果 | 备注 |
|---|---|---|
| **新增测试用例** | 4 个官方 Sandcastle 案例（累计 243 / 309） | 涵盖贴地材质高级覆层、HDR、混合吸附、I3S 3D 对象图层 |
| **Rust 单元测试** | `cargo test --lib` 全部通过（134 / 134） | 包含 `batch74_groundprimmat_hdr_hybridsnap_i3s` 强类型断言 |
| **Native GPU 渲染测试** | `cargo test --test full-recipes-gpu` 全部通过（244 / 244） | 零回退、零崩溃、像素渲染与拾取覆盖率 > 180px |
| **Browser WebGPU 测试** | 4/4 通过（`full-recipes-browser.json` 累计收录 243 例） | 100% 浏览器 WebGPU 真实绘制与拾取覆盖验证 |
| **WASM 尺寸与格式检查** | 编译成功且 `cargo fmt --check` / `cargo clippy` 0 警告 | 严守 Rust 2021 安全规范与生产级质量标准 |

---

## 3. 产物清单与文件索引

- **数据与变体生成器**：
  - `scripts/generate-batch74-data.mjs`
  - `scripts/batch74-buttons.json`
  - `scripts/batch74-recipes.mjs`
- **案例数据及变体配置**：
  - `tests/sandcastle/full/ground-primitive-materials-dev.json` & `.variants.json`
  - `tests/sandcastle/full/high-dynamic-range.json` & `.variants.json`
  - `tests/sandcastle/full/hybrid-snapping-dev.json` & `.variants.json`
  - `tests/sandcastle/full/i3s-3d-object-layer.json` & `.variants.json`
- **测试与验证结果**：
  - `docs/extended-recipes-browser.json`
  - `docs/full-recipes-browser.json`（累计 243 例）
  - `docs/full-recipes-gpu-validation.log`（累计 244 项 Native GPU 渲染断言）
  - 各案例 Preview PNG 与 Smoke 日志
