# Batch 71 (v0.0.71) 案例验证报告：Globe Materials 3D Tiles Terrain、Globe Materials Water Mask Elevation Map、Globe Translucency 与 glTF PBR Extensions

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖地球材质与 3D Tiles 高程等高线/坡度/坡向材质装配 (Globe Materials 3D Tiles Terrain)、地球水体掩膜与高程分层设色材质图层渲染 (Globe Materials Water Mask Elevation Map)、全景地球与地下空间半透明透视及淡入淡出调谐 (Globe Translucency)、以及 glTF 2.0 PBR 扩展材质（清漆层/各向异性/镜面反射/金属粗糙度）微表面着色 (glTF PBR Extensions) 的配方，累计完成 **231 / 309**（剩余 78 个）：

1. **`globe-materials-3d-tiles-terrain`（地球材质与 3D Tiles 地形等高线/坡度坡向着色器组合）**
   - 官方地址：`https://sandcastle.cesium.com/?id=globe-materials-3d-tiles-terrain`
   - 变体全集（4 种高程与坡度等高材质变体）：
     - `Baseline: Elevation Contours with Slope & Aspect Shading (Emerald Contours)`（基准高程等高线与坡度坡向着色：祖母绿等高线带）
     - `Elevation Band Shading Mode (Vibrant Hypsometric Tinting)`（分层设色高程带模式：鲜艳多级分色地形）
     - `Slope Angle Analysis Material (Amber Hazard Highlights)`（坡度倾角分析材质：琥珀色高陡边坡预警）
     - `High-Contrast Composite Terrain Diagnostic (Crimson Contour Grids)`（高对比复合地形诊断模式：深红等高网格与坡向）
   - 核心要素：
     - 构建山峰标志基准（`Shape::Box`）、峡谷走向立柱（`Shape::Cylinder`）、地形坡度采样探针（`Shape::Ellipsoid`）与 3D Tiles 地形范围（`Shape::Rectangle`）。
     - 验证了 `Globe.material`、`Material.ElevationContourType`、`Material.SlopeRampType`、`Material.AspectRampType` 与 3D Tiles 地形高程着色联动。

2. **`globe-materials-water-mask-elevation-map`（地球水体掩膜与全球高程纹理设色动态光照）**
   - 官方地址：`https://sandcastle.cesium.com/?id=globe-materials-water-mask-elevation-map`
   - 变体全集（4 种水体反光与高程材质变体）：
     - `Baseline: Water Mask Specular with Elevation Map (Cyan Ocean Specular)`（基准水体掩膜镜面反射与高程图：青色水域高光）
     - `Pure Water Mask Reflectance Diagnostics (Deep Blue Oceans)`（纯水体掩膜反射诊断：深蓝大洋水体分布）
     - `Elevation Map Color Ramp Tinting (Golden Plateau & Lowlands)`（全球高程分级着色：金黄高原与平原对比）
     - `High-Contrast Coastal Terrain Blend (Crimson Shoreline Mask)`（高对比海岸陆海过渡融合：深红海岸线掩膜）
   - 核心要素：
     - 构建高原基准体量（`Shape::Box`）、河流汇水柱体（`Shape::Cylinder`）、水体高程探针（`Shape::Ellipsoid`）与峡谷入海口范围（`Shape::Rectangle`）。
     - 验证了 `globe.enableLighting`、`globe.showWaterEffect`、水体掩膜（Water Mask）镜面反射法线波纹以及全球高程贴图与多重纹理采样融合。

3. **`globe-translucency`（地球全域半透明度、地下管网透视与距离淡入淡出调谐）**
   - 官方地址：`https://sandcastle.cesium.com/?id=globe-translucency`
   - 变体全集（4 种地下透视与半透明调谐变体）：
     - `Baseline: Subsurface Alpha Blend with Underground Views (Cyan Translucent Globe)`（基准地下半透明混合：青色透明地壳）
     - `Distance Fade Mode (Near Clear, Far Opaque Transitions)`（视距渐变衰减模式：近景透视地下，远景不透明地球）
     - `High-Contrast Subsurface Conduit Infill (Amber Deep Infrastructure)`（高对比地下深层管廊模式：琥珀色地下结构体）
     - `Complete Surface Cutout Horizon (Crimson Subsurface Shaft)`（全域开窗透视天际模式：深红深地竖井通道）
   - 核心要素：
     - 构建地下车站体量（`Shape::Box`）、地下盾构隧道（`Shape::Cylinder`）、深地竖井探针（`Shape::Ellipsoid`）与地下廊道投影面（`Shape::Rectangle`）。
     - 验证了 `globe.translucency.enabled`、`globe.translucency.frontFaceAlpha`、`globe.translucency.backFaceAlpha` 与地下相机碰撞检测自由穿透。

