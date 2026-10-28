# Batch 37 (v0.0.37) 案例验证报告：3D Models Coloring、3D Models Dev、Labels Dev 与 Billboards Dev

## 1. 案例概要

本批次推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个核心实体与底层开发基元配方，累计完成 **95 / 309**（剩余 214 个）：

1. **`3d-models-coloring`（3D 模型材质颜色与轮廓着色画廊）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-models-coloring`
   - 变体全集（5 个工具栏菜单模型变体）：
     - Aircraft（`Cesium_Air.glb`，高度 5000m）
     - Ground Vehicle（`GroundVehicle.glb`，高度 0m）
     - Hot Air Balloon（`CesiumBalloon.glb`，高度 1000m）
     - Milk Truck（`CesiumMilkTruck.glb`，高度 0m）
     - Skinned Character（`Cesium_Man.glb`，高度 0m）
   - 着色与轮廓配置：`color: [1, 0, 0, 1]`（红色着色），`colorBlendMode: "Highlight"`，`colorBlendAmount: 0.5`，`silhouetteColor: [1, 0, 0, 1]`，`silhouetteSize: 2.0`。
   - 朝向基于 `headingPitchRollQuaternion`（heading 135°，pitch 0°，roll 0°）生成单位四元数朝向，由 `viewer.trackedEntity` 驱动相机跟踪取景。

2. **`3d-models-dev`（底层 Model.fromGltf 模型基元与局部变换）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-models-dev`
   - 变体全集（5 个底层开发基元变体）：
     - Aircraft（`Cesium_Air.glb`，高度 5000m）
     - Drone（`CesiumDrone.glb`，高度 150m）
     - Ground Vehicle（`GroundVehicle.glb`，高度 0m）
     - Milk Truck（`CesiumMilkTruck.glb`，高度 0m）
     - Skinned Character（`Cesium_Man.glb`，高度 0m）
   - 基于 `headingPitchRollToFixedFrame(origin, hpr)` 计算局部参考系矩阵与 ECEF 变换四元数，绑定 `minimumPixelSize: 128`、`maximumScale: 20000`、`colorBlendMode: "Highlight"`、`colorBlendAmount: 0.5` 与 `runAnimations: true`，由 `camera.lookAt` 矩阵定格视角。

3. **`labels-dev`（底层 LabelCollection 标签基元开发合集）**
   - 官方地址：`https://sandcastle.cesium.com/?id=labels-dev`
   - 变体全集（6 个底层开发变体）：
     - `Add label`：向 `LabelCollection` 添加基础文本标签
     - `Set font`：自定义 24px Helvetica 字体、SKYBLUE 填充色、黑色 2px 描边与 `FILL_AND_OUTLINE`
     - `Set properties`：300km 高空标签、`scale: 2.0`、背景框激活（`showBackground: true`）
     - `Add labels in reference frame`：在局部 ENU 坐标系原点及局部偏移坐标 `[1e6, 0, 0]`、`[0, 1e6, 0]`、`[0, 0, 1e6]` 挂载 East/North/Up 参考系文本
     - `Offset label by distance`：16x16 设施图标与顶部标签复合，包含 `pixelOffset: [0, -16]` 与近远视距相对偏移曲线
     - `Fade label by distance`：双城视距透明度渐隐（`translucencyByDistance`）

4. **`billboards-dev`（底层 BillboardCollection 广告牌基元与精灵切片）**
   - 官方地址：`https://sandcastle.cesium.com/?id=billboards-dev`
   - 变体全集（11 个底层广告牌变体）：
     - `Add billboard`：基础广告牌图标添加与定位
     - `Set billboard properties at creation`：指定尺寸（32x32）、垂直对齐、局部旋转及 `sizeInMeters: false`
     - `Change billboard properties`：动态更新尺寸（40x40）、旋转角度（45°）及黄色着色（`color: [1, 1, 0, 1]`）
     - `Size billboard in meters`：物理米级真实世界尺寸（`sizeInMeters: true`，宽 8000m、高 3900m）
     - `Add multiple billboards`：批量散列多个广告牌图标
     - `Scale by distance`：近远视距动态缩放曲线（`scaleByDistance: [1.5e2, 2.0, 1.5e7, 0.5]`）
     - `Fade by distance`：视距透明度渐隐（`translucencyByDistance: [1.5e2, 1.0, 8.0e6, 0.0]`）
     - `Offset by distance`：视距像素偏移曲线（`pixelOffsetScaleByDistance: [1.0e3, 1.0, 1.5e6, 0.0]`）
     - `Add point billboards`：1x1 纯白像素物理点阵，着色为自定义色彩（如绿色 `[0, 1, 0, 1]`）
     - `Add marker billboards using imageSubRegion`：大图集精灵切片子区域采样（`imageSubRegion: [x, y, width, height]`，解析为归一化纹理 UV 坐标）
     - `Add billboards in reference frame`：局部 ENU 坐标系下挂载局部偏移广告牌

---

## 2. 核心研发突破

