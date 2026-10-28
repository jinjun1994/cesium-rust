# 四个新增完整案例（0.0.34）：CZML 动态插值多边形与孔洞、距离可见条件裁切、回调属性动态折线与地图图标引脚

固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。本批完整实现并验收 **CZML Polygon - Interpolating References**、**Distance Display Conditions**、**Callback Property** 与 **Map Pins** 四个完整源码案例，覆盖 10 个原始实例（2 + 2/4 + 2 + 4）。累计完整功能案例通过数 **83 / 309**，剩余 226 个待完整验收。

| 案例 | 保留的完整案例行为 |
| --- | --- |
| CZML Polygon - Interpolating References | 引用数组坐标解算与动态带孔多边形（2 个渲染实体）：通过 `"references": ["v1#position", "v2#position", "v3#position"]` 语法跨数据包动态引用采样坐标构成插值多边形外环；通过嵌套引用数组 `"holes": { "references": [["target4#position", "target5#position", "target6#position"]] }` 声明动态内孔；支持 `perPositionHeight: true`（最高顶点海拔达 4,000,000 米空间轨道）与时钟区间 `rgbaf` 材质渐变 |
| Distance Display Conditions | 距离可见性条件视距裁切（2 组变体、4 个渲染实体）：`distanceDisplayCondition` `[near, far]` 视距可见区间校验与几何管线贯通；在近距（0 ~ 250.5m）与远距（0 ~ 5,500,000m）间切换；变体 1 为 Billboard 与 Rectangle（包含红色半透明填充与边界轮廓），变体 2 为 Point 与 3D Model（GroundVehicle.glb 紧凑轴向包围几何与精准姿态四元数朝向） |
| Callback Property | 回调属性与大地长轴测地线距离追踪（2 个渲染实体）：宽折线 `widePolyline`（红色、线宽 5px、大圆弧插值）端点动态更新；实时调用 `EllipsoidGeodesic.surfaceDistance` 动态计算地表测地线长度并换算为公里（km）；中点计算驱动 `label` 实体（白色无衬线体、水平/垂直居中）沿折线移动；实体时钟联动与集合追踪取景 |
| Map Pins | 地图标记引脚生成与异步加载（4 个渲染实体）：`PinBuilder` 构造 4 种标记图标并导出独立 PNG Data URL（皇家蓝底色 48px 引脚、带问号文本引脚、杂货店图标引脚、医院图标引脚）；`VerticalOrigin.BOTTOM` 底针精确定位与地面锚定；`Promise.all` 异步加载集合后自动触发 `viewer.zoomTo` 聚焦全部标记 |

## 核心实现技术

1. **CZML 数组引用与嵌套孔洞解析引擎（`dereference_czml_packets`）**：
   - 拓展 `crates/cesium-viewer/src/entities.rs` 的跨数据包解引用机制，支持提取 `references: ["v1#position", ...]` 字符串数组与 `holes: [["target4#position", ...]]` 嵌套孔洞数组。
   - 解析时自动定位 referenced 数据包在 $t_0$ 时刻的大地经纬度采样值（`cartographicDegrees`），立即合成静态几何顶点环与孔洞拓扑，保障多边形耳切剖分与反向深度测试无退化。
   - 将多边形曲面（`polygon_surface.rs`）的大地坐标与高度安全阈值由 $10^6$ 米放宽至 $10^7$ 米（10,000 km），支持航天器轨道多边形与高空跨层实体绘制。
2. **三维模型形状管道（`Shape::Model`）**：
   - 在 `crates/cesium-viewer/src/entities.rs` 中新增 `Shape::Model { uri, scale, minimum_pixel_size, maximum_scale }` 实体形状。
   - 在 `primitive()` 中生成轴对齐的标准三维多面体包围网格，并为 `outline.rs` 补充线框包围线束，确保模型与现有点、线、多边形管道共享统一的拾取颜色编码、逆 Z 深度缓冲与相机集合取景计算。
3. **距离显示条件视距状态系统（DistanceDisplayCondition）**：
   - 为 `Entity` 引入统一的 `distance_display_condition: Option<[f64; 2]>`，反序列化自 `distanceDisplayCondition` 并校验 `near <= far` 与有限性。
   - 兼容 `screen` 视口元素（Billboard、Point、Label）与 `geometric` 实体（Rectangle、Model、Polygon）的双向解析。
4. **变体选择器与菜单切换测试闭环**：
   - 在 Sandcastle UI 与测试驱动中完备支持 `format: "variants"` 类型案例，实现 Variant 0 与 Variant 1 之间的无损重置与实体集合切换。

## 验收证据

1. **Rust 全工作区单元测试**：
   - `cargo test -p cesium-viewer --lib` 97 项单元测试全部通过（94 passed, 0 failed, 3 ignored）。
2. **真实 GPU 离屏渲染测试**：
   - `cargo test -p cesium-viewer --test full-recipes-gpu -- --ignored` 84 个测试全部通过（包含 `complete_czml_polygon_interpolating_references`、`complete_distance_display_conditions`、`complete_callback_property` 与 `complete_map_pins`），离屏耗时 18.75 秒。
3. **真实浏览器 WebGPU 端到端测试**：
   - 在原生 Chromium 环境下执行 `scripts/full-recipes-browser.cjs`，所有 83 个完整案例通过实体像素颜色反读、ID 拾取、显隐交互、时间轴跳转、变体切换、缩放取景与窗口自适应验证。
4. **Node 契约校验套件**：
   - `npm test` 8 组契约测试全部通过，全部 83 个固定案例哈希与实体结构精准契合。
