# Batch 63 (v0.0.63) 案例验证报告：Design Model – Category Filtering、Design Model – Metadata Styling、AEC Snapping 与 Design Model – Tileset Metadata Filtering

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 Autodesk Revit 建筑设计构件 CategoryID 分类过滤与独立隔离 (Design Model – Category Filtering)、基于三维瓦片嵌入式元数据与双滑块动态着色暖通机电管线 (Design Model – Metadata Styling)、混合式几何捕捉与边缘高亮测绘标注 (AEC Snapping)、以及瓦片集层级并行元数据数组解析与构件拾取过滤面板 (Design Model – Tileset Metadata Filtering) 的配方，累计完成 **199 / 309**（剩余 110 个）：

1. **`aec-isolate-by-category`（Autodesk Revit 斯诺登大厦构件 CategoryID 隔离过滤、场地开挖与图层控制）**
   - 官方地址：`https://sandcastle.cesium.com/?id=aec-isolate-by-category`
   - 变体全集（6 种构件分类隔离与过滤变体）：
     - `Show All Categories`（展示所有构件分类基准）
     - `Isolate Walls (OST_Walls -2000011)`（仅隔离展示墙体构件）
     - `Isolate Floors (OST_Floors -2000032)`（仅隔离展示楼板构件）
     - `Isolate Stairs (OST_Stairs -2000120)`（仅隔离展示核心筒楼梯构件）
     - `Isolate Doors (OST_Doors -2000023)`（仅隔离展示出入口门构件）
     - `Isolate Lighting (OST_LightingFixtures -2001120)`（仅隔离展示照明灯具构件）
   - 核心要素：
     - 构建建筑墙体主体（`Shape::Box`）、水平楼板结构（`Shape::Box`）、核心筒楼梯（`Shape::Cylinder`）、出入门洞（`Shape::Box`）、照明灯具椭球体（`Shape::Ellipsoid`）与场地开挖矩形轮廓（`Shape::Rectangle`）。
     - 验证了 `Cesium3DTileStyle` 中以 `${feature['category']} === '...'` 动态生成过滤规则，并支持在无地球环境（`globe: false`）下的场地裁剪与环境光遮蔽（SSAO）。

2. **`aec-metadata-styling`（嵌入式机电元数据动态着色、双滑块管线长度区间过滤与幽灵建筑半透衬托）**
   - 官方地址：`https://sandcastle.cesium.com/?id=aec-metadata-styling`
   - 变体全集（4 种管线长度区间与高亮着色变体）：
     - `Highlight All Pipe Lengths (0m - 50m)`（高亮全范围 0m~50m HVAC 管线基准）
     - `Highlight Short Terminal Runs (0m - 10m)`（仅高亮 0m~10m 末端送风散流管段）
     - `Highlight Medium Branch Ducts (10m - 25m)`（仅高亮 10m~25m 支路送风风管）
     - `Highlight Long Supply Mains (25m - 50m)`（仅高亮 25m~50m 主干送风管路）
   - 核心要素：
     - 构建幽灵半透建筑轮廓方块（`Shape::Box`，`lightblue` 0.18 透明度）、主管路圆柱（`Shape::Cylinder`）、支风管圆柱（`Shape::Cylinder`）、散流末端方块（`Shape::Box`）与 MEP 场地覆盖矩形（`Shape::Rectangle`）。
     - 验证了基于属性表达式 `(${length} !== undefined && ${length} >= min) && (${length} <= max)` 进行条件材质着色（`color('red')`）与多瓦片集视距屏幕误差（SSE=4）调度。

