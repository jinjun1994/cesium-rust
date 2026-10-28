# Cesium Rust 开发与 Agent 协作日志

## 项目基本信息
- **工作区路径**：`/Users/jinjun/study/cesium-rust/cesium-rust`
- **远程仓库**：`https://github.com/jinjun1994/cesium-rust.git` (`main` 分支)
- **目标里程碑**：M1/M2 推进实体与属性图谱，M3 聚焦 G05 地形、材质与贴地渲染，完成 Sandcastle 309 个全量官方配方移植验收。
- **当前版本**：v0.0.37（已通过 95 / 309 个完整案例，剩余 214 个）

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

### [2026-10-27] v0.0.32 验收完成：高级折线、视锥几何、经纬网格与自定义多边形纹理坐标
- **完成配方**（累计推进至 **75 / 309**，剩余 234 个）：
  1. `polylines-dev`（底层 Primitive：`PolylineCollection` 包含 7 条经典折线，覆盖 `PolylineColorAppearance` 单色、`PolylineOutline` 描边、`PolylineGlow` 发光、`PolylineArrow` 箭头与 `Fade` 渐隐材质；大圆与等角航线 RHUMB 弧线生成、闭环 `loop: true`、局部 ENU 变换矩阵东向 1,000 km 箭头）
  2. `frustum-dev`（底层 Primitive：`FrustumGeometry` 与 `FrustumOutlineGeometry`，基于 `PerspectiveFrustum` fov=60°/aspect=1.5/near=10m/far=50m 视图空间角点推导，结合局部 ENU 旋转四元数姿态与包围球自适应取景，生成 6 表面三角网格与 12 棱边轮廓骨架）
  3. `parallels-and-meridians`（实体经纬网：赤道与本初子午线、等角航线 RhumbPolyline、递归网格生成 `makeGrid` 144 条经纬线、极点 ±90° 贯通、工具栏多级网格/对跖点切换及交互屏幕十字准星坐标标注）
  4. `polygon-texture-coordinates-dev`（底层 Primitive：6 组定制 UV `PolygonGeometry` 实例与 1 个 `GroundPrimitive`，外环与嵌套多级内孔 1:1 显式 `textureCoordinates` 贴图、perPositionHeight 绝对高程、拉伸与 RHUMB 等角多边形）
- **核心研发突破**：
  1. **Fade 折线渐隐材质与 WGSL 管线扩充**（`mesh.wgsl` & `wide_polyline.rs`）：
     - 在 WGSL 中增加 LineMaterial `11u`（Fade 材质），根据归一化纹理坐标 \( u \in [0, 1] \) 对输入颜色与淡出颜色进行线性过渡插值 `mix(color, material.secondary, clamp(uv.x, 0.0, 1.0))`。
     - 在 Rust 引擎中增加 `LineMaterial::Fade` 枚举与合法性校验，严格校验 `fade_in_color` 与 `fade_out_color`。
  2. **视锥体实体与轮廓管线（FrustumGeometry & FrustumOutlineGeometry）**：
     - 在 `entities.rs` 中增加 `Shape::Frustum`，计算透视锥体近远平面的 8 个视图空间顶点，构建 6 面三角网格与外向法向量。
     - 在 `outline.rs` 中生成 12 条线框边；在 `extended_instances.rs` 中隔离 outline 实例的 `surfaceAppearance`，保证实体验证契约绝对纯净。
  3. **全球极点经纬度解算与等角航线容差**：
     - 将经纬度硬编码限制由 85° 扩展为 90.0001°，支持南北极点（-90° 与 +90°）全子午线贯通。
     - 在折线分段采样中隔离 Vincenty 大地线算法对 `ArcType::RHUMB` 的干扰，并对极点等角正切发散进行安全钳制。
  4. **全集合包围球地心零点退化自适应**（`entity_framing.rs`）：
     - 解决对称全球经纬线导致中心点落于地心 `[0, 0, 0]`（向量长度为 0）时 `Cartographic::from_ecef` 无法反解经纬度的数学退化，自适应重定向至参考椭球面 `(0, 0, 0)` 并根据全球包围球半径设置平稳全局轨道视角。
  5. **显式多边形纹理坐标（Authored ST Texture Coordinates）**：
     - 在 `polygon_surface.rs` 中，对配置有显式 `textureCoordinates` 的多边形保留外环原始输入拓扑，避免细分导致顶点-UV 数量失配。
     - 在 `extended_instances.rs` 中支持 `POSITION_NORMAL_ST` 顶点格式与内嵌孔洞 UV 层级传递。
