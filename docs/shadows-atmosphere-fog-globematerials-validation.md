# Batch 48 (v0.0.48) 案例验证报告：Shadows、Atmosphere、Fog Post Process 与 Globe Materials

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖真实地形与多载体动态阴影投射 (Shadows)、地球高动态范围地面与天空大气散射系统 (Atmosphere)、基于深度缓冲的距离雾效后期处理管线 (Fog Post Process)、以及地形高程等高线/坡度/坡向复合材质库 (Globe Materials) 的配方，累计完成 **139 / 309**（剩余 170 个）：

1. **`shadows`（地表、建筑、3D 模型与几何体动态阴影渲染）**
   - 官方地址：`https://sandcastle.cesium.com/?id=shadows`
   - 变体全集（4 个全球地形与轨道光照变体）：
     - `Exton Location (Ground & Terrain Shadows)`（基准陆地视态：宾夕法尼亚 Exton 基地部署 Cesium Air 航空器、地面特种车辆、Cesium Man 人物、木质了望塔、暗影测试建筑群、红色阴影立方体、棋盘格阴影体与半透明投影球体 8 具全要素实体，启用地形与阴影贴图最大 10km 覆盖）
     - `Half Dome Location (High Altitude Terrain)`（高海拔峡谷地形视态：整体机动至加州半圆顶花岗岩绝壁，验证极端起伏地形对航空器与建筑投射阴影的无缝吸附）
     - `Mount Everest Location (Extreme Mountain Ridge)`（极高峰峦山脊视态：整体机动至珠峰 8848m 极高海拔山脊，验证高落差陡峭雪峰阴影遮蔽与交错）
     - `Space Orbital Location (2000km Low Earth Orbit)`（2000km 近地轨道视态：整体机动至外太空近地轨道，验证大尺度空间日光照明角与天体遮挡）
   - 核心要素：
     - 支持模型类实体（`Shape::Model`）、立方体（`Shape::Box`）与球椭球体（`Shape::Ellipsoid`）的高保真阴影模式（`shadows: Cesium.ShadowMode.ENABLED`）。
     - 验证了多类型形体共处同一光照系统下的深度图投射与阴影接收。

2. **`atmosphere`（地球大气瑞利散射、米氏气溶胶与 HDR 光照）**
   - 官方地址：`https://sandcastle.cesium.com/?id=atmosphere`
   - 变体全集（4 个大气成分与光学散射变体）：
     - `Earth Standard Atmosphere (Clear Sky Rayleigh & Mie)`（地球标准纯净晴空：标准瑞利散射系数 [5.5e-6, 13.0e-6, 28.4e-6]、米氏气溶胶系数 [2.0e-5, 2.0e-5, 2.0e-5]、尺度高 8000m、大气光强 20.0）
     - `Golden Hour Sunset (Enhanced Red/Amber Rayleigh Shift)`（金色黄昏与日落红移：高瑞利红光长波散射、金橙色地平线地表与天空红移）
     - `Mars Thin Dust Atmosphere (High Aerosol Haze)`（火星稀薄扬尘与高气溶胶：米氏各向异性 0.9、黄褐/红棕色天空与地表尘暴散射）
     - `High Dynamic Range (HDR Atmosphere 50.0 Intensity)`（高动态范围 HDR 大气：HDR 曝光启用、高光强 50.0、平流层臭氧深紫吸收带）
   - 核心要素：
     - 在场景管线中统一支持物理级瑞利（Rayleigh）蓝光波长散射与米氏（Mie）白光前向散射参数化调谐。
     - 部署赤道、北极、黄昏交界面与平流层 4 具实体传感器，构建大气参数监测与光学标定阵列。

3. **`fog-post-process`（基于深度缓冲的距离雾效后期处理阶段）**
   - 官方地址：`https://sandcastle.cesium.com/?id=fog-post-process`
   - 变体全集（4 个能见度与气象距离雾变体）：
     - `Dense Near Fog (10m - 200m, Black)`（极低能见度暗黑浓雾：near=10m, far=200m，近距黑色深度衰减与高对比轮廓）
     - `Morning Mountain Mist (30m - 500m, Pale Gray)`（晨间山间薄雾：near=30m, far=500m，淡灰白高透光散射雾）
     - `Hazardous Industrial Smog (15m - 300m, Amber Yellow)`（工业有害浓烟与黄雾：near=15m, far=300m，琥珀黄有色烟雾遮蔽）
     - `Night Coastal Fog (20m - 250m, Deep Navy)`（夜间深海浓雾：near=20m, far=250m，午夜深蓝低光雾）
   - 核心要素：
     - 采用眼睛坐标系距离解算（`czm_unpackDepth` + `czm_windowToEyeCoordinates`），并结合四维标量 `fogByDistance`（near, nearAlpha, far, farAlpha）实现平滑线性插值与 Alpha 混合。
     - 阶梯式分布近距 10m、中距 60m、远距 140m 与极限 200m 探针，严格校验深度雾效随距离阶跃渐变的像素着色。

