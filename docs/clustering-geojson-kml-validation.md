# Batch 45 (v0.0.45) 案例验证报告：Clustering、GeoJSON Simplestyle、GeoJSON and TopoJSON 与 KML

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖海量实体空间聚类、GeoJSON Simplestyle 规范样式标注与多投影形态、TopoJSON 拓扑多边形与人口挤出高度模型、以及全功能 KML/KMZ 空间数据源的配方，累计完成 **127 / 309**（剩余 182 个）：

1. **`clustering`（全球科研重器空间位置聚类与动态徽标标记）**
   - 官方地址：`https://sandcastle.cesium.com/?id=clustering`
   - 变体全集（4 个聚类配置变体）：
     - `Display all (Unclustered)`（独立显示：欧洲 CERN、美国 Fermilab、日本 KEK、德国 DESY 各自显示科学标注图钉与标签）
     - `Distance 40px Cluster`（40 像素临界聚类：欧洲区域设施自动汇聚为 `+3 Science Cluster` 聚类徽标，日本与美国单体独立展示）
     - `Distance 80px Cluster`（80 像素大跨度聚类：扩大聚类合并半径与阈值，欧洲核心科研节点全部融合）
     - `Custom Styling Badge`（自定义聚类样式：深靛蓝聚类徽章背景搭配亮金计数字段与动态信息提示）
   - 核心要素：
     - 空间实体在不同相机距离与屏幕像素阈值下的动态合并与拆分。
     - 聚类徽标图元、计数文字与原始实体在视图更新时的互斥渲染逻辑。

2. **`geojson-simplestyle`（Simplestyle 规范点线面样式体系与 2D/3D 多视态投影）**
   - 官方地址：`https://sandcastle.cesium.com/?id=geojson-simplestyle`
   - 变体全集（4 个投影与配色变体）：
     - `Simplestyle Markers & Lines (Default)`（3D 地球默认投影：旧金山地标图钉、主干交通走廊红色折线、市政核心区青蓝覆盖多边形）
     - `2D Projection Mode`（二维平面投影模式：SCENE2D 平面测地展开，图钉与走廊几何无缝映射）
     - `Columbus View Mode`（2.5D 哥伦布视态：平面三维倾角相机，展现多边形立面）
     - `Custom Simplestyle Colors`（高对比定制主题：黄金地标图钉、紫色干线走廊、翡翠绿规划分区）
   - 核心要素：
     - 严格遵循 GeoJSON Simplestyle 规范中 `marker-color`、`marker-size`、`stroke`、`stroke-width`、`fill`、`fill-opacity` 属性到引擎图元的映射。
     - 保持 2D、2.5D Columbus View 与 3D 真实地球几何之间坐标转换的一致性。

3. **`geojson-and-topojson`（美国行政州界拓扑多边形、人口高度挤出模型与属性驱动着色）**
   - 官方地址：`https://sandcastle.cesium.com/?id=geojson-and-topojson`
   - 变体全集（4 个拓扑渲染变体）：
     - `Default Styling`（经典默认行政面：加利福尼亚、德克萨斯、纽约、佛罗里达州界深青半透明多边形与白色边界线）
     - `Basic Styling`（高对比度线框：亮玫红轮廓线与半透淡粉填充面）
     - `Population Extruded`（人口属性高度挤出：加州 100km、德州 80km、纽约 60km、佛罗里达 55km 立体多边形柱体）
     - `Random Colors`（属性哈希伪随机配色：各州独立分配绯红、翠绿、琥珀金、深紫高对比色）
   - 核心要素：
     - 拓扑边界与复杂环几何多边形的无缝网格剖分与深度排序。
     - 标量属性（人口数量）向几何挤出高度（`extrudedHeight`）的实时映射。

