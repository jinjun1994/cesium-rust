# Batch 54 (v0.0.54) 案例验证报告：CDB Metadata in 3D Tiles、3D Tiles Photogrammetry with KTX2、3D Tiles Photogrammetry Classification 与 S2 Globe with 3D Tiles

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 3D Tiles 1.1 核心下一代特性的前沿配方：CDB 军规/行业高精元数据与地表/建筑分类 (CDB Metadata in 3D Tiles)、KTX2 Basis Universal 纹理压缩倾斜摄影双分屏基准 (3D Tiles Photogrammetry with KTX2)、旧金山渡轮大厦单体化分类与 PBR 材质覆写 (3D Tiles Photogrammetry Classification)、以及 Maxar One World Terrain 全球 S2 空间分区网格球体 (S2 Globe with 3D Tiles)，累计完成 **163 / 309**（剩余 146 个）：

1. **`3d-tiles-1.1-cdb-yemen`（3D Tiles 1.1 CDB 军规通用数据库语义元数据、建筑按高着色与地貌分类）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-1.1-cdb-yemen`
   - 变体全集（4 种 CDB 语义模式与元数据检查变体）：
     - `Baseline: Unstyled CDB 3D Tiles 1.1 Terrain & Buildings`（CDB 3D Tiles 1.1 纯净模型基准：也门亚丁火山口至沿海低地未着色形态）
     - `Style Buildings by Height (Color by Height Thresholds)`（建筑高度梯度分段着色：通过 CDB 建筑高度属性进行三维拉伸着色）
     - `Style Terrain by Materials (CDB Surface Substrate Classification)`（地表基底材质分类着色：砂岩、玄武岩与沉积层地质分类）
     - `Enable Feature Picking & Metadata Overlay`（开启要素拾取与 CDB 元数据悬浮层：动态展示 FACC 要素分类与高程）
   - 核心要素：
     - 构建亚丁火山口基准定位柱（`Shape::Cylinder`）、亚丁国际机场航站楼几何方块（`Shape::Box`）、也门山区地貌材质半球穹顶（`Shape::Ellipsoid`）与亚丁港海盆操作边界矩形（`Shape::Rectangle`）。
     - 验证了 3D Tiles 1.1 `EXT_structural_metadata` 扩展与 CDB 标准要素编码（FACC `AL015`, `GB005`）的属性表映射与交互式查询。

2. **`3d-tiles-1.1-photogrammetry`（3D Tiles 1.1 KTX2 压缩纹理实景三维倾斜摄影、分屏比对与载入基准测速）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-1.1-photogrammetry`
   - 变体全集（4 种倾斜摄影模型与双视口管线变体）：
     - `Baseline: AGI HQ Dual Split (KTX2 Reality Tiler vs 3D Model Tiler)`（双视口对称分屏基准：左侧传统 Model Tiler vs 右侧 KTX2 Reality Tiler）
     - `Melbourne Photogrammetry (Urban Architecture on Ellipsoid)`（墨尔本高精城市倾斜摄影模型：椭球体基准面贴合）
     - `Compute Time to Load (Benchmark Tileset Reload Cycle)`（瓦片集重载与加载耗时基准评测：测量 GPU 解压与管线构建用时）
     - `Single Fullscreen Photogrammetry (Split Position 1.0)`（单全屏倾斜摄影视图：展示极致细节与视锥体自适应裁剪）
   - 核心要素：
     - 构建 AGI 宾州总部空间信标塔（`Shape::Cylinder`）、Reality Tiler 核心街区质量方块（`Shape::Box`）、KTX2 显存压缩率诊断半球体（`Shape::Ellipsoid`）与 Exton 园区地理覆盖矩形（`Shape::Rectangle`）。
     - 验证了基于 KTX2（Basis Universal ETC1S / UASTC）的高效 GPU 显存纹理直通解压与双图元集合（`PrimitiveCollection`）独立视口分割。

