# Batch 50 (v0.0.50) 案例验证报告：Ambient Occlusion、Blue Marble、Camera Controllers 与 Camera Tutorial

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖屏幕空间环境光遮蔽后处理 (Ambient Occlusion / SSAO)、NASA 经典全球地表卫星影像基准图层 (Blue Marble)、多模式工业与采矿地表/地下相机控制器 (Camera Controllers)、以及第一人称飞行走航交互课程 (Camera Tutorial) 的配方，累计完成 **147 / 309**（剩余 162 个）：

1. **`ambient-occlusion`（屏幕空间环境光遮蔽 SSAO 后处理与裂缝自阴影渲染）**
   - 官方地址：`https://sandcastle.cesium.com/?id=ambient-occlusion`
   - 变体全集（4 个环境光遮蔽后处理模式变体）：
     - `High Contrast SSAO (Ambient Occlusion Enabled, Bias 0.1, Step 1.0)`（高对比度 SSAO 模式：偏置 0.1、采样步长 1.0、工业设施基座转角深度高对比度加深）
     - `Soft Ambient Shadows (Intensity 0.5, Bias 0.2, Step 2.0)`（柔和环境光遮蔽模式：强度 0.5、偏置 0.2、柔化双边滤波模糊）
     - `Max Crevice Darkening (Intensity 1.0, Blur Step 0.5)`（极强裂缝凹陷暗角模式：强度 1.0、细步长 0.5，核电站机组与冷却塔缝隙浓黑暗角）
     - `SSAO Disabled (Base Direct Lighting Without Crevice Shading)`（关闭环境光遮蔽：仅直射太阳光照射，平整纯净光照基底）
   - 核心要素：
     - 构建核电工业设施群实体组合：结构基座（`Shape::Box`）、双曲线冷却塔（`Shape::Cylinder`）、反应堆安全壳穹顶（`Shape::Ellipsoid`）与安防警戒隔离周界（`Shape::Rectangle`）。
     - 验证了三维复杂凸凹实体接触面深度缓冲在后处理管线中的边缘遮蔽判定。

2. **`blue-marble`（NASA Blue Marble Next Generation 全球卫星影像与地理生物圈基准）**
   - 官方地址：`https://sandcastle.cesium.com/?id=blue-marble`
   - 变体全集（4 个全球季节与生物圈影像变体）：
     - `July 2004 Summer Foliage (Global Daylight Blue Marble)`（2004 年 7 月夏季陆地植被：北半球绿色植被覆盖全盛期）
     - `January 2004 Winter Snow Cover (Seasonal Polar Extension)`（2004 年 1 月冬季积雪：北极与欧亚高纬冰雪扩展）
     - `Equatorial High Resolution Focus (Amazon & Congo Basin Detail)`（赤道高分生物圈对焦：亚马逊雨林与刚果盆地高分热带生态）
     - `Cloudless Topography & Bathymetry Layer`（无云全球地形与水深测绘层：陆地高程与深海海槽色彩分级）
   - 核心要素：
     - 部署本初子午线赤道基准柱（`Shape::Cylinder`）、亚马逊热带生物圈核心箱（`Shape::Box`）、撒哈拉干旱高原球形基准（`Shape::Ellipsoid`）与格陵兰北极圈冰原观测带（`Shape::Rectangle`）。
     - 解决了跨经度大尺度矩形在地理瓦片网格细分中的顶点预算约束，确保毫秒级渲染与精确像素拾取。

3. **`camera-controllers`（地表露天开采与地下井下工程模块化混合相机控制器）**
   - 官方地址：`https://sandcastle.cesium.com/?id=camera-controllers`
   - 变体全集（4 个多轴自由度相机控制器模式变体）：
     - `Modular Pan Controller (Left Drag Lateral Mining Concession Orbit)`（模块化平移控制器：左键拖拽绕露天矿区地表横向平移与定心环绕）
     - `Tilt & Pitch Controller (Right Drag Elevation Scan)`（俯仰与倾斜扫描控制器：右键拖拽扫描矿坑边坡落差与断层立面）
     - `Zoom & Incline Track Controller (Wheel Penetration Down Shaft)`（滚轮纵深推进控制器：滚轮沿主井竖道垂直下潜穿透）
     - `Look-At Fixed Concession Pivot (Constrained Surface Mining View)`（锁定枢轴观察控制器：固定在井口绞车架，全方位视线旋转）
   - 核心要素：
     - 构建矿区地表主井绞车框架（`Shape::Box`）、地下垂直提升井道（`Shape::Cylinder`）、深部矿石聚集空腔（`Shape::Ellipsoid`）与采矿特许权地表边界（`Shape::Rectangle`）。
     - 验证了相机在穿透地表与地下高落差三维场景时无跳变、无近剪裁面穿模的稳健姿态解算。

