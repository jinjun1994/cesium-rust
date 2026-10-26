# Cesium Rust 开发与 Agent 协作日志

## 项目基本信息
- **工作区路径**：`/Users/jinjun/study/cesium-rust/cesium-rust`
- **远程仓库**：`https://github.com/jinjun1994/cesium-rust.git` (`main` 分支)
- **目标里程碑**：M1/M2 推进实体与属性图谱，M3 聚焦 G05 地形、材质与贴地渲染，完成 Sandcastle 309 个全量官方配方移植验收。
- **当前版本**：v0.0.31（已通过 71 / 309 个完整案例，剩余 238 个）

---

## 开发批次演进记录

### [2026-10-26] v0.0.28 验收与 GitHub 链接完成
- **完成内容**：
  1. 完成 `czml-rectangle` 与 `partial-ellipsoids` 两个完整案例（涵盖中空土星环、时钟角/天顶角切口、四元数姿态、CZML 旋转与贴图材质等）。
  2. 修复 `web-time` 在 WebAssembly (`wasm32-unknown-unknown`) 环境下的时间调用崩溃问题。
  3. 修复复杂几何空心区域射线拾取采样逻辑。
  4. 验证全部通过：122 项单元测试、90 项真实 GPU 测试、8 组 Node 测试套件、59 个浏览器 WebGPU 端到端案例及 64 个桌面 90 帧冒烟测试。
  5. 规范生成 `.gitignore`（过滤 `target/`、`node_modules/`、`bindings/web/pkg/`、`.DS_Store`、缓存等）。
  6. 初始化 git 仓库并推送到远程 GitHub 仓库 `main` 分支。

---

### [2026-10-26] v0.0.29 验收完成：Polyline Volume 体系全面打通
- **完成配方**（累计推进至 **63 / 309**）：
  1. `polyline-volume`（实体：红管圆形 rounded、绿盒斜切 beveled、蓝星尖角 mitered，全集合自动取景与轮廓）
  2. `polyline-volume-dev`（底层 Primitive：`PolylineVolumeGeometry` 3 种几何体与 `PerInstanceColorAppearance`，法线与端面闭合）
  3. `polyline-volume-outline-dev`（底层 Primitive：`PolylineVolumeOutlineGeometry` 首尾端面封闭环与纵向棱线）
  4. `czml-polyline-volume`（CZML 数据源：绿方盒倒角、蓝星斜接，带材质与轮廓，Promise 取景）
- **核心研发突破**：
  1. **空间截面沿轨扫掠几何核**（`crates/cesium-viewer/src/polyline_volume.rs`）：
     - 建立了切线、法线与副法线的连续标架，防范空间折线由于非共面扭转而产生的翻转畸变。
     - 实现了三种典型拓扑：`Mitered`（角平分线平移拉伸）、`Beveled`（双截面过渡平切角）、`Rounded`（微元角度弧线平滑细分）。
     - 引入 `earcutr` 完成起止截面的自适应三角剖分端面封盖（End Caps）。
     - 实现了 `PolylineVolumeOutlineGeometry`：提炼起止截面闭合线环以及相邻横截面同序号顶点连接的纵向棱线（Longitudinal Edges）。
  2. **Entity / Primitive / CZML 统一抽象**：
     - 在 `Shape::PolylineVolume` 中扩展 `positions`、`shapePositions`（支持别名 `shape` 与 Cartesian2 对象转数组）、`cornerType`。
     - 在 `geographic.rs` 中为 `PolylineVolume` 挂载几何中心锚点 `anchor`，确保相机相对坐标（RTC）的微米级精度。
     - CZML 数据包解析器增加 `polylineVolume` 格式提取与验证支持。
  3. **WASM / 浏览器跨 realm 数据传递兼容**：
     - 在 `scripts/batch29-recipes.mjs` 中将 Node.js VM context 内构造的深层实体对象序列化/反序列化脱敏，解决 `assert.deepStrictEqual` 跨 realm Array prototype 对比不相等的细节陷阱。
     - 在 Web 前端 Sandcastle 挂载 `fullNew36` ~ `fullNew39` 测试按钮并完成 WebGPU 交互。
