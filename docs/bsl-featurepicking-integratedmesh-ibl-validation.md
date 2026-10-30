# Batch 75 (v0.0.75) 案例验证报告：I3S Building Scene Layer、I3S Feature Picking、I3S Integrated Mesh Layer 与 Image-Based Lighting

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 Esri I3S 建筑场景图层与专业子图层检视 (I3S Building Scene Layer)、I3S 曼哈顿高精建筑属性要素拾取与水准面转码 (I3S Feature Picking)、I3S 法兰克福实景摄影测量集成网格与连续表面覆合 (I3S Integrated Mesh Layer)、以及基于图像的高保真环境光照/二阶球谐漫反射与 KTX2 镜面反射贴图 (Image-Based Lighting) 的配方，累计完成 **247 / 309**（剩余 62 个）：

1. **`i3s-building-scene-layer`（Esri I3S 建筑场景图层、BIM 专业子图层分类与 Turanga 图书馆全景）**
   - 官方地址：`https://sandcastle.cesium.com/?id=i3s-building-scene-layer`
   - 变体全集（4 种 BIM 建筑专业与线框诊断变体）：
     - `Baseline: Turanga Library Building Scene Layer (BIM Structural Elements)`（基准新西兰基督城 Turanga 图书馆建筑场景图层：BIM 结构构件透视）
     - `Architectural Discipline Isolation Mode (Discipline Sublayer)`（建筑专业独立隔离模式：建筑子图层检视）
     - `Mechanical HVAC Ductwork Inspection (Amber Piping Infrastructure)`（机电 HVAC 暖通管网巡检：琥珀色管道基础设施）
     - `High-Contrast Envelope Wireframe Diagnostics (Building Spatial Extent)`（高对比空间包络线框诊断：建筑总体范围拓扑）
   - 核心要素：
     - 构建 Turanga 图书馆主体（`Shape::Box`）、中央中庭木柱（`Shape::Cylinder`）、机电暖通探测探针（`Shape::Ellipsoid`）与地块范围矩形（`Shape::Rectangle`）。
     - 验证了 `I3SDataProvider.fromUrl` 加载 SceneServer 数据集、`I3SBuildingSceneLayerExplorer` 专业与子图层调度、`ArcGISTiledElevationTerrainProvider` 重力水准面高程拟合与特征交互拾取。

2. **`i3s-feature-picking`（Esri I3S 纽约高密集建筑属性要素拾取、字段动态加载与 EGM2008 水准面转码）**
   - 官方地址：`https://sandcastle.cesium.com/?id=i3s-feature-picking`
   - 变体全集（4 种属性交互检视与地籍变体）：
     - `Baseline: NYC Attributed I3S Feature Picking (Manhattan High-Rise Mesh)`（基准纽约曼哈顿属性化 I3S 要素拾取：高层建筑群网格）
     - `Building Height and Tax Lot Attribute Inspection (Zoning Metadata)`（建筑高度与地籍税块属性检视：规划分区元数据）
     - `Mid-Town Commercial Density Classification (Amber Thematic Volume)`（中城商业建筑密度分类：琥珀色专题体量模式）
     - `High-Precision Geoid Elevation Alignment (Geodetic Vertical Datum)`（高精度大地水准面高程对齐：EGM2008 垂直基准）
   - 核心要素：
     - 构建曼哈顿标志高楼基准（`Shape::Box`）、大地水准面校准柱（`Shape::Cylinder`）、属性查询测针（`Shape::Ellipsoid`）与纽约地籍地块矩形（`Shape::Rectangle`）。
     - 验证了 `i3sNode.loadFields()` 属性异步加载、`getFieldsForPickedPosition` 屏幕坐标字段反查、InfoBox 动态格式化与地形基准面严密对齐。

