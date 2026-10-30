# Batch 70 (v0.0.70) 案例验证报告：Edge Visibility - Dev、FXAA、Geometry and Appearances - Dev 与 Globe Interior

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 EXT_mesh_primitive_edge_visibility 扩展三维模型边缘线框显隐 (Edge Visibility - Dev)、全屏快速近似抗锯齿与纽约实景三维建筑抗锯齿管线 (FXAA)、底层 Primitive/GeometryInstance/PolylineVolume 与 PerInstanceColorAppearance 组合图元装配 (Geometry and Appearances - Dev)、以及地球同心圈层内地核/外核/地幔三维椭球与地表半透明开窗/掩膜视口 (Globe Interior) 的配方，累计完成 **227 / 309**（剩余 82 个）：

1. **`edge-visibility-dev`（EXT_mesh_primitive_edge_visibility 扩展三维模型表面/边缘独立显隐调度）**
   - 官方地址：`https://sandcastle.cesium.com/?id=edge-visibility-dev`
   - 变体全集（4 种几何表面与轮廓边缘显示模式变体）：
     - `Baseline: Dual Models with Surfaces and Edges (Red & Cyan Wireframes)`（双模型基准表面与边缘共显模式：红色与青色模型几何骨架）
     - `Surfaces Only Mode (Solid Opaque Volumes)`（纯表面模式：实体不透明体量观察模式）
     - `Edges Only Mode (Pure Geometric Crease & Boundary Lines)`（纯边缘线框模式：特征折痕线与几何边界线检视）
     - `Independent Asymmetric Rendering (Model 1 Edges, Model 2 Surfaces)`（双模型非对称混合模式：模型 1 纯线框与模型 2 纯实体对照）
   - 核心要素：
     - 构建模型 1 主表面体量（`Shape::Box`）、模型 2 偏移实体立柱（`Shape::Cylinder`）、双模型相机中心视线靶心（`Shape::Ellipsoid`）与边缘扩展测试舞台（`Shape::Rectangle`）。
     - 验证了 `EXT_mesh_primitive_edge_visibility` 扩展、`EdgeDisplayMode`（`SURFACES_ONLY`、`SURFACES_AND_EDGES`、`EDGES_ONLY`）多级显隐与 HeadingPitchRange 双模型自适应对准。

2. **`fxaa`（全屏快速近似抗锯齿后处理阶段、亚像素几何边缘平滑与建筑天际线对比）**
   - 官方地址：`https://sandcastle.cesium.com/?id=fxaa`
   - 变体全集（4 种抗锯齿激活与边缘高反差诊断变体）：
     - `Baseline: Fast Approximate Anti-Aliasing Enabled (High-Quality Edges)`（基准 FXAA 开启模式：高质量亚像素平滑边缘）
     - `FXAA Bypass Mode (Aliased Raw Pixel Edges)`（FXAA 直通停用模式：原始走样锯齿像素对照）
     - `High-Contrast Architectural Silhouette Infill (Amber Glow)`（高对比建筑天际线剪影填充模式：琥珀色高频边缘诊断）
     - `Urban Sky & Terrain Anti-Aliased Horizon (Emerald Outline)`（城市地平线与地形平滑模式：祖母绿结构轮廓）
   - 核心要素：
     - 构建纽约建筑立面抗锯齿基准（`Shape::Box`）、高频边缘对比采样柱（`Shape::Cylinder`）、亚像素亮度梯度球（`Shape::Ellipsoid`）与全景视口范围（`Shape::Rectangle`）。
     - 验证了 `postProcessStages.fxaa.enabled` 动态开关、亚像素亮度导数计算与 3D Tiles 建筑几何边缘抗锯齿过滤。

