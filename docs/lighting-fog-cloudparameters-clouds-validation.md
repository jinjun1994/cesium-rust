# Batch 49 (v0.0.49) 案例验证报告：Lighting、Fog、Cloud Parameters 与 Clouds

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖动态多光源系统 (Lighting)、地表气象多高度层雾效 (Fog)、程序化积云微观形态参数化调谐 (Cloud Parameters)、以及宏观山地层云与逆温层天气 (Clouds) 的配方，累计完成 **143 / 309**（剩余 166 个）：

1. **`lighting`（动态多光源照明系统：日光、月光、相机手电筒与定制聚光灯）**
   - 官方地址：`https://sandcastle.cesium.com/?id=lighting`
   - 变体全集（4 个光源照明模式变体）：
     - `Sun Light Mode (Dynamic Solar Position & Shadows)`（太阳自然光模式：动态时钟驱动的太阳天球轨迹，直射 CesiumBalloon 航空气球、基座圆柱体、半透明反射球与参考方块）
     - `Moon Light Mode (Soft Lunar Light, Blue Hue 0.5 Intensity)`（月球冷光模式：通过 ICRF 惯性系至地球固定系转换解算月球方向，产生淡蓝冷色夜间低照度光影）
     - `Flashlight Mode (Camera View-Aligned Headlamp, 3.0 Intensity)`（相机探照灯模式：光束法向动态绑定相机视线方向，3.0 强照度近距高光打亮）
     - `Custom Directional Light (Warm Amber #deca7c Light)`（定制暖琥珀色平行光模式：[0.24, 0.88, 0.40] 方向入射，高饱和度金黄黄昏斜射效果）
   - 核心要素：
     - 支持模型类实体（`Shape::Model`）、圆柱体（`Shape::Cylinder`）、球椭球体（`Shape::Ellipsoid`）与立方体（`Shape::Box`）在多光源模式下的材质光照响应。
     - 验证了时间序列驱动的天体方向（`Simon1994PlanetaryPositions`）在 GPU 光栅化下的稳定着色表现。

2. **`fog`（地表气象多高度层大气雾效与视距衰减）**
   - 官方地址：`https://sandcastle.cesium.com/?id=fog`
   - 变体全集（4 个雾气浓度与视距变体）：
     - `Standard Atmospheric Fog (Density 0.0002, Balanced Falloff)`（标准山地大气雾：密度 0.0002、高度衰减 0.5、阿尔卑斯山脉高程逆温层边界）
     - `High Altitude Alpine Fog (Max Height 4000m, Snow Peak Shroud)`（极高海拔雪峰雾障：最大高度 4000m，覆盖戈特哈德山口高海拔冰川与监测站）
     - `Valley Dense Ground Fog (Height Falloff 0.8, Dense Base)`（山谷逆温浓雾：高度衰减 0.8，浓重谷底低空冷雾堆积与清晰雪山对立）
     - `Clear Sky Minimal Fog (Density 0.00002, Crisp Alpine Vista)`（清朗晴空极低雾：密度 0.00002，高通透度俯瞰瑞士山地全景）
   - 核心要素：
     - 在场景管线中统一支持基于地表相对高度的垂直衰减雾（Height Falloff）与水平视距雾（Distance Fog）。
     - 部署山口哨所、谷底气象站、峰顶透射标杆与大范围逆温矩形边框，验证不同海拔高度下的几何遮蔽与雾浓度渐变。

3. **`cloud-parameters`（程序化积云三维形态参数化调谐系统）**
   - 官方地址：`https://sandcastle.cesium.com/?id=cloud-parameters`
   - 变体全集（4 个云朵形态与光学变体）：
     - `White Cumulus Cloud (Slice 0.36, Brightness 1.0)`（标准白积云：切片 0.36 保留膨胀半球体、X/Y/Z 尺度 25x12x15、亮度 1.0）
     - `Golden Sunset Cloud (Warm Amber Glow, Brightness 1.2)`（金橙日落积云：暖琥珀色散射、高亮度 1.2、多层云朵穹顶暖色溢出）
     - `Storm Grey Nimbus Cloud (Dark Silver, Brightness 0.5)`（风暴深灰雨云：暗银灰色质感、低亮度 0.5、浓厚压迫感阴影基底）
     - `High Altitude Cirrus (Scaled Flat, Slice 0.5, Brightness 1.0)`（高空卷云扁平层：薄片扁平拉伸、切片 0.5、羽翼状透光形体）
   - 核心要素：
     - 支持三维云朵椭球基元（`Shape::Ellipsoid`）的多切片（`slice`）几何剖切与自发光亮度调制。
     - 包含云底翻滚波团、主云核、穹顶云冠与地表投影衰减面，构建微观单云朵高保真着色。

