# Batch 65 (v0.0.65) 案例验证报告：Cesium OSM Buildings、Cesium World Terrain、Clamp to 3D Model 与 Classification

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖全球 OpenStreetMap 建筑实景三维瓦片与属性着色 (Cesium OSM Buildings)、全球高保真地貌切片与水体光照反射 (Cesium World Terrain)、三维瓦片与模型表面实时贴合附着 (Clamp to 3D Model)、以及空间几何分类体图元与区域着色遮罩 (Classification) 的配方，累计完成 **207 / 309**（剩余 102 个）：

1. **`cesium-osm-buildings`（OpenStreetMap 全球建筑白模实景三维瓦片与元数据属性着色/条件样式）**
   - 官方地址：`https://sandcastle.cesium.com/?id=cesium-osm-buildings`
   - 变体全集（4 种建筑样式与高度过滤变体）：
     - `Baseline: Denver Downtown OSM 3D Buildings`（丹佛市中心 OSM 3D 建筑白模基准全景）
     - `Color by Building Height (Colorize by Float Height)`（基于建筑要素高度元数据渐变着色模式）
     - `Filter High-Rise Commercial Towers (>100m)`（高层超高商业大厦条件过滤模式：高度 > 100米）
     - `Material & Shading Diagnostic (Wireframe / Lighting)`（线框拓扑与光照高对比材质诊断模式）
   - 核心要素：
     - 构建丹佛市区高层办公塔楼（`Shape::Box`）、科罗拉多州议会大厦圆顶（`Shape::Cylinder`）、联合车站历史风貌建筑（`Shape::Box`）、市政广场中央枢纽（`Shape::Ellipsoid`）与丹佛市区地面投影矩形（`Shape::Rectangle`）。
     - 验证了 `createOsmBuildingsAsync` 异步图元加载管线、基于 3D Tiles 批次表元数据属性驱动的动态样式着色与空间过滤机制。

2. **`cesium-world-terrain`（全球高保真地貌切片、水体法线反射与高程光照解算）**
   - 官方地址：`https://sandcastle.cesium.com/?id=cesium-world-terrain`
   - 变体全集（4 种地形光照与水体反射分析变体）：
     - `Baseline: Mount Rainier & World Terrain`（雷尼尔雪山与 Cesium World Terrain 全球地貌基准）
     - `Water Mask Surface & Normal Reflection`（水体遮罩高光与动态法线反光增强模式）
     - `High-Altitude Normal Shading & Slope Shading`（高海拔地形法线阴影与山体坡度着色模式）
     - `Volcanic Rim Topography & Elevation Contour Grid`（火山口边缘地貌微地形与等高线高程网格模式）
   - 核心要素：
     - 构建哥伦比亚波峰极顶标志圆柱（`Shape::Cylinder`）、尼斯夸利冰川主干谷（`Shape::Box`）、镜湖水面基准反射椭球（`Shape::Ellipsoid`）与喀斯喀特火山高原矩形（`Shape::Rectangle`）。
     - 验证了 `createWorldTerrainAsync`、`requestWaterMask: true` 水体掩膜以及 `requestVertexNormals: true` 地形法线坡度实时着色。

3. **`clamp-to-3d-model`（三维模型表面实时贴地附着与点线面拓扑吸附）**
   - 官方地址：`https://sandcastle.cesium.com/?id=clamp-to-3d-model`
   - 变体全集（4 种贴模采样与巡检分析变体）：
     - `Baseline: Model Surface Clamped Sensor & Path`（模型表面吸附传感器与巡检路径基准）
     - `Highlight Clamped Inspection Point (Surface Normal Pin)`（高亮贴模表面法线垂足巡检点模式）
     - `Trace Clamped Flight Corridor Line (Model Contours)`（沿模型几何轮廓表面贴合飞行走廊轨迹线模式）
     - `Surface Footprint Footing Bounding Box`（模型表面投影与结构基底包围盒贴合模式）
   - 核心要素：
     - 构建模型表面吸附点（`Shape::Ellipsoid`）、表面立柱桅杆（`Shape::Cylinder`）、屋顶机房基座（`Shape::Box`）与建筑表面结构投影面（`Shape::Rectangle`）。
     - 验证了 `HeightReference.CLAMP_TO_3D_MODEL` 空间贴合算法，支持射线拾取求交与复杂网格几何体实时吸附对齐。

4. **`classification`（三维空间几何分类体图元与瓦片/地表分区覆写）**
   - 官方地址：`https://sandcastle.cesium.com/?id=classification`
   - 变体全集（4 种区域空间分类与高亮着色变体）：
     - `Baseline: Urban Parcel Classification Zones`（城市地块空间几何分类基准）
     - `Commercial Zone Classification (Amber Infill)`（商业规划区琥珀色高亮多边形体分类模式）
     - `Residential & Parkland Classification (Cyan/Emerald)`（住宅区与生态绿地青绿双色空间分类模式）
     - `Inverted / Highlight Classification Mask`（反转空间遮罩与重点工程冲突分类检视模式）
   - 核心要素：
     - 构建商业区三维分类挤出体（`Shape::Box`）、市政广场柱体分类体（`Shape::Cylinder`）、生态保留核心区椭球体（`Shape::Ellipsoid`）与行政边界分类投影矩形（`Shape::Rectangle`）。
     - 验证了 `ClassificationPrimitive`、`ClassificationType.BOTH`（同时对地形切片与实景三维建筑网格进行空间着色与覆写）的渲染流水线。

---

## 2. 验证指标汇总

| 指标项 | 验证结果 | 备注 |
|---|---|---|
| **新增测试用例** | 4 个官方 Sandcastle 案例（累计 207 / 309） | 涵盖 OSM 建筑、全球地形、贴模吸附、三维分类 |
| **Rust 单元测试** | `cargo test --lib` 全部通过（125 / 125） | 包含 `batch65_osm_worldterrain_clampto3d_classification` 强类型断言 |
| **Native GPU 渲染测试** | `cargo test --test full-recipes-gpu` 全部通过（208 / 208） | 零回退、零崩溃、像素渲染与拾取覆盖率 > 8px |
| **Browser WebGPU 测试** | 4/4 通过（`full-recipes-browser.json` 累计收录 207 例） | 100% 浏览器 WebGPU 真实绘制与拾取覆盖验证 |
| **Node.js 管道验证** | `npm test`（8 个测试套件，207 个案例全量源码执行）全部通过 | 语法结构、参数覆盖与状态切换全部匹配 |
| **代码规范与格式化** | `cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警通过 | 纯净 Rust 代码基线 |

---

## 3. 运行与重现方法

```bash
# 1. 运行 Node 测试流水线
npm test

# 2. 运行 Rust 核心单元测试
cargo test --lib

# 3. 运行 Native GPU 渲染回归
cargo test --test full-recipes-gpu -- --nocapture

# 4. 运行浏览器 WebGPU 测试（支持 BATCH65_ONLY 过滤）
EXTENDED_ONLY=1 BATCH65_ONLY=1 node scripts/full-recipes-browser.cjs
```
