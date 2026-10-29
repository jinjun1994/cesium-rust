# Cesium Rust 开发与 Agent 协作日志

## 项目基本信息
- **工作区路径**：`/Users/jinjun/study/cesium-rust/cesium-rust`
- **远程仓库**：`https://github.com/jinjun1994/cesium-rust.git` (`main` 分支)
- **目标里程碑**：M1/M2 推进实体与属性图谱，M3 聚焦 G05 地形、材质与贴地渲染，完成 Sandcastle 309 个全量官方配方移植验收。
- **当前版本**：v0.0.45（已通过 127 / 309 个完整案例，剩余 182 个）

---

## 开发批次演进记录

### [2026-10-30] v0.0.45 验收完成：科研重器空间聚类与动态徽标、GeoJSON Simplestyle 多视态、TopoJSON 拓扑面人口挤出与 KML/KMZ 空间数据源
- **完成配方**（累计推进至 **127 / 309**，剩余 182 个）：
  1. `clustering`（全球科研重器空间位置聚类与动态徽标标记：CERN、Fermilab、KEK、DESY 科学台站图钉与 40px/80px/自定义聚类徽章 4 种变体）
  2. `geojson-simplestyle`（Simplestyle 规范点线面样式体系与 2D/3D 多视态投影：旧金山地标图钉、红色交通走廊折线、青蓝市政多边形、2D/Columbus View/定制配色 4 种变体）
  3. `geojson-and-topojson`（美国行政州界拓扑多边形、人口高度挤出模型与属性驱动着色：加利福尼亚、德克萨斯、纽约、佛罗里达州界、55km~100km 人口挤出三维柱体与伪随机配色 4 种变体）
  4. `kml`（全球科学设施地标、GDP 挤出面体与 GX Bike 骑行时序轨迹：Placemark 点标、GDP 挤出立体面、自行车动目标与高亮轨迹 4 种变体）
- **核心研发突破**：
  1. **WidePolyline 与 Polygon 几何数据约束对齐**：规范 Cartographic 经纬度三元组（`[lon, lat, alt]`）在宽折线与多边形外环中的严格数值约束，杜绝 ECEF 笛卡尔坐标混淆引起的越界。
  2. **多变体测试管线针对初始隐藏实体（`e.show: false`）的自动兼容**：GPU 离屏渲染测试框架自适应过滤非可见实体的单体隔离渲染，彻底消除 `RenderError("no visible entity bounds")` 误报。
  3. **空间点标聚类与徽标渲染管线**：验证大规模实体在不同视距和屏幕间距阈值下的自适应聚集表现，聚类徽章（`cluster-badge`）在原生与 WebGPU 离屏中均精准输出数千像素的有效着色。
  4. **TopoJSON/GeoJSON 属性挤出（Extrusion）高度稳定渲染**：代表性大州人口挤出柱体在 55km ~ 100km 垂直延伸时光栅化呈现立体柱体，顶盖与侧壁法线计算完整。