4. **`clouds`（宏观山地层云、低空云海与气象覆盖系统）**
   - 官方地址：`https://sandcastle.cesium.com/?id=clouds`
   - 变体全集（4 个云海天气与海拔变体）：
     - `Cascade Mountain Cloud Layer (5 Cloud Formations, Low Ceiling)`（喀斯喀特山脉低空云层：5 组百米级层云覆盖俄勒冈波特兰群山）
     - `High Altitude Sea of Clouds (Elevated Stratus +200m)`（高海拔翻腾云海：整体抬升 200m，山脊破云而出，纯白高光积聚）
     - `Scattered Fair Weather Cumulus (Low Density, Soft Haze)`（晴好疏散淡积云：低密度软光霭气，点缀山林地表）
     - `Dense Overcast Mountain Weather (High Opacity Nimbus Haze)`（密闭阴霾山地风暴：高不透明度阴云天幕全面笼罩地表）
   - 核心要素：
     - 验证了大尺度（长径达 2000m、短径 300m）三维椭球云团在起伏地形上方的空间排布与视景体剔除。
     - 结合地表气象天幕矩形，形成宏观地形天气与低空云层立体的协同渲染管线。

---

## 2. 核心研发突破

1. **多光源动态着色与天体轨道解算无缝衔接**：
   - 统一了太阳光、月球光、相机探照灯与平行光在 glTF 3D 模型、圆柱体与球体上的漫反射与环境光响应，离屏渲染与 WebGPU 均稳定贡献数百至数千像素。
2. **多海拔层级山地大气雾（Elevation-Aware Fog）**：
   - 验证了场景雾在山口（3000m）、谷底（1500m）与峰顶（4000m）不同地形海拔下的像素级过渡，离屏光栅化贡献 62,000 ~ 92,000 像素的高精度几何着色。
3. **程序化积云微观形态与切片几何（Cloud Slice & Modulation）**：
   - 实现了基于半球切片（slice）的非完整椭球云团模型，验证了多重云核在不同亮度与色温变体下的精确光栅化与鼠标拾取。
4. **大尺度山地气象云层群（Volumetric Mountain Cloud Collection）**：
   - 在 1500m ~ 2000m 尺度下排布多层层云与低空天幕，产生 11,000 ~ 66,000 像素的高密度气象渲染效果。

---

## 3. 验收结果矩阵

| 案例 ID | 实体数 | 变体数 | Native GPU 渲染 | 浏览器 WebGPU | 像素拾取 | 交互控制 | 状态 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `lighting` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (1,696 px) | 4 种光源模式（日光/月光/手电筒/平行光）切换 | **Passed** |
| `fog` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (293,078 px) | 4 种雾浓度/高度衰减/视点切换 | **Passed** |
| `cloud-parameters` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (704 px) | 4 种积云色温/切片/形态切换 | **Passed** |
| `clouds` | 5 | 4 | **Passed** (90 帧无错误) | **Passed** (5 实体) | **Passed** (174,281 px) | 4 种山地云海天气变体切换 | **Passed** |

---

## 4. 验证日志索引

- Native GPU 单体验收日志：
  - `docs/full-lighting-smoke.log`
  - `docs/full-lighting-variant-1-smoke.log` ~ `docs/full-lighting-variant-3-smoke.log`
  - `docs/full-fog-smoke.log`
  - `docs/full-fog-variant-1-smoke.log` ~ `docs/full-fog-variant-3-smoke.log`
  - `docs/full-cloud-parameters-smoke.log`
  - `docs/full-cloud-parameters-variant-1-smoke.log` ~ `docs/full-cloud-parameters-variant-3-smoke.log`
  - `docs/full-clouds-smoke.log`
  - `docs/full-clouds-variant-1-smoke.log` ~ `docs/full-clouds-variant-3-smoke.log`
- Native GPU 整体回归日志：`docs/full-recipes-gpu-validation.log` (144 个测试全部 Passed)
- 浏览器 WebGPU 综合验收日志：`docs/full-recipes-browser.json` (143 个 Sandcastle 案例全部 Passed)
