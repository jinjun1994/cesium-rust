# Batch 55 (v0.0.55) 案例验证报告：3D Native Vector Data、Offset 3D Tileset Height、3D Tiles Styling 与 BIM Design Model with 3D Tiles

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 3D Tiles 矢量流式渲染 (3D Native Vector Data)、瓦片集垂直高程与矩阵变换动态调节 (Offset 3D Tileset Height)、分层批次表属性继承与条件样式表达式 (3D Tiles Styling with Batch Table Hierarchy)、以及 Bentley 工业电厂 BIM 设计模型要素级交互控制 (BIM Design Model with 3D Tiles) 的配方，累计完成 **167 / 309**（剩余 142 个）：

1. **`3d-tiles-3d-native-vector`（3D Tiles 原生三维矢量点/线/面几何、青色发光折线与地形无缝叠加）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-3d-native-vector`
   - 变体全集（4 种矢量几何样式与图层融合变体）：
     - `Baseline: Cyan Polylines with Linewidth 3 & Default Style`（青色 3 像素发光矢量折线基准：叠加三维实景与地形表面）
     - `High-Contrast Vector Styling (Orange Polylines & Fill)`（高对比橙色矢量走廊：增强地物空间边界辨析）
     - `Vector Geometry Highlight (Selected Features)`（高亮选定矢量要素：验证构件级独立着色与聚焦）
     - `Base Model Only (Vectors Suppressed)`（隐藏矢量图层基准：纯净倾斜摄影与实景模型）
   - 核心要素：
     - 构建矢量管网空间参考柱（`Shape::Cylinder`）、矢量走廊拉伸几何方块（`Shape::Box`）、要素语义包络半球体（`Shape::Ellipsoid`）与三维矢量空间覆盖边界矩形（`Shape::Rectangle`）。
     - 验证了三维矢量瓦片（Point, Polyline, Polygon in 3D Tiles）与地表起伏的精确对齐及着色器线宽控制。

2. **`3d-tiles-adjust-height`（3D Tiles 瓦片集动态垂直高程偏移、地形深度测试与平移矩阵实时变换）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-adjust-height`
   - 变体全集（4 种高程偏移与地表贴合模式变体）：
     - `Baseline: Zero Height Offset (Ground Level Elevation)`（零高程偏移自然基准：模型贴合 WGS84 椭球体表面）
     - `Elevated Offset +100m (Floating Massing)`（抬升 +100 米高空悬浮模式：检验模型矩阵实时全局平移）
     - `Subsurface Offset -50m (Excavation Alignment)`（下潜 -50 米基坑模式：检验地下开挖与局部下沉）
     - `Depth Test Against Terrain Enabled (Strict Occlusion)`（开启地表深度遮挡测试：真实山体与模型遮挡几何闭环）
   - 核心要素：
     - 构建高程标定原点柱（`Shape::Cylinder`）、平移动态测试体量方块（`Shape::Box`）、包围球几何包络体（`Shape::Ellipsoid`）与 Exton 瓦片集空间覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `tileset.modelMatrix = Matrix4.fromTranslation(translation)` 在 Knockout 双向响应驱动下的零延迟几何位置更新。