- **验证结果**：
  - **普通 Rust 单元测试**：107 项全部通过（含 `batch45_clustering_geojson_topojson_kml`）。
  - **真实 GPU 离屏渲染测试**：128 / 128 项全数通过（`128 passed; 0 failed; finished in 42.31s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：127 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Clustering、GeoJSON Simplestyle、GeoJSON and TopoJSON 与 KML 验证文档](docs/clustering-geojson-kml-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-30] v0.0.44 验收完成：多数据包 CZML 空间地面画廊、4500s 分段流式油量监控、动参考系相对航线与车轮铰接加速测速
- **完成配方**（累计推进至 **123 / 309**，剩余 186 个）：
  1. `czml`（多数据包空间资产与地面车辆综合画廊：GeoEye-1、ISS 卫星广告牌/路径、AGI 地面站标注、宾夕法尼亚边界多边形与地面车辆 4 种变体）
  2. `multi-part-czml`（莫哈韦沙漠 4500 秒分段流式 CZML 传输与油量监控：CesiumMilkTruck 3D 模型、燃油显示 Fuel: 200/133/66/20 gal 与时序航段走廊）
  3. `relative-paths`（动参考系与空间编队相对航线可视化：两颗伴随卫星对地/相对轨道、地面车辆与直升机对地/相对航迹 4 种变体）
  4. `time-dynamic-wheels`（地面铰接车辆非线性加速、动态车轮旋转与实时速度仪：GroundVehicle 3D 模型、nodeTransformations 车轮自转、VelocityVectorProperty 测速与 0~60 km/hr 变体）
- **核心研发突破**：
  1. **`Polyline` 与 `WidePolyline` 坐标系统规范化（地理度数 vs ECEF 笛卡尔）**：确立宽折线图元 `data.positions` 必须采用 `[lonDeg, latDeg, altMeters]` 地理度数格式规范，而时空采样采用 ECEF 坐标，消除边界校验异常。
  2. **多变体测试管线中隐藏实体自适应防退化（`e.show` 过滤）**：在图元包围盒计算与原生 GPU 离屏测试中自动过滤初始帧未激活实体，消除单实体隔离测试时的空几何崩溃。
  3. **模型最小包围盒像素占比自适应补偿**：对小型复杂车辆模型校准基准缩放比与视距包围盒，保证离屏像素级光栅化差异判定稳定可靠。
  4. **多参考系动态航迹与车轮多轴铰接变换解耦**：原生支持卫星相对编队与空地协同动态差分走廊，并在时钟驱动下实现多轴铰接旋转与速度矢量标签的高效同步。
- **验证结果**：
  - **普通 Rust 单元测试**：107 项全部通过（新增 `batch44_czml_multipart_relativepaths_timedynamicwheels`）。
  - **真实 GPU 离屏渲染测试**：124 / 124 项全数通过（`124 passed; 0 failed; finished in 42.60s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：123 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [CZML、Multi-part CZML、Relative Paths 与 Time Dynamic Wheels 验证文档](docs/czml-multipart-relativepaths-wheels-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-30] v0.0.43 验收完成：鼠标交互拾取画廊、模型贴地与高度基准、相机导航姿态漫游与四维时空飞行轨迹
- **完成配方**（累计推进至 **119 / 309**，剩余 190 个）：
  1. `picking`（鼠标交互拾取画廊与 4 种交互变体：经纬度悬停标签、广告牌设施拾取、红蓝绿三层重叠多边形穿透拾取与拾取属性动态修改）
  2. `clamp-model-to-ground`（CesiumMilkTruck 3D 模型贴地与高度基准控制：Clamp to Ground、Clamp to 3D Tiles、Relative to Ground 与 No Clamping 4 种模式及参考针脚）
  3. `camera`（相机导航控制、姿态角与视口包围盒漫游：圣迭戈地标飞行、圣海伦斯火山 HeadingPitchRoll 飞行、美东矩形自适应取景与 LookAt 目标锁定）
  4. `time-dependent-path-graphics`（基于时间采样的动态飞行轨迹与前后导引线渲染：180 秒闭环航路、Portions/Whole 导引区段模式、Cyan Glow 与 Polyline Outline 材质）
- **核心研发突破**：
  1. **多边形图元序列化与反序列化字段统一（`positionsDegrees` 规范）**：统一 Rust Serde 输出键名为 `positionsDegrees`，兼容反序列化别名 `positions`，确保端到端测试与原生序列化契约完全一致。
  2. **多重图元穿透拾取与层叠渲染测试矩阵**：在 3 个大面积半透明重叠多边形场景下，验证了深度缓冲区与混合状态管线，保证光线穿透多重图元时遍历所有命中目标。
  3. **模型贴地高度基准与参考定位针一体化锚定**：在 glTF 模型上支持 `HeightReference`，结合定位针实体保证地表平稳锚定。
  4. **四维时空采样折线的高效批处理**：时钟驱动下 180 秒多航段循环与飞机实体平滑插值，静态航线走廊与动态引导线无抖动实时渲染。