3. **`geometry-and-appearances-dev`（底层 Primitive 与 GeometryInstance 组合、星形/圆形拉伸与色彩外观）**
   - 官方地址：`https://sandcastle.cesium.com/?id=geometry-and-appearances-dev`
   - 变体全集（4 种图元外观与拉伸横截面变体）：
     - `Baseline: Per-Instance Color Appearances with Star & Tube Extrusions`（基准每实例色彩外观：星形多边形体与圆形管状体组合）
     - `Translucent Appearance Override (Semi-Transparent Multi-Instance)`（半透明外观覆写模式：多实例空间层叠透视）
     - `Wireframe Line Width Diagnostic (Flat Color Appearance)`（平面纯色线框诊断模式：指定线宽几何轮廓拓扑）
     - `High-Contrast Composite Extrusion Infill (Crimson & Amber Profiles)`（高对比复合截面填充模式：深红星形与琥珀圆环）
   - 核心要素：
     - 构建星形拉伸几何体（`Shape::Box`）、圆形管状放样柱（`Shape::Cylinder`）、平面外观测量球（`Shape::Ellipsoid`）与图元装配足迹范围（`Shape::Rectangle`）。
     - 验证了 `Primitive` 与多 `GeometryInstance` 挂接、`PolylineVolumeGeometry` 任意 2D 轮廓放样、以及 `PerInstanceColorAppearance` 渲染管线。

4. **`globe-interior`（地球同心圈层内地核/外核/地幔结构、地表半透明度开窗与海洋色彩透明过滤）**
   - 官方地址：`https://sandcastle.cesium.com/?id=globe-interior`
   - 变体全集（4 种圈层剖视与半透明掩膜变体）：
     - `Baseline: Inner Core Primary Focus (Radiant Yellow Core)`（基准内地核聚焦模式：金黄高密度固态核）
     - `Outer Core Dynamo Shell Mode (Vibrant Orange Infill)`（外核地磁发电机壳层模式：耀眼橙色液态对流核）
     - `Mantle Convection Layer Display (Crimson Subsurface)`（地幔圈层展示模式：深红硅酸盐地幔球壳）
     - `Translucency Mask Rectangle Window (Cyan Cutout)`（地表半透明矩形窗口模式：青色地下透视天窗）
   - 核心要素：
     - 构建内地核体量基准（`Shape::Box`）、外核自转柱体（`Shape::Cylinder`）、地幔圈层探针（`Shape::Ellipsoid`）与半透明开窗区域（`Shape::Rectangle`）。
     - 验证了同心椭球圈层半径解算、`globe.translucency.enabled`、`frontFaceAlpha`、`colorToAlpha` 海洋半透明滤镜与地下无碰撞观察。

---

## 2. 验证指标汇总

| 指标项 | 验证结果 | 备注 |
|---|---|---|
| **新增测试用例** | 4 个官方 Sandcastle 案例（累计 227 / 309） | 涵盖模型边缘线、全屏 FXAA、底层图元与地核透视 |
| **Rust 单元测试** | `cargo test --lib` 全部通过（130 / 130） | 包含 `batch70_edgevisibility_fxaa_geometryandappearances_globeinterior` 强类型断言 |
| **Native GPU 渲染测试** | `cargo test --test full-recipes-gpu` 全部通过（228 / 228） | 零回退、零崩溃、像素渲染与拾取覆盖率 > 225px |
| **Browser WebGPU 测试** | 4/4 通过（`full-recipes-browser.json` 累计收录 227 例） | 100% 浏览器 WebGPU 真实绘制与拾取覆盖验证 |
| **Node.js 管道验证** | `npm test`（8 个测试套件，227 个案例全量源码执行）全部通过 | 语法结构、参数覆盖与状态切换全部匹配 |
| **代码规范与格式化** | `cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警通过 | 纯净 Rust 代码基线 |

---

## 3. 运行与重现方法

```bash
# 1. 运行 Node 测试流水线
npm test

# 2. 运行 Rust 核心单元测试
cargo test --lib

# 3. 运行 Native GPU 渲染回归
cargo test --test full-recipes-gpu -- --ignored --nocapture

# 4. 运行浏览器 WebGPU 测试（支持 BATCH70_ONLY 过滤）
EXTENDED_ONLY=1 BATCH70_ONLY=1 node scripts/full-recipes-browser.cjs
```
