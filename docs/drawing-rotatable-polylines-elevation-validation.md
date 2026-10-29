# Batch 47 (v0.0.47) 案例验证报告：Drawing on Terrain、Rotatable 2D Map、Polylines On Terrain - Dev 与 Elevation Band Material

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖真实地形交互式点线面标绘 (Drawing on Terrain)、二维平面旋转地图全向航向漫游 (Rotatable 2D Map)、大峡谷复杂地形多路径贴地折线与地表分类面体 (Polylines On Terrain - Dev)、以及喜马拉雅山脉多级海拔等高线带谱与梯度材质 (Elevation Band Material) 的配方，累计完成 **135 / 309**（剩余 174 个）：

1. **`drawing-on-terrain`（地表交互绘制线面几何与顶点吸附系统）**
   - 官方地址：`https://sandcastle.cesium.com/?id=drawing-on-terrain`
   - 变体全集（4 个标绘模式变体）：
     - `Both Line and Polygon on Terrain`（点线面协同全要素模式：地表贴地纯白宽折线、青蓝半透明多边形填充面与 2 具高亮红色顶点图元）
     - `Line Drawing Mode Only`（折线单体绘制模式：聚焦地表测地走廊折线与顶点定位）
     - `Polygon Drawing Mode Only`（多边形单体绘制模式：聚焦地表面域空间范围）
     - `Custom High-Visibility Styles`（高对比定制主题：亮黄加粗航线、翡翠绿规划多边形与高饱和度顶点标绘）
   - 核心要素：
     - 支持交互式连续点标输入时的贴地约束（`clampToGround: true`）与线面几何动态生成。
     - 确立了点图元（`Shape::Point`）与线面几何图元在同一空间参考系中的深度对齐与层叠规范。

2. **`rotatable-2d-map`（二维平面投影全向自由旋转与航向倾角姿态控制）**
   - 官方地址：`https://sandcastle.cesium.com/?id=rotatable-2d-map`
   - 变体全集（4 个航向旋转姿态变体）：
     - `Heading -45 Degrees (Northwest Incline)`（西北 45° 倾角视态：波士顿新英格兰区域大范围俯视、矩形覆盖范围与 42°N 恒向纬线引导）
     - `Heading 0 Degrees (Standard North)`（正北航向基准视态：翠绿高透检验矩形与标准坐标系对齐）
     - `Heading 90 Degrees (Eastward Orientation)`（正东航向侧转视态：黄金地标图钉与侧向展开）
     - `Heading 180 Degrees (Southward Orientation)`（正南倒转全景视态：洋红高亮引导折线与逆向坐标对齐）
   - 核心要素：
     - 在 `SceneMode::SCENE2D` 模式下原生实现 `MapMode2D::ROTATE` 全角度自由旋转投影。
     - 确保广告牌地标、大面积覆盖矩形与恒向线（`RHUMB`）在大幅度姿态旋转时光栅化无撕裂与畸变。

3. **`polylines-on-terrain-dev`（大峡谷崎岖山谷地面折线与地形地物分类基元）**
   - 官方地址：`https://sandcastle.cesium.com/?id=development/Polylines%20On%20Terrain`
   - 变体全集（4 个地形特征变体）：
     - `Grand Canyon Trails and Ground Rectangle`（山谷步道与地表分类面：发卡弯主航迹、东部环线发光轨迹与峡谷底地表矩形）
     - `Main Hairpin Trail Focus`（发卡弯主路径聚焦：8px 亮黄宽折线突出大落差盘旋路段）
     - `Eastern Loop Trail Focus`（东部闭合环线聚焦：8px 翠绿发光宽折线环绕山脊）
     - `Composite Terrain Classification`（复合地表分类与深度排序：深紫高饱和度半透明矩形面体叠加多条穿透山谷折线）
   - 核心要素：
     - 处理极端高差山地（大峡谷落差达 1500m 以上）的地表折线无缝贴合。
     - 验证了 `GroundPrimitive`（贴地矩形）与 `GroundPolylinePrimitive`（贴地折线）在共面与穿插情况下的稳定深度缓冲表现。

