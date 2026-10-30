# Batch 62 (v0.0.62) 案例验证报告：Vertical Exaggeration with 3D Tiles、Design Model – Layer Control and Metadata Picking、Design Model – Terrain Inset 与 Design Model – Elements API Query

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖谷歌实景三维高程垂直夸张参数调谐 (Vertical Exaggeration with 3D Tiles)、Autodesk Revit 斯诺登大厦多专业 BIM 设计图层与属性拾取 (Design Model – Layer Control and Metadata Picking)、建筑基地 GeoJSON 多边形地形开挖与设计模型嵌合 (Design Model – Terrain Inset)、以及 Cesium ion 资产构件级 Elements API 结构化检索与空间定位 (Design Model – Elements API Query) 的配方，累计完成 **195 / 309**（剩余 114 个）：

1. **`3d-tiles-vertical-exaggeration`（谷歌高精实景三维瓦片集地表垂直夸张、相对基准高度调谐与阿尔卑斯山系视角）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-vertical-exaggeration`
   - 变体全集（4 种垂直夸张与高程基准变体）：
     - `Baseline: 3x Vertical Exaggeration (scene.verticalExaggeration = 3.0)`（3 倍地形垂直起伏夸张基准）
     - `5x Extreme Alpine Exaggeration (scene.verticalExaggeration = 5.0)`（5 倍极致高山峡谷夸张模式：强化山脊地形梯度）
     - `1x Realistic True-Scale Relief (scene.verticalExaggeration = 1.0)`（1 倍真实物理尺度真实地形起伏基准）
     - `Exaggeration with Non-Zero Relative Height Offset (relativeHeight = 1500.0)`（引入 1500 米非零相对高度基准面偏移模式）
   - 核心要素：
     - 构建山脊测绘基准柱（`Shape::Cylinder`）、夸张地貌采样方块（`Shape::Box`）、高山大地测量球体（`Shape::Ellipsoid`）与富士山起伏覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `scene.verticalExaggeration`、`scene.verticalExaggerationRelativeHeight` 在非地球环境（`globe: false`）下的 WebGPU 顶点着色变换。

2. **`aec-architectural-design`（Autodesk Revit 斯诺登大厦多专业协同 BIM、基地多边形开挖裁剪与属性表拾取）**
   - 官方地址：`https://sandcastle.cesium.com/?id=aec-architectural-design`
   - 变体全集（4 种专业图层显隐与环境光遮蔽变体）：
     - `Baseline: Snowdon Towers Architecture & Facade Active with Site Clipping`（建筑与幕墙图层基准激活、场地多边形裁剪）
     - `Structural & MEP Engineering Focus (Structural, HVAC, Plumbing Active)`（结构工程与暖通机电专业聚焦模式）
     - `Full Multi-Discipline BIM Convergence (All 7 Tileset Layers Active)`（建筑、幕墙、结构、电气、暖通、给排水、场地七大专业全开）
     - `Pure Google Photorealistic Context without Architectural Inset`（关闭建筑模型模式：纯实景三维环境上下文对比）
   - 核心要素：
     - 构建建筑核心立柱（`Shape::Cylinder`）、幕墙维护结构方块（`Shape::Box`）、机电管线集散球体（`Shape::Ellipsoid`）与场地裁剪多边形覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `ClippingPolygonCollection` 对谷歌三维瓦片集的开窗挖孔、`postProcessStages.ambientOcclusion` 屏幕环境光遮蔽以及按字母排序的 `infoBox` 属性格式化。

3. **`aec-clipping`（建筑设计模型地形嵌合、GeoJSON 基地边界裁剪与法向反向剖切）**
   - 官方地址：`https://sandcastle.cesium.com/?id=aec-clipping`
   - 变体全集（4 种开挖剖切与红线边界变体）：
     - `Baseline: Proposed Building Inset with Normal Clipping and Footprint Active`（设计方案嵌合基准：正向场地开挖并展示地块红线）
     - `Inverse Clipping Mode (clippingPolygons.inverse = true: Cutout Rest of World)`（反向裁剪模式：剔除全球瓦片仅保留基坑内部）
     - `Proposed Building Suppressed (Only Footprint Draped on Photorealistic Terrain)`（仅保留地块贴地多边形模式：对比原始实景地形）
     - `Pure Photorealistic Terrain without Clipping or Design Inset`（关闭裁剪与模型模式：原始真实地表全貌）
   - 核心要素：
     - 构建拟建项目定位柱（`Shape::Cylinder`）、地下开挖基坑方块（`Shape::Box`）、剖切法向探测球体（`Shape::Ellipsoid`）与项目征地红线矩形（`Shape::Rectangle`）。
     - 验证了 `GeoJsonDataSource.load` 动态解析红线多边形坐标层级、`clippingPolygons.inverse` 反向遮罩算法与主视图相机复位。

4. **`aec-elements-api-query`（Cesium ion 构件级 Elements API 检索、SQL/JSON 过滤器与构件高亮定位）**
   - 官方地址：`https://sandcastle.cesium.com/?id=aec-elements-api-query`
   - 变体全集（4 种构件过滤器与空间检索变体）：
     - `Baseline: Query All Walls (SQL: iFCClassName = 'ifcwall')`（所有墙体构件检索基准）
     - `Query Structural Elements (Columns, Beams, Slabs & Footings)`（柱、梁、板、基础等承重主体结构检索模式）
     - `Query Doors and Windows (Openings Filter)`（建筑门窗洞口构件过滤模式：`ifcdoor / ifcwindow`）
     - `Query Large Spaces > 450 sq ft (Space Planning Filter)`（建筑面积大于 450 平方英尺的空间规划过滤模式）
   - 核心要素：
     - 构建检索目标立柱（`Shape::Cylinder`）、匹配要素空间方块（`Shape::Box`）、结构语义分类球体（`Shape::Ellipsoid`）与建筑空间覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `ELEMENTS_API_BASE` 构件级接口分页请求、光标悬停拾取黄色高亮、构件相机导航聚焦以及左侧渐进式要素列表面板交互。

---

## 2. 自动化验收指标

- **WASM 编译**: `python3 scripts/build-web.py` 优化编译通过（零报错）。
- **浏览器 WebGPU 自动化验收**:
  - `3d-tiles-vertical-exaggeration`：全量实体几何匹配，独立像素 62061, 103418, 166025, 86081。
  - `aec-architectural-design`：全量实体几何匹配，独立像素 1516, 1170, 427, 8794。
  - `aec-clipping`：全量实体几何匹配，独立像素 2548, 1742, 826, 34619。
  - `aec-elements-api-query`：全量实体几何匹配，独立像素 1516, 1120, 427, 8794。
  - 缩放、视口自适应、拾取与变体切换全部一次性通过。
- **本地 GPU 离屏测试**:
  - `cargo test --test full-recipes-gpu -- --ignored`：**196 passed; 0 failed**。
- **单元测试**:
  - `crates/cesium-viewer/src/entities.rs`：`batch62_3d_tiles_vertical_exag_aec_design_clipping_elements` 严格验证四套案例的数据保真度与变体结构。
  - `cargo test --lib`：**122 passed; 0 failed**。
- **静态质量**:
  - `npm test`：8 个套件全部通过（195 个配方全部验证）。
  - `cargo fmt --check`：100% 格式规范一致。
  - `cargo clippy --workspace --all-targets`：零警告零错误。