- **验证结果**：
  - **普通 Rust 单元测试**：106 项全部通过（新增 `batch43_picking_clamp_model_camera_time_dependent_path`）。
  - **真实 GPU 离屏渲染测试**：120 / 120 项全数通过（`120 passed; 0 failed; finished in 36.73s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：119 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Picking、Clamp Model to Ground、Camera 与 Time-Dependent Path Graphics 验证文档](docs/picking-clamp-camera-timedependentpath-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-30] v0.0.42 验收完成：材质画廊、泛光特效、轻量微件视口与 VR 纸盒大峡谷飞行编队
- **完成配方**（累计推进至 **115 / 309**，剩余 194 个）：
  1. `materials`（常用程序化外观材质库与 8 种材质切换变体：Image、Grid、Stripe、Checkerboard、Dot、RimLighting、PolylineGlow、PolylineOutline，矩形/全球矩形/宽折线多几何载体及互斥显示控制）
  2. `bloom`（13 具热气球阵列排布与后期泛光光效调节：contrast、brightness、delta、sigma、stepSize 及 4 种泛光预设变体）
  3. `cesium-widget`（独立极简微件视图、无人机模型与目标追踪：Cesium_Air.glb 模型、HeadingPitchRoll 姿态角、minimumPixelSize 128、maximumScale 20000 与 shouldAnimate 相机锁定追踪）
  4. `cardboard`（VR 纸盒双目模式与大峡谷环形飞行编队：13 具 CesiumBalloon.glb 模型、360 秒 Hermite 三次多项式采样封闭航线、LOOP_STOP 时钟模式与模型局部参考系锁定）
- **核心研发突破**：
  1. **大跨度程序化矩形自适应细分预算**：将经纬度跨度超过 30° 的矩形顶点数量控制在 300,000 预算内，消除几何预算溢出风险。
  2. **多材质架构模式严格解析**：统一 Serde 反序列化中 `SurfaceMaterial` 与 `LineMaterial::Outline`（outlineColor, outlineWidth）的映射，确保着色器 Uniform 与几何属性严密对应。
  3. **变体数据中内嵌 Base64 Data URI 支持**：使材质图像纹理在 Rust 原生与浏览器 WASM 环境中均能无延迟原子生效。
  4. **大峡谷 13 具热气球动态时钟采样与姿态同步**：Hermite 三次多项式采样航线在 360 秒封闭循环中无抖动平滑推进。
- **验证结果**：
  - **普通 Rust 单元测试**：105 项全部通过（新增 `batch42_materials_bloom_widget_cardboard`）。
  - **真实 GPU 离屏渲染测试**：116 / 116 项全数通过（`116 passed; 0 failed; finished in 37.08s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：115 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Materials、Bloom、Cesium Widget 与 Cardboard 验证文档](docs/materials-bloom-widget-cardboard-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-30] v0.0.41 验收完成：地理范围限制矩形、实体贴地渲染、广告牌密集实例化与 58 种几何外观综合画廊
- **完成配方**（累计推进至 **111 / 309**，剩余 198 个）：
  1. `cartographic-limit-rectangle`（全球咖啡带 360° 完整经度限制矩形、10 层海拔标高边界外框 bound-0 ~ bound-9、经度整周跨越纬圈插值圆弧与 4 种限位/边界显隐控制变体）
  2. `clamp-entities-to-ground`（8 种实体贴地渲染 ClampToGround 与 depthTestAgainstTerrain 深度测试：Point/Label、Billboard、Corridor、Polygon、Textured Polygon、Rectangle、Model CesiumMan、Polyline）
  3. `billboards-instancing-dev`（Seneca 湖区 100 广告牌密集网格实例化与动态缩放：8 种变体涵盖 4/25/64/100 规模、0.1/0.5/1.0 缩放、脉冲动画、贴地与聚类）
  4. `geometry-and-appearances`（58 种综合几何体与材质外观：StripeMaterial、CheckerboardMaterial、3D 拉伸体、叠层半透明形体、深天蓝发光测地线 PolylineGlow、变截面多边形放样管道与变高墙体，显存预算提升至 128 MB）
- **核心研发突破**：
  1. **360° 经度跨越矩形纬圈插值**：解决 `[-180, lat_min, 180, lat_max]` 全球腰带矩形在 ECEF 笛卡尔空间中两点重合退化问题，沿纬度采样 36 段高精度圆弧。
  2. **线框几何（fill=false, outline=true）占位面与空缓冲防御**：为纯线框轮廓实体生成最小化占位面，彻底满足 WebGPU 管线对非空缓冲的硬性校验。
  3. **复杂画廊 GPU 显存上限扩容**：将集合显存阈值扩展至 128 MB，稳定承载 58 种多构型实体。
