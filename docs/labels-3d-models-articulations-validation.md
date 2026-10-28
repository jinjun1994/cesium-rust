# Batch 36 (v0.0.36) 案例验证报告：Labels、3D Models、CZML 关节动画与节点变换

## 1. 案例概要

本批次推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个核心实体与数据驱动配方，累计完成 **91 / 309**（剩余 218 个）：

1. **`labels`（Labels 样式变体画廊）**
   - 官方地址：`https://sandcastle.cesium.com/?id=labels`
   - 变体全集（8 个工具栏菜单变体）：
     - `Add label`：基础文本标签渲染与默认样式
     - `Set font`：自定义 24px Helvetica 字体、SKYBLUE 填充色、黑色 2px 描边与 `FILL_AND_OUTLINE` 复合渲染
     - `Set properties`：300km 高空标签、`scale: 2.0`、背景框激活（`showBackground: true`）
     - `Offset label by distance`：16x16 设施图标与顶部标签复合，包含 `pixelOffset: [0, -16]`、`scaleByDistance` 与 `pixelOffsetScaleByDistance` 视距缩放与相对偏移动态缩放
     - `Fade label by distance`：纽约与亚特兰大双城视距透明度渐隐（`translucencyByDistance`）
     - `Scale label by distance`：费城视距缩放曲线（`scaleByDistance: [150, 2.0, 1.5e7, 0.5]`）
     - `Set label with right-to-left language`：希伯来语 / 英语双向多语言文本渲染（`Master (אדון): Hello\nתלמיד (student): שלום`）
     - `Animate Label`：外描边宽度与字体尺寸随时间动态插值动画
2. **`3d-models`（glTF 3D 模型集合画廊）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-models`
   - 变体全集（10 个工具栏菜单模型变体）：
     - Aircraft（`Cesium_Air.glb`，高度 5000m）
     - Drone（`CesiumDrone.glb`，高度 150m）
     - Ground Vehicle（`GroundVehicle.glb`，高度 0m）
     - Hot Air Balloon（`CesiumBalloon.glb`，高度 1000m）
     - Milk Truck（`CesiumMilkTruck.glb`，高度 0m）
     - Skinned Character（`Cesium_Man.glb`，高度 0m）
     - Unlit Box（`BoxUnlit.gltf`，高度 10m）
     - Draco Compressed Model（`CesiumMilkTruck.gltf`，高度 0m）
     - KTX2 Compressed Balloon（`CesiumBalloonKTX2.glb`，高度 1000m）
     - Instanced Box（`BoxInstanced.gltf`，高度 15m）
   - 基于 `headingPitchRollQuaternion`（heading 135°，pitch 0°，roll 0°）生成单位四元数朝向，绑定 `minimumPixelSize: 128` 与 `maximumScale: 20000`，由 `viewer.trackedEntity` 驱动相机跟踪取景。
3. **`czml-model-node-transformations`（CZML 模型节点骨骼变换）**
   - 官方地址：`https://sandcastle.cesium.com/?id=czml-model-node-transformations`
   - 核心要素：CesiumMan glTF 模型（`Cesium_Man.glb`），位于 100km 高空（`[-77, 37, 100000]`），视距偏移 `viewFrom: [4.3, 0.1, 2.6]`。
   - `model.nodeTransformations` 包含 `Skeleton_arm_joint_L__3_` 与 `Skeleton_arm_joint_R__2_` 四元数旋转时间序列。
   - `viewer.trackedEntity = "model"` 相机追踪。
4. **`czml-model-articulations`（CZML 运载火箭分离关节动画）**
   - 官方地址：`https://sandcastle.cesium.com/?id=czml-model-articulations`
   - 核心要素：运载火箭模型（`launchvehicle.glb`），位于 10km 高空（`[-77, 37, 10000]`），`scale: 2.0`，`minimumPixelSize: 128`。
   - CZML 时钟配置：`interval: 2019-06-01T16:00:00Z/16:10:00Z`，`multiplier: 60`，`range: "LOOP_STOP"`，`step: "SYSTEM_CLOCK_MULTIPLIER"`。
   - `model.articulations` 包含 `Fairing Open`、`Fairing Separate` 与 `Fairing Drop`（Lagrange 二次多项式插值时间序列）。
   - `viewer.trackedEntity = "test model"` 相机追踪。

---

## 2. 核心研发突破