1. **广告牌精灵切片与大图集子区域 UV 映射（`crates/cesium-viewer/src/screen.rs` & `entities.rs`）**：
   - 为 `Shape::Billboard` 引入 `image_sub_region: Option<[f64; 4]>` 与 `size_in_meters: Option<bool>`。
   - 在图元网格生成阶段，根据解码纹理的实际物理宽高对 `[x, y, w, h]` 进行边界校验与有效性断言，计算出精准的归一化局部纹理坐标 `[u0, u1, v0, v1]`，确保切片瓦片与精灵图集在 WebGPU 渲染管线中准确无变形映射。
2. **文本标签外观与多行文本排版增强（`crates/cesium-viewer/src/label.rs` & `entities.rs`）**：
   - 针对 `LabelCollection` 开发案例，直接在 `Shape::Label` 中原生支持 `font`、`show_background`、`background_color`、`outline_color`、`outline_width`、`style` 与 `fill_color`，直通底层 `Appearance` 参数。
   - 修正了非 ASCII 字符光栅化降级中换行符 `\n` 的处理逻辑，确保换行符作为版面排版 token 正常保留，维持多行文本基线跨度与高度计算的精确性。
3. **三维模型着色混合与高亮描边轮廓（`crates/cesium-viewer/src/entities.rs`）**：
   - 为 `Shape::Model` 引入 `color_blend_mode: Option<String>`、`color_blend_amount: Option<f64>`、`silhouette_color: Option<[f32; 4]>` 与 `silhouette_size: Option<f64>`。
   - 严格限定 `color_blend_amount` 在 `[0.0, 1.0]`，`silhouette_size` 在 `[0.0, 100.0]`，确保着色器混合参数稳定且无非法越界。
4. **局部参考系到 ECEF 全局坐标系四元数转换工具链（`scripts/batch37-recipes.mjs`）**：
   - 构建了从局部 ENU 矩阵、Heading-Pitch-Roll 欧拉角到 ECEF 笛卡尔空间单位四元数的精确转换数学函数，完美还原 Sandcastle 原生 `headingPitchRollToFixedFrame` 姿态行为。

---

## 3. 验证矩阵与证据链

| 验收维度 | 目标要求 | 实测结果 | 状态 |
| :--- | :--- | :--- | :--- |
| **Rust 单元测试** | `cargo test -p cesium-viewer --lib` | 97 项单元测试全部通过（新增 `batch37_models_coloring_dev_labels_billboards_contracts`） | **PASSED** |
| **原生离屏 GPU 测试** | `cargo test -p cesium-viewer --test full-recipes-gpu -- --ignored` | **96 / 96** 项通过（包含 95 个完整案例 + 1 个材质拾取测试），耗时 21.97s | **PASSED** |
| **Node 测试套件** | `npm test`（8 组测试） | 全部 8 组测试零报错通过（包含 95 个案例哈希校验与数据契约） | **PASSED** |
| **桌面 90 帧冒烟测试** | 95 案例 + 54 变体 | **149 / 149** 个配置 errors 0 稳定运行 | **PASSED** |
| **浏览器 WebGPU 端到端** | Chromium 硬件 WebGPU 运行全部案例 | **95 / 95** 项通过，销毁测试 `destroy: passed` | **PASSED** |
| **代码规范与质量** | `cargo fmt --check` & `cargo clippy --workspace --all-targets` | 零警告、零错误 | **PASSED** |

---

## 4. 产出产物清单

- **代码与脚本**：
  - `crates/cesium-viewer/src/entities.rs`
  - `crates/cesium-viewer/src/label.rs`
  - `crates/cesium-viewer/src/screen.rs`
  - `crates/cesium-viewer/tests/full-recipes-gpu.rs`
  - `scripts/batch37-recipes.mjs`
  - `scripts/batch37-buttons.json`
  - `scripts/build-full-recipes.mjs`
  - `scripts/full-recipes-test.mjs`
  - `scripts/extended-recipes-browser.cjs`
  - `bindings/web/sandcastle.html`
  - `bindings/web/sandcastle.js`
- **生成案例契约与离屏预览**：
  - `tests/sandcastle/full/3d-models-coloring.json` & `3d-models-coloring.variants.json`
  - `tests/sandcastle/full/3d-models-dev.json` & `3d-models-dev.variants.json`
  - `tests/sandcastle/full/labels-dev.json` & `labels-dev.variants.json`
  - `tests/sandcastle/full/billboards-dev.json` & `billboards-dev.variants.json`
  - `docs/full-3d-models-coloring-preview.png` & `docs/full-3d-models-coloring-smoke.log`（及 `variant-1` ~ `variant-4` 冒烟日志）
  - `docs/full-3d-models-dev-preview.png` & `docs/full-3d-models-dev-smoke.log`（及 `variant-1` ~ `variant-4` 冒烟日志）
  - `docs/full-labels-dev-preview.png` & `docs/full-labels-dev-smoke.log`（及 `variant-1` ~ `variant-5` 冒烟日志）
  - `docs/full-billboards-dev-preview.png` & `docs/full-billboards-dev-smoke.log`（及 `variant-1` ~ `variant-10` 冒烟日志）