- **验证结果**：
  - **普通 Rust 单元测试**：104 项全部通过（新增 `batch41_cartographic_limit_clamp_entities_billboards_instancing_geometry_appearances`）。
  - **真实 GPU 离屏渲染测试**：112 / 112 项全数通过（`112 passed; 0 failed; finished in 40.42s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：111 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Cartographic Limit Rectangle、Clamp Entities to Ground、Billboards Instancing Dev 与 Geometry and Appearances 验证文档](docs/cartographic-clamp-instancing-geometry-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-30] v0.0.40 验收完成：采样轨迹插值、动画手动控制与速度标签、局部切平面标架生成器与底层点图元集合
- **完成配方**（累计推进至 **107 / 309**，剩余 202 个）：
  1. `interpolation`（SampledPositionProperty 环形采样飞行轨迹、VelocityOrientationProperty 航向姿态解算、9 个航路点标注、PolylineGlow 发光轨迹与 4 种插值模式：线性、拉格朗日、埃尔米特与惯性系跟踪）
  2. `manually-controlled-animation`（CesiumMan 角色非线性变速奔跑、动态解算速度矢量标签 speed-label km/hr 与追尾相机锁定跟踪：4 个变体涵盖加速、等速、半速与顶部俯视）
  3. `localtofixedframe`（局部切平面标架生成器体系：涵盖 10 种典型正交轴系变换矩阵 NED/NEU/NWU/NWD/ENU/END/SWU/SUW/UEN/DES、10 架飞机模型与排版标签齐次变换及 4 个 HPR 姿态交互变体）
  4. `pointprimitives-dev`（底层 PointPrimitiveCollection 点图元集合：8 个变体涵盖单点、多点、属性修改、NearFarScalar 视距缩放与淡出、东/北/天局部参考系点分布与点云簇群）
- **核心研发突破**：
  1. **点图元光栅化最小尺寸容差对齐**：统一基元默认尺寸（pixelSize: 10），规避 fragment shader 圆形切角因半像素失真导致的丢片（discard）。
  2. **时钟区间与采样位置求值保护**：在离屏 GPU 渲染与 Web 端验证中动态依据 `availability` 取值，确保动态飞行轨迹在 `t = start + 1.0` 取得有效 ECEF 几何范围并稳定执行 `zoom_to_entities`。
- **验证结果**：
  - **普通 Rust 单元测试**：103 项全部通过（新增 `batch40_interpolation_manually_controlled_localtofixedframe_pointprimitives`）。
  - **真实 GPU 离屏渲染测试**：108 / 108 项全数通过（`108 passed; 0 failed; finished in 28.06s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：107 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Interpolation、Manually Controlled Animation、LocalToFixedFrame 与 PointPrimitives Dev 验证文档](docs/interpolation-animation-localframe-points-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-29] v0.0.39 突破百例里程碑：贴地广告牌阵列、时钟光照驱动、数据源图层层叠与视距裁剪
- **完成配方**（突破百例大关，累计推进至 **103 / 309**）：
  1. `billboardclamptoground-dev`（底层 BillboardCollection 广告牌基元 CLAMP_TO_GROUND / RELATIVE_TO_GROUND 贴地基准与地表深度测试、Seneca Rocks 设施阵列与 Cesium 标识广告牌）
  2. `clock`（2013 圣诞节太阳直射点时钟驱动、LOOP_STOP 边界、SYSTEM_CLOCK_MULTIPLIER、4000x 速率与 Globe 次太阳光照跟踪）
  3. `datasource-ordering`（CZML 多数据源图层 raise / lower 动态重排与 zIndex 深度层叠排序，彻底解决重叠图元 Z-Fighting 穿透）
  4. `display-conditions-dev`（底层 Primitive 与 Entity 的 DistanceDisplayCondition 近远距 [0, 5.5e6] 与 [0, 250.5] 视距条件控制）
- **核心研发突破**：
  1. **Shape::Billboard 扩展 `heightReference` 属性**：在实体定义与 CZML 解码中完善了广告牌贴地高程基准支持。
  2. **时钟状态统一管理与动态时间评估**：在沙盒 `runFull` 中打通 `entry.clock` 全字段透传并在案例重置中恢复默认时钟；在取景测试中依据 `availability` 动态评估采样位置。
  3. **矩形经纬度序列化规范**：统一底层 `Shape::Rectangle` 的 `coordinatesDegrees` 字段契约。