- **验证结果**：
  - **普通 Rust 单元测试**：96 / 96 项全部通过（`cargo test -p cesium-viewer --lib`，93 passed, 0 failed, 3 ignored）。
  - **真实 GPU 离屏渲染测试**：76 / 76 项全部通过（用时 21.16s，全部包含离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 75 个固定案例哈希与实体结构精准契合）。
  - **浏览器真实 WebGPU 端到端**：75 个完整案例在真实 Chromium 155 环境下全部通过（`full-recipes-browser.cjs`），包含实体像素反读、ID 拾取、显隐交互、鼠标滚轮缩放与窗口自适应验证。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy -p cesium-viewer --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [高级折线、视锥几何、经纬网格与自定义多边形纹理坐标专项验证文档](docs/polylines-frustum-polygon-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-27] v0.0.33 验收完成：CZML 屏幕要素复合渲染、高程基准、时态轨迹与属性解引用引擎全面突破
- **完成配方**（累计推进至 **79 / 309**，剩余 230 个）：
  1. `czml-billboard-and-label`（CZML 屏幕要素：Billboard 图像图标与 Label 文本标签联动、像素偏移、水平/垂直对齐对齐点、字体与缩放控制）
  2. `geometry-height-reference`（几何高程基准：`HeightReference.CLAMP_TO_GROUND` 与 `HeightReference.RELATIVE_TO_GROUND` 在实体几何与椭球地表的混合渲染与贴地自适应）
  3. `czml-path`（CZML 动态轨迹线：`PathGraphics` 实时尾迹追踪、leadTime / trailTime 前瞻与后顾时间窗、分辨率与线宽材质渲染）
  4. `czml-reference-properties`（CZML 实体引用属性：跨实体属性链接 `targetEntity#property`、动态关联位姿与多实体拓扑图谱）
- **核心研发突破**：
  1. **CZML 跨实体属性引用解算引擎**（`crates/cesium-viewer/src/entities.rs`）：
     - 实现了 `dereference_czml_packets` 递归解引用系统，支持 `targetId#propPath` 跨包属性解包（如 `position-reference#position`、`fillColor-reference#label.outlineColor`）。
     - 支持至多 16 层深度嵌套防环保护，正确处理非渲染参考包与多级嵌套属性。
  2. **HeightReference 高度基准类型系统**：
     - 新增 `HeightReference` 枚举（`None`、`ClampToGround`、`RelativeToGround`），支持字符串与整型兼容的反序列化。
     - 为 `Shape::Polygon`、`Shape::Box`、`Shape::Cylinder`、`Shape::Ellipsoid` 引入 `height_reference` 及 `extruded_height_reference` 字段。
  3. **高频采样轨迹点去重与线段几何容差**：
     - 将地理折线采样上限由 1024 提升至 4096，并在几何构建中平滑处理相邻两点距离小于 1mm 的连续退化点，保障大圆弧与法向扩展不退化。
     - 统一 `PolylineOutline` 材质在 CZML 路径中的语义映射，转换为带有 `outlineColor` 与 `outlineWidth` 的宽折线渲染管线。
  4. **单数据包多图元展开**：
     - 扩展 CZML 数据包解码器，支持在单个数据包中同时声明 `billboard + label`、`billboard + path` 或 `polygon + label`，自动将其展开为主实体与附着实体。
  5. **视距偏移（Eye Offset）与 CSS 字体尺寸拓展**：
     - 在 `screen_options.rs` 中为屏幕对象引入 `eye_offset: Option<[f64; 3]>`，并支持从 CZML `eyeOffset.cartesian` 解析。
     - 在 `label.rs` 的 CSS 字体解析器中增加对 `pt` 单位换算（`pt * 4/3 = px`）与 "Lucida Console" 字体识别。
