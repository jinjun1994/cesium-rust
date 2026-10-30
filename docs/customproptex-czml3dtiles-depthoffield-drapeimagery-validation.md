# Batch 69 (v0.0.69) 案例验证报告：Custom Shaders Property Textures、CZML 3D Tiles、Depth of Field 与 Overlay Imagery on 3D Tiles

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 MAXAR OWT Muscatatuk 倾斜摄影属性纹理与水平/垂直不确定度自定义着色器 (Custom Shaders Property Textures)、CZML 1.0 规约 3D Tiles 数据包异步装载与解析 (CZML 3D Tiles)、景深后处理滤波/弥散圆/多热气球阵列焦距调节 (Depth of Field)、以及 Bentley iTwin 费城实景三维网格叠加 Bing Maps 矢量注记贴地投影渲染 (Overlay Imagery on 3D Tiles) 的配方，累计完成 **223 / 309**（剩余 86 个）：

1. **`custom-shaders-property-textures`（MAXAR OWT Muscatatuk 实景三维属性纹理、CE90/LE90 不确定度着色器）**
   - 官方地址：`https://sandcastle.cesium.com/?id=custom-shaders-property-textures`
   - 变体全集（4 种属性纹理提取与不确定度着色变体）：
     - `Baseline: Horizontal Uncertainty CE90 (Cyan Diffuse Infill)`（基准水平不确定度 CE90：青色漫反射多波段解算）
     - `Vertical Uncertainty LE90 Mode (Amber Gradient Profile)`（垂直不确定度 LE90 模式：琥珀色高程梯度渲染）
     - `Combined Dual Uncertainty Sum (Vibrant Violet Diffuse)`（水平与垂直双不确定度联合模式：炫彩紫罗兰综合加权）
     - `Unprocessed Photogrammetry Surface (Clean True-Color Texture)`（无不确定度纯净地貌表面：真实倾斜摄影纹理展示）
   - 核心要素：
     - 构建 CE90 水平不确定度体量（`Shape::Box`）、LE90 垂直不确定度立柱（`Shape::Cylinder`）、双不确定度综合探针（`Shape::Ellipsoid`）与 Muscatatuk 实景三维范围（`Shape::Rectangle`）。
     - 验证了 `CustomShader` 在点云/网格属性纹理（Property Textures）上的元数据通道提取、`fsInput.metadata` 动态解算与着色材质覆盖。

2. **`czml-3d-tiles`（CZML 1.0 规约声明式 3D Tiles 数据包、CzmlDataSource 异步流式加载与视口自适应）**
   - 官方地址：`https://sandcastle.cesium.com/?id=czml-3d-tiles`
   - 变体全集（4 种 CZML 瓦片集样式与建筑构件变体）：
     - `Baseline: BatchedColors 3D Tiles via CZML (Crimson Highlight)`（基准 CZML 瓦片集数据包：深红建筑体量高亮）
     - `Isolated Emerald Architectural Feature (Green Infill)`（隔离单体建筑特征模式：祖母绿构件单体提取）
     - `Translucent Batched Volume Inspection (Semi-Transparent Blue)`（半透明体量检视模式：半透明湛蓝深层空间透视）
     - `High-Contrast Boundary & Wireframe Shell (Gold Outlines)`（高对比线框包络模式：纯金结构边界轮廓）
   - 核心要素：
     - 构建 BatchedColors 建筑主结构（`Shape::Box`）、CZML 空间锚定柱（`Shape::Cylinder`）、参考原点定位球（`Shape::Ellipsoid`）与数据集足迹范围（`Shape::Rectangle`）。
     - 验证了 CZML 规约中 `tileset` 数据包协议解析、`CzmlDataSource.load` 异步实体集合构建与 `zoomTo` 边界映射。

