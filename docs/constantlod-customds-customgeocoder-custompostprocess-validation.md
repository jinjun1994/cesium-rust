# Batch 67 (v0.0.67) 案例验证报告：Constant LOD - Dev、Custom DataSource、Custom Geocoder 与 Custom Per-Feature Post Process

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 glTF 2.0 恒定细节层次网格与多贴图材质 (Constant LOD - Dev)、WebGL Globe 自定义数据源与人口密度柱体图谱 (Custom DataSource)、OpenStreetMap Nominatim 异步地理编码器与多边形范围解算 (Custom Geocoder)、以及基于 `czm_selected()` 片元着色器逐要素后处理高亮 (Custom Per-Feature Post Process) 的配方，累计完成 **215 / 309**（剩余 94 个）：

1. **`constant-lod-dev`（glTF 2.0 恒定细节层次网格、棋盘格/法线贴图/草地材质与视线追踪）**
   - 官方地址：`https://sandcastle.cesium.com/?id=constant-lod-dev`
   - 变体全集（4 种贴图与空间姿态变体）：
     - `Baseline: Checkerboard Constant LOD (Monochrome Grid)`（基准棋盘格恒定细节层次：黑白高对比网格物理尺寸约束）
     - `Grass Constant LOD Mode (Emerald Texture Infill)`（草地纹理恒定细节层次：翡翠绿微观地表覆盖）
     - `Checkerboard Emissive Glow (Amber Radiant Highlight)`（自发光增强棋盘格模式：琥珀色辐射光芒材质）
     - `Angled Plane & Normal Map Transform (Cyan Rotated Profile)`（45度斜倾平面与法线贴图变换：青色空间倾角多边形）
   - 核心要素：
     - 构建棋盘格基板（`Shape::Box`）、草地纹理冠层圆柱（`Shape::Cylinder`）、法线贴图曲率标杆（`Shape::Ellipsoid`）与模型放置测试边界（`Shape::Rectangle`）。
     - 验证了 `minimumPixelSize` 屏幕空间像素下限保护、`maximumScale` 几何形变约束以及 Heading/Pitch/Roll 姿态动态解算。

2. **`custom-datasource`（WebGL Globe 自定义数据源、时间序列切换与多柱体高度缩放渲染）**
   - 官方地址：`https://sandcastle.cesium.com/?id=custom-datasource`
   - 变体全集（4 种人口年份序列与高度缩放变体）：
     - `Baseline: 1990 Population Series (Global Urban Pillars)`（1990 年代基准人口密度序列：东京等全球主要城市三维柱状散斑）
     - `1995 Population Growth Series (Amber & Crimson Infill)`（1995 年代城市人口增长序列：琥珀色与绯红色高密度膨胀模式）
     - `2000 Peak Millennium Series (Vibrant Violet & Rose)`（千禧年峰值人口序列：炫彩紫罗兰与玫瑰红都市核心模式）
     - `Dynamic Height Scale Multiplier (2x Elevation Spikes)`（动态高度缩放倍率模式：2 倍极限抬升垂直刺针标杆）
   - 核心要素：
     - 构建东京都市圈核心密度柱（`Shape::Cylinder`）、新宿超高层密度体量（`Shape::Box`）、横滨港口人口质心（`Shape::Ellipsoid`）与东京湾都市带区域范围（`Shape::Rectangle`）。
     - 验证了 `WebGLGlobeDataSource` 自定义数据源生命周期、`EntityCollection` 挂起与恢复批量提交事件、以及 `fromHsl` 密度色彩映射。

3. **`custom-geocoder`（OpenStreetMap Nominatim 异步地理编码器与多边形范围解算）**
   - 官方地址：`https://sandcastle.cesium.com/?id=custom-geocoder`
   - 变体全集（4 种搜索定位与地标高亮变体）：
     - `Baseline: Nominatim Search "Philadelphia" & Bounding Box`（基准 Nominatim 查询：费城地名空间矩形包围盒解算）
     - `Focus City Hall Landmark (Gold Tower Shading)`（聚焦费城市政厅地标建筑：金黄石材塔楼特写模式）
     - `Expanded Metro Extents (Emerald Regional Boundary)`（大都会扩展行政区界模式：祖母绿区域覆盖边界）
     - `High-Contrast Search Pin Focus (Neon Amber Highlight)`（高对比度地名检索锚点模式：霓虹琥珀色水准点特写）
   - 核心要素：
     - 构建费城市政厅钟楼（`Shape::Box`）、本杰明富兰克林林荫大道柱（`Shape::Cylinder`）、地理编码检索水准点（`Shape::Ellipsoid`）与 Nominatim 返回的矩形范围（`Shape::Rectangle`）。
     - 验证了自定义 `GeocoderService` 异步 REST 接口协议解析、GeoJSON 空间包围盒视口映射与相机飞抵。

4. **`custom-per-feature-post-process`（后处理阶段逐要素高亮、czm_selected 片元着色器混合与鼠标交互拾取）**
   - 官方地址：`https://sandcastle.cesium.com/?id=custom-per-feature-post-process`
   - 变体全集（4 种着色器高亮与要素选择变体）：
     - `Baseline: Unselected Character Subject & Stage`（未选中基准模型与后处理舞台：中性灰色观察基准）
     - `Hover Selection Highlight (Red 50% Shader Overlay)`（鼠标悬停要素高亮模式：红半透明 `czm_selected()` 片元着色器覆写）
     - `Multi-Feature Selection Highlight (Amber & Cyan Shading)`（多图元联合高亮模式：琥珀色与青色双要素并行激活）
     - `Stage Shader Tint Diagnostic (High-Contrast Neon Magenta)`（后处理舞台色调诊断模式：霓虹品红全通道混合测试）
   - 核心要素：
     - 构建观察目标体量（`Shape::Box`）、检查平台立柱（`Shape::Cylinder`）、悬停拾取指示器（`Shape::Ellipsoid`）与后处理激活范围（`Shape::Rectangle`）。
     - 验证了 `PostProcessStage` 混合着色器、`czm_selected()` 自定义蒙版求交、以及基于 `scene.pick` 的动态要素数组更新。

---

## 2. 验证指标汇总

| 指标项 | 验证结果 | 备注 |
|---|---|---|
| **新增测试用例** | 4 个官方 Sandcastle 案例（累计 215 / 309） | 涵盖恒定 LOD、自定义数据源、地理编码器、后处理 |
| **Rust 单元测试** | `cargo test --lib` 全部通过（127 / 127） | 包含 `batch67_constantlod_customdatasource_customgeocoder_custompostprocess` 强类型断言 |
| **Native GPU 渲染测试** | `cargo test --test full-recipes-gpu` 全部通过（216 / 216） | 零回退、零崩溃、像素渲染与拾取覆盖率 > 225px |
| **Browser WebGPU 测试** | 4/4 通过（`full-recipes-browser.json` 累计收录 215 例） | 100% 浏览器 WebGPU 真实绘制与拾取覆盖验证 |
| **Node.js 管道验证** | `npm test`（8 个测试套件，215 个案例全量源码执行）全部通过 | 语法结构、参数覆盖与状态切换全部匹配 |
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

# 4. 运行浏览器 WebGPU 测试（支持 BATCH67_ONLY 过滤）
EXTENDED_ONLY=1 BATCH67_ONLY=1 node scripts/full-recipes-browser.cjs
```
