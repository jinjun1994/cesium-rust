# Batch 38 (v0.0.38) 案例验证报告：3D Models Articulations Dev、3D Models Node Explorer Dev、HeadingPitchRoll 与 Entity Tracking

## 1. 案例概要

本批次推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个核心三维模型控制、骨骼节点变换与相机跟踪定位配方，累计完成 **99 / 309**（剩余 210 个）：

1. **`3d-models-articulations-dev`（底层 Model.fromGltfAsync 运载火箭铰接动画阶段控制）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-models-articulations-dev`
   - 变体全集（4 个铰接阶段控制变体）：
     - `Fairing Open: 0°`（整流罩闭合初始态，Move 0.0）
     - `Fairing Open: 60°`（整流罩展开半开态，Move 60.0）
     - `Fairing Separate: -25`（整流罩展开并平移分离，Open 120.0，Separate -25.0）
     - `Fairing Drop: -50`（整流罩下落，Open 120.0，Separate -50.0，Drop -50.0）
   - 运载火箭模型（`launchvehicle.glb`）位于 220km 高空（`[-74.693, 28.243, 220000.0]`），基于 `headingPitchRollToFixedFrame(origin, hpr)` 局部矩阵与 `minimumPixelSize: 128` 渲染，由 `camera.lookAt` 锁定矩阵视角。

2. **`3d-models-node-explorer-dev`（底层 Model.fromGltfAsync 蒙皮角色节点骨骼探索器）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-models-node-explorer-dev`
   - 变体全集（4 个骨骼节点变换变体）：
     - `Default Pose`（默认站立骨骼姿态）
     - `Left Arm Rotated (Roll 45°)`（左臂关节 `Skeleton_arm_joint_L__3_` 绕局部轴翻滚 45°）
     - `Right Arm Rotated (Pitch 45°)`（右臂关节 `Skeleton_arm_joint_R__2_` 绕局部轴俯仰 45°）
     - `Hide Man (show: false)`（节点/模型显隐切换）
   - CesiumMan 模型位于 250km 高空（`[-123.0744619, 44.0503706, 250000.0]`），由 `camera.lookAt` 以 heading 230°、pitch -20° 定位定格。

3. **`headingpitchroll`（飞机姿态欧拉角交互、西北天 NWU 局部坐标系与 PolylineGlow 飞行轨迹）**
   - 官方地址：`https://sandcastle.cesium.com/?id=headingpitchroll`
   - 变体全集（5 个姿态键盘交互变体）：
     - `Default (Heading 0°, Pitch 0°, Roll 0°, Speed 10)`（平飞基准态）
     - `Pitch Down (ArrowDown, Pitch -3°)`（俯冲姿态）
     - `Pitch Up (ArrowUp, Pitch +3°)`（爬升姿态）
     - `Turn Right (ArrowRight, Heading +3°)`（右转偏航姿态）
     - `Turn Left (ArrowLeft, Heading -3°)`（左转偏航姿态）
   - 核心要素：
     - 飞机模型（`Cesium_Air.glb`，高度 5000m），基于 `localFrameToFixedFrameGenerator("north", "west")` 西北天（North-West-Up, NWU）局部坐标系正交基矩阵到 ECEF 四元数解算。
     - 轨迹实体（`path`）：基于 `SampledPositionProperty` 采样轨迹，绑定 `PolylineGlow` 发光材质（`glowPower: 0.3`、`taperPower: 0.3`、淡金色 `Color.PALEGOLDENROD`，线宽 10px）。
     - 相机 lookAt 绑定 `fromBehind` 追尾跟随视角。

4. **`entity-tracking`（CZML 卫星与无人机跨实体相机追踪与多种跟踪参考系）**
   - 官方地址：`https://sandcastle.cesium.com/?id=entity-tracking`
   - 变体全集（5 个实体与参考系变体）：
     - `Satellites (ISS Tracked)`（锁定跟踪国际空间站卫星，视距偏移 `viewFrom: [-300, 20, 100]`，时钟 30x 循环）
     - `Drone (Drone Tracked)`（锁定跟踪无人机模型，视距偏移 `viewFrom: [-50, 0, 5]`，时钟 1x 循环）
     - `Tracking reference frame: Inertial`（惯性坐标系跟踪，`trackingReferenceFrame: "INERTIAL"`）
     - `Tracking reference frame: Velocity`（速度矢量标架跟踪，`trackingReferenceFrame: "VELOCITY"`）
     - `Tracking reference frame: East-North-Up`（东北天局部切平面跟踪，`trackingReferenceFrame: "ENU"`）
   - 数据源包含 `Satellite/ISS`（公告牌图元、文本标签、路径折线与 INERTIAL 拉格朗日 5 阶多项式插值坐标序列）与 `CesiumDrone`（无人机 glTF 模型与线性插值飞行路径）。

---

## 2. 核心研发突破