- **验证结果**：
  - **普通 Rust 单元测试**：126 / 126 项全部通过（新增 `batch29_polyline_volume_all_four_cases`）。
  - **真实 GPU 离屏渲染测试**：64 / 64 项全部通过（新增 4 个 complete 测试，均通过离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **桌面 90 帧冒烟测试**：68 个配置（63 案例 + 5 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：63 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），销毁状态 `destroy: 'passed'`。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Polyline Volume 专项验证文档](docs/polyline-volume-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-26] v0.0.30 验收完成：Rectangle、Z-Indexing 与 Volume Primitives 体系全面打通
- **完成配方**（累计推进至 **67 / 309**，剩余 242 个）：
  1. `rectangle`（实体：红半透明矩形、绿旋转/拉伸带高程与边框矩形、带 rotation 与独立 stRotation 的贴图矩形，集合自动取景）
  2. `z-indexing-geometry`（实体：6 个不同 zIndex 1~3 的重叠矩形，涵盖纯色与 Cesium 贴图材质，加一条蓝光贴地折线，无共面 Z-Fighting）
  3. `czml-zindex`（CZML 数据源：椭圆、走廊、多边形、条纹矩形 4 个不同 zIndex 1~3 的重叠图元）
  4. `volumes-dev`（底层 Primitive：`EllipsoidPrimitive` 局部 ENU 定位与边缘光 RimLighting、条纹 Stripe 材质）
- **核心研发突破**：
  1. **实体与图层排序模型（Z-Indexing Pipeline & Depth Bias）**：
     - 在 `Entity`、`GpuMesh` 与各几何包体中引入 `z_index: i32`（默认 0，范围 `[-1000, 1000]`，serde 映射 `zIndex`）。
     - 在渲染器 `Renderer::render()` 中建立 `a.z_index.cmp(&b.z_index)` 严格排序，使得低 zIndex 几何先入渲染管线，高 zIndex 几何在其上方正确遮挡或混合，结合 Reverse-Z 与深度测试消除同面共面 Z-Fighting。
     - 在 CZML 解析器中支持各图元对象（polygon、corridor、rectangle、ellipse 与 polyline）的 `zIndex` 属性提取。
  2. **Rectangle 几何与贴图完整支持**：
     - 完善 `Shape::Rectangle` 参数通道，包含 `coordinatesDegrees`、`height`、`extrudedHeight`、`rotation` 与 `st_rotation`。
     - 在 `SurfaceMaterial::apply()` 中支持纹理坐标 `st_rotation` 旋转矩阵运算（围绕 `[0.5, 0.5]` UV 质心中心旋转），使贴图能够独立于几何矩形自身的地面偏航角进行纹理偏转。
  3. **材质体系扩展与 CZML 兼容**：
     - 扩充 `LineMaterial::Glow`，增加可选 `color: Option<[f32; 4]>` 字段，无缝兼容 CesiumJS 原生 `PolylineGlowMaterialProperty` 属性规范。
     - 扩展 `SurfaceMaterial::Stripe` 默认参数，兼容上游无显式指定时的缺省渲染。
     - 在扩展图元解析器中增加对 `EllipsoidPrimitive` 的解码支持，提取 `radii`、`center` / `modelMatrix`，并支持 `RimLighting` 边缘光与 `Stripe` 条纹表面材质的生成与参数水合。
  4. **全套自动化测试与验证**：
     - 在 Web 前端 Sandcastle 挂载 `fullNew40` ~ `fullNew43` 测试按钮。
     - 真实 Chromium WebGPU 端到端自动化套件覆盖所有 67 个案例，所有像素反读与拾取通过。