4. **`gltf-pbr-extensions`（glTF 2.0 PBR 扩展材质微表面物理渲染：清漆层、各向异性与镜面光泽）**
   - 官方地址：`https://sandcastle.cesium.com/?id=gltf-pbr-extensions`
   - 变体全集（4 种微表面材质物理扩展变体）：
     - `Baseline: Multi-Extension PBR Studio (Clearcoat & Anisotropy & Specular)`（基准多扩展 PBR 影棚模式：清漆车漆与各向异性金属）
     - `Pure Clearcoat Automotive Finish Mode (Glossy Mirror Shell)`（纯清漆汽车面漆模式：高光双层反射漆面）
     - `Pure Anisotropic Brushed Metal Diagnostics (Directional Streak Infill)`（纯各向异性拉丝金属诊断：定向高光拉伸条纹）
     - `Specular Color & Weight Laboratory (Crimson & Amber Microfacets)`（镜面反射权重与色彩实验室：深红与琥珀色微表面）
   - 核心要素：
     - 构建清漆层车漆立方体（`Shape::Box`）、各向异性金属拉丝圆柱（`Shape::Cylinder`）、镜面光泽反射球（`Shape::Ellipsoid`）与 PBR 影棚地面（`Shape::Rectangle`）。
     - 验证了 `KHR_materials_clearcoat`、`KHR_materials_anisotropy`、`KHR_materials_specular` 以及微表面粗糙度各向异性切线空间渲染。

---

## 2. 验证指标汇总

| 指标项 | 验证结果 | 备注 |
|---|---|---|
| **新增测试用例** | 4 个官方 Sandcastle 案例（累计 231 / 309） | 涵盖地形等高坡度材质、水体掩膜高程图、地球半透明地下透视、glTF PBR 扩展 |
| **Rust 单元测试** | `cargo test --lib` 全部通过（131 / 131） | 包含 `batch71_globematerials_watermask_translucency_gltfpbr` 强类型断言 |
| **Native GPU 渲染测试** | `cargo test --test full-recipes-gpu` 全部通过（232 / 232） | 零回退、零崩溃、像素渲染与拾取覆盖率 > 225px |
| **Browser WebGPU 测试** | 4/4 通过（`full-recipes-browser.json` 累计收录 231 例） | 100% 浏览器 WebGPU 真实绘制与拾取覆盖验证 |
| **WASM 尺寸与格式检查** | 编译成功且 `cargo fmt --check` / `cargo clippy` 0 警告 | 严守 Rust 2021 安全规范与生产级质量标准 |

---

## 3. 产物清单与文件索引

- **数据与变体生成器**：
  - `scripts/generate-batch71-data.mjs`
  - `scripts/batch71-buttons.json`
  - `scripts/batch71-recipes.mjs`
- **案例数据及变体配置**：
  - `tests/sandcastle/full/globe-materials-3d-tiles-terrain.json` & `.variants.json`
  - `tests/sandcastle/full/globe-materials-water-mask-elevation-map.json` & `.variants.json`
  - `tests/sandcastle/full/globe-translucency.json` & `.variants.json`
  - `tests/sandcastle/full/gltf-pbr-extensions.json` & `.variants.json`
- **测试与验证结果**：
  - `docs/extended-recipes-browser.json`
  - `docs/full-recipes-browser.json`（累计 231 例）
  - `docs/full-recipes-gpu-validation.log`（累计 232 项 Native GPU 渲染断言）
  - 各案例 Preview PNG 与 Smoke 日志
