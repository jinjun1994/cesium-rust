# 四个新增完整案例（0.0.33）：CZML 复合广告牌与标签、几何高度参考基准、飞行路径与跨实体引用属性

固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。本批完整实现并验收 **CZML Billboard and Label**、**Geometry Height Reference**、**CZML Path** 与 **CZML Reference Properties** 四个完整源码案例，覆盖 24 个原始实例（2 + 16 + 2 + 4）。累计完整功能案例通过数 **79 / 309**，剩余 230 个待完整验收。

| 案例 | 保留的完整案例行为 |
| --- | --- |
| CZML Billboard and Label | 复合图元数据包（2 个渲染实体）：单个 CZML 数据包同时声明 `billboard`（内置 Base64 嵌入式 PNG 图标、缩放 1.5 倍）与 `label`（文本 "AGI"、深褐色带透明度背景 `#705939C8`、内边距 `[7, 5]`、像素偏移 `[8, 0]`、居左对齐），精准空间定位与集合取景 |
| Geometry Height Reference | 16 个相邻多边形贴地/相对高度渲染矩阵：保留原案例 4x4 网格地理布局，每个多边形独立配置随机色、`heightReference: HeightReference.RELATIVE_TO_GROUND`（高程 40m）与 `extrudedHeightReference: HeightReference.CLAMP_TO_GROUND`（拉伸基底贴地 0m）；支持 Sandcastle 菜单动态切换 16 个 Box / Cylinder / Ellipsoid 变体几何；支持 lookAt 偏航/俯仰角集合对焦 |
| CZML Path | 真实滑翔翼飞行记录轨迹动画（2 个渲染实体）：1790 个高频连续采样大地经纬度点构成的飞行员 GPS 航迹；CZML `path` 声明带描边折线材质 `PolylineOutline`（品红填充/白色描边/线宽 2）、前导/后延时间与动态插值；运动中的 `billboard` 实体携带局部空间视距偏移 `eyeOffset: [0, 0, -10]`；时钟倍速 10x 自动跟随与追踪视锥渲染 |
| CZML Reference Properties | 跨实体属性引用解算（4 个渲染实体）：通过 `"targetId#property.subproperty"` 语法实现动态引用；`fillColor-reference` 引用独立数据包 `position-reference#position` 获得大地经纬度定位；`polygon` 实体的面填充颜色通过 `reference: "fillColor-reference#label.outlineColor"` 动态引用标签的描边紫色 RGBA，实现多实体样式联动与属性传递 |

## 核心实现技术

1. **CZML 跨实体属性解引用引擎（`dereference_czml_packets`）**：
   - 在 `crates/cesium-viewer/src/entities.rs` 中设计并实现了递归解引用预处理机制。在数据包进入实体反序列化之前，预先建立 `id -> packet` 索引表，对任何形如 `targetId#propPath` 的 reference 节点进行跨数据包递归解包（支持至多 16 层深度的嵌套防环保护），支持点号路径访问子对象（如 `label.outlineColor`）与非渲染数据包（如仅含位置坐标的 `position-reference`）。
2. **HeightReference 高度基准类型系统**：
   - 增加 `HeightReference` 枚举（`None`、`ClampToGround`、`RelativeToGround`），支持字符串与整型兼容的反序列化器。
   - 为 `Shape::Polygon`、`Shape::Box`、`Shape::Cylinder`、`Shape::Ellipsoid` 引入 `height_reference` 及 `extruded_height_reference` 字段，满足地形起伏与相对高度标注需求。
3. **高频采样轨迹点去重与线段几何容差（Wide Polyline Path）**：
   - 针对高频 GPS 飞行日志（如 1790 个点）中存在的大量停留或同坐标时间戳，在 `wide_polyline.rs` 中将地理折线采样上限由 1024 提升至 4096，并在几何构建中平滑处理相邻两点距离小于 1mm 的连续退化点，保障大圆弧与法向扩展不退化。
   - 统一 `PolylineOutline` 材质在 CZML 路径中的语义映射，转换为带有 `outlineColor` 与 `outlineWidth` 的宽折线渲染管线。
4. **单数据包多图元拆解（Multi-graphic Packet Decomposition）**：
   - 扩展 CZML 数据包解码器，支持在单个数据包中同时声明 `billboard + label`、`billboard + path` 或 `polygon + label`，自动将其展开为主实体与附着实体，共享基础数据包属性（名称、位置、时钟区间）并保持独立图元样式与几何拾取能力。
5. **视距偏移（Eye Offset）与 CSS 字体尺寸拓展**：
   - 在 `screen_options.rs` 中为屏幕对象引入 `eye_offset: Option<[f64; 3]>`，并支持从 CZML `eyeOffset.cartesian` 解析，让贴合地面的广告牌在深度缓冲中适当靠近相机避免遮挡。
   - 在 `label.rs` 的 CSS 字体解析器中增加对 `pt` 单位的换算支持（`pt * 4/3 = px`）与 "Lucida Console" 字体识别。

## 验收证据

1. **Rust 全工作区单元测试**：
   - `cargo test -p cesium-viewer --lib` 97 项单元测试全部通过（94 passed, 0 failed, 3 ignored）。
2. **真实 GPU 离屏渲染测试**：
   - `cargo test -p cesium-viewer --test full-recipes-gpu -- --ignored` 80 个测试全部通过（包含 `complete_czml_billboard_and_label`、`complete_geometry_height_reference`、`complete_czml_path` 与 `complete_czml_reference_properties`），离屏耗时 22.42 秒。
3. **真实浏览器 WebGPU 端到端测试**：
   - 在原生 Chromium 环境下执行 `scripts/full-recipes-browser.cjs`，所有 79 个完整案例通过实体像素颜色反读、ID 拾取、显隐交互、时间轴跳转、缩放取景与窗口自适应验证。
4. **Node 契约校验套件**：
   - `npm test` 8 组契约测试全部通过，全部 79 个固定案例哈希与实体结构精准契合。
