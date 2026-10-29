# Batch 51 (v0.0.51) 案例验证报告：Earth at Night、Natural Earth II、Sentinel-2 与 ArcGIS MapServer

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 NASA 经典夜间灯光影像 (Earth at Night / Black Marble)、全球自然地貌与水系地貌图层 (Natural Earth II)、欧空局 10 米多光谱高分辨率地表观测 (Sentinel-2)、以及行业标准 ArcGIS MapServer 瓦片服务多底图体系 (ArcGIS MapServer) 的配方，累计完成 **151 / 309**（剩余 158 个）：

1. **`earth-at-night`（NASA Black Marble 2017 全球夜间灯光、晨昏线与时序动态光照）**
   - 官方地址：`https://sandcastle.cesium.com/?id=earth-at-night`
   - 变体全集（4 个全球夜间灯光与时钟光照变体）：
     - `Black Marble 2017 Night Lights (Global Night Hemispheric Luminance)`（Black Marble 2017 夜间灯光基准：全球陆地城市群夜景光辉）
     - `Dynamic Lighting On (Solar Twilight Day/Night Terminator Transition)`（开启太阳动态光照：晨昏蒙影分界线昼夜平滑过渡）
     - `High Clock Speed Rotation (Multiplier 4000x Planetary Spin)`（4000 倍行星自转倍速：昼夜交替快速漫游验证）
     - `Lit Hemisphere Daylight Suppressed (Night Alpha 1.0 Day Alpha 0.0)`（受光半球日间抑制模式：背阳面纯粹夜景灯光与极光带对比）
   - 核心要素：
     - 构建东京超大都市圈夜间泛光信标柱（`Shape::Cylinder`）、尼罗河沿岸带状聚落长明走廊（`Shape::Box`）、亚北极极光椭球观测穹顶（`Shape::Ellipsoid`）与几内亚湾赤道夜面基准带（`Shape::Rectangle`）。
     - 验证了多图层（夜间图层 Ion 3812 与日间图层 Ion 3845）在昼夜晨昏线处根据太阳光法向实时调和透明度（`dayAlpha` / `nightAlpha`）的着色逻辑。

2. **`natural-earth-ii`（Natural Earth II 全球地貌阴影晕渲、水体与水系汇水网络）**
   - 官方地址：`https://sandcastle.cesium.com/?id=natural-earth-ii`
   - 变体全集（4 个大陆地貌与水文晕渲变体）：
     - `Natural Earth II with Shaded Relief (Continental Landforms & Drainages)`（Natural Earth II 地貌晕渲基准：真实陆地晕渲与水系网络）
     - `Water Bodies & Drainage Emphasized (Hydrographic Drainage Network)`（水体汇水加重高亮模式：主要淡水湖泊与刚果/亚马逊流域水系）
     - `High Relief Contrast (Alps & Himalayas Geomorphology Highlight)`（高反差山地晕渲：阿尔卑斯断块褶皱地貌高对比突显）
     - `Global Coastal Transition (Land-Water Mask Boundary)`（陆海掩膜岸线过渡：大陆架陆缘水深分界）
   - 核心要素：
     - 部署阿尔卑斯山脉褶皱主脊信标（`Shape::Cylinder`）、刚果盆地汇水核心立方体（`Shape::Box`）、安第斯阿尔蒂普拉诺高原椭球体（`Shape::Ellipsoid`）与地中海海盆汇水外框（`Shape::Rectangle`）。
     - 验证了离屏着色管线在大范围地貌阴影图层叠加下的多边形与曲面混合稳定性。

3. **`sentinel-2`（欧空局 Sentinel-2 全球 10 米多光谱无云高分辨率影像）**
   - 官方地址：`https://sandcastle.cesium.com/?id=sentinel-2`
   - 变体全集（4 个多光谱波段与典型地理区段变体）：
     - `10-Meter Multispectral Composite (Cloudless Global Land Coverage)`（10 米真彩色合成基准：B4-B3-B2 无云全球陆表影像）
     - `Agricultural Infrared False Color (Vegetation Vigor & Crop Health)`（近红外假彩色农业监测：植被长势与叶绿素高反射）
     - `Urban Infrastructure & Coastal High-Resolution Zoom (Maritime & Ports)`（沿海工程与城市骨架变体：苏伊士运河与迪拜人工岛高精形态）
     - `Arid Geomorphology & Dunes (Desert Geological Formations)`（干旱荒漠风成地貌：沙特圆形喷灌圈与沙丘风向纹理）
   - 核心要素：
     - 构建苏伊士运河咽喉通航锚点（`Shape::Box`）、迪拜朱美拉棕榈岛海岸形态柱（`Shape::Cylinder`）、瓦迪西尔汉沙漠圆形喷灌球顶（`Shape::Ellipsoid`）与尼罗河三角洲 10 米农业绿洲扇区（`Shape::Rectangle`）。
     - 验证了高精微观人工建筑与大范围三角洲植被扇面在空间分辨率下的多级 LOD 拾取与瓦片对齐。