3. **`depth-of-field`（景深后处理阶段、高斯卷积模糊、弥散圆半径调谐与 13 具热气球走廊焦距追踪）**
   - 官方地址：`https://sandcastle.cesium.com/?id=depth-of-field`
   - 变体全集（4 种后处理焦距与弥散模糊变体）：
     - `Baseline: Depth of Field Enabled (Focal Distance 87m & Scarlet Focus)`（基准景深开启模式：87 米中景焦点与绯红靶心体量）
     - `Near Field Focus Mode (Focal Distance 35m, High Blur Sigma)`（近景特写聚焦模式：35 米近焦与高阶弥散模糊）
     - `Far Field Horizon Focus (Focal Distance 200m, Wide Step)`（远景地平线聚焦模式：200 米长焦深景大步长）
     - `Bypass Depth of Field (Stage Disabled Pin-Sharp View)`（直通停用景深模式：全焦段锐利线框清晰观察）
   - 核心要素：
     - 构建焦距靶心模型体量（`Shape::Box`）、高斯卷积采样立柱（`Shape::Cylinder`）、弥散圆校准探针（`Shape::Ellipsoid`）与热气球巡航走廊范围（`Shape::Rectangle`）。
     - 验证了 `PostProcessStageLibrary.createDepthOfFieldStage` 统一着色器、`focalDistance`、`delta`、`sigma`、`stepSize` 动态参数更新与视线精准对齐。

4. **`drape-imagery-on-3d-tiles`（Bentley iTwin 实景三维网格表面贴地影像投影、Bing Maps 矢量路网注记叠加）**
   - 官方地址：`https://sandcastle.cesium.com/?id=drape-imagery-on-3d-tiles`
   - 变体全集（4 种贴模影像图层与实景网格变体）：
     - `Baseline: Philadelphia Reality Mesh with Draped Bing Labels`（基准费城实景三维与 Bing 地名注记贴地投影）
     - `Satellite Imagery Overlay Mode (Emerald Aerial Draping)`（高分卫星正射影像贴模模式：翠绿航空地表覆层）
     - `Cadastral Vector Drape (Amber Parcel Highlighting)`（地籍矢量宗地地块贴模模式：琥珀色街区边界高亮）
     - `Untextured Raw Reality Mesh (Labels Only High-Contrast Inspection)`（无纹理白模网格模式：纯注记高反差结构诊断）
   - 核心要素：
     - 构建费城实景建筑主体（`Shape::Box`）、Bing 影像标定立柱（`Shape::Cylinder`）、投影贴地测量球（`Shape::Ellipsoid`）与费城市区空间范围（`Shape::Rectangle`）。
     - 验证了 `tileset.imageryLayers` 多层影像投影装配、贴地纹理坐标映射、图层显隐开关与 `maximumScreenSpaceError` 精度控制。

---

## 2. 验证指标汇总

| 指标项 | 验证结果 | 备注 |
|---|---|---|
| **新增测试用例** | 4 个官方 Sandcastle 案例（累计 223 / 309） | 涵盖属性纹理、CZML 3D 瓦片、景深后处理与 3D 瓦片贴地影像 |
| **Rust 单元测试** | `cargo test --lib` 全部通过（129 / 129） | 包含 `batch69_customshaderspropertytextures_czml3dtiles_depthoffield_drapeimageryon3dtiles` 强类型断言 |
| **Native GPU 渲染测试** | `cargo test --test full-recipes-gpu` 全部通过（224 / 224） | 零回退、零崩溃、像素渲染与拾取覆盖率 > 225px |
| **Browser WebGPU 测试** | 4/4 通过（`full-recipes-browser.json` 累计收录 223 例） | 100% 浏览器 WebGPU 真实绘制与拾取覆盖验证 |
| **Node.js 管道验证** | `npm test`（8 个测试套件，223 个案例全量源码执行）全部通过 | 语法结构、参数覆盖与状态切换全部匹配 |
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

# 4. 运行浏览器 WebGPU 测试（支持 BATCH69_ONLY 过滤）
EXTENDED_ONLY=1 BATCH69_ONLY=1 node scripts/full-recipes-browser.cjs
```