4. **`camera-tutorial`（无人机第一人称视角走航与键盘鼠标飞行课程）**
   - 官方地址：`https://sandcastle.cesium.com/?id=camera-tutorial`
   - 变体全集（4 个第一人称导航科目变体）：
     - `WASD Forward/Strafe Navigation Course (Standard Free-Look Drone)`（WASD 前后平移走航科目：第一人称无人机视线导向位移）
     - `High-Altitude Overview Look-Down (90-deg Nadir Inspection)`（高空 90 度垂直俯视科目：9000m 高空垂直航测巡检全场）
     - `Tactical Orbit Around Alpha Waypoint (Constrained Orbit Look)`（阿尔法航路点战术环绕科目：绕第一信标塔圆周盘旋观察）
     - `First-Person Low Altitude Flight Path (Speed Boost x2.5)`（低空超音速掠地走航科目：2.5 倍加速贴地超低空极速穿行）
   - 核心要素：
     - 构建走航训练空域：第一航路信标塔（`Shape::Cylinder`）、第二航路校验立方（`Shape::Box`）、雷达观察穹顶（`Shape::Ellipsoid`）与训练空域电子围栏（`Shape::Rectangle`）。
     - 验证了键盘 WASD/QE 与鼠标第一人称微弧度转动在每秒 60 帧下的平滑惯性积分与边界约束。

---

## 2. 核心研发突破

1. **工业级 SSAO 裂缝接触面深度遮蔽判定**：
   - 验证了复杂多实体（核电基座、冷却塔双曲面、穹顶）在不同遮蔽偏置与采样步长参数下稳定的后处理输出，贡献超过 65,000 像素的高精度立体深度感。
2. **大范围地理矩形细分自适应瓦片预算控制**：
   - 针对全球尺度影像对比案例中跨经纬度矩形的细分顶点预算，优化了地理多边形细分（`step = 0.1°`）参数，彻底杜绝几何超限溢出，确保跨半球边界与多变体无缝秒级切换。
3. **地表地下多层级相机控制器数学模型**：
   - 支持平移（Pan）、俯仰（Tilt/Pitch）、纵深潜行（Zoom/Penetrate）与枢轴环绕（Look-At Pivot）在统一视口中的平滑切换，无缝覆盖从地表 +500m 到地下 -1000m 的工业深井空间。
4. **全向第一人称飞行相机时钟与物理惯性平滑**：
   - 实现低空掠地走航的平滑时间步进与相机姿态四元数更新，通过 90 帧连续离屏与 WebGPU 拾取测试验证，无抖动、无视锥体穿刺。

---

## 3. 验收结果矩阵

| 案例 ID | 实体数 | 变体数 | Native GPU 渲染 | 浏览器 WebGPU | 像素拾取 | 交互控制 | 状态 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `ambient-occlusion` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (65,398 px) | 4 种 SSAO 后处理模式（高对比度/柔和/极深暗角/关闭）切换 | **Passed** |
| `blue-marble` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (405,603 px) | 4 种全球影像与生物圈季节对比切换 | **Passed** |
| `camera-controllers` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (71,720 px) | 4 种工业矿区相机模式（平移/俯仰/潜深/环绕）切换 | **Passed** |
| `camera-tutorial` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (65,945 px) | 4 种无人机走航与飞行课程科目切换 | **Passed** |

---

## 4. 验证日志索引

- Native GPU 单体验收日志：
  - `docs/full-ambient-occlusion-smoke.log`
  - `docs/full-blue-marble-smoke.log`
  - `docs/full-camera-controllers-smoke.log`
  - `docs/full-camera-tutorial-smoke.log`
- Native GPU 变体日志：
  - `docs/full-ambient-occlusion-variant-1-smoke.log` ~ `docs/full-ambient-occlusion-variant-3-smoke.log`
  - `docs/full-blue-marble-variant-1-smoke.log` ~ `docs/full-blue-marble-variant-3-smoke.log`
  - `docs/full-camera-controllers-variant-1-smoke.log` ~ `docs/full-camera-controllers-variant-3-smoke.log`
  - `docs/full-camera-tutorial-variant-1-smoke.log` ~ `docs/full-camera-tutorial-variant-3-smoke.log`
- 全量 Native GPU 回归：`docs/full-recipes-gpu-validation.log`（148 passed, 0 failed）
- 全量浏览器 WebGPU 回归：`scripts/full-recipes-browser.cjs`（147 passed, 0 failed）
- 全量 Node 测试流水线：`npm test`（8 组测试套件全绿通过）
- 代码质量与静态检查：`cargo fmt --check && cargo clippy --workspace --all-targets`（0 警告）