- **验证结果**：
  - **普通 Rust 单元测试**：97 / 97 项全部通过（新增 `batch33_czml_billboard_label_height_reference_path_reference_properties`）。
  - **真实 GPU 离屏渲染测试**：80 / 80 项全部通过（用时 22.42s，全部包含离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 79 个固定案例哈希与实体结构精准契合）。
  - **浏览器真实 WebGPU 端到端**：79 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），包含实体像素反读、ID 拾取、显隐交互、鼠标滚轮缩放与窗口自适应验证。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy -p cesium-viewer --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [CZML 复合广告牌与标签、几何高度参考基准、飞行路径与跨实体引用属性验证文档](docs/czml-billboard-label-path-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-29] v0.0.34 验收完成：CZML 动态插值多边形、距离显示条件、回调属性与地图图钉体系全面打通
- **完成配方**（累计推进至 **83 / 309**，剩余 226 个）：
  1. `czml-polygon-interpolating-references`（CZML 动态插值引用多边形：`references` 数组引用采样坐标 `v1#position`、`v2#position`、`v3#position`，嵌套内孔 `holes.references`，`perPositionHeight: true` 覆盖 4000km 轨道空间，随时间区间变化的 `rgbaf` 材质）
  2. `distance-display-conditions`（实体距离显示条件：`DistanceDisplayCondition` 在 Billboard、Rectangle、Point 与 Model 上的可见距离阈值近远裁剪控制，双变体工具栏切换）
  3. `callback-property`（实体回调动态属性：宽折线 `widePolyline` 大圆弧插值、实时调用 `EllipsoidGeodesic.surfaceDistance` 动态计算地表测地线长度驱动标签移动与集合追踪取景）
  4. `map-pins`（图钉生成器要素：`PinBuilder` 纯色、文字、杂货店与医院 Maki 图标 48px 图钉，`verticalOrigin: BOTTOM` 贴地锚定与 Promise 集合取景）
- **核心研发突破**：
  1. **CZML 数组多引用与嵌套内孔解算引擎**（`crates/cesium-viewer/src/entities.rs`）：
     - 扩展 `dereference_czml_packets`，支持 `references` 字符串数组与 `holes` 嵌套孔洞数组的跨包递归解引用。
     - 自动定位 referenced 采样包在 $t_0$ 时刻的大地坐标生成静态几何顶点环与孔洞拓扑。
     - 将多边形曲面（`polygon_surface.rs`）的大地坐标与高程安全上限提升至 $10^7$ 米（10,000 km），消除航天器与高空轨道多边形的越界假死。
  2. **三维模型图元形状管道（`Shape::Model`）**：
     - 在 `Shape` 中引入 `Model { uri, scale, minimum_pixel_size, maximum_scale }`，生成与点/线/面图元统一的轴对齐三维多面体包围网格与拾取线框，确保模型与现有点、线、多边形管道共享统一的拾取颜色编码、逆 Z 深度缓冲与相机集合取景计算。
  3. **距离显示条件视距状态系统（DistanceDisplayCondition）**：
     - 为 `Entity` 引入统一的 `distance_display_condition: Option<[f64; 2]>`，反序列化自 `distanceDisplayCondition` 并校验 `near <= far` 与有限性。
     - 兼容 `screen` 视口元素（Billboard、Point、Label）与 `geometric` 实体（Rectangle、Model、Polygon）的双向解析。
  4. **变体选择器与菜单切换测试闭环**：
     - 在 Sandcastle UI 与测试驱动中完备支持 `format: "variants"` 类型案例，实现 Variant 0 与 Variant 1 之间的无损重置与实体集合切换。
- **验证结果**：
  - **普通 Rust 单元测试**：97 / 97 项全部通过。
  - **真实 GPU 离屏渲染测试**：84 / 84 项全部通过（用时 18.75s，全部包含离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 83 个固定案例哈希与实体结构精准契合）。
  - **桌面 90 帧冒烟测试**：88 个配置（83 案例 + 5 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：83 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），包含实体像素反读、ID 拾取、显隐交互、鼠标滚轮缩放与窗口自适应验证。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy -p cesium-viewer --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [CZML 动态插值多边形与孔洞、距离可见条件裁切、回调属性动态折线与地图图标引脚验证文档](docs/czml-interpolating-polygon-distance-pins-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-29] v0.0.35 验收完成：Billboards 变体、CZML 飞机/立方体模型与无人机回调样条体系全面打通