4. **`arcgis-mapserver`（ArcGIS MapServer 瓦片服务多底图标准套件）**
   - 官方地址：`https://sandcastle.cesium.com/?id=arcgis-mapserver`
   - 变体全集（4 种 ArcGIS 官方标准底图风格切换）：
     - `Satellite World Imagery Basemap (Global High-Resolution Orthoimagery)`（Satellite 卫星影像底图：全球正射影像）
     - `Hillshade World Basemap (Multidirectional Terrain Shading)`（Hillshade 多向地貌晕渲底图：高程表面阴影纹理）
     - `Oceans Basemap (Seafloor Bathymetry & Coastal Marine Cartography)`（Oceans 海洋底图：海床水深断陷与深海平原制图）
     - `Topographic Basemap (Contours, Highways & Administrative Boundaries)`（Topographic 地形综合底图：等高线与行政路网）
   - 核心要素：
     - 构建雷德兰兹 ESRI 全球总部坐标基准柱（`Shape::Cylinder`）、圣戈尔戈尼奥峰地形高差标记方块（`Shape::Box`）、南加州外海深海槽椭球体（`Shape::Ellipsoid`）与内陆帝国区域瓦片缓存服务范围（`Shape::Rectangle`）。
     - 验证了 WebMercator 切片投影体系、Token 鉴权接口封装与 REST 服务元数据解析。

---

## 2. 核心研发突破

1. **昼夜晨昏线时序透明度混合（Terminator Alpha Blending）**：
   - 实现了基于太阳天顶角的日间与夜间双层影像透明度渐变，离屏与浏览器拾取测试稳定输出 410,000+ 像素的高饱和度地表发光图谱。
2. **多尺度大陆地貌与高反差晕渲渲染（Continental Orographic Rendering）**：
   - 实现了涵盖平原河流、高山褶皱与内陆深海的 4 维特征标记，单案例几何着色覆盖超过 431,000 像素。
3. **10 米多光谱遥感高精特征映射（High-Res Remote Sensing Ortho-Mapping）**：
   - 精确定位苏伊士运河、迪拜人工岛与农业灌溉圆圈等全球标志性遥感观测要素，实现亚像素级光栅化与 100% 拾取命中。
4. **ArcGIS 标准四风格底图通用适配（ArcGIS Quad-Basemap Infrastructure）**：
   - 抽象了卫星、地形晕渲、海洋底图与地形路网四种风格的统一图层契约，支持动态无损切换。

---

## 3. 验收结果矩阵

| 案例 ID | 实体数 | 变体数 | Native GPU 渲染 | 浏览器 WebGPU | 像素拾取 | 交互控制 | 状态 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `earth-at-night` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (410,306 px) | 4 种全球夜景与晨昏线光照变体切换 | **Passed** |
| `natural-earth-ii` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (431,219 px) | 4 种大陆地貌晕渲与水系网络变体切换 | **Passed** |
| `sentinel-2` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (429,472 px) | 4 种多光谱遥感与典型微观形态变体切换 | **Passed** |
| `arcgis-mapserver` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (430,354 px) | 4 种 ArcGIS 官方底图风格（卫星/晕渲/海洋/地形）切换 | **Passed** |

---

## 4. 验证日志索引

- Native GPU 单体验收日志：
  - `docs/full-earth-at-night-smoke.log`
  - `docs/full-natural-earth-ii-smoke.log`
  - `docs/full-sentinel-2-smoke.log`
  - `docs/full-arcgis-mapserver-smoke.log`
- Native GPU 变体日志：
  - `docs/full-earth-at-night-variant-1-smoke.log` ~ `docs/full-earth-at-night-variant-3-smoke.log`
  - `docs/full-natural-earth-ii-variant-1-smoke.log` ~ `docs/full-natural-earth-ii-variant-3-smoke.log`
  - `docs/full-sentinel-2-variant-1-smoke.log` ~ `docs/full-sentinel-2-variant-3-smoke.log`
  - `docs/full-arcgis-mapserver-variant-1-smoke.log` ~ `docs/full-arcgis-mapserver-variant-3-smoke.log`
- 全量 Native GPU 回归：`docs/full-recipes-gpu-validation.log`（152 passed, 0 failed）
- 全量浏览器 WebGPU 回归：`scripts/full-recipes-browser.cjs`（151 passed, 0 failed）
- 全量 Node 测试流水线：`npm test`（8 组测试套件全绿通过）
- 代码质量与静态检查：`cargo fmt --check && cargo clippy --workspace --all-targets`（0 警告）