4. **`globe-materials`（地表高程等高线、坡度与坡向综合材质库）**
   - 官方地址：`https://sandcastle.cesium.com/?id=globe-materials`
   - 变体全集（4 个地形分析材质变体）：
     - `Elevation Ramp & Contours (150m Spacing, 2px Width)`（高程渐变色谱与等高线：150m 等高距、2px 红色等高线、复合 ElevationColorContour 材质）
     - `Slope Angle Shading (Steep Terrain Visualization)`（地形坡度着色：基于地表法线计算坡角，高亮陡峭半圆顶崖壁与险峰）
     - `Aspect Orientation Shading (Compass Direction Ramp)`（山体坡向着色：罗盘 360° 方位角映射彩虹色谱，分析向阳背阴面）
     - `High-Density Elevation Contours (50m Spacing, Red Contours)`（高密精细等高线：50m 密集等高线覆盖喜马拉雅峰群）
   - 核心要素：
     - 实现复合地表材质结构（Composite Material Fabric），支持 diffuse 与 alpha 在 ramp 材质与 contour 材质之间的动态分支。
     - 部署喜马拉雅极峰、半圆顶陡崖、温哥华基准面与大范围等高线带谱实体，校验地形分析着色全流程。

---

## 2. 核心研发突破

1. **多模型、几何体与阴影模式协同渲染**：
   - 统一了 glTF 模型与几何体（Box、Ellipsoid）在阴影投射和接收下的着色表现，离屏光栅化验证像素级变化与拾取一致性。
2. **多波长物理大气与高动态范围（HDR）管线闭环**：
   - 确立了大气层散射参数与三维传感器探测阵列的空间对齐，离屏渲染产生 69,000 ~ 164,000 像素的高饱和度光学效果。
3. **深度缓冲后期处理距离雾效（Depth-based Fog Post-Process）**：
   - 基于深度纹理重构视点空间距离，通过 10m ~ 200m 阶梯探针精准验证了线性雾效插值。
4. **地形分析与复合地表材质库（Composite Globe Materials）**：
   - 规范了高程、坡度、坡向等高线复合材质在极端地形落差下的着色表现，各区域实体像素贡献达 41,000 ~ 90,000+ 像素。

---

## 3. 验收结果矩阵

| 案例 ID | 实体数 | 变体数 | Native GPU 渲染 | 浏览器 WebGPU | 像素拾取 | 交互控制 | 状态 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `shadows` | 8 | 4 | **Passed** (90 帧无错误) | **Passed** (8 实体) | **Passed** (300+ px) | 4 个全球地形与轨道机动变体 | **Passed** |
| `atmosphere` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (431,144 px) | 4 种大气散射成分与光照变体 | **Passed** |
| `fog-post-process` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (370 px) | 4 种距离雾能见度变体 | **Passed** |
| `globe-materials` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (242,386 px) | 4 种高程/坡度/坡向材质变体 | **Passed** |

---

## 4. 验证日志索引

- Native GPU 单体验收日志：
  - `docs/full-shadows-smoke.log`
  - `docs/full-shadows-variant-1-smoke.log` ~ `docs/full-shadows-variant-3-smoke.log`
  - `docs/full-atmosphere-smoke.log`
  - `docs/full-atmosphere-variant-1-smoke.log` ~ `docs/full-atmosphere-variant-3-smoke.log`
  - `docs/full-fog-post-process-smoke.log`
  - `docs/full-fog-post-process-variant-1-smoke.log` ~ `docs/full-fog-post-process-variant-3-smoke.log`
  - `docs/full-globe-materials-smoke.log`
  - `docs/full-globe-materials-variant-1-smoke.log` ~ `docs/full-globe-materials-variant-3-smoke.log`
- Native GPU 整体回归日志：`docs/full-recipes-gpu-validation.log` (140 个测试全部 Passed)
- 浏览器 WebGPU 综合验收日志：`docs/full-recipes-browser.json` (139 个 Sandcastle 案例全部 Passed)