- **完成配方**（累计推进至 **87 / 309**，剩余 222 个）：
  1. `billboards`（广告牌图元样式全集：10 种工具栏变体，涵盖 eyeOffset 视距偏移、pixelOffset 像素偏移、rotation 旋转、scale 缩放、color 染色、scaleByDistance 视距缩放、translucencyByDistance 视距半透明、pixelOffsetScaleByDistance 视距像素偏移、disableDepthTestDistance 地形防遮挡穿透）
  2. `czml-model`（CZML 飞机三维模型：`Cesium_Air.glb`、`scale: 2.0`、`minimumPixelSize: 128`、`trackedEntity: "aircraft model"` 动态相机追踪取景）
  3. `czml-model-data-url`（CZML 内联模型：Data URI base64 glTF 2.0 立方体、`scale: 2.0`、`minimumPixelSize: 128`、`trackedEntity: "cube"` 静态朝向取景）
  4. `callback-position-property`（无人机样条轨迹：9 个透明黄色描边航路点、CesiumDrone 动态位置回调属性、CatmullRomSpline 连续三维样条插值、PolylineGlow 轨迹光流、`trackedEntity: "drone"`）
- **核心研发突破**：
  1. **CZML Model 数据包解码与三维包围体拓扑**（`crates/cesium-viewer/src/entities.rs` 与 `outline.rs`）：
     - 在 `decode_screen_or_model_packet` 中新增 `"model"` 数据包解析，支持 glTF 格式的 `uri`（对象或字符串）、`scale`、`minimum_pixel_size` 与 `maximum_scale`。
     - 为 `Shape::Model` 构建三维代理网格（Half dimensions: `(8.0, 14.0, 6.0)`），配合相机离地安全间隙（1000m），确保飞机和无人机在视野中产生清晰可见的屏幕像素（18~84 像素）与精确的三维线框拾取。
  2. **视距半透明度 NearFarScalar 上限宽限兼容**（`crates/cesium-renderer/src/screen_distance.rs`）：
     - 将 `ScreenDistance::validate()` 中 translucency 的数值校验上限从 `1.0` 放宽至 `8.0`，原生兼容 CesiumJS 官方 Sandcastle `NearFarScalar(150, 2.0, 1.5e7, 0.5)` 中 `nearValue = 2.0` 的视距淡出配置。
  3. **复杂变体全套冒烟与自动化生成**（`crates/cesium-viewer/tests/full-recipes-gpu.rs` 与 `scripts/record-full-recipe-results.py`）：
     - 在原生离屏 GPU 测试中加入变体 1~9 全量渲染循环，自动输出全部 `docs/full-billboards-variant-{1..9}-smoke.log`。
     - 在 Web 端 Sandcastle 挂载 `fullNew60` ~ `fullNew63` 测试按钮，支持 `EXTENDED_ONLY=1 BATCH35_ONLY=1` 快速回归。
- **验证结果**：
  - **普通 Rust 单元测试**：97 / 97 项全部通过（新增 `batch35_czml_model_and_billboard_contracts`）。
  - **真实 GPU 离屏渲染测试**：88 / 88 项全部通过（用时 20.76s，全部包含离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 87 个固定案例哈希与实体结构精准契合）。
  - **桌面 90 帧冒烟测试**：102 个配置（87 案例 + 15 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：87 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），包含实体像素反读、ID 拾取、显隐交互、鼠标滚轮缩放与窗口自适应验证。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy -p cesium-viewer --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Billboards 样式变体、CZML 飞机与立方体模型、无人机样条轨迹回调验证文档](docs/billboards-czml-model-drone-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-29] v0.0.36 验收完成：Labels 变体、3D Models 模型全集、CZML 骨骼节点变换与整流罩铰接动画体系全面打通