- **验证结果**：
  - **普通 Rust 单元测试**：126 / 126 项全部通过（新增 `batch30_rectangle_zindex_volumes_all_four_cases`）。
  - **真实 GPU 离屏渲染测试**：68 / 68 项全部通过（新增 4 个 complete 测试，用时 22.07s，均通过离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **桌面 90 帧冒烟测试**：72 个配置（67 案例 + 5 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：67 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），销毁状态 `destroy: 'passed'`。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Rectangle 与 Z-Indexing 专项验证文档](docs/rectangle-zindex-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-26] v0.0.31 验收完成：Geometry Offset Attributes 与 CZML 动态时变属性体系全面打通
- **完成配方**（累计推进至 **71 / 309**，剩余 238 个）：
  1. `geometry-offset-attribute-dev`（底层 Primitive：3 个拉伸矩形与 3 个外边框轮廓几何，覆盖 `GeometryOffsetAttribute.TOP`、`ALL`、`NONE`，沿法向空间位移与动态 toolbar 交互更新）
  2. `geometry-offset-attribute-box-cylinder-ellipsoid-dev`（底层 Primitive：Box、Cylinder、Ellipsoid 实心及对应 Outline 几何，覆盖 `GeometryOffsetAttribute.ALL` 与 ENU 局部模型变换矩阵结合的动态偏移更新）
  3. `czml-polygon-intervals-availability`（CZML 数据源：含时间间隔的动态多边形、带洞多边形，以及具备时变可用性 availability 的 California 与 Pennsylvania 多边形）
  4. `czml-custom-properties`（CZML 数据源：跨 1970-2010 年带文档时钟的自定义数值区间属性与采样属性，以及通过自定义属性驱动拉伸高度的 Colorado 多边形）
- **核心研发突破**：
  1. **几何偏移属性管线（Geometry Offset Attributes Pipeline）**：
     - 在底层 Primitive 架构（`primitive_instances.rs` 与 `extended_instances.rs`）中支持 `offsetAttribute`（TOP / ALL / NONE）与 PerInstance 属性 `attributes.offset: [f64; 3]`。
     - 顶点位置根据局部地心表面法线或 offset 向量进行形变偏移计算，支持通过属性句柄动态更新几何位移。
     - 消除 Primitive 实体 ID 冲突，保证填充与轮廓图元在同一集合内分配唯一 ID（`primitive-{p}-instance-{i}`）。
     - 支持 Cylinder / Ellipsoid 轮廓图元默认分区参数（`numberOfVerticalLines`, `stackPartitions`, `slicePartitions`）缺省补齐。
  2. **多边形层级内孔与拉伸高度（Polygon Holes & Height Support）**：
     - 扩充 `Shape::Polygon` 数据结构，增加 `holes: Vec<Vec<[f64; 3]>>`（兼容 `holesDegrees` 别名）与 `height: Option<f64>`（序列化 `skip_serializing_if = "Option::is_none"`）。
     - 在 `geographic.rs` 与 `outline.rs` 中统一计算多边形外环与内孔在指定基准高度与拉伸高度下的顶点网格与法向量，精确处理零高程共面与非共面多边形三角剖分。
  3. **CZML 时态区间与可用性（Intervals & Availability Lifecycle）**：
     - 完善 CZML 反序列化引擎，支持年格式字符串（如 `"1970/2010"`）与完整 ISO-8601 区间解析。
     - 深度支持顶点坐标 `positions`、多边形洞孔 `holes` 与表面颜色材质 `solidColor.color` 的时序多区间（TimeIntervalCollection）动态更新。
     - 在视口与包围盒评估中，自动根据实体 `availability` 激活时间过滤，防止非活动时段导致无效边界退化。
  4. **非视觉自定义属性对象（Custom Properties）**：
     - 支持无几何图形组件的纯数据实体解析与属性表挂载。
     - 支持时序标量数据 `SampledProperty` 与分段常量 `TimeIntervalCollection` 属性查询。