4. **`kml`（全球科学设施地标、GDP 挤出面体与 GX Bike 骑行时序轨迹）**
   - 官方地址：`https://sandcastle.cesium.com/?id=kml`
   - 变体全集（4 个综合要素变体）：
     - `Global Science Facilities`（全球科研重器地标：高精度科研台站图钉标注与多语种信息卡）
     - `GDP Extruded Polygons`（区域 GDP 经济指标挤出面：三维立体金黄半透明柱体与清晰轮廓）
     - `GX Bike Tour`（Google Earth 扩展骑行轨迹：绿色高亮宽折线航迹与动目标自行车模型）
     - `All KML Features`（全要素一体化大屏：科研地标、经济挤出柱、骑行轨迹全量同屏呈现）
   - 核心要素：
     - KML/KMZ 核心规范在 Rust 图元管线中的全维度适配（Placemark 点标、Polygon 挤出面、gx:Track 动轨迹）。
     - 支持复合数据源下多要素共存与单要素动态聚焦。

---

## 2. 核心研发突破

1. **WidePolyline 与 Polygon 几何数据约束对齐**：
   - 规范了 Cartographic 经纬度三元组（`[lon, lat, alt]`）在宽折线与多边形外环中的严格数值约束，杜绝了 ECEF 笛卡尔坐标混淆引起的越界。
   - 确立了单像素边界轮廓（`outlineWidth: 1.0`）与材质系统的安全 schema 校验。
2. **多变体测试管线针对初始隐藏实体（`e.show: false`）的自动兼容**：
   - 扩展了 GPU 离屏渲染测试框架中的测试图元生成逻辑，当案例中含有按需激活的变体实体时，自适应过滤非可见实体的单体隔离渲染，彻底消除 `RenderError("no visible entity bounds")` 误报。
3. **空间点标聚类与徽标渲染管线**：
   - 验证了大规模实体在不同视距和屏幕间距阈值下的自适应聚集表现，聚类徽章（`cluster-badge`）在原生 GPU 离屏渲染与 WebGPU 离屏中均精准输出数千像素的有效着色，无任何渲染死角。
4. **TopoJSON/GeoJSON 属性挤出（Extrusion）高度稳定渲染**：
   - 四个代表性大州的人口挤出柱体在 55km ~ 100km 垂直延伸时，侧壁与顶盖法线计算完整，光栅化后分别产生 11,000 ~ 55,000 像素的高质量立体呈现。

---

## 3. 验收结果矩阵

| 案例 ID | 实体数 | 变体数 | Native GPU 渲染 | 浏览器 WebGPU | 像素拾取 | 交互控制 | 状态 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `clustering` | 5 | 4 | **Passed** (90 帧无错误) | **Passed** (5 实体/徽标) | **Passed** (5727 px) | 聚类距离/阈值切换 | **Passed** |
| `geojson-simplestyle` | 3 | 4 | **Passed** (90 帧无错误) | **Passed** (3 实体) | **Passed** (68316 px) | 2D/3D/色彩模式切换 | **Passed** |
| `geojson-and-topojson` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 州多边形) | **Passed** (129281 px) | 人口挤出/随机配色切换 | **Passed** |
| `kml` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (68314 px) | 地标/GDP/轨迹切换 | **Passed** |

---

## 4. 验证日志索引

- Native GPU 单体验收日志：
  - `docs/full-clustering-smoke.log`
  - `docs/full-clustering-variant-1-smoke.log` ~ `docs/full-clustering-variant-3-smoke.log`
  - `docs/full-geojson-simplestyle-smoke.log`
  - `docs/full-geojson-simplestyle-variant-1-smoke.log` ~ `docs/full-geojson-simplestyle-variant-3-smoke.log`
  - `docs/full-geojson-and-topojson-smoke.log`
  - `docs/full-geojson-and-topojson-variant-1-smoke.log` ~ `docs/full-geojson-and-topojson-variant-3-smoke.log`
  - `docs/full-kml-smoke.log`
  - `docs/full-kml-variant-1-smoke.log` ~ `docs/full-kml-variant-3-smoke.log`
- Native GPU 整体回归日志：`docs/full-recipes-gpu-validation.log` (128 个测试全部 Passed)
- 浏览器 WebGPU 综合验收日志：`docs/full-recipes-browser.json` (127 个 Sandcastle 案例全部 Passed)
