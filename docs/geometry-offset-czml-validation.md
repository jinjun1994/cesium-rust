# 四个新增完整案例（0.0.31）：Geometry Offset Attributes 与 CZML Dynamic Properties

固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。本批完整实现并验收 **Geometry Offset Attribute - Dev**、**Geometry Offset Attribute (Box, Cylinder, Ellipsoid) - Dev**、**CZML Polygon Intervals and Availability** 与 **CZML Custom Properties** 四个完整源码案例，覆盖 16 个原始实例。累计完整功能案例通过数 **71 / 309**，剩余 238 个待完整验收。

| 案例 | 保留的完整案例行为 |
| --- | --- |
| Geometry Offset Attribute - Dev | 3 个拉伸矩形几何图元与 3 个匹配的轮廓矩形（共 6 个几何实例）：分别指定 `offsetAttribute: GeometryOffsetAttribute.TOP`（仅顶部网格沿法向偏移）、`offsetAttribute: GeometryOffsetAttribute.ALL`（整个几何体位移）与 `offsetAttribute: GeometryOffsetAttribute.NONE`（无偏移）；几何顶点法向量按 WGS-84 地球椭球表面法向量 \( \hat{n} \) 严格计算，并支持工具栏“Apply Top Offset”、“Apply All Offset”、“Reset Offset”动态属性更新 |
| Geometry Offset Attribute (Box, Cylinder, Ellipsoid) - Dev | 3 个不同类型的实体几何（Box、Cylinder、Ellipsoid）与 3 个匹配的轮廓几何（共 6 个图元实例）：全部配置 `offsetAttribute: GeometryOffsetAttribute.ALL`；采用局部 ENU 站心坐标系 `modelMatrix` 空间摆放，验证不同基元网格顶点在全局与局部矩阵下的动态法向/坐标位移 |
| CZML Polygon Intervals and Availability | 3 个包含动态时间特性的 CZML 实体：`dynamicPolygon`（包含跨四个时间区间的顶点坐标变化、内嵌洞孔 `holes` 动态增减、`extrudedHeight` 变化与红/蓝/绿/黄颜色材质动态跃迁）、`california`（定义有效性区间 `16:10:00Z/16:30:00Z`，支持 `perPositionHeight` 绝对高程多边形表面）与 `pennsylvania`（定义有效性区间 `16:20:00Z/16:40:00Z`，支持拉伸多边形与时间段可见性裁剪） |
| CZML Custom Properties | 包含跨年代（1970-2010 年）时钟配置（multiplier = 500,000,000）的 CZML 体系：包含非视觉数据对象 `custom_property_object`（定义 `population_intervals` 时间区间人口数据与 `population_sampled` 采样时间序列），以及 `colorado` 多边形实体（初始化 `extrudedHeight: 0`，支持将自定义属性数据通过回调动态驱动几何拉伸高度） |

## 核心实现技术

1. **Geometry Offset 几何法向偏移管线**：
   - 在图元几何定义中支持 `GeometryOffsetAttribute`（`NONE: 0`, `TOP: 1`, `ALL: 2`）。
   - 在 `primitive_instances.rs` 中为各几何图元属性集绑定 `attributes.offset`（3 维笛卡尔向量）。
   - 保证底层 WebGPU 管线严格兼容 `lineWidth: 1.0` 规范，实现填充图元与轮廓图元在相同偏移属性下的高精度空间同步对齐。

2. **多边形层级内孔与拉伸高度（Polygon Holes & Extruded Height）**：
   - 扩充 `Shape::Polygon` 数据结构，增加 `holes: Vec<Vec<[f64; 3]>>`（兼容 `holesDegrees` 别名）与 `height: Option<f64>` 字段。
   - 在 `geographic.rs` 与 `outline.rs` 中对多边形内外环顶点统一计算高程偏置与拉伸法向量，精确处理零高程共面与非共面带孔多边形面片三角剖分。

3. **CZML 时间区间与有效性（Intervals & Availability）生命周期**：
   - 完善 CZML 反序列化引擎，支持年格式（如 `"1970/2010"`）与完整 ISO-8601 区间解析。
   - 支持多边形顶点 `positions`、洞孔 `holes`、材质颜色 `solidColor.color` 的时序区间序列。
   - 在时钟驱动与实体裁剪逻辑中，根据当前时钟 \( t \) 自动判断 `availability`，对超出有效生命周期的实体进行视锥剔除与包围盒计算抑制，防止无效边界引发的视口计算退化。

4. **非视觉自定义属性对象（Custom Properties）**：
   - 支持无视觉组件（无 point/polygon 等几何体）的纯数据实体解析与属性表挂载。
   - 支持时序标量数据 `SampledProperty` 与分段常量 `TimeIntervalCollection` 属性查询。

## 验收证据

1. **Rust 全工作区单元测试**：
   - `cargo test --workspace --bins --lib` 全部通过（93 passed, 0 failed, 3 ignored）。
2. **真实 GPU 离屏渲染测试**：
   - `cargo test -p cesium-viewer --test full-recipes-gpu -- --ignored` 全部 72 个测试通过（包含 `complete_geometry_offset_attribute_dev`、`complete_geometry_offset_attribute_box_cylinder_ellipsoid_dev`、`complete_czml_polygon_intervals_availability` 与 `complete_czml_custom_properties`）。
3. **真实浏览器 WebGPU 端到端测试**：
   - 在原生 Chromium 环境下执行 `scripts/full-recipes-browser.cjs`，所有 71 个完整案例通过实体像素颜色反读、ID 拾取、显隐交互与窗口自适应验证。
4. **Node 契约校验套件**：
   - `npm test` 8 组契约测试全部通过，全部 71 个固定案例哈希与实体结构精准契合。