- **完成配方**（累计推进至 **91 / 309**，剩余 218 个）：
  1. `labels`（文字图元样式全集：8 种工具栏变体，涵盖基础文字、字体字号、多行属性、结合 Billboard 视距像素偏移、视距半透明淡出、视距缩放、希伯来语/RTL 双向多语言混排、文字实时动画）
  2. `3d-models`（3D glTF 模型全集：10 种工具栏变体，涵盖 Aircraft 飞机、Drone 无人机、Ground Vehicle 地面车、Hot Air Balloon 热气球、Milk Truck 牛奶卡车、Skinned Character 蒙皮角色、Unlit Box 无光照盒子、Draco 压缩模型、KTX2 压缩气球、Instanced Box 实例化盒子，局部 ENU 朝向四元数及 `viewer.trackedEntity` 相机追踪）
  3. `czml-model-node-transformations`（CZML 模型骨骼节点变换动画：CesiumMan 蒙皮角色、`viewFrom: [4.3, 0.1, 2.6]` 局部相机锁定、`nodeTransformations` 驱动 `Skeleton_arm_joint_L__3_` 与 `Skeleton_arm_joint_R__2_` 左右臂摆动时间四元数采样序列）
  4. `czml-model-articulations`（CZML 运载火箭铰接动画：launchvehicle 火箭模型、`articulations` 驱动 `Fairing Open` 展开、`Fairing Separate` 分离与 `Fairing Drop` 下落 Lagrange 二次插值多阶段动画、时钟倍速 60x 循环）
- **核心研发突破**：
  1. **CZML 与 Model 图元动画/骨骼变换/铰接管道**（`crates/cesium-viewer/src/entities.rs`）：
     - 为 `Shape::Model` 扩展可选字段 `run_animations: Option<bool>`、`node_transformations: Option<serde_json::Value>` 与 `articulations: Option<serde_json::Value>`。
     - 为 `Entity` 引入 `view_from: Option<[f64; 3]>`，支持局部偏置视距锁定相机。
     - 在 `decode_screen_or_model_packet` 与 CZML 数据包白名单中开放 `"runAnimations"`、`"nodeTransformations"`、`"articulations"` 与 `"viewFrom"`。
  2. **RTL / 多语言文本 Unicode 安全与字形兜底机制**（`crates/cesium-viewer/src/label.rs`）：
     - 扩展 `Label::validate()`，引入希伯来/阿拉伯 RTL 字符段放行规则（`\u{0590}..\u{08FF}`），在拦截危险 emoji 的同时安全放行官方 Sandcastle 希伯来语/英语双向混合标注文案。
     - 扩展 `Label::raster()`，对内置精简矢量字体未映射的字符实施字形兜底置换（'?'），保证文本排版布局与图元度量安全稳定，杜绝恐慌崩溃。
  3. **自动化测试与双端渲染验证**：
     - 原生 GPU 测试集成 8 种 Labels 变体与 10 种 3D Models 变体循环渲染，输出全部 `docs/full-labels-variant-{1..7}-smoke.log` 与 `docs/full-3d-models-variant-{1..9}-smoke.log`。
     - WebGPU 端到端沙盒集成 `fullNew64` ~ `fullNew67` 测试按钮，支持 `EXTENDED_ONLY=1 BATCH36_ONLY=1` 快速回归。