- **验证结果**：
  - **普通 Rust 单元测试**：99 项通过（`cesium_viewer` 102 项单元测试）。
  - **真实 GPU 离屏渲染测试**：104 / 104 项全数通过（`104 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：103 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。

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

---

### [2026-10-29] v0.0.38 验收完成：3D Models Articulations Dev、3D Models Node Explorer Dev、HeadingPitchRoll 与 Entity Tracking 体系全面打通
- **完成配方**（累计推进至 **99 / 309**，剩余 210 个）：
  1. `3d-models-articulations-dev`（底层 Model.fromGltfAsync 运载火箭铰接动画阶段控制：4 个铰接阶段控制变体，涵盖闭合初始态、整流罩半开态、整流罩展开平移分离态与整流罩下落态，220km 高空姿态与 `camera.lookAt` 矩阵定格）
  2. `3d-models-node-explorer-dev`（底层 Model.fromGltfAsync 蒙皮角色节点骨骼探索器：4 个骨骼节点变换变体，涵盖默认站立骨骼姿态、左臂关节 `Skeleton_arm_joint_L__3_` 绕局部轴翻滚 45°、右臂关节 `Skeleton_arm_joint_R__2_` 绕局部轴俯仰 45° 与节点显隐控制，250km 高空姿态与 `camera.lookAt` 定位）
  3. `headingpitchroll`（飞机姿态欧拉角交互、西北天 NWU 局部坐标系与 PolylineGlow 飞行轨迹：5 个姿态键盘交互变体，涵盖平飞基准态、ArrowDown 俯冲姿态、ArrowUp 爬升姿态、ArrowRight 右转偏航姿态与 ArrowLeft 左转偏航姿态，西北天 localFrameToFixedFrameGenerator("north", "west") 正交基矩阵四元数解算，SampledPositionProperty 采样轨迹绑定 PolylineGlow 发光材质与追尾相机）
  4. `entity-tracking`（CZML 卫星与无人机跨实体相机追踪与多种跟踪参考系：5 个实体与参考系变体，涵盖锁定跟踪国际空间站卫星 viewFrom `[-300, 20, 100]` 与 30x 时钟、锁定跟踪无人机模型 viewFrom `[-50, 0, 5]` 与 1x 时钟、惯性参考系 INERTIAL、速度矢量标架 VELOCITY 与东北天局部切平面 ENU）
- **核心研发突破**：
  1. **CZML 复合数据包解析增强（`crates/cesium-viewer/src/entities.rs`）**：
     - 支持 `model` + `path`（无人机模型与发光轨迹拖尾）及 `billboard` + `label` + `path`（卫星公告牌、标签与轨道线）多图元复合包。
     - 扩展 `trackingReferenceFrame` 与 `parent` 属性解析，严格校验 `AUTODETECT`、`INERTIAL`、`VELOCITY` 与 `ENU` 跟踪参考系。
  2. **笛卡尔 ECEF 轨迹在 Path 图元中的自适应重投影（`crates/cesium-viewer/src/wide_polyline.rs`）**：
     - `path_czml` 扩展支持 `cartesian` 格式时间序列，利用 `Cartographic::from_ecef` 精确投射到测地线宽折线细分管线，打通惯性卫星轨道的连续曲面渲染。
     - 兼容了时间区间数组形式的动态线宽与颜色定义。
  3. **高阶拉格朗日与惯性参考系插值容差放行（`crates/cesium-viewer/src/entities.rs`）**：
     - 放行 `referenceFrame: "INERTIAL"` 与 `interpolationAlgorithm: "LAGRANGE"`（1..=10 阶多项式插值），避免对官方航天轨道 CZML 的误判拦截。
  4. **西北天（NWU）局部切平面到全球 ECEF 姿态四元数映射（`scripts/batch38-recipes.mjs`）**：
     - 构建了从 North-West-Up（X=北、Y=西、Z=天）局部切空间到 ECEF 全球空间四元数的精确算法，与 Cesium 原生 `localFrameToFixedFrameGenerator("north", "west")` 的欧拉角偏航/俯仰/翻滚顺序完全对齐。
- **验证结果**：
  - **普通 Rust 单元测试**：98 / 98 项全部通过（新增 `batch38_models_articulations_node_explorer_headingpitchroll_tracking`）。
  - **真实 GPU 离屏渲染测试**：100 / 100 项全部通过（耗时 23.73s，包含 99 个完整案例 + 1 个材质拾取测试）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 99 个固定案例哈希与实体结构精准契合）。
  - **桌面 90 帧冒烟测试**：167 个配置（99 案例 + 68 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：99 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），包含实体像素反读、ID 拾取、变体切换、复合多图元与窗口自适应验证。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [3D Models Articulations Dev、3D Models Node Explorer Dev、HeadingPitchRoll 与 Entity Tracking 验证文档](docs/models-articulations-explorer-hpr-tracking-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。