1. **CZML 复合数据包解析增强（`crates/cesium-viewer/src/entities.rs`）**：
   - 突破了以往仅支持单图元或特定两图元组合的限制，全面支持复杂 CZML 包：
     - `model` + `path` 双图元组合（如无人机实体同时包含 glTF 3D 模型与其动态发光拖尾）。
     - `billboard` + `label` + `path` 三图元深度复合（如高空卫星实体同时包含图表点阵、文字标注及轨道历史线）。
   - 扩展支持了 `"trackingReferenceFrame"` 与 `"parent"` 属性，规范了 `AUTODETECT`、`INERTIAL`、`VELOCITY` 与 `ENU` 跟踪参考系语义。
2. **笛卡尔 ECEF 轨迹在 Path 图元中的自适应重投影（`crates/cesium-viewer/src/wide_polyline.rs`）**：
   - 扩展 `path_czml`，同时支持 `cartographicDegrees` 经纬度和 `cartesian` 笛卡尔坐标格式的飞行采样点序列。
   - 对笛卡尔坐标 chunk，利用 `Cartographic::from_ecef` 精准转换出对应大地坐标，无缝喂入测地线宽折线（`GEODESIC`）几何细分管线，彻底打通航天卫星惯性轨道在屏幕及地球表面的投射绘制。
   - 兼容了 CZML 时间区间数组形式的 `width` 与 `color` 动态属性。
3. **高阶拉格朗日与惯性参考系插值容差放行（`crates/cesium-viewer/src/entities.rs`）**：
   - 针对航天与复杂动力学案例，放行 `referenceFrame: "INERTIAL"` 以及 `interpolationAlgorithm: "LAGRANGE"`（多项式阶数 1..=10），确保轨道计算模块在加载官方复杂 CZML 时零拒绝、零崩溃。
4. **西北天（NWU）正交标架到全球 ECEF 姿态四元数映射（`scripts/batch38-recipes.mjs`）**：
   - 构建了从 North-West-Up（X=北、Y=西、Z=天）局部切空间到 ECEF 全球空间四元数的精确算法，与 Cesium 原生 `localFrameToFixedFrameGenerator("north", "west")` 的欧拉角偏航/俯仰/翻滚顺序完全对齐。

---

## 3. 验证矩阵与证据链

| 验收维度 | 目标要求 | 实测结果 | 状态 |
| :--- | :--- | :--- | :--- |
| **Rust 单元测试** | `cargo test -p cesium-viewer --lib` | 98 项单元测试全部通过（新增 `batch38_models_articulations_node_explorer_headingpitchroll_tracking`） | **PASSED** |
| **原生离屏 GPU 测试** | `cargo test -p cesium-viewer --test full-recipes-gpu -- --ignored` | **100 / 100** 项通过（包含 99 个完整案例 + 1 个材质拾取测试），耗时 23.73s | **PASSED** |
| **Node 测试套件** | `npm test`（8 组测试） | 全部 8 组测试零报错通过（包含 99 个案例哈希校验与数据契约） | **PASSED** |
| **桌面 90 帧冒烟测试** | 99 案例 + 68 变体 | **167 / 167** 个配置 errors 0 稳定运行 | **PASSED** |
| **浏览器 WebGPU 端到端** | Chromium 硬件 WebGPU 运行全部案例 | **99 / 99** 项通过，销毁测试 `destroy: passed` | **PASSED** |
| **代码规范与质量** | `cargo fmt --check` & `cargo clippy --workspace --all-targets` | 零警告、零错误 | **PASSED** |

---

## 4. 产出产物清单

- **代码与脚本**：
  - `crates/cesium-viewer/src/entities.rs`
  - `crates/cesium-viewer/src/wide_polyline.rs`
  - `crates/cesium-viewer/tests/full-recipes-gpu.rs`
  - `scripts/batch38-recipes.mjs`
  - `scripts/batch38-buttons.json`
  - `scripts/build-full-recipes.mjs`
  - `scripts/full-recipes-test.mjs`
  - `scripts/extended-recipes-browser.cjs`
  - `bindings/web/sandcastle.html`
  - `bindings/web/sandcastle.js`
- **生成案例契约与离屏预览**：
  - `tests/sandcastle/full/3d-models-articulations-dev.json` & `.variants.json`
  - `tests/sandcastle/full/3d-models-node-explorer-dev.json` & `.variants.json`
  - `tests/sandcastle/full/headingpitchroll.json` & `.variants.json`
  - `tests/sandcastle/full/entity-tracking.json` & `.variants.json`
  - `docs/full-3d-models-articulations-dev-preview.png` & `docs/full-3d-models-articulations-dev-smoke.log`（及 `variant-1` ~ `variant-3` 冒烟日志）
  - `docs/full-3d-models-node-explorer-dev-preview.png` & `docs/full-3d-models-node-explorer-dev-smoke.log`（及 `variant-1` ~ `variant-3` 冒烟日志）
  - `docs/full-headingpitchroll-preview.png` & `docs/full-headingpitchroll-smoke.log`（及 `variant-1` ~ `variant-4` 冒烟日志）
  - `docs/full-entity-tracking-preview.png` & `docs/full-entity-tracking-smoke.log`（及 `variant-1` ~ `variant-4` 冒烟日志）