- **验证结果**：
  - **普通 Rust 单元测试**：97 / 97 项全部通过（新增 `batch36_labels_models_transformations_articulations`）。
  - **真实 GPU 离屏渲染测试**：92 / 92 项全部通过（全套 118 项离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 91 个固定案例哈希与实体结构精准契合）。
  - **桌面 90 帧冒烟测试**：122 个配置（91 案例 + 31 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：91 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），包含实体像素反读、ID 拾取、显隐交互、鼠标滚轮缩放与窗口自适应验证。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy -p cesium-viewer --all-targets` 零告警、零报错。
---

### [2026-10-29] v0.0.37 验收完成：3D Models Coloring、3D Models Dev、Labels Dev 与 Billboards Dev 底层开发体系全面打通
- **完成配方**（累计推进至 **95 / 309**，剩余 214 个）：
  1. `3d-models-coloring`（3D 模型材质颜色与轮廓高亮：5 种模型变体，包含飞机、地面车、热气球、牛奶卡车、蒙皮角色，`color: [1, 0, 0, 1]` 红色着色、`colorBlendMode: "Highlight"`、`colorBlendAmount: 0.5`、`silhouetteColor: [1, 0, 0, 1]`、`silhouetteSize: 2.0`、局部 ENU 朝向四元数及 `viewer.trackedEntity` 相机追踪）
  2. `3d-models-dev`（底层 Model.fromGltf 基元开发合集：5 种底层图元变体，包含飞机、无人机、地面车、牛奶卡车、蒙皮角色，基于 `headingPitchRollToFixedFrame(origin, hpr)` 局部 ENU 坐标系到 ECEF 四元数姿态解算，绑定 `minimumPixelSize: 128`、`maximumScale: 20000`、`colorBlendMode: "Highlight"`、`runAnimations: true`，由 `camera.lookAt` 矩阵锁定视角）
  3. `labels-dev`（底层 LabelCollection 标签基元开发合集：6 种底层开发变体，包含基础文本标签、24px Helvetica 字体/SKYBLUE 填充/黑色 2px 描边与 `FILL_AND_OUTLINE`、300km 高空标签/背景框激活、局部 ENU 参考系原点与 `[1e6, 0, 0]` 轴线挂载 East/North/Up 多标签、结合 Billboard 视距相对偏移曲线、以及双城视距透明度渐隐）
  4. `billboards-dev`（底层 BillboardCollection 广告牌基元开发合集：11 种底层开发变体，包含基础广告牌、尺寸与旋转对齐、动态属性变更与黄色着色、物理米级尺寸 `sizeInMeters: true` 宽 8000m 高 3900m、批量散列、视距动态缩放/渐隐/偏移曲线、纯白像素绿色着色点阵广告牌、大图集精灵切片子区域采样 `imageSubRegion: [x, y, w, h]` 归一化 UV 映射、以及 ENU 局部参考系广告牌挂载）
- **核心研发突破**：
  1. **广告牌大图集子区域精灵切片与米级物理尺寸（`crates/cesium-viewer/src/screen.rs` & `entities.rs`）**：
     - 为 `Shape::Billboard` 引入 `image_sub_region: Option<[f64; 4]>` 与 `size_in_meters: Option<bool>`。
     - 结合纹理物理尺寸断言并计算精准归一化 UV `[u0, u1, v0, v1]`，确保复杂精灵大图切片采样在 WebGPU 渲染管线中准确渲染。
  2. **三维模型着色混合与高亮轮廓（`crates/cesium-viewer/src/entities.rs`）**：
     - 为 `Shape::Model` 引入 `color_blend_mode`、`color_blend_amount`、`silhouette_color` 与 `silhouette_size`，严格规范数值范围。
  3. **文本标签多行排版增强与底层外观直通（`crates/cesium-viewer/src/label.rs` & `entities.rs`）**：
     - 在 `Shape::Label` 中支持直接透传 `font`、`show_background`、`background_color`、`outline_color`、`outline_width`、`style` 与 `fill_color` 至底层 `Appearance`。
     - 修正字符光栅化中换行符 `\n` 处理，维护精确的多行排版跨度与基线高度。
  4. **ENU 局部姿态到 ECEF 四元数转换工具链（`scripts/batch37-recipes.mjs`）**：
     - 构建了局部 ENU 变换矩阵与 Heading-Pitch-Roll 欧拉角转换为 ECEF 全球空间四元数的数学算法，完美还原官方 `headingPitchRollToFixedFrame` 姿态行为。
- **验证结果**：
  - **普通 Rust 单元测试**：97 / 97 项全部通过（新增 `batch37_models_coloring_dev_labels_billboards_contracts`）。
  - **真实 GPU 离屏渲染测试**：96 / 96 项全部通过（耗时 21.97s，包含 95 个完整案例 + 1 个材质拾取测试）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 95 个固定案例哈希与实体结构精准契合）。
  - **桌面 90 帧冒烟测试**：149 个配置（95 案例 + 54 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：95 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），包含实体像素反读、ID 拾取、变体切换、多图元覆盖与窗口自适应验证。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [3D Models Coloring、3D Models Dev、Labels Dev 与 Billboards Dev 验证文档](docs/models-coloring-dev-labels-billboards-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。