4. **`elevation-band-material`（喜马拉雅珠峰多阶海拔高程等高分级材质带）**
   - 官方地址：`https://sandcastle.cesium.com/?id=Elevation%20Band%20Material`
   - 变体全集（4 个海拔梯度变体）：
     - `All Elevation Bands (4200m - 8848m)`（全海拔带谱：4200m 基底带、7000m 中高带、7500m 高海拔带、与 8000m~8848m 珠峰冲顶峰带）
     - `High Altitude Focus (7500m+)`（极高海拔雪线聚焦：7500m 以上金黄与赤红冲顶面体）
     - `Everest Summit Peak Only (8000m - 8848m)`（珠峰极顶单体：纯白雪顶金边立体挤出面）
     - `Gradient Multi-Tiered Visualization`（多级平滑渐变色彩分级：深蓝、青绿、金黄多色谱交织呈现）
   - 核心要素：
     - 标量高程阈值（4200m ~ 8848m）到垂直几何立体柱体（`height` 与 `extrudedHeight`）的精准投影。
     - 大尺度（8000m 级）垂直几何立面光栅化产生 70,000 ~ 102,000 像素的高精度立体着色。

---

## 2. 核心研发突破

1. **`Shape::Rectangle` 与 `Shape::Point` Serde 契约严密规范化**：
   - 统一确立了矩形图元必须采用 `coordinatesDegrees: [west, south, east, north]` 坐标数组规范，杜绝散装字段导致的解析异常；同时严格确立了点图元内置 `outlineWidth` 与 `outlineColor` 属性与端到端反序列化完全同构。
2. **2D 旋转地图（`SCENE2D + ROTATE`）全景渲染管线闭环**：
   - 验证了在二维哥伦布展开与平面坐标系下相机航向偏航（Heading -45° ~ 180°）时，点标、矩形与折线的齐次变换与离屏像素拾取无缝匹配。
3. **超高海拔垂直多阶分层（4200m ~ 8848m）立体光栅化**：
   - 验证了四个梯级高度分层立体面在极限高程差下的无缝拼装与法线朝向，离屏光栅化后各层分别贡献 70,000 ~ 100,000+ 像素的高质量立体视觉表现。

---

## 3. 验收结果矩阵

| 案例 ID | 实体数 | 变体数 | Native GPU 渲染 | 浏览器 WebGPU | 像素拾取 | 交互控制 | 状态 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `drawing-on-terrain` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (47253 px) | 线/面/顶点绘制模式切换 | **Passed** |
| `rotatable-2d-map` | 3 | 4 | **Passed** (90 帧无错误) | **Passed** (3 实体) | **Passed** (63718 px) | 航向角度 -45°~180° 旋转切换 | **Passed** |
| `polylines-on-terrain-dev` | 3 | 4 | **Passed** (90 帧无错误) | **Passed** (3 实体) | **Passed** (71703 px) | 山谷折线/分类面切换 | **Passed** |
| `elevation-band-material` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 阶海拔面) | **Passed** (324018 px) | 全带谱/雪线/极顶海拔切换 | **Passed** |

---

## 4. 验证日志索引

- Native GPU 单体验收日志：
  - `docs/full-drawing-on-terrain-smoke.log`
  - `docs/full-drawing-on-terrain-variant-1-smoke.log` ~ `docs/full-drawing-on-terrain-variant-3-smoke.log`
  - `docs/full-rotatable-2d-map-smoke.log`
  - `docs/full-rotatable-2d-map-variant-1-smoke.log` ~ `docs/full-rotatable-2d-map-variant-3-smoke.log`
  - `docs/full-polylines-on-terrain-dev-smoke.log`
  - `docs/full-polylines-on-terrain-dev-variant-1-smoke.log` ~ `docs/full-polylines-on-terrain-dev-variant-3-smoke.log`
  - `docs/full-elevation-band-material-smoke.log`
  - `docs/full-elevation-band-material-variant-1-smoke.log` ~ `docs/full-elevation-band-material-variant-3-smoke.log`
- Native GPU 整体回归日志：`docs/full-recipes-gpu-validation.log` (136 个测试全部 Passed)
- 浏览器 WebGPU 综合验收日志：`docs/full-recipes-browser.json` (135 个 Sandcastle 案例全部 Passed)