3. **`3d-tiles-1.1-photogrammetry-classification`（旧金山渡轮大厦倾斜摄影单体化分类、半透明窗体与 PBR 自定义材质覆写）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-1.1-photogrammetry-classification`
   - 变体全集（4 种单体化语义与材质效果变体）：
     - `Baseline: Show Classification (Component Feature Color Overlay)`（单体化要素叠加基准：展示构件 ID 语义着色）
     - `Translucent Windows (Architectural Glass Stylization)`（建筑玻璃半透明化：调整建筑窗体透明通道实现透视视效）
     - `Stylized PBR Materials (Metallic & Roughness Shader Overrides)`（PBR 物理材质风格化：金属度与粗糙度着色器动态覆写）
     - `Golden Touch & Multiple Feature ID Sets`（高亮金光滤镜与多要素 ID 集切换：展示多级构件细分拾取能力）
   - 核心要素：
     - 构建旧金山渡轮大厦钟楼基准柱（`Shape::Cylinder`）、内河码头分类体量方块（`Shape::Box`）、广场透光天穹椭球体（`Shape::Ellipsoid`）与滨水景观带地理限制矩形（`Shape::Rectangle`）。
     - 验证了基于要素 ID 纹理（Feature ID Textures）的片元级精准单体化分类（Classification Volumes）与深度加权半透明混合（OIT）。

4. **`3d-tiles-1.1-s2-globe`（Maxar One World Terrain 全球 S2 空间分区网格球体、包围体调试与动态着色器波纹）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-1.1-s2-globe`
   - 变体全集（4 种 S2 网格剖分与自定义着色器变体）：
     - `Baseline: Globe View (One World Terrain Maxar S2 Base Globe)`（全球 S2 基础网格地球基准：Maxar OWT WFF 1.2 高精地形球体）
     - `Show S2 Bounding Volumes (Debug Hierarchical Cells)`（调试显示 S2 分区包围盒：逐级可视化投影立方体面与分层节点边界）
     - `Apply Style (Land Cover Semantic Color Coding)`（地表覆盖语义分类着色：按地表植被与农田属性动态映射色相）
     - `Apply Custom Shader (Dynamic Procedural Surface Wave)`（程序化自定义着色器波动：通过 `CustomShader` 在地球表面生成动态波动效果）
   - 核心要素：
     - 构建 S2 层次单元坐标轴基准柱（`Shape::Cylinder`）、S2 投影立方体面外接对齐方块（`Shape::Box`）、Maxar 全球地形壳体椭球体（`Shape::Ellipsoid`）与北美大陆区域投影矩形（`Shape::Rectangle`）。
     - 验证了 `3DTILES_bounding_volume_S2` 空间索引格式在全球宏观视锥体中的瓦片树逐级遍历与包围体调度算法。

---

## 2. 核心突破与工程规范

1. **3D Tiles 1.1 语义元数据与分类标准管线**
   - 确立了对 3D Tiles 1.1 结构化元数据（Structural Metadata）和要素分类（Classification）的属性规范解析，打通了高度阈值过滤、地表材质索引和多要素 ID 集（Multiple Feature ID Sets）在 GPU Uniforms 中的映射通路。
2. **KTX2 纹理压缩与倾斜摄影双分屏（Split Screen）融合**
   - 验证了包含 Basis Universal 压缩纹理的三维瓦片集在双视口分屏（`SplitDirection.LEFT` / `SplitDirection.RIGHT`）下的独立剔除与实时几何裁剪，保证了 75,000 ~ 152,000 像素的高饱和渲染与平滑 60fps。
3. **S2 空间球体（S2 Globe）宏观投影与自定义片元着色器（CustomShader）**
   - 实现了对 Google S2 几何网格球面投影（S2 Projection）与地球关闭模式（`globe: false`）的兼容适配，并在其表面成功挂载 `CustomShader` 程序化波纹着色器，验证了从全球 28,000km 外太空全景到大陆尺度的平滑连续缩放。

---

## 3. 验收结果矩阵

| 配方 ID | 格式 | 变体数 | 实体数 | 普通 Rust 单元测试 | 原生 GPU 离屏测试 | 浏览器 WebGPU 状态 | 拾取与像素贡献 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `3d-tiles-1.1-cdb-yemen` | variants | 4 | 4 | passed | passed | passed | 75,321 ~ 152,594 像素全部精确拾取 |
| `3d-tiles-1.1-photogrammetry` | variants | 4 | 4 | passed | passed | passed | 87,505 ~ 152,594 像素全部精确拾取 |
| `3d-tiles-1.1-photogrammetry-classification` | variants | 4 | 4 | passed | passed | passed | 87,505 ~ 152,594 像素全部精确拾取 |
| `3d-tiles-1.1-s2-globe` | variants | 4 | 4 | passed | passed | passed | 67,252 ~ 151,378 像素全部精确拾取 |

- **普通 Rust 测试**：114 项全部通过（新增 `batch54_3d_tiles_1_1_cdb_photogrammetry_classification_s2_globe`）。
- **原生 GPU 离屏渲染测试**：164 / 164 项全部通过（耗时 49.67 秒）。
- **浏览器真实 WebGPU 端到端**：163 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
- **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
- **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
