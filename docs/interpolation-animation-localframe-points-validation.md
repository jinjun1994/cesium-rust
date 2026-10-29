# Batch 40 (v0.0.40) 案例验证报告：Interpolation、Manually Controlled Animation、LocalToFixedFrame 与 PointPrimitives Dev

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个核心采样插值轨迹、模型动画手动控制与速度标签、局部切平面标架变换生成器矩阵体系、以及底层点图元集合视距缩放淡出与局部坐标系分布配方，累计完成 **107 / 309**（剩余 202 个）：

1. **`interpolation`（SampledPositionProperty 环形采样飞行轨迹、航向姿态解算与多项式插值模式）**
   - 官方地址：`https://sandcastle.cesium.com/?id=interpolation`
   - 变体全集（4 个多项式插值与参考系变体）：
     - `Interpolation: Linear Approximation (Degree 1)`（线性插值，一阶近似）
     - `Interpolation: Lagrange Polynomial (Degree 5)`（拉格朗日多项式插值，五阶多项式拟合环形飞行）
     - `Interpolation: Hermite Polynomial (Degree 3)`（埃尔米特多项式插值，三阶平滑三次样条）
     - `Tracking reference frame: Inertial`（惯性坐标系相机跟踪）
   - 核心要素：
     - 包含 9 个经纬度高度航路点（`waypoint-0` ~ `waypoint-8`，以 45° 间隔环形分布），黄色外描边空心标示。
     - `Cesium_Air.glb` 飞机模型，基于 `VelocityOrientationProperty` 根据采样位置导数实时解算朝向四元数，`minimumPixelSize: 64`。
     - `flight-path`：测地线发光轨迹（`PolylineGlow`，`glowPower: 0.1`、`taperPower: 0.1`、黄色高亮发光）。
     - 仿真时钟：2015-03-25 16:00 UTC 起始，360 秒周期，10x 速率，`ClockRange.LOOP_STOP`。

2. **`manually-controlled-animation`（CesiumMan 非线性变速跑动、实时速度矢量标签与相机跟随）**
   - 官方地址：`https://sandcastle.cesium.com/?id=manually-controlled-animation`
   - 变体全集（4 个运动速率与相机观察视角变体）：
     - `Default (Accelerated Run)`（非线性加速跑动，速度随时间二次多项式增长）
     - `Constant Speed (Linear Lerp)`（等速巡航模式）
     - `Half Speed`（0.5x 减速模式）
     - `Top-down Track (viewFrom [0, 0, 50])`（顶部俯视跟踪模式）
   - 核心要素：
     - `cesium-man`：带非线性采样位置时序序列的奔跑角色模型。
     - `speed-label`：实时动态解算位置一阶差分速度，以 km/hr 显示速度文本。
     - 相机以 `viewFrom: [-30.0, -10.0, 10.0]` 锁定跟随。

3. **`localtofixedframe`（局部切平面标架生成器、10 种正交轴系变换矩阵与姿态欧拉角控制）**
   - 官方地址：`https://sandcastle.cesium.com/?id=localtofixedframe`
   - 变体全集（4 个局部标架与 HeadingPitchRoll 键盘交互变体）：
     - `Default (Heading 0°, Pitch 0°, Roll 0°)`（10 种局部坐标系基准态）
     - `Pitch Down (ArrowDown, Pitch -15°)`（俯冲姿态调整）
     - `Pitch Up (ArrowUp, Pitch +15°)`（仰角姿态调整）
     - `Roll Right (Shift+ArrowRight, Roll +30°)`（横滚姿态调整）
   - 核心要素：
     - 涵盖 10 个 `Cesium_Air.glb` 飞机模型与 10 个对应的排版文本标签（`plane-0` ~ `plane-9`，`label-0` ~ `label-9`，共 20 个实例）。
     - 10 种典型局部坐标系生成器完整覆盖：
       - `NED`（North East Down）
       - `NEU`（North East Up）
       - `NWU`（North West Up）
       - `NWD`（North West Down）
       - `ENU`（East North Up）
       - `END`（East North Down）
       - `SWU`（South West Up）
       - `SUW`（South Up West）
       - `UEN`（Up East North）
       - `DES`（Down East South）
     - 通过 `headingPitchRollToFixedFrame(pos, hpr, ellipsoid, converter)` 矩阵合成，验证局部坐标系任意轴向排列到地球 ECEF 固定坐标系的齐次变换。

4. **`pointprimitives-dev`（底层 PointPrimitiveCollection 视距动态缩放与局部参考系点云簇）**
   - 官方地址：`https://sandcastle.cesium.com/?id=pointprimitives-dev`
   - 变体全集（8 个工具栏菜单变体）：
     - `Add point`（添加单个黄色点图元）
     - `Set point properties at creation`（创建时配置颜色、尺寸与描边：SkyBlue 10px，黄色 3px 描边）
     - `Change point properties`（动态修改点图元参数：白色 20px 半透明）
     - `Add multiple points`（添加红/蓝/青绿多色点阵）
     - `Scale by viewer distance`（近远视距缩放曲线：`NearFarScalar(150, 2.0, 15000000, 0.5)`）
     - `Fade by viewer distance`（近远视距透明度曲线：`NearFarScalar(150, 1.0, 15000000, 0.2)`）
     - `Add points in reference frame`（局部切平面参考系：原点中心点与沿东、北、天三个方向各偏移 1000km 的定标基准点）
     - `Add point cluster`（添加密集点云簇群）
   - 核心要素：
     - 验证 `ScreenOptions` 中 `scaleByDistance` 与 `translucencyByDistance` 视距衰减曲线对点图元（Point）的着色器绑定与渲染。
     - 规范了点图元在亚像素/微像素情况下的几何栅格化稳定性。

---

## 2. 核心研发突破

1. **点图元亚像素栅格化与尺寸规范（`tests/sandcastle/full/pointprimitives-dev.json` & `variants.json`）**：
   - 针对点图元（`Shape::Point`）内切圆反走样着色器算法（`mesh.wgsl` extras 5u），确保在屏幕取景测试与拾取判定中拥有足够几何面积和稳定像素变化（`pixelSize: 10`），杜绝极端 1px 细微点在离散屏幕像素栅格中丢失的问题。
2. **多局部坐标系标架矩阵与欧拉角变换（`crates/cesium-viewer/tests/full-recipes-gpu.rs`）**：
   - 全面打通 10 种局部切平面标架到 ECEF 固定坐标系模型的齐次变换矩阵验证，确保 `modelMatrix` 列主序 4x4 变换下法线、切线、位置与姿态的绝对一致。
3. **时钟时序与非线性运动采样（`scripts/extended-recipes-browser.cjs`）**：
   - 在前端 WebGPU 自动化回归链路中，为 `interpolation` 与 `manually-controlled-animation` 配置了时间时钟自适应评估和快进/跳转时间戳匹配，确保带时钟区间实体在变焦取景与逐实体测试中全部命中。

---

## 3. 验收全景与指标汇总

- **普通 Rust 测试**：103 项通过（`cesium_viewer` 103 项单元测试全部通过）。
- **实际 GPU 测试**：108 项完整案例 GPU 渲染测试全数通过（`108 passed; 0 failed`）。
- **Node.js 测试套件**：8 组测试全绿（107 个配方校验通过）。
- **浏览器 WebGPU 自动化**：107 个配方在 Chromium WebGPU 环境下 100% 验收通过（无一失败）。
- **代码规范**：`cargo clippy --workspace --all-targets` 零警告，`cargo fmt --check` 格式一致。