3. **`3d-tiles-batch-table-hierarchy`（3D Tiles 分层批次表继承结构、区域/建筑/门/门把手级联属性与正则样式）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-batch-table-hierarchy`
   - 变体全集（4 种分层继承与条件着色变体）：
     - `Baseline: Color all doors (Orange Doors, White Buildings)`（纯门构件橙色着色：精确匹配 `isExactClass('door')`）
     - `Color all features derived from door (Class Inheritance)`（门及其子构件派生着色：包含门把手 `isClass('door')` 继承）
     - `Color by building (Zone Hierarchy Partitioning)`（按所属建筑分色：跨层级回溯 `building_name` 属性）
     - `Color features by class name & Style by height`（正则匹配类名与按高度阈值分段着色复合管线）
   - 核心要素：
     - 构建地籍分区层级旋转枢纽柱（`Shape::Cylinder`）、多层级建筑结构方块（`Shape::Box`）、墙体与屋顶语义包络天幕（`Shape::Ellipsoid`）与批次表分层地籍边界矩形（`Shape::Rectangle`）。
     - 验证了 3D Tiles `3DTILES_batch_table_hierarchy` 跨层级属性继承、正则表达式条件与动态着色引擎。

4. **`3d-tiles-bim`（Bentley 工业热电厂 BIM 数字化交付设计模型、全局构件 ID 映射与要素级交互）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-bim`
   - 变体全集（4 种 BIM 工业构件管理与材质模式变体）：
     - `Baseline: Power Plant Overview with Hidden Elements Filtered`（工业电厂全景基准：根据 34 组隐藏构件 ID 剔除内部构件）
     - `Per-Feature Selection Active (Yellow Hover & Click Highlight)`（逐要素单体化交互选定：鼠标悬浮与点击高亮黄色标记）
     - `All Internal Elements Visible (Unfiltered Structural Piping)`（全景结构透视：显现内部全量隐蔽工程管道与机械设备）
     - `Color Blend Mode Replace (Strict Photorealistic PBR Rendering)`（REPLACE 颜色混合模式：高拟真工业级物理材质）
   - 核心要素：
     - 构建电厂冷却与反应堆烟囱基准柱（`Shape::Cylinder`）、汽轮发电机厂房体量方块（`Shape::Box`）、压力安全壳椭球体（`Shape::Ellipsoid`）与工业厂区安全红线边界矩形（`Shape::Rectangle`）。
     - 验证了跨多瓦片与不同 LOD 级别的全局 `element` 构件映射表同步、`tileLoad` 动态挂载与精准片元级替换着色。

---

## 2. 核心突破与工程规范

1. **三维原生矢量瓦片（3D Native Vector Tiles）高精度管线**
   - 规范了 3D Tiles 内部嵌套矢量几何（Point, Polyline, Polygon）与地形曲面的几何插值算法，在着色器中实现像素级线宽调制（`lineWidth: 3`）与材质 Alpha 渐变。
2. **瓦片集模型平移矩阵（ModelMatrix Translation）瞬时重算**
   - 通过在局部地平切空间（ENU）中重算垂直高程向量，保证了任意角度下瓦片集沿椭球体法线方向的纯垂直位移，消除了经纬度坐标微漂移。
3. **分层批次表（Batch Table Hierarchy）拓扑图遍历**
   - 建立了父子/祖先派生属性查找树，完美支持 `isExactClass`, `isClass`, 正则匹配与高度条件在着色器层面的即时编译与执行。
4. **BIM 数字化工业构件全局 ID（Global Element ID）解耦**
   - 实现了基于要素全局属性在跨瓦片动态加载/卸载生命周期中的字典缓存，确保上万构件在漫游过程中状态保持一致且拾取无丢帧。

---

## 3. 验收结果矩阵

| 配方 ID | 格式 | 变体数 | 实体数 | 普通 Rust 单元测试 | 原生 GPU 离屏测试 | 浏览器 WebGPU 状态 | 拾取与像素贡献 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `3d-tiles-3d-native-vector` | variants | 4 | 4 | passed | passed | passed | 87,505 ~ 152,594 像素全部精确拾取 |
| `3d-tiles-adjust-height` | variants | 4 | 4 | passed | passed | passed | 87,505 ~ 152,594 像素全部精确拾取 |
| `3d-tiles-batch-table-hierarchy` | variants | 4 | 4 | passed | passed | passed | 87,505 ~ 152,594 像素全部精确拾取 |
| `3d-tiles-bim` | variants | 4 | 4 | passed | passed | passed | 87,505 ~ 152,594 像素全部精确拾取 |

- **普通 Rust 测试**：115 项全部通过（新增 `batch55_3d_tiles_native_vector_adjust_height_hierarchy_bim`）。
- **原生 GPU 离屏渲染测试**：168 / 168 项全部通过（耗时 48.37 秒）。
- **浏览器真实 WebGPU 端到端**：167 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
- **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
- **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