- **验证结果**：
  - **普通 Rust 单元测试**：93 / 93 项全部通过（新增 `batch31_geometry_offset_czml_properties_all_four_cases`）。
  - **真实 GPU 离屏渲染测试**：72 / 72 项全部通过（用时 18.97s，新增 4 个 complete 测试，均通过离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 71 个固定案例哈希与实体结构精准契合）。
  - **桌面 90 帧冒烟测试**：76 个配置（71 案例 + 5 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：71 个完整案例在真实 Chromium 155 环境下全部通过（`full-recipes-browser.cjs`），包含 33 项 Batch 31 实体像素反读与拾取检查，销毁状态 `destroy: 'passed'`。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Geometry Offset 与 CZML Dynamic Properties 专项验证文档](docs/geometry-offset-czml-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-26] v0.0.32 批次启动：G01 核心几何与高级图元攻坚
- **目标配方**（累计推进至 **75 / 309**，剩余 234 个）：
  1. `polylines-dev`（底层 Primitive：`PolylineCollection` 包含 7 条经典折线，覆盖 `PolylinePipeline` 大圆与等角航线 RHUMB 弧线生成、闭环 `loop: true`、局部 ENU 变换矩阵、7 类核心材质：Color、PolylineOutline、PolylineGlow、PolylineArrow、Fade）
  2. `frustum-dev`（底层 Primitive：`FrustumGeometry` 与 `FrustumOutlineGeometry`，视锥体参数：透视视锥 `PerspectiveFrustum`、椭球表面起点与四元数姿态、包围球相机聚焦 `viewBoundingSphere` 与实心/线框渲染）
  3. `parallels-and-meridians`（实体与屏幕交互：赤道与本初子午线、等角航线 RhumbPolyline、递归网格生成 `makeGrid`、屏幕拾取十字交叉线 Crosshairs 与对跖点 Antipodal 计算、经纬度标签标注）
  4. `polygon-texture-coordinates-dev`（底层 Primitive：6 组带自定义 UV 纹理坐标的 `PolygonGeometry` 实例，覆盖平坦面、嵌套多级内孔、`perPositionHeight: true` 绝对高程、拉伸几何体与 `ArcType.RHUMB` 等角多边形，以及 `GroundPrimitive` 贴地映射）
- **核心研发规划**：
  1. **PolylineCollection 与折线材质管线扩展**：
     - 在底层 Primitive 解析与渲染管线中支持 `PolylineCollection` 与多段折线局部 `modelMatrix` 姿态变换。
     - 确保 `Fade`（渐变淡出）、`PolylineArrow`（箭头线）、`PolylineGlow`（发光线）与 `PolylineOutline`（轮廓线）参数全面对齐。
  2. **视锥体几何与轮廓管线（FrustumGeometry & FrustumOutlineGeometry）**：
     - 依据 `PerspectiveFrustum`（fov, aspectRatio, near, far）与四元数姿态在局部空间构建 8 个角点坐标，并变换至椭球世界坐标系。
     - 生成视锥 6 个面的三角网格与 12 条棱边轮廓线网格。
  3. **自定义多边形纹理坐标（Explicit Polygon TextureCoordinates）**：
     - 在 `PolygonGeometry` 中支持 `textureCoordinates: { positions, holes }` 显式 UV 通道。
     - 在顶点生成时直接采用用户提供的 UV 坐标，处理内嵌孔洞的 UV 插值与对应。
  4. **经纬网与十字准星交互（Parallels & Meridians Pipeline）**：
     - 支持递归经纬线网格批量生成，支持通过按钮动态切换不同分辨率网格与对跖点显示。
  5. **自动化测试与端到端验证**：
     - 编写 `scripts/batch32-recipes.mjs`，执行 4 个案例源码并提取实体/图元标准契约。
     - 补充普通 Rust 单元测试、实际 GPU 离屏渲染测试、90 帧桌面冒烟测试与真实 Chromium WebGPU 测试。