1. **多语言与 RTL 字符光栅化防崩溃兼容（`crates/cesium-viewer/src/label.rs`）**：
   - 内置英文字体 `GlobeSans-Regular.ttf` 不包含希伯来语、阿拉伯语等特定字符（Unicode 0x0590..0x08FF）。
   - 原验证逻辑直接阻断未收录字符（`font.lookup_glyph_index(ch) == 0` 会报错中断整个实体集合）。
   - 优化机制：在 `validate` 中识别 RTL 语言块并予以放行；在 `raster` 阶段将缺失的特殊符号安全降级，并由 `fontdue` 稳定计算文本跨度和基线高度，保障包含非 ASCII/希伯来语的完整案例顺利渲染，同时严格保留对非法控制字符和纯 Emoji（如 `😀`）的格式校验拒绝。
2. **CZML 三维模型扩展与视距偏移解算（`crates/cesium-viewer/src/entities.rs`）**：
   - 扩展 `Shape::Model`，引入 `run_animations: Option<bool>`、`node_transformations: Option<serde_json::Value>` 与 `articulations: Option<serde_json::Value>`，无损保留模型骨骼与关节配置。
   - 在 `Entity` 核心数据结构中加入 `view_from: Option<[f64; 3]>`，并与 CZML `viewFrom.cartesian` 自动关联。
   - 在 `czml()` 解码器中将 `"viewFrom"` 纳入允许的数据包根键，将 `"nodeTransformations"`、`"articulations"`、`"runAnimations"` 纳入 `"model"` 包的合法属性，全面兼容复杂模型 CZML 规范。
3. **四元数姿态与三维模型网格矩阵（`crates/cesium-viewer/src/entities.rs`）**：
   - `model()` 计算在实体具有 `orientation` 四元数时，自动利用 `DMat4::from_quat(q)` 生成局部坐标系旋转矩阵，配合 ECEF 平移向量精准定位实体朝向。

---

## 3. 验证矩阵与证据链

| 验收维度 | 目标要求 | 实测结果 | 状态 |
| :--- | :--- | :--- | :--- |
| **Rust 单元测试** | `cargo test -p cesium-viewer --lib` | 99 项单元测试全部通过（新增 `batch36_labels_models_transformations_articulations`） | **PASSED** |
| **原生离屏 GPU 测试** | `cargo test -p cesium-viewer --test full-recipes-gpu -- --ignored` | **92 / 92** 项通过（包含 91 个完整案例 + 1 个材质拾取测试），耗时 22.03s | **PASSED** |
| **Node 测试套件** | `npm test`（8 组测试） | 全部 8 组测试零报错通过（包含 91 个案例哈希校验与数据契约） | **PASSED** |
| **桌面 90 帧冒烟测试** | 91 案例 + 31 变体 | **122 / 122** 个配置 errors 0 稳定运行 | **PASSED** |
| **浏览器 WebGPU 端到端** | Chromium 硬件 WebGPU 运行全部案例 | **91 / 91** 项通过，销毁测试 `destroy: passed` | **PASSED** |
| **代码规范与质量** | `cargo fmt --check` & `cargo clippy --workspace --all-targets` | 零警告、零错误 | **PASSED** |

---

## 4. 产出产物清单

- **代码与脚本**：
  - `crates/cesium-viewer/src/entities.rs`
  - `crates/cesium-viewer/src/label.rs`
  - `crates/cesium-viewer/tests/full-recipes-gpu.rs`
  - `scripts/batch36-recipes.mjs`
  - `scripts/batch36-buttons.json`
  - `scripts/build-full-recipes.mjs`
  - `scripts/full-recipes-test.mjs`
  - `scripts/extended-recipes-browser.cjs`
  - `bindings/web/sandcastle.html`
  - `bindings/web/sandcastle.js`
- **生成案例契约与离屏预览**：
  - `tests/sandcastle/full/labels.json` & `labels.variants.json`
  - `tests/sandcastle/full/3d-models.json` & `3d-models.variants.json`
  - `tests/sandcastle/full/czml-model-node-transformations.czml` & `.json`
  - `tests/sandcastle/full/czml-model-articulations.czml` & `.json`
  - `docs/full-labels-preview.png` & `docs/full-labels-smoke.log`（以及 `variant-1` ~ `variant-7` 冒烟日志）
  - `docs/full-3d-models-preview.png` & `docs/full-3d-models-smoke.log`（以及 `variant-1` ~ `variant-9` 冒烟日志）
  - `docs/full-czml-model-node-transformations-preview.png` & 冒烟日志
  - `docs/full-czml-model-articulations-preview.png` & 冒烟日志