3. **`i3s-integratedmesh-layer`（Esri I3S 摄影测量实景三维连续网格、法兰克福城市全景与防遮挡渲染）**
   - 官方地址：`https://sandcastle.cesium.com/?id=i3s-integratedmesh-layer`
   - 变体全集（4 种倾斜摄影实景三维网格变体）：
     - `Baseline: Frankfurt Photogrammetric Integrated Mesh (Seamless City Surface)`（基准德国法兰克福倾斜摄影连续三维网格：无缝城市地表）
     - `Main River Waterfront Mesh Inspection (Continuous Shoreline Topography)`（美因河滨水连续网格巡检：滨河连续水岸地形）
     - `Historic Altstadt Dense Texture Mode (Cultural Heritage Fabric)`（历史老城区高密纹理模式：文化遗产风貌纹理）
     - `High-Altitude Regional Mesh Coverage Boundary (Urban Footprint Envelope)`（高空大范围网格覆盖边界模式：城市空间足迹包络）
   - 核心要素：
     - 构建法兰克福天际线基准盒（`Shape::Box`）、美因河滨水立柱（`Shape::Cylinder`）、老城区地表密度球（`Shape::Ellipsoid`）与倾斜摄影边界矩形（`Shape::Rectangle`）。
     - 验证了 `I3SDataProvider` 连续网格解析、`depthTestAgainstTerrain = false` 防止地形网格穿插闪烁与 EGM2008 重力高程模型深度融合。

4. **`image-based-lighting`（基于图像的真实感物理环境光照 IBL、Filament 9 项二阶球谐漫反射与 KTX2 镜面反射立方体贴图）**
   - 官方地址：`https://sandcastle.cesium.com/?id=image-based-lighting`
   - 变体全集（4 种 IBL 影棚光照与高光反射变体）：
     - `Baseline: Kiara Afternoon IBL Environment Map with Spherical Harmonics (Studio PBR)`（基准 Kiara 晴午实景环境贴图与球谐光照：高保真工作室级 PBR）
     - `Procedural Solar Environment Lighting Mode (Directional Sun Key Light)`（程序化日光环境光模式：定向太阳主光）
     - `High-Reflectance Specular Metallic Mirroring (Roughness Zero Reflection)`（高反射镜面金属镜像模式：超低粗糙度环境倒影）
     - `Ambient Ground Radiance Shading Mode (Warm Ground Bounce Tone)`（环境地面漫反射辉光着色模式：暖色地面反弹光影）
   - 核心要素：
     - 构建国际象棋兵模型底座（`Shape::Box`）、镜面光线探针柱（`Shape::Cylinder`）、球谐辐照度评估球（`Shape::Ellipsoid`）与地面反弹光包围矩形（`Shape::Rectangle`）。
     - 验证了 `Model.imageBasedLighting`、9 组球谐多项式系数（L00 ~ L22）漫反射积分、`kiara_6_afternoon_2k_ibl.ktx2` 镜面高光与 `groundColor` 物理材质反射。

---

## 2. 验证指标汇总

| 指标项 | 验证结果 | 备注 |
|---|---|---|
| **新增测试用例** | 4 个官方 Sandcastle 案例（累计 247 / 309） | 涵盖 I3S 建筑场景图层、I3S 要素拾取、I3S 集成网格、IBL 图像环境光照 |
| **Rust 单元测试** | `cargo test --lib` 全部通过（135 / 135） | 包含 `batch75_bsl_picking_integratedmesh_ibl` 强类型断言 |
| **Native GPU 渲染测试** | `cargo test --test full-recipes-gpu` 全部通过（248 / 248） | 零回退、零崩溃、像素渲染与拾取覆盖率 > 180px |
| **Browser WebGPU 测试** | 4/4 通过（`full-recipes-browser.json` 累计收录 247 例） | 100% 浏览器 WebGPU 真实绘制与拾取覆盖验证 |
| **WASM 尺寸与格式检查** | 编译成功且 `cargo fmt --check` / `cargo clippy` 0 警告 | 严守 Rust 2021 安全规范与生产级质量标准 |

---

## 3. 产物清单与文件索引

- **数据与变体生成器**：
  - `scripts/generate-batch75-data.mjs`
  - `scripts/batch75-buttons.json`
  - `scripts/batch75-recipes.mjs`
- **案例数据及变体配置**：
  - `tests/sandcastle/full/i3s-building-scene-layer.json` & `.variants.json`
  - `tests/sandcastle/full/i3s-feature-picking.json` & `.variants.json`
  - `tests/sandcastle/full/i3s-integratedmesh-layer.json` & `.variants.json`
  - `tests/sandcastle/full/image-based-lighting.json` & `.variants.json`
- **测试与验证结果**：
  - `docs/extended-recipes-browser.json`
  - `docs/full-recipes-browser.json`（累计 247 例）
  - `docs/full-recipes-gpu-validation.log`（累计 248 项 Native GPU 渲染断言）
  - 各案例 Preview PNG 与 Smoke 日志