3. **`aec-snapping`（BIM 构件混合式端点与边缘捕捉、几何精度优化与热度距离剔除）**
   - 官方地址：`https://sandcastle.cesium.com/?id=aec-snapping`
   - 变体全集（4 种几何捕捉模式与端点反馈变体）：
     - `Edge Snapping Mode (Cyan Edge Hover + Lime Client + Red Server)`（边缘捕捉模式：青色悬停边缘、青柠瞬时客户端捕捉、红色服务器精化端点）
     - `Surface Snapping Mode (Yellow Surface Hover + Surface Snap)`（表面捕捉模式：黄色悬停表面、表面橙红捕捉端点）
     - `Terrain Hover Mode (Gray Feedback, Reject Server Snap)`（地形悬停模式：灰色光标反馈、防误触剔除远距离服务器捕捉）
     - `All Snapping Calibration Markers Enabled`（全捕捉标定端点激活模式）
   - 核心要素：
     - 构建 BIM 结构柱构件（`Shape::Box`）、青色边缘悬停标记（`Shape::Ellipsoid`）、青柠客户端捕捉标记（`Shape::Ellipsoid`）、红色服务器精化标记（`Shape::Ellipsoid`）、灰色地形悬停标记（`Shape::Ellipsoid`）与基础承台矩形（`Shape::Rectangle`）。
     - 验证了 `edgeDisplayMode = SurfacesAndEdges` 边缘显示控制、`Scene.snap` 客户端屏幕拾取光线、服务器几何捕捉距离阈值（`IonSnapHeat.NONE` 结合像素距离防漂移保护）。

4. **`aec-tileset-metadata-filtering`（三维瓦片集并行元数据结构解析、CTRL+点击单选与拾取隐藏模式）**
   - 官方地址：`https://sandcastle.cesium.com/?id=aec-tileset-metadata-filtering`
   - 变体全集（4 种元数据层级过滤与显隐变体）：
     - `All Categories and Subcategories Visible`（所有构件分类与子分类可见基准）
     - `Solo Structural Columns (CTRL+Click Solo Cat 101)`（CTRL+点击单选：仅孤立展示结构立柱构件）
     - `Filter Subcategories (Hide Mullions Sub 202)`（子分类过滤：隐藏幕墙竖框子构件）
     - `Hide by Pick Mode (Hide Roof & Floors)`（交互拾取隐藏模式：点击隐藏屋顶与楼板构件）
   - 核心要素：
     - 构建现浇混凝土立柱（`Shape::Cylinder`）、幕墙系统与竖框（`Shape::Box`）、结构楼板混凝土垫层（`Shape::Box`）、组合钢结构屋顶（`Shape::Box`）与高精元数据场地矩形（`Shape::Rectangle`）。
     - 验证了并行元数据数组（`categoryIds`, `categoryNames`, `subcategoryIds`, `subcategoryNames`, `subcategoryParentIds`）映射树构建、基于 `Set` 的双层自定义求值过滤函数与交互拾取遮罩。

---

## 2. 跨平台与四层验收指标

| 案例 ID | 上游源码 SHA-256 | 实体数 | 变体数 | 原生 GPU (Metal/Vulkan) | 真实浏览器 WebGPU | 像素拾取判定 |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| `aec-isolate-by-category` | `488fbff81b79f62f7d13897e4ab0b77a522ed9ac1930fc7b3b4409a9bd86dfe9` | 6 | 6 | `passed (90f)` | `passed` | 6/6 构件与场地边界全拾取 |
| `aec-metadata-styling` | `378df88c7fdf9e2cff408ea6bc1aee2ec467f5cf40e340842fe9ebf4aeb912cf` | 5 | 4 | `passed (90f)` | `passed` | 5/5 幽灵建筑与机电全拾取 |
| `aec-snapping` | `8cba50f555628b05fcfe6d4b4fb24e351829e701ea55f135b914aa036e7dd78a` | 6 | 4 | `passed (90f)` | `passed` | 6/6 构件与捕捉端点全拾取 |
| `aec-tileset-metadata-filtering` | `410830d4c3f73da110df78458abede4032ae3916095784aaff3f8d9e8ffad430` | 5 | 4 | `passed (90f)` | `passed` | 5/5 各级分类构件全拾取 |

---

## 3. 自动化测试与工程一致性保障

- **单元测试**：`cargo test --lib batch63` 通过（验证了全部几何、材质、分类属性与元数据结构体反序列化）。
- **GPU 离屏测试**：`cargo test --test full-recipes-gpu -- --ignored` 执行 200 项测试全部通过（0 失败，覆盖 90 帧状态留痕与相机投影计算）。
- **浏览器 WebGPU**：通过 Playwright 在 Chromium 环境中执行无头 WebGPU 渲染测试，全部 4 项案例通过像素差异检验（`samples.count > 8`）与鼠标拾取测试。
- **静态质量保障**：`npm test` 8 个测试套件全绿，`cargo fmt --check` 严格格式化，`cargo clippy --workspace --all-targets` 0 警告通过。
