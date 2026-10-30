# Cesium Rust 开发与 Agent 协作日志

## 项目基本信息
- **工作区路径**：`/Users/jinjun/study/cesium-rust/cesium-rust`
- **远程仓库**：`https://github.com/jinjun1994/cesium-rust.git` (`main` 分支)
- **目标里程碑**：M1/M2 推进实体与属性图谱，M3 聚焦 G05 地形、材质与贴地渲染，完成 Sandcastle 309 个全量官方配方移植验收。
- **当前版本**：v0.0.90（已通过 307 / 309 个完整案例，剩余最后 2 个）

---

## 开发批次演进记录

### [2026-10-30] v0.0.82 验收完成：Particle System Weather、PBR Lighting - Dev、Per-Feature Post Processing 与 Physically-Based Materials
- **完成配方**（累计推进至 **275 / 309**，剩余 34 个）：
  1. `particle-system-weather`（高山喜马拉雅大范围降雪/暴雨天气、SphereEmitter 半径 100km 粒子发射与雾气光学迁移：基准高山暴风雪模式、倾盆大雨暴雨模式、冰霰暴风雪混合模式、雪后初霁晴空模式 4 种变体）
  2. `pbr-lighting-dev`（基于物理渲染 PBR 模型多因子光照、ImageBasedLighting 环境贴图与动态环境贴图管理器：基准完整 PBR 图像光照、纯漫反射环境光照模式、纯镜面反射高光模式、直射日光影棚模式 4 种变体）
  3. `per-feature-post-processing`（逐要素交互式后处理阶段、鼠标悬停拾取与轮廓描边/黑白阶调阶化：基准鼠标悬停高亮石灰绿轮廓线阶段、鼠标悬停黑白灰度量化阶段、双阶段复合渲染模式、直通无过滤基础模型渲染 4 种变体）
  4. `physically-based-materials`（工程运输卡车真实感物理材质、地形贴地与全天 24 小时动态光照漫游：基准正前方高光反射视角、45 度侧角日出掠射视角、正上方正午直射视角、仰角黄昏地平线边缘光视角 4 种变体）
- **核心研发突破**：
  1. **大范围气象粒子动力学与大气雾气协同（Wide-Area Weather Dynamics & Atmospheric Fog Coupling）**：支持半径 100km 超大发射半径随相机平移，实现基于视距的反比透明度渐变与雨雪重力加速度场，无缝驱动大气散射色移与雾气浓淡。
  2. **高精物理环境光照与漫反射/镜面二元解耦（Dual-Factor Image-Based Lighting & Dynamic Ground Calibration）**：解耦 IBL 漫反射/镜面反射权重向量，实现 `DynamicEnvironmentMapManager` 对地面反照率、大气散射倍率与 Gamma 的闭环重置调控。
  3. **逐要素屏幕空间后处理掩膜隔离（Per-Feature Post-Process Isolation & Selective Staging）**：结合 `PostProcessStageLibrary` 动态管理要素选择集 `stage.selected`，仅对特定图元实施石灰绿高光轮廓描边或多阶黑白片元着色。
  4. **全天时太阳光照追踪与地形贴模 PBR 材质（Diurnal Solar Tracking & Terrain-Clamped PBR Shading）**：支持 `HeightReference.CLAMP_TO_GROUND` 地形贴模解算，通过时钟时间推移驱动太阳光照矢量连续变化，在微表面高光与阴影级联中呈现细腻金属质感。
- **验证结果**：
  - **普通 Rust 单元测试**：142 项全部通过（含 `batch82_weather_pbr_postprocess_materials`）。
  - **真实 GPU 离屏渲染测试**：276 / 276 项全数通过（`276 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，275 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：275 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Particle System Weather、PBR Lighting - Dev、Per-Feature Post Processing 与 Physically-Based Materials 验证文档](docs/weather-pbr-postprocessing-pbm-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.81 验收完成：Panorama、Particle System、Particle System Fireworks 与 Particle System Tails
- **完成配方**（累计推进至 **271 / 309**，剩余 38 个）：
  1. `panorama`（360 度全景沉浸式球体/立方体贴图漫游、相机 lookAt 姿态锁定与鼠标滚轮视场 FOV 缩放：基准等距长方圆柱 360 全景、六面成立方体天空盒全景模式、望远长焦光学变焦模式、超广角沉浸式全景模式 4 种变体）
  2. `particle-system`（运奶车运动轨迹跟踪、Circle/Sphere/Cone 多型发射器与局部引力扰动：基准圆形发射器尾气烟流、球形发射器全向膨胀模式、高速方向性锥体喷流、重力偏转下沉烟羽模式 4 种变体）
  3. `particle-system-fireworks`（夜空多级爆裂烟花、随机三维空间偏移与径向速度衰减截止：基准多级烟花齐射、高空翡翠与青蓝星芒、极速连发压轴华章模式、近地超重型礼炮单团绽放 4 种变体）
  4. `particle-system-tails`（超音速客机尾迹、彗尾冰晶流与火箭热能喷注双模切换：基准彗尾深冷粒子飘带、火箭助推热力加力模式、双翼尖对称空气动力学涡流凝结尾迹模式、低速大气烟带宽幅漂移弥散模式 4 种变体）
- **核心研发突破**：
  1. **沉浸式 360 全景渲染与视场交互（Equirectangular & Cubemap Panoramas & Dynamic FOV）**：支持 `EquirectangularPanorama` 与 `CubeMapPanorama` 局部定向变换矩阵装配，解算 lookAt 观察点与滚轮视场平滑缩放。
  2. **物理引力粒子动力学管线（Physics-Based Particle Emitters & Local Gravity Field）**：集成 `CircleEmitter` / `SphereEmitter` / `ConeEmitter` / `BoxEmitter` 多几何发射器与 `ParticleBurst` 脉冲爆发，支持以地心法向单位向量实施局部引力速度场动态解算。
  3. **大规模多级烟火粒子散布与截止截断（Multi-Stage Firework Salvos & Velocity Cutoff Force）**：支持 20 组随机空间三维偏移矩阵与色彩生成，结合 `updateCallback` 在达到爆心半径阈值时强行衰减截断速度。
  4. **大数量超音速尾流粒子群与微分螺旋发散（High-Density Contrail Swarms & Cartographic Spiral Integration）**：支持 50 ~ 100 组微型粒子发射器级联阵列，实现基于大地经纬度步长与局部速度切向混合的动态尾迹流淌。
- **验证结果**：
  - **普通 Rust 单元测试**：141 项全部通过（含 `batch81_panorama_particles_fireworks_tails`）。
  - **真实 GPU 离屏渲染测试**：272 / 272 项全数通过（`272 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，271 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：271 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Panorama、Particle System、Particle System Fireworks 与 Particle System Tails 验证文档](docs/panorama-particles-fireworks-tails-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.80 验收完成：Multifrustum Snapping - Dev、Multiple Shadows - Dev、Multiple Synced Views 与 PAMAP Terrain
- **完成配方**（累计推进至 **267 / 309**，剩余 42 个）：
  1. `multifrustum-snapping-dev`（Cesium ion 几何吸附多平截头体近远裁切面穿透、多视锥高精度拾取与对齐孔径控制：基准多平截头体离子模型吸附、高灵敏度大孔径吸附模式、中距离平截头体边界吸附模式、多视锥线段端点对准模式 4 种变体）
  2. `multiple-shadows-dev`（多动态光源阴影图投影、方向光/聚光灯/点光源阴影级联混合与偏置解析度调优：基准多光源投射阴影、聚光灯窄锥角锐利阴影模式、日光主导高分辨率瀑布级联阴影模式、低空多色微弱辅助阴影模式 4 种变体）
  3. `multiple-synced-views`（多相机视口联动同步、主从视口三维空间姿态解算与多地标分屏观览：基准多视口双向实时相机姿态联动、俯视正射主从对齐模式、地平线追踪双屏漫游模式、高空全景广角与微观细节对比模式 4 种变体）
  4. `pamap-terrain`（宾夕法尼亚高分辨率激光雷达 PAMAP 地形数据、局部高精高程流与地貌等高线剖析：基准 PAMAP 高分辨率局部地形、阿勒格尼高原山脉高程增强模式、宾州局部水系低洼地貌对比模式、无地形基准椭球体对照模式 4 种变体）
- **核心研发突破**：
  1. **多平截头体几何吸附与视锥深度穿透（Multifrustum Snapping & Deep Picking）**：支持多平截头体渲染架构下的空间几何点与线段对准，跨深度分层消除 Z-fighting 并精确维持近远景拾取精度。
  2. **多光源级联阴影管线与偏置调制（Multi-Source Cascade Shadow Mapping）**：集成多方向光与聚光灯阴影投射矩阵，动态计算级联阴影深度图偏置，消除粉刺走样与阴影断层。
  3. **多视口相机姿态同步与主从对齐（Synchronized Camera Views & Split-View Orchestration）**：实现主从视图实时变换矩阵双向联动，支持俯视导航雷达与斜视巡检视角的平滑姿态镜像。
  4. **高分辨率局部激光雷达地形流（High-Resolution PAMAP Terrain Streaming）**：集成宾夕法尼亚高精度 PAMAP 高程瓦片集，支持地势起伏分层增强与局域等高线微地貌解析。
- **验证结果**：
  - **普通 Rust 单元测试**：140 项全部通过（含 `batch80_multifrustum_shadows_synced_pamap`）。
  - **真实 GPU 离屏渲染测试**：268 / 268 项全数通过（`268 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，267 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：267 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Multifrustum Snapping - Dev、Multiple Shadows - Dev、Multiple Synced Views 与 PAMAP Terrain 验证文档](docs/multifrustum-shadows-synced-pamap-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.79 验收完成：Material with Custom GLSL、Montreal Point Cloud、Moon 与 MSAA
- **完成配方**（累计推进至 **263 / 309**，剩余 46 个）：
  1. `material-with-custom-glsl`（Cesium Material Fabric 规范、自定义顶点/片元着色器、时钟 Uniform 驱动与动态 Phong 高光：基准自定义 Fabric GLSL 材质外观、正弦时间周期 RGB 色度动态调制模式、双扫描明亮透明度光带模式、连续 Z 轴顶点旋转模式 4 种变体）
  2. `montreal-point-cloud`（加拿大蒙特利尔 20cm 高分辨率激光点云、皇家山地貌与多视角导览：基准蒙特利尔 20cm 分辨率激光雷达点云、皇家山公园植被地形点云分类模式、老港历史滨水街区近景巡检模式、圣劳伦斯河沿岸水域勘测边界模式 4 种变体）
  3. `moon`（月球全境 3D 瓦片地形、LROC 月海玄武岩边界与阿波罗/阿尔忒弥斯着陆点：基准月球 3D 瓦片地形、阿尔忒弥斯 3 号南极永久阴影坑科学考察模式、LROC 全球月海玄武岩边界贴地 GeoJSON 矢量模式、月球高地斜长岩风化层反射率光谱分析模式 4 种变体）
  4. `msaa`（场景级多重采样抗锯齿 MSAA、亚像素级边缘平滑与硬件功能检测：基准 MSAA 8x 多重采样抗锯齿激活、MSAA 4x 性能平衡抗锯齿模式、MSAA 2x 轻量边缘平滑模式、MSAA 1x 走样基准直通模式 4 种变体）
- **核心研发突破**：
  1. **Fabric 架构自定义材质与动态着色器注入（Fabric Custom Shaders & Dynamic Uniforms）**：支持 `Cesium.Material` 的 Fabric 架构与 `MaterialAppearance` 顶点/片元自定义代码注入，结合 `czm_phong` 与时间周期实现平滑的正弦颜色过渡与光带扫描。
  2. **高密度城市倾斜激光雷达点云漫游（Urban Lidar Point Cloud Streaming & FlyTos）**：集成蒙特利尔 20cm 级点云模型，支持精确的三维直角坐标视点导引与相机四元数定向飞行。
  3. **月球全域实景三维与玄武岩地质图（Lunar 3D Terrain & Geological Overlay）**：支持 `Ellipsoid.MOON` 参考椭球体与高程瓦片碰撞，结合 LROC 全球月海玄武岩贴地 GeoJSON 矢量图层展示多时期人类探月成果。
  4. **硬件级多重采样抗锯齿调控（Hardware Multisample Anti-Aliasing Pipeline）**：支持 `scene.msaaSamples`（1x、2x、4x、8x）动态调优与硬件兼容性守卫，消除模型边缘与高程地形之间的亚像素锯齿走样。
- **验证结果**：
  - **普通 Rust 单元测试**：139 项全部通过（含 `batch79_glsl_montreal_moon_msaa`）。
  - **真实 GPU 离屏渲染测试**：264 / 264 项全数通过（`264 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，263 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：263 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Material with Custom GLSL、Montreal Point Cloud、Moon 与 MSAA 验证文档](docs/glsl-montreal-moon-msaa-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.78 验收完成：KHR_mesh_primitive_restart - Dev、LensFlare、Many Clipping Planes - Dev 与 Mars
- **完成配方**（累计推进至 **259 / 309**，剩余 50 个）：
  1. `khr-mesh-primitive-restart-dev`（glTF KHR_mesh_primitive_restart 图元内联重启索引批处理、Hilbert 空间填充曲线与 WebGL 2 原生管线：基准希尔伯特空间填充曲线图元重启、线环多不相交拓扑模式、三角带批处理几何模式、三角扇不相交拓扑检视模式 4 种变体）
  2. `lensflare`（相机镜头眩光后处理全屏阶段、彩虹色散光环、光学镜片污迹与深空天体观察：基准太阳镜头眩光后处理阶段、高色散彩虹光环模式、高强度光学表面污渍模拟模式、微光大气边缘眩光模式 4 种变体）
  3. `many-clipping-planes-dev`（大数量径向剖切面几何裁剪、32/64 面圆柱剖开剖切体与多类型图元裁剪适配：基准 32 径向剖切面、高密度 64 剖切面多面体高精竖井钻孔模式、并集外壳反向剖切模式、地下地形径向扇区剖切包络模式 4 种变体）
  4. `mars`（火星全境 3D 瓦片地形、火星大气散射参数、好奇号与毅力号漫游车实迹交互：基准火星 3D 瓦片实景地形与火星大气层、盖尔撞击坑好奇号漫游车穿越轨迹、毅力号与机智号直升机侦察飞行范围、《火星救援》跨大陆远征路线 4 种变体）
- **核心研发突破**：
  1. **glTF 图元内联索引重启管线（Primitive Restart Indices & WebGL 2 Buffer Binding）**：支持 `KHR_mesh_primitive_restart` 规范，实现基于 `0xFF / 0xFFFF / 0xFFFFFFFF` 重启值的线/带/扇多拓扑单图元高效批处理渲染。
  2. **全屏镜头眩光后处理与天际光晕（Lens Flare Post-Process & Optical Dispersion）**：集成 `PostProcessStageLibrary.createLensFlareStage`，支持彩虹鬼影色散、光环宽度与镜面污迹多参数着色器 Uniforms 实时联动。
  3. **大数量径向剖切面几何裁剪系统（Radial Multi-Plane Clipping & Excavation）**：支持 32/64 面大数量剖切平面数组批处理生成，结合距离矩阵与反向裁剪实现任意半径圆柱井筒精准掏空开挖。
  4. **火星天体全域地形与物理散射大气（Mars Ellipsoid & Planetary Atmospheric Scattering）**：集成 `Ellipsoid.MARS` 与 3D 瓦片实景三维，精确配置火星特有瑞利/米氏大气光学散射系数，结合时态 CZML 实现多漫游车火星表面全地形时空回放。
- **验证结果**：
  - **普通 Rust 单元测试**：138 项全部通过（含 `batch78_restart_lensflare_clipping_mars`）。
  - **真实 GPU 离屏渲染测试**：260 / 260 项全数通过（`260 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，259 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：259 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [KHR_mesh_primitive_restart - Dev、LensFlare、Many Clipping Planes - Dev 与 Mars 验证文档](docs/khrrestart-lensflare-clipping-mars-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.77 验收完成：Ion Snapping - Dev、iTwin Feature Service、iTwin Gaussian Splats 与 Japan Buildings
- **完成配方**（累计推进至 **255 / 309**，剩余 54 个）：
  1. `ion-snapping-dev`（Cesium ion 服务端几何对齐吸附、最近/关键点/中心多模式吸附与交互容差孔径：基准 Ion BIM/CAD 服务端几何吸附、关键点吸附模式、中心吸附模式、高容差搜索孔径模式 4 种变体）
  2. `itwin-feature-service`（Bentley iTwin 地理要素服务、点线面多尺度现实数据分类渲染与图层控制台：基准 Bentley iTwin 要素服务、点状基础设施隔离模式、线性工程标线模式、鸟瞰航拍地籍多边形区域 4 种变体）
  3. `itwin-gaussian-splats`（Bentley iTwin 3D 高斯泼溅真实感模型、模型矩阵高程微调与全球地形结合：基准 3D 高斯泼溅真实数据、大坝坝顶近景巡检模式、地形基底矩阵偏移校准、勘测外包范围边界模式 4 种变体）
  4. `japan-buildings`（日本全境 3D 建筑物大数据集、局部地形加载、多地标漫游与动态相机光源：基准日本 3D 建筑大数据集与区域地形、京都二条城文化遗产地标视角、相机动态平行光源、富士山-御殿场地势高程模式 4 种变体）
- **核心研发突破**：
  1. **Ion 服务端高精几何吸附对齐（Ion Snap Service & Topo Alignment）**：集成 `IonSnapService.fromAssetId` 与 `IonSnapMode`，支持最近点、顶点/边缘关键点与形心多模式几何捕捉对齐与像素级动态容差搜索。
  2. **iTwin 地理要素服务与多源图层控制（iTwin Geospatial Features & Layer Management）**：支持 `ITwinData.loadGeospatialFeatures` 异步批量加载点/线/面要素，结合 Maki 图标 PinBuilder 与独立分类控制台动态切换。
  3. **3D 高斯泼溅网格与全球地形贴合（3D Gaussian Splats & World Terrain Registration）**：集成 `ITwinData.createTilesetForRealityDataId`，支持高斯核协方差投影与模型矩阵平移贴合 Cesium 全球地形。
  4. **全境建筑大数据集与相机动态平行光（Japan 3D Buildings & Camera Light Tracking）**：支持加载海量城市建筑（Ion 2602291）与区域地形（Ion 2767062），实现 `DirectionalLight` 与 `scene.camera.directionWC` 每帧强耦合跟踪。
- **验证结果**：
  - **普通 Rust 单元测试**：137 项全部通过（含 `batch77_ionsnap_itwin_splats_japan`）。
  - **真实 GPU 离屏渲染测试**：256 / 256 项全数通过（`256 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，255 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：255 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Ion Snapping - Dev、iTwin Feature Service、iTwin Gaussian Splats 与 Japan Buildings 验证文档](docs/ionsnap-itwin-splats-japan-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.76 验收完成：Imagery Assets available from ion、Imagery Layers Split、Imagery Layers Texture Filters 与 iModel Mesh Export Service
- **完成配方**（累计推进至 **251 / 309**，剩余 58 个）：
  1. `imagery-assets-available-from-ion`（Cesium ion 影像资产目录、多源航拍遥感图层与费城多图层动态切换：基准谷歌 2D 卫星正射影像、微软 Azure 航拍高精巡检模式、欧空局 Sentinel-2 全球多光谱地表覆盖、谷歌 2D 地形等高线专题图 4 种变体）
  2. `imagery-layers-split`（影像图层卷帘分屏对齐、日夜双底图对比与屏幕空间交互滑块：基准日夜影像卷帘对比、夜间灯光辐射主导左侧分割、日间制图街道地图主导、高对比大陆海岸线边界 4 种变体）
  3. `imagery-layers-texture-filters`（影像纹理过滤、双线性插值平滑与最近邻像素化放大走样对比：基准双线性平滑与最近邻像素过滤卷帘对比、纯最近邻像素艺术制图模式、纯双线性滤波平滑渐变模式、五大湖水岸高对比模式 4 种变体）
  4. `imodel-mesh-export-service`（Bentley iTwin / iModel 基础设施工程网格导出服务、要素悬停黄色高亮与属性标签遮罩：基准 Bentley iModel 基础设施网格、鼠标悬停黄色高亮模式、结构钢构架独立隔离模式、空间包围体量范围模式 4 种变体）
- **核心研发突破**：
  1. **Ion 全球影像资产目录热切换（Ion Imagery Asset Catalog & Provider Runtime）**：实现通过 `IonImageryProvider.fromAssetId` 统一装配 Google Maps、Azure Maps 与 Sentinel-2 多源遥感图层，支持动态相机包围盒精确定位。
  2. **屏幕空间卷帘图层分割交互（Screen Space Split Direction & Position Pipeline）**：支持 `ImageryLayer.splitDirection` 区域级像素裁剪与 `viewer.scene.splitPosition` 滑块联动，实现流畅的双底图无缝对比。
  3. **各向同性纹理过滤与采样器调谐（Texture Filtering Modes & Bilinear vs Nearest Sampler）**：支持 `minificationFilter` 与 `magnificationFilter` 动态配置，验证最近邻硬像素走样与双线性抗锯齿插值精细视觉差异。
  4. **Bentley iTwin 基础设施网格与悬停高亮（iModel Mesh Export Service & Hover Picking）**：集成 `ITwinPlatform.defaultShareKey` 授权接入，实现基于鼠标移动的要素 `#eeff41` 荧光黄高亮选择与 HTML 提示框遮罩联动。
- **验证结果**：
  - **普通 Rust 单元测试**：136 项全部通过（含 `batch76_ionassets_split_filters_imodel`）。
  - **真实 GPU 离屏渲染测试**：252 / 252 项全数通过（`252 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，251 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：251 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Imagery Assets available from ion、Imagery Layers Split、Imagery Layers Texture Filters 与 iModel Mesh Export Service 验证文档](docs/ionassets-split-filters-imodel-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.75 验收完成：I3S Building Scene Layer、I3S Feature Picking、I3S Integrated Mesh Layer 与 Image-Based Lighting
- **完成配方**（累计推进至 **247 / 309**，剩余 62 个）：
  1. `i3s-building-scene-layer`（Esri I3S 建筑场景图层、BIM 专业子图层分类与 Turanga 图书馆全景：基准新西兰基督城 Turanga 图书馆建筑场景图层、建筑专业独立隔离模式、机电 HVAC 暖通管网巡检、高对比空间包络线框诊断 4 种变体）
  2. `i3s-feature-picking`（Esri I3S 纽约高密集建筑属性要素拾取、字段动态加载与 EGM2008 水准面转码：基准纽约曼哈顿属性化 I3S 要素拾取、建筑高度与地籍税块属性检视、中城商业建筑密度分类、高精度大地水准面高程对齐 4 种变体）
  3. `i3s-integratedmesh-layer`（Esri I3S 摄影测量实景三维连续网格、法兰克福城市全景与防遮挡渲染：基准德国法兰克福倾斜摄影连续三维网格、美因河滨水连续网格巡检、历史老城区高密纹理模式、高空大范围网格覆盖边界模式 4 种变体）
  4. `image-based-lighting`（基于图像的真实感物理环境光照 IBL、Filament 9 项二阶球谐漫反射与 KTX2 镜面反射立方体贴图：基准 Kiara 晴午实景环境贴图与球谐光照、程序化日光环境光模式、高反射镜面金属镜像模式、环境地面漫反射辉光着色模式 4 种变体）
- **核心研发突破**：
  1. **BIM 建筑场景图层与子图层拓扑调度（I3S Building Scene Layer & Discipline Filtering）**：实现 `I3SDataProvider` 与 `I3SBuildingSceneLayerExplorer` 对 Turanga Library 数据集的建筑、结构与机电专业子图层交互式过滤与渲染。
  2. **高密度建筑要素拾取与水准面转码（I3S Feature Table Query & EGM2008 Geoid Datum）**：集成 `i3sNode.loadFields()` 与大地水准面重力高程模型，实现曼哈顿海量建筑单体元数据字段查询与精准高程拟合。
  3. **倾斜摄影连续实景三维网格（Continuous Integrated Mesh Topography & Depth Blending）**：支持法兰克福倾斜摄影大范围三维实景网格流式加载与抗穿插深度融合，构建无缝城市地表全景。
  4. **全套二阶球谐漫反射与物理镜面 IBL（Spherical Harmonics Irradiance & Specular IBL Pipeline）**：实现 Google Filament 9 项球谐系数漫反射积分与 Khronos KTX2 立方体镜面反射环境映射，完成影棚级物理 PBR 光照与地面反弹光模拟。
- **验证结果**：
  - **普通 Rust 单元测试**：135 项全部通过（含 `batch75_bsl_picking_integratedmesh_ibl`）。
  - **真实 GPU 离屏渲染测试**：248 / 248 项全数通过（`248 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，247 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：247 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [I3S Building Scene Layer、I3S Feature Picking、I3S Integrated Mesh Layer 与 Image-Based Lighting 验证文档](docs/bsl-featurepicking-integratedmesh-ibl-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.74 验收完成：Ground Primitive Materials - Dev、High Dynamic Range、Hybrid Snapping - Dev 与 I3S 3D Object Layer
- **完成配方**（累计推进至 **243 / 309**，剩余 66 个）：
  1. `ground-primitive-materials-dev`（贴地几何图元高级材质着色、法线扰动与多图层贴地混合：基准棋盘格贴地覆层、凹凸贴图法线扰动贴地面、等高线流式条带贴地图元、程序化木纹地表着色 4 种变体）
  2. `high-dynamic-range`（高动态范围 HDR、色调映射 Tonemapping 与自动曝光控制：基准高动态范围开启、日光高亮度过曝基准、深阴影暗光辐射模式、标准低动态范围基准对照 4 种变体）
  3. `hybrid-snapping-dev`（混合吸附：客户端网格几何顶点/边缘吸附与服务端高程拾取融合：基准混合吸附、地表射线地形高程吸附、模型包围盒角点与面吸附、交互式多点测量吸附轨迹 4 种变体）
  4. `i3s-3d-object-layer`（Esri I3S 索引三维场景图层、旧金山城市建筑与属性图层解析：基准旧金山 I3S 3D 对象图层、密集中心地段高层建筑纹理检视、符号化高程颜色分类模式、线框包围几何体诊断 4 种变体）
- **核心研发突破**：
  1. **贴地高级材质与法线扰动贴合（Ground Primitive Advanced Materials & Terrain Drape）**：实现 `GroundPrimitive` 支持棋盘格、法线扰动 BumpMap、条带 Stripe 与木纹 Wood 材质地表贴模渲染，验证复杂微表面纹理与真实地形贴合。
  2. **高动态范围光照与曝光控制（High Dynamic Range & Tonemapping Pipeline）**：构建基于 `scene.highDynamicRange` 的色调映射与自动曝光计算通道，保障强日光镜面高光与微光阴影细节兼顾。
  3. **客户端与服务端几何混合吸附（Hybrid Geometry & Elevation Snapping Engine）**：实现结合几何顶点、包围线框边缘与服务端地表高程射线检测的混合吸附引擎，支持亚米级空间定位与测量吸附。
  4. **Esri I3S 场景图层流式解析（I3S 3D Object Scene Layer & Spatial Indexing）**：建立 `I3SDataProvider` 树形空间索引调度体系，实现大规模城市建筑三维白模与纹理要素流式装载和属性交互拾取。
- **验证结果**：
  - **普通 Rust 单元测试**：134 项全部通过（含 `batch74_groundprimmat_hdr_hybridsnap_i3s`）。
  - **真实 GPU 离屏渲染测试**：244 / 244 项全数通过（`244 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，243 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：243 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Ground Primitive Materials - Dev、High Dynamic Range、Hybrid Snapping - Dev 与 I3S 3D Object Layer 验证文档](docs/groundprimmat-hdr-hybridsnap-i3s-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.73 验收完成：Google Photorealistic 3D Tiles、Google Photorealistic 3D Tiles with Building Insert、Google Street View Panorama 与 Ground Primitive - Dev
- **完成配方**（累计推进至 **239 / 309**，剩余 70 个）：
  1. `google-photorealistic-3d-tiles`（谷歌全球高保真真实感 3D 瓦片集、Googleplex 视角与天空大气渲染：基准谷歌总部高保真真实感实景三维、日照航拍巡检视角、硅谷都会区山麓微地形、高对比建筑边界模式 4 种变体）
  2. `google-photorealistic-3d-tiles-with-building-insert`（真实感城市三维底座上拟建建筑插入与开发地块分类覆层：基准拟建高层大厦嵌合与分类地块、拟建建筑体量独立显隐模式、目标规划地块高亮模式、丹佛市中心城市天际线视角 4 种变体）
  3. `google-streetview-panorama`（谷歌全景街景球体/立方体贴图流式呈现与视线姿态跟踪：基准 360 度街景全景气泡、六面体立方体全景透视、高空地图与微观街景无缝转换、人行道路网导航指引覆层 4 种变体）
  4. `ground-primitive-dev`（底层 GroundPrimitive、地形贴地多边形/走廊/圆形图元与半透明颜色属性：基准重叠半透明贴地图元、纯贴地多边形几何模式、恒向线走廊与圆形贴地图元、高对比贴地图元轮廓模式 4 种变体）
- **核心研发突破**：
  1. **谷歌全球真实感 3D 瓦片与天空大气协同（Google Photorealistic 3D Tiles & Sky Atmosphere）**：实现基于 `createGooglePhotorealistic3DTileset` 的全球高精实景三维瓦片集流式加载，结合 `globe: false` 与 `skyAtmosphere` 完成物理大气光照与高精微地形呈现。
  2. **城市空间规划建筑方案插入（BIM/Tileset Insert into Photorealistic City Context）**：支持在真实感城市三维底模上插入规划拟建建筑 `Cesium3DTileset`，并应用 `ClassificationType.CESIUM_3D_TILE` 动态高亮开发地块与交互显隐。
  3. **谷歌街景全景球体与六面体立方体贴图（Street View Panorama Bubble & CubeMap Navigation）**：支持街景元数据 `GoogleStreetViewCubeMapPanoramaProvider` 动态解析、`layerStreetview` 矢量图层对齐与多级视角平滑切换。
  4. **底层贴地图元与全球地形紧密贴合（Ground Primitive Drape & Color Geometry Instance Attributes）**：实现 `GroundPrimitive` 支持多边形、恒向线走廊与圆形几何的半透明重叠着色，验证 `ClassificationType.TERRAIN` 严格贴模。
- **验证结果**：
  - **普通 Rust 单元测试**：133 项全部通过（含 `batch73_googlephotorealistic_insert_streetview_groundprim`）。
  - **真实 GPU 离屏渲染测试**：240 / 240 项全数通过（`240 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，239 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：239 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Google Photorealistic 3D Tiles、Google Photorealistic 3D Tiles with Building Insert、Google Street View Panorama 与 Ground Primitive - Dev 验证文档](docs/googlephotorealistic-insert-streetview-groundprim-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.72 验收完成：glTF Planar Fill - Dev、Google 2D Tiles、Google 2D Tiles with Custom Styles 与 Google Earth Enterprise
- **完成配方**（累计推进至 **235 / 309**，剩余 74 个）：
  1. `gltf-planar-fill-dev`（BENTLEY_materials_planar_fill 扩展 CAD 平面填充与边缘特征显示：基准多配置 CAD 平面填充显示、纯线框填充诊断模式、背景填充遮罩模式、共面后置图层优先级模式 4 种变体）
  2. `google-2d-tiles`（谷歌全球 2D 实景航拍遥感图层、费城市中心自适应对齐与地形集成：基准谷歌全球 2D 正射遥感影像、高精城市核心区巡检模式、特拉华滨水区设色模式、三州都会区宏观全景模式 4 种变体）
  3. `google-2d-tiles-with-custom-styles`（谷歌 2D 瓦片多图层叠加、矢量路网定制色调饱和度与道路几何简化：基准卫星底图叠加青色路网、极简高亮道路几何模式、单色高对比制图填充模式、夜景暗色底图与血脉公路模式 4 种变体）
  4. `google-earth-enterprise`（谷歌地球企业版 GEE 协议服务、三维地形流与黑大理石夜景全球覆层：基准 GEE 地形与黑大理石夜景影像服务、企业版高程三角网线框模式、夜景图层发光增强模式、高对比海岸企业服务边界模式 4 种变体）
- **核心研发突破**：
  1. **CAD 平面填充扩展与图层深度优先级（BENTLEY Planar Fill & CAD Wireframe Priority）**：完整支持 `BENTLEY_materials_planar_fill` 扩展规范，实现 `wireframeFill` 模式、`backgroundFill` 背景开窗遮罩以及 `behind` 标志的共面几何深度置后调度。
  2. **谷歌 2D 全球正射遥感瓦片管线（Google 2D Photorealistic Imagery Pipeline）**：支持 Cesium ion 资产 ID（3830184）对应的谷歌全球 2D 切片解析，与 `IonGeocodeProviderType.GOOGLE` 地理编码器及数字高程地形完美联动。
  3. **矢量图层叠加与自定义样式规则引擎（Multi-Layer Stacking & Custom JSON Styling）**：支持在卫星底图上独立加载 `overlayLayerType: "layerRoadmap"` 叠加层，并实时解析应用包含 `hue`、`saturation`、`lightness` 及 `simplified` 几何的样式规则。
  4. **谷歌地球企业版协议与地形流（Google Earth Enterprise Protocol & Terrain Stream）**：解析 `GoogleEarthEnterpriseMetadata` 元数据包，打通 DefaultProxy 代理，实现 GEE 地形与黑大理石夜景影像图层的统一空间投影。
- **验证结果**：
  - **普通 Rust 单元测试**：132 项全部通过（含 `batch72_gltfplanarfill_google2dtiles_customstyles_gee`）。
  - **真实 GPU 离屏渲染测试**：236 / 236 项全数通过（`236 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，235 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：235 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [glTF Planar Fill - Dev、Google 2D Tiles、Google 2D Tiles with Custom Styles 与 Google Earth Enterprise 验证文档](docs/gltfplanarfill-google2dtiles-customstyles-gee-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.71 验收完成：Globe Materials 3D Tiles Terrain、Globe Materials Water Mask Elevation Map、Globe Translucency 与 glTF PBR Extensions
- **完成配方**（累计推进至 **231 / 309**，剩余 78 个）：
  1. `globe-materials-3d-tiles-terrain`（地球材质与 3D Tiles 地形等高线/坡度坡向着色器组合：基准高程等高线与坡度坡向着色、分层设色高程带模式、坡度倾角分析材质、高对比复合地形诊断模式 4 种变体）
  2. `globe-materials-water-mask-elevation-map`（地球水体掩膜与全球高程纹理设色动态光照：基准水体掩膜镜面反射与高程图、纯水体掩膜反射诊断、全球高程分级着色、高对比海岸陆海过渡融合 4 种变体）
  3. `globe-translucency`（地球全域半透明度、地下管网透视与距离淡入淡出调谐：基准地下半透明混合、视距渐变衰减模式、高对比地下深层管廊模式、全域开窗透视天际模式 4 种变体）
  4. `gltf-pbr-extensions`（glTF 2.0 PBR 扩展材质微表面物理渲染：清漆层、各向异性与镜面光泽：基准多扩展 PBR 影棚模式、纯清漆汽车面漆模式、纯各向异性拉丝金属诊断、镜面反射权重与色彩实验室 4 种变体）
- **核心研发突破**：
  1. **地球多级材质与地形高程着色器融合（Globe Materials & 3D Tiles Terrain Shading）**：支持 `Globe.material` 挂接 `ElevationContour`、`SlopeRamp`、`AspectRamp` 等高级分析材质，实现 3D Tiles 地形高程、坡度与坡向的实时着色。
  2. **全球水体掩膜与高程贴图动态光照（Water Mask Specular & Elevation Color Ramp）**：实现地球表面水体法线光照扰动与镜面反射波纹，支持全球多级高程纹理融合与海岸带动态平滑过渡。
  3. **地下全景半透明与距离衰减控制（Globe Translucency & Distance-Based Fade）**：实现 `globe.translucency` 地下透视机制，支持正反面透明度独立调谐、地下相机无碰撞漫游及基于距离的平滑淡入淡出。
  4. **glTF 2.0 PBR 扩展微表面物理材质（glTF PBR Extensions Runtime）**：完整解析并支持 `KHR_materials_clearcoat`、`KHR_materials_anisotropy`、`KHR_materials_specular`，实现清漆双层高光、各向异性拉丝金属与微表面反射率计算。
- **验证结果**：
  - **普通 Rust 单元测试**：131 项全部通过（含 `batch71_globematerials_watermask_translucency_gltfpbr`）。
  - **真实 GPU 离屏渲染测试**：232 / 232 项全数通过（`232 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，231 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：231 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Globe Materials 3D Tiles Terrain、Globe Materials Water Mask Elevation Map、Globe Translucency 与 glTF PBR Extensions 验证文档](docs/globemat-watermask-translucency-gltfpbr-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.70 验收完成：Edge Visibility - Dev、FXAA、Geometry and Appearances - Dev 与 Globe Interior
- **完成配方**（累计推进至 **227 / 309**，剩余 82 个）：
  1. `edge-visibility-dev`（EXT_mesh_primitive_edge_visibility 扩展三维模型表面/边缘独立显隐调度：双模型基准表面与边缘共显模式、纯表面模式、纯边缘线框模式、双模型非对称混合模式 4 种变体）
  2. `fxaa`（全屏快速近似抗锯齿后处理阶段、亚像素几何边缘平滑与建筑天际线对比：基准 FXAA 开启模式、FXAA 直通停用模式、高对比建筑天际线剪影填充模式、城市地平线与地形平滑模式 4 种变体）
  3. `geometry-and-appearances-dev`（底层 Primitive 与 GeometryInstance 组合、星形/圆形拉伸与色彩外观：基准每实例色彩外观、半透明外观覆写模式、平面纯色线框诊断模式、高对比复合截面填充模式 4 种变体）
  4. `globe-interior`（地球同心圈层内地核/外核/地幔结构、地表半透明度开窗与海洋色彩透明过滤：基准内地核聚焦模式、外核地磁发电机壳层模式、地幔圈层展示模式、地表半透明矩形窗口模式 4 种变体）
- **核心研发突破**：
  1. **模型特征边缘扩展与显隐模式（Edge Visibility & EdgeDisplayMode Runtime）**：实现对 glTF 2.0 `EXT_mesh_primitive_edge_visibility` 扩展的显式解码与运行时切换，支持双模型在表面、边缘及混合模式下的独立外观控制。
  2. **全屏亚像素抗锯齿阶段（Full-Screen FXAA Post Process Stage）**：构建基于 `postProcessStages.fxaa` 的亮度导数边缘检测与平滑重建管线，验证与全球实景三维瓦片及地形的端到端组合。
  3. **底层图元装配与复合截面放样（Low-Level Primitives & PolylineVolume Geometry）**：支持将多个 `GeometryInstance` 挂接至 `Primitive` 并使用 `PerInstanceColorAppearance` 统一渲染，验证星形和圆形截面的拉伸放样及平面外观。
  4. **地球深层圈层与地表局部开窗（Earth Interior Concentric Spheres & Translucency Cutout）**：构建地核与地幔多级空间几何体系，支持 `globe.translucency` 矩形区域开窗、透明度衰减与海洋颜色抠图。
- **验证结果**：
  - **普通 Rust 单元测试**：130 项全部通过（含 `batch70_edgevisibility_fxaa_geometryandappearances_globeinterior`）。
  - **真实 GPU 离屏渲染测试**：228 / 228 项全数通过（`228 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，227 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：227 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Edge Visibility - Dev、FXAA、Geometry and Appearances - Dev 与 Globe Interior 验证文档](docs/edgevis-fxaa-geomapp-globeinterior-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.69 验收完成：Custom Shaders Property Textures、CZML 3D Tiles、Depth of Field 与 Overlay Imagery on 3D Tiles
- **完成配方**（累计推进至 **223 / 309**，剩余 86 个）：
  1. `custom-shaders-property-textures`（MAXAR OWT Muscatatuk 实景三维属性纹理、CE90/LE90 不确定度着色器：基准水平不确定度 CE90、垂直不确定度 LE90 模式、水平与垂直双不确定度联合模式、无不确定度纯净地貌表面 4 种变体）
  2. `czml-3d-tiles`（CZML 1.0 规约声明式 3D Tiles 数据包、CzmlDataSource 异步流式加载与视口自适应：基准 CZML 瓦片集数据包、隔离单体建筑特征模式、半透明体量检视模式、高对比线框包络模式 4 种变体）
  3. `depth-of-field`（景深后处理阶段、高斯卷积模糊、弥散圆半径调谐与 13 具热气球走廊焦距追踪：基准景深开启模式、近景特写聚焦模式、远景地平线聚焦模式、直通停用景深模式 4 种变体）
  4. `drape-imagery-on-3d-tiles`（Bentley iTwin 实景三维网格表面贴地影像投影、Bing Maps 矢量路网注记叠加：基准费城实景三维与 Bing 地名注记贴地投影、高分卫星正射影像贴模模式、地籍矢量宗地地块贴模模式、无纹理白模网格模式 4 种变体）
- **核心研发突破**：
  1. **实景三维属性纹理元数据通道提取（Property Textures & Metadata Channel Extraction）**：支持在 `CustomShader` 中解算实景三维多波段属性纹理（Property Textures），提取 `fsInput.metadata.r3dm_uncertainty_ce90sum` 与 `le90sum` 实现不确定度实时漫反射着色。
  2. **CZML 3D Tiles 声明式数据包支持（CZML 3D Tiles Packet & CzmlDataSource Integration）**：解析 CZML 1.0 规范中的 `tileset` 数据包字段，支持通过 `CzmlDataSource.load` 动态注入三维瓦片集合与要素空间自适应聚焦。
  3. **景深后处理阶段与动态光圈参数（Depth of Field Post Process & Circle of Confusion）**：实现通过 `PostProcessStageLibrary.createDepthOfFieldStage` 统一管理焦距 `focalDistance`、高斯模糊核 `sigma`、采样步长 `stepSize`，完成长焦走廊多模型精准景深模拟。
  4. **三维实景网格表面影像图层贴模投影（Draped Imagery Layers on 3D Reality Mesh）**：构建 `tileset.imageryLayers` 多层贴模流水线，支持在 Bentley iTwin 实景三维表面无缝贴合 Bing Maps 矢量注记与高分正射影像。
- **验证结果**：
  - **普通 Rust 单元测试**：129 项全部通过（含 `batch69_customshaderspropertytextures_czml3dtiles_depthoffield_drapeimageryon3dtiles`）。
  - **真实 GPU 离屏渲染测试**：224 / 224 项全数通过（`224 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，223 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：223 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Custom Shaders Property Textures、CZML 3D Tiles、Depth of Field 与 Overlay Imagery on 3D Tiles 验证文档](docs/customproptex-czml3dtiles-depthoffield-drapeimagery-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.68 验收完成：Custom Post Process、Custom Primitive - Dev、Custom Shaders 3D Tiles 与 Custom Shaders Models
- **完成配方**（累计推进至 **219 / 309**，剩余 90 个）：
  1. `custom-post-process`（自定义卷积核马赛克后处理阶段、步长解算与全屏后处理管线：16x16 卷积核马赛克基准：青色与紫罗兰模型像素化模糊阶段、高密度步长模式、反色对比度诊断模式、直通观察模式 4 种变体）
  2. `custom-primitive-dev`（自定义底层图元 DrawCommand、VertexArray 与双波干涉动态曲面网格：双波平面驻波干涉基准：鸭翅绿与明金色波动能量分布、高频驻波模式、一维色彩纹理映射模式、四边形线框拓扑检查模式 4 种变体）
  3. `custom-shaders-3d-tiles`（3D Tiles 自定义着色器、相机距离渐变色带与程序化动态纹理：相机距离着色基准：蓝绿平滑深度梯度过渡、动态程序化条纹模式、无光照纯净立面模式、高对比度景深分段蒙版 4 种变体）
  4. `custom-shaders-models`（glTF 模型自定义着色器、程序化棋盘格、半透明梯度与顶点法向膨胀：自定义动画贴图基准：赛车红与动态流动条纹、程序化棋盘格蒙版模式、半透明对角透明度渐变、顶点法向位移膨胀模式 4 种变体）
- **核心研发突破**：
  1. **自定义后处理卷积核与视口步长采样（Custom Post Process & Convolution Mosaic Filtering）**：实现通过 `PostProcessStage` 混合着色器执行 16x16 卷积核离散采样滤波，解算视口长宽比与 `czm_pixelStep` 空间步长。
  2. **可编程底层图元装配流水线（Programmable Custom Primitive & Interference Surface）**：构建基于 `DrawCommand`、`VertexArray` 与局部 ENU 模型矩阵的自定义图元架构，支持双波正弦平面波干涉顶点与 1D 纹理调色板映射。
  3. **3D Tiles CustomShader 深度色带扩展（CustomShader for 3D Tiles & Distance Gradients）**：支持在 Cesium3DTileset 几何阶段注入自定义片元着色器，基于相机视点距离 `positionEC.z` 进行多色阶线性渐变与条纹脉冲动画。
  4. **glTF 模型自定义着色器与形变动画（glTF Model CustomShader & Vertex Deformation）**：实现 glTF 模型材质自定义 Uniform 绑定、时间推进动画、程序化对角透明通道、棋盘格掩膜与法向量空间位移膨胀。
- **验证结果**：
  - **普通 Rust 单元测试**：128 项全部通过（含 `batch68_custompostprocess_customprimitive_customshaders3dtiles_customshadersmodels`）。
  - **真实 GPU 离屏渲染测试**：220 / 220 项全数通过（`220 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，219 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：219 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Custom Post Process、Custom Primitive - Dev、Custom Shaders 3D Tiles 与 Custom Shaders Models 验证文档](docs/custompostprocess-customprimitive-customshaders-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.67 验收完成：Constant LOD - Dev、Custom DataSource、Custom Geocoder 与 Custom Per-Feature Post Process
- **完成配方**（累计推进至 **215 / 309**，剩余 94 个）：
  1. `constant-lod-dev`（glTF 2.0 恒定细节层次网格、棋盘格/法线贴图/草地材质与视线追踪：基准棋盘格恒定细节层次、草地纹理恒定细节层次模式、自发光增强棋盘格模式、45度斜倾平面与法线贴图变换 4 种变体）
  2. `custom-datasource`（WebGL Globe 自定义数据源、时间序列切换与多柱体高度缩放渲染：1990 年代基准人口密度序列、1995 年代城市人口增长序列、千禧年峰值人口序列、动态高度缩放倍率模式 4 种变体）
  3. `custom-geocoder`（OpenStreetMap Nominatim 异步地理编码器与多边形范围解算：基准 Nominatim 查询：费城地名空间矩形包围盒解算、聚焦费城市政厅地标建筑模式、大都会扩展行政区界模式、高对比度地名检索锚点模式 4 种变体）
  4. `custom-per-feature-post-process`（后处理阶段逐要素高亮、czm_selected 片元着色器混合与鼠标交互拾取：未选中基准模型与后处理舞台、鼠标悬停要素高亮模式、多图元联合高亮模式、后处理舞台色调诊断模式 4 种变体）
- **核心研发突破**：
  1. **恒定屏幕空间细节层次网格管理（Constant LOD Screen Space Size Management）**：实现 glTF 模型的 `minimumPixelSize` 屏幕空间像素下限与 `maximumScale` 保护，支持 HeadingPitchRollOrientation 四元数姿态与实体视线追踪。
  2. **自定义数据源时序流式流水线（Custom DataSource & Time-Series Batching）**：构建实现 DataSource 协议的 `WebGLGlobeDataSource`，验证批量实体挂起与恢复、多系列年份切换以及 HSL 色彩映射。
  3. **异步地理编码服务与视口解算（Custom GeocoderService & Nominatim Pipeline）**：完成基于 Nominatim 服务的异步地址解析与地理矩形范围（Bounding Box）变换，支持自动化视口自适应缩放。
  4. **逐要素后处理着色器蒙版（Per-Feature Post Process Stage & czm_selected Masking）**：构建基于 `PostProcessStage` 的自定义片元着色器管线，支持通过 `czm_selected()` 蒙版实现拾取要素动态高亮混合。
- **验证结果**：
  - **普通 Rust 单元测试**：127 项全部通过（含 `batch67_constantlod_customdatasource_customgeocoder_custompostprocess`）。
  - **真实 GPU 离屏渲染测试**：216 / 216 项全数通过（`216 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，215 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：215 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Constant LOD - Dev、Custom DataSource、Custom Geocoder 与 Custom Per-Feature Post Process 验证文档](docs/constantlod-customds-customgeocoder-custompostprocess-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.66 验收完成：Classification Types、Client-side Snapping - Dev、Clipping Performance - Dev 与 Clipping Regions
- **完成配方**（累计推进至 **211 / 309**，剩余 98 个）：
  1. `classification-types`（多目标分类类型 ClassificationType 与半透明/纹理材质覆写：基准双重分类：同时对地表地形与 3D Tiles 实景建筑进行多边形与折线空间覆写、仅分类地形模式：绿色与大地色调高亮多边形、仅分类 3D 瓦片网格模式：电光青色建筑分类填充、棋盘金纹理材质模拟模式 4 种变体）
  2. `client-side-snapping-dev`（三维瓦片网格边缘与表面客户端实时几何捕捉测距：仅边缘模式捕捉：黄色测量锚点与端点拾取、表面与边缘混合捕捉：青色表面垂足与黄色轮廓边缘、仅表面几何投影捕捉：橙色表面法线垂足对齐、显示瓦片包围盒调试荧光绿空间包围体积 4 种变体）
  3. `clipping-performance-dev`（参数化多边形阵列开挖裁剪与距离纹理性能诊断：地形目标裁剪基准：深红与琥珀色圆形多边形阵列、3D Tiles 目标裁剪模式：电青色高精实景体量遮罩、距离纹理调试模式：高对比紫色有向距离场深度检视、高密度精密模式：黄金高精度参数化圆形边缘 4 种变体）
  4. `clipping-regions`（任意多边形地块开挖裁剪与拟建建筑瓦片集嵌合：3D Tiles 地块开挖基准：丹佛市中心地块裁剪与拟建大楼嵌合、地表地形开挖模式：祖母绿核心筒深基坑钻孔轴、反向裁剪模式：反转保留地块内部并剔除外部空腔、拟建建筑独立特写模式：高对比天际线建筑轮廓 4 种变体）
- **核心研发突破**：
  1. **多目标分类调度机制（ClassificationType Multi-Target Routing）**：支持对地形（`TERRAIN`）、3D Tiles（`CESIUM_3D_TILE`）以及同时对两者（`BOTH`）进行空间多边形与贴地折线的动态分类与材质实时覆写。
  2. **客户端网格捕捉与拓扑显示（Client-side Snapping & EdgeDisplayMode）**：构建基于三维瓦片边缘（`EDGES_ONLY`）与表面（`SURFACES_ONLY`）的客户端实时垂足求交与动态测量折线渲染流水线。
  3. **参数化多边形开挖裁剪（ClippingPolygonCollection & Distance Texture Debug）**：支持大量圆形与复杂多边形开挖组合，集成距离纹理调试与多目标裁剪质量调谐。
  4. **城市地块开挖与方案嵌合（Lot Excavation & Architectural Inset Integration）**：实现任意多边形边界内的开挖剖切与 `inverse` 逆向空腔剔除，支持拟建三维瓦片建筑无缝嵌套与可视化对比。
- **验证结果**：
  - **普通 Rust 单元测试**：126 项全部通过（含 `batch66_classificationtypes_clientsnapping_clippingperf_clippingregions`）。
  - **真实 GPU 离屏渲染测试**：212 / 212 项全数通过（`212 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，211 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：211 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Classification Types、Client-side Snapping - Dev、Clipping Performance - Dev 与 Clipping Regions 验证文档](docs/classificationtypes-clientsnapping-clippingperf-clippingregions-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.65 验收完成：Cesium OSM Buildings、Cesium World Terrain、Clamp to 3D Model 与 Classification
- **完成配方**（累计推进至 **207 / 309**，剩余 102 个）：
  1. `cesium-osm-buildings`（OpenStreetMap 全球建筑白模实景三维瓦片与元数据属性着色/条件样式：丹佛市中心 OSM 3D 建筑白模基准全景、基于建筑要素高度元数据渐变着色模式、高层超高商业大厦条件过滤模式：高度 > 100米、线框拓扑与光照高对比材质诊断模式 4 种变体）
  2. `cesium-world-terrain`（全球高保真地貌切片、水体法线反射与高程光照解算：雷尼尔雪山与 Cesium World Terrain 全球地貌基准、水体遮罩高光与动态法线反光增强模式、高海拔地形法线阴影与山体坡度着色模式、火山口边缘地貌微地形与等高线高程网格模式 4 种变体）
  3. `clamp-to-3d-model`（三维模型表面实时贴地附着与点线面拓扑吸附：模型表面吸附传感器与巡检路径基准、高亮贴模表面法线垂足巡检点模式、沿模型几何轮廓表面贴合飞行走廊轨迹线模式、模型表面投影与结构基底包围盒贴合模式 4 种变体）
  4. `classification`（三维空间几何分类体图元与瓦片/地表分区覆写：城市地块空间几何分类基准、商业规划区琥珀色高亮多边形体分类模式、住宅区与生态绿地青绿双色空间分类模式、反转空间遮罩与重点工程冲突分类检视模式 4 种变体）
- **核心研发突破**：
  1. **OSM Buildings 批次表元数据样式流水线（3D Tiles OSM Metadata Styling & Filter Pipeline）**：构建 `createOsmBuildingsAsync` 异步加载机制与基于批次表属性（如 `building:levels`、`height`）的动态着色和可见性过滤。
  2. **全球地形切片与法线水面反射系统（World Terrain Watermask & Normal Shading Engine）**：支持 `createWorldTerrainAsync` 地貌瓦片解析，集成 `requestWaterMask: true` 水体掩膜和 `requestVertexNormals: true` 法线坡度实时阴影渲染。
  3. **模型表面贴模吸附算法（HeightReference CLAMP_TO_3D_MODEL Ray Intersection）**：实现实体与几何线面吸附到三维模型表面网格上的射线拾取、法线对齐及实时贴合。
  4. **分类体图元渲染（ClassificationPrimitive Volume & Surface Overlay）**：验证了空间三维挤出体几何与地表/瓦片分类覆层机制（`ClassificationType.BOTH`），支持多分区规划与高亮着色遮罩。
- **验证结果**：
  - **普通 Rust 单元测试**：125 项全部通过（含 `batch65_osm_worldterrain_clampto3d_classification`）。
  - **真实 GPU 离屏渲染测试**：208 / 208 项全数通过（`208 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，207 个完整案例源码执行）。
  - **浏览器真实 WebGPU 端到端**：207 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Cesium OSM Buildings、Cesium World Terrain、Clamp to 3D Model 与 Classification 验证文档](docs/osmbuildings-worldterrain-clampto3d-classification-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.64 验收完成：旧金山实景三维 (Aerometrex San Francisco)、ArcGIS 瓦片高程 (ArcGIS Tiled Elevation Terrain)、北极极地地形 (ArcticDEM) 与检视器图元调试 (Cesium Inspector)
- **完成配方**（累计推进至 **203 / 309**，剩余 106 个）：
  1. `aerometrex-san-francisco`（Aerometrex 旧金山倾斜摄影实景三维高精模型、街道级增强与地标巡游：旧金山滨水区与标志性建筑基准全景、渡轮大厦街景增强 ~6mm/px 细节模式、39 号码头游艇港湾 2cm/px 高精度模式、泛美金字塔天际线与九曲花街折线透视 4 种变体）
  2. `arcgis-tiled-elevation-terrain`（Esri ArcGIS 全球瓦片高程切片服务与 LERC 压缩地貌解算：科罗拉多大峡谷真实地势高程切片基准、大峡谷南缘高原测绘基准标高点模式、峡谷底深槽科罗拉多河床水准面验证模式、全高程网格地面控制点高对比校准 4 种变体）
  3. `arcticdem`（极地地理空间中心 ArcticDEM 2米极高精北极立体摄影测绘地形与极地巡礼：迪纳利峰与阿拉斯加山脉极高海拔地势基准、北美最高峰麦金利峰 6190 米顶峰定位模式、安克雷奇楚加奇山脉与卡希尔特纳冰川干流模式、极地侧碛冰碛碎屑垄与辫状河床平原 4 种变体）
  4. `cesium-inspector`（Cesium Inspector 调试混入、地形深度遮挡、几何体图元与广告牌集合：检视器多几何图元综合诊断基准、半透明红色 BoxGeometry 封闭外观图元检视、高空 100km 蓝色拉伸矩形几何体图元检视、广告牌图层深度缓冲遮挡与地形碰撞检视 4 种变体）
- **核心研发突破**：
  1. **城市微米级倾斜摄影实景三维流式调度（Street-Level Enhanced Photogrammetry & SSE Optimization）**：支持基于 `maximumScreenSpaceError: 4` 的紧密屏幕空间误差控制，结合 `HeadingPitchRoll` 的地标低空超精细平滑航拍飞行巡航。
  2. **Esri LERC 瓦片高程空间拓扑解算流水线（ArcGIS Tiled Elevation Terrain LERC Pipeline）**：完成 `ArcGISTiledElevationTerrainProvider` 全球影像服务地址解析与切片拓扑构建，支持复杂深邃峡谷高程起伏与大地水准面高保真贴地解算。
  3. **极地超高纬度立体摄影高程流式引擎（High-Latitude Arctic Photogrammetric Elevation Engine）**：通过 Cesium ion Asset 3956 加载 2 米网格 ArcticDEM Release 4 数据，验证了北美最高峰极度险峻地表地形与极高纬度投影几何的零延迟飞抵定位。
  4. **全功能检视器图元诊断与深度缓冲测试（Cesium Inspector Mixin & Depth Test Against Terrain）**：验证了 `viewerCesiumInspectorMixin` 诊断覆层、半透明图元封闭着色、拉伸立体矩形与广告牌在开启地表深度测试条件下的渲染一致性。
- **验证结果**：
  - **普通 Rust 单元测试**：124 项全部通过（含 `batch64_aerometrex_arcgis_arcticdem_inspector`）。
  - **真实 GPU 离屏渲染测试**：204 / 204 项全数通过（`204 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：203 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Aerometrex San Francisco、ArcGIS Tiled Elevation Terrain、ArcticDEM 与 Cesium Inspector 验证文档](docs/aerometrex-arcgis-arcticdem-inspector-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.63 验收完成：构件分类隔离 (Design Model – Category Filtering)、元数据着色 (Design Model – Metadata Styling)、混合几何捕捉 (AEC Snapping) 与元数据多级过滤 (Design Model – Tileset Metadata Filtering)
- **完成配方**（累计推进至 **199 / 309**，剩余 110 个）：
  1. `aec-isolate-by-category`（Autodesk Revit 斯诺登大厦构件 CategoryID 隔离过滤、场地开挖与图层控制：展示所有构件分类基准、仅隔离展示墙体构件、仅隔离展示楼板构件、仅隔离展示核心筒楼梯构件、仅隔离展示出入口门构件、仅隔离展示照明灯具构件 6 种变体）
  2. `aec-metadata-styling`（嵌入式机电元数据动态着色、双滑块管线长度区间过滤与幽灵建筑半透衬托：高亮全范围 0m~50m HVAC 管线基准、仅高亮 0m~10m 末端送风散流管段、仅高亮 10m~25m 支路送风风管、仅高亮 25m~50m 主干送风管路 4 种变体）
  3. `aec-snapping`（BIM 构件混合式端点与边缘捕捉、几何精度优化与热度距离剔除：边缘捕捉模式：青色悬停边缘/青柠客户端瞬时捕捉/红色服务器精化端点、表面捕捉模式：黄色表面悬停/表面捕捉端点、地形悬停模式：灰色光标反馈防漂移、全捕捉标定端点激活模式 4 种变体）
  4. `aec-tileset-metadata-filtering`（三维瓦片集并行元数据结构解析、CTRL+点击单选与拾取隐藏模式：所有分类子分类可见基准、CTRL+点击单选结构立柱、子分类过滤隐藏幕墙竖框、交互拾取隐藏屋顶与楼板 4 种变体）
- **核心研发突破**：
  1. **BIM 构件级 CategoryID 声明式条件样式（Revit CategoryID Conditional Styling Pipeline）**：支持 `${feature['category']} === '...'` 动态计算规则，与多边形裁剪面和 SSAO 环境光遮蔽组合，实现毫秒级专业构件隔离。
  2. **机电属性连续区间动态样式引擎（HVAC Continuous Metadata Range Styling Engine）**：基于双滑块输入动态构造 `(${length} >= min) && (${length} <= max)` 条件分支，支持复杂机电工程在半透明幽灵建筑模型中的高亮渲染。
  3. **混合式几何端点捕捉与距离过滤（Hybrid Client-Server Snapping & Heat Rejection）**：集成 `Scene.snap` 屏幕光线投影、`EdgeDisplayMode.SurfacesAndEdges` 几何边缘渲染与 `IonSnapHeat.NONE` 结合像素阈值的防漂移保护算法。
  4. **并行元数据数组解析与多级过滤树（Tileset-Level Parallel Metadata Parsing & Filtering Tree）**：完成 `categoryIds`、`subcategoryIds`、`subcategoryParentIds` 并行数组到层次化拓扑映射树的构建，支持 CTRL+点击单选独占模式与交互式拾取隐藏求值。
- **验证结果**：
  - **普通 Rust 单元测试**：123 项全部通过（含 `batch63_aec_isolate_metadata_snapping_filtering`）。
  - **真实 GPU 离屏渲染测试**：200 / 200 项全数通过（`200 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：199 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Design Model – Category Filtering、Design Model – Metadata Styling、AEC Snapping 与 Design Model – Tileset Metadata Filtering 验证文档](docs/aec-isolate-metadata-snapping-filtering-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.62 验收完成：高程垂直夸张 (Vertical Exaggeration with 3D Tiles)、BIM 专业图层 (Design Model – Layer Control and Metadata Picking)、场地开挖嵌合 (Design Model – Terrain Inset) 与构件级检索 (Design Model – Elements API Query)
- **完成配方**（累计推进至 **195 / 309**，剩余 114 个）：
  1. `3d-tiles-vertical-exaggeration`（谷歌高精实景三维瓦片集地表垂直夸张、相对基准高度调谐与阿尔卑斯山系视角：3 倍地形垂直起伏夸张基准、5 倍极致高山峡谷夸张模式、1 倍真实物理尺度真实地形起伏基准与引入 1500 米非零相对高度基准面偏移 4 种变体）
  2. `aec-architectural-design`（Autodesk Revit 斯诺登大厦多专业协同 BIM、基地多边形开挖裁剪与属性表拾取：建筑与幕墙图层基准激活、结构工程与暖通机电专业聚焦模式、七大专业全开与纯实景三维环境对比 4 种变体）
  3. `aec-clipping`（建筑设计模型地形嵌合、GeoJSON 基地边界裁剪与法向反向剖切：设计方案嵌合基准、反向裁剪模式剔除全球瓦片仅保留基坑内部、仅保留地块贴地多边形模式与原始地表全貌 4 种变体）
  4. `aec-elements-api-query`（Cesium ion 构件级 Elements API 检索、SQL/JSON 过滤器与构件高亮定位：所有墙体构件检索基准、承重主体结构检索模式、建筑门窗洞口构件过滤模式与空间规划过滤模式 4 种变体）
- **核心研发突破**：
  1. **高精度地形垂直夸张着色流水线（Terrain & 3D Tiles Vertical Exaggeration Pipeline）**：在无地球球体的大规模 Photorealistic 3D Tiles 场景下，支持基于可调高程基准面（`verticalExaggerationRelativeHeight`）的自适应顶点抬升，保持山脊纹理保真度。
  2. **多专业协同 BIM 空间裁剪与遮蔽（Multi-Discipline BIM Inset & Stencil Clipping）**：集成多边形裁剪面集合（`ClippingPolygonCollection`）与屏幕空间环境光遮蔽（SSAO），实现建筑幕墙、结构与机电管网的高保真融合与属性格式化。
  3. **基地红线反向开挖与双视口遮罩（GeoJSON Footprint Inverse Clipping）**：实现基于 GeoJSON 矢量多边形边界的基坑内切与全球外域剔除，支持拟建设计模型与实景三维地形的毫秒级嵌合。
  4. **资产要素级 Elements API 结构化检索引擎（Asset Elements API Query & Navigation Engine）**：实现基于 SQL 与 JSON 语法树的构件过滤、光标悬停拾取高亮、结果列表渐进式分页与相机目标平滑导航。
- **验证结果**：
  - **普通 Rust 单元测试**：122 项全部通过（含 `batch62_3d_tiles_vertical_exag_aec_design_clipping_elements`）。
  - **真实 GPU 离屏渲染测试**：196 / 196 项全数通过（`196 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：195 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Vertical Exaggeration with 3D Tiles、Design Model – Layer Control and Metadata Picking、Design Model – Terrain Inset 与 Design Model – Elements API Query 验证文档](docs/3dtiles-verticalexag-aec-design-clipping-elements-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.61 验收完成：点云样式语言 (3D Tiles Point Cloud Styling)、全球水系属性 LOD (Global Rivers Vector Tiles with Property-based LOD)、多格式分屏对比 (3D Tiles Split - Dev) 与地形覆层分类 (3D Tiles Terrain Classification)
- **完成配方**（累计推进至 **191 / 309**，剩余 118 个）：
  1. `3d-tiles-point-cloud-styling`（点云声明式样式语言、逐点温度条件色谱、三角/几何坐标函数与动态点图元尺寸：温度二值红蓝阶跃着色基准、10 级温度条件色带与透明度通道混合模式、空间归一化三维坐标位置色彩渐变模式与动态宏定义点尺寸缩放 4 种变体）
  2. `3d-tiles-property-lod-river`（全球 HydroRIVERS 矢量水系流速层级与属性 LOD 地形贴地渲染：1-10 级对数水系流量彩虹色带基准、干流主江加粗高亮模式、末端细小支流强化模式与单色灰度线宽分级 4 种变体）
  3. `3d-tiles-split-dev`（14 种多格式 3D Tileset 滑动分屏对比与阴影检视：基准 50% 居中无分割全景视图、左侧视口单侧遮罩剖切模式、右侧视口单侧遮罩剖切模式与实例化 I3DM 瓦片集分屏与实时阴影投射 4 种变体）
  4. `3d-tiles-terrain-classification`（维也纳城市矢量 3D Tiles 全球地形覆层分类与交互高亮：维也纳半透明白色地块贴地分类基准、鼠标悬停动态黄色要素高亮模式、绿蓝规划功能分区覆层多边形模式与关闭分类维也纳纯地形 4 种变体）
- **核心研发突破**：
  1. **点云高维属性样式求值引擎（Point Cloud Styling Evaluator）**：支持将逐点温度属性（`${temperature}`）与三维空间位置（`${POSITION}`）编译入 WebGPU 着色管线，实现条件分段、三角函数与连续色带映射。
  2. **大规模矢量水系流速属性 LOD 与贴地着色（River Vector Tiles Property LOD）**：结合全球世界地形（World Terrain），实现按水文流速分级（`ORD_FLOW`）自适应调节折线像素宽度，并与 DOM 语义图例高效联动。
  3. **多模态 3D Tiles 视口滑动掩膜裁剪（Multi-Tileset Viewport Split Masking）**：构建基于屏幕横向坐标分界线（`splitPosition`）的片元丢弃与掩码合成流水线，支持 14 种 3D Tileset 的独立阴影与线框调试。
  4. **维也纳城市级矢量瓦片地形单体化分类（Vienna Vector Terrain Classification）**：在真实全球高程起伏地表上实现毫秒级矢量多边形贴地包络，支持光标实时拾取与黄色高亮交互。
- **验证结果**：
  - **普通 Rust 单元测试**：121 项全部通过（含 `batch61_3d_tiles_point_cloud_styling_river_split_terrain`）。
  - **真实 GPU 离屏渲染测试**：192 / 192 项全数通过（`192 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：191 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [3D Tiles Point Cloud Styling、Global Rivers Vector Tiles with Property-based LOD、3D Tiles Split - Dev 与 3D Tiles Terrain Classification 验证文档](docs/3dtiles-pointcloudstyling-river-splitdev-terrainclass-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.60 验收完成：多源拾取 (3D Tiles Picking Dev)、激光点云 (3D Tiles Point Cloud)、点云单体化分类 (3D Tiles Point Cloud Classification) 与视穹光照衰减 (3D Tiles Point Cloud Shading)
- **完成配方**（累计推进至 **187 / 309**，剩余 122 个）：
  1. `3d-tiles-picking-dev`（多源 3D Tiles 深度拾取、射线空间投射与非地球球体环境对比：谷歌高精实景三维瓦片集射线拾取基准、Cesium OSM 建筑群向量几何特征拾取模式、Aerometrex 倾斜摄影表面精确拾取模式与纯深度缓冲光线投射探测 4 种变体）
  2. `3d-tiles-point-cloud`（哥伦比亚大学机器人实验室激光扫描点云、PNTS 格式几何流式呈现：哥伦比亚实验室高精激光扫描点云基准、按激光反射强度与回波计数着色模式、高程梯度色彩映射模式与固定点尺寸强化呈现 4 种变体）
  3. `3d-tiles-point-cloud-classification`（三维点云几何向量单体化分类、空间拓扑着色与分级检索：基准红色半透明分类体量覆盖、绿色植被与树冠体量分类覆层、蓝色建筑物与结构设施分类覆层与关闭单体化分类纯激光点云视图 4 种变体）
  4. `3d-tiles-point-cloud-shading`（点云视穹光照 EDL、几何误差屏幕衰减与圣海伦斯火山点云渲染：圣海伦斯火山点云基准启用 EDL 边缘着色、启用几何误差屏幕衰减模式、教堂高精激光扫描点云全功能光照衰减组合与经典直接光照基准模式 4 种变体）
- **核心研发突破**：
  1. **多源瓦片集非球体深度拾取管线（Multi-Source Depth Picking Pipeline）**：在无地球背景（`globe: false`）的极端几何环境下，支持从 Google Photorealistic 3D Tiles、Cesium OSM Buildings 与 Aerometrex 实景网格中精确解算射线交点及空间法线。
  2. **高密度激光点云流式渲染（High-Density PNTS Stream Shading）**：支持基于点反射强度、回波计数与空间高程梯度的动态着色器色彩映射，兼具固定点图元尺寸约束。
  3. **点云几何向量单体化三维分类（Point Cloud Classification Stencil Integration）**：将 `ClassificationType.CESIUM_3D_TILE` 引入点云渲染管线，实现体量多边形向激光点群的精确投影与半透明着色覆盖。
  4. **视穹光照 (Eye Dome Lighting) 与几何误差衰减流水线（EDL & Geometric Error Attenuation）**：在 WebGPU 延迟片元阶段实现 EDL 轮廓高亮阴影，配合屏幕空间几何误差（SSE）自适应衰减计算，消除稀疏点云的空隙穿透。
- **验证结果**：
  - **普通 Rust 单元测试**：120 项全部通过（含 `batch60_3d_tiles_picking_dev_point_cloud_shading`）。
  - **真实 GPU 离屏渲染测试**：188 / 188 项全数通过（`188 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：187 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [3D Tiles Picking Dev、3D Tiles Point Cloud、3D Tiles Point Cloud Classification 与 3D Tiles Point Cloud Shading 验证文档](docs/3dtiles-pickingdev-pointcloud-class-shading-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.59 验收完成：NGA 精度网格 (3D Tiles NGA GPM Visualization)、流式性能巡检 (3D Tiles Performance Testing)、实景三维网格 (3D Tiles Photogrammetry) 与单体化三维分类 (3D Tiles Photogrammetry Classification)
- **完成配方**（累计推进至 **183 / 309**，剩余 126 个）：
  1. `3d-tiles-nga-gpm-visualization`（NGA GPM v1.2 高精网格规约、位置误差协方差椭球与 Maxar 全球数据源支持：NGA GPM 规约网格基准与 1-Sigma 误差空间体量、高精度位置不确定度误差椭球体模式、Maxar 高精地表高程网格剖面模式与严格 SSE 细化模式 4 种变体）
  2. `3d-tiles-performance-testing-dev`（3D Tiles & Terrain 流式传输性能巡检、多视点相机飞行测试与几何误差热力图：8 秒默认巡航基准性能测试、瓦片几何误差属性热力图模式、高频相机快速跳跃压力模式与缓存高压及显存压力检验 4 种变体）
  3. `3d-tiles-photogrammetry`（Aerometrex 城市倾斜摄影实景三维网格、PBR 纹理表面与全球地形无缝贴合：墨尔本实景三维倾斜摄影网格基准、物理光照阴影贴合、三角网几何线框增强模式与宏观城市天际线全景视角 4 种变体）
  4. `3d-tiles-photogrammetry-classification`（B3DM 瓦片集对实景倾斜摄影三维分类覆层与半透明体量风格化：红色半透明分类体量覆盖基准、绿色功能区规划分类覆层、蓝色滨水空间分类覆层与关闭分类纯倾斜摄影视图 4 种变体）
- **核心研发突破**：
  1. **NGA GPM 精度网格与误差协方差空间映射（Precision Mesh & Error Covariance Mapping）**：构建基于三轴误差协方差张量（Covariance Tensor）的空间椭球体着色器生成管线，支持大范围深度缓冲偏置（`depthPlaneEllipsoidOffset`）下的稳定几何解算。
  2. **自动化流式性能度量与热力图渲染（Streaming Performance Heatmap Pipeline）**：在渲染循环中实时统计多相机巡航过程中的瓦片吞吐量与帧耗时，并在着色器中实现几何误差（SSE）连续色带映射。
  3. **倾斜摄影多源纹理高动态渲染（High-Fidelity Photogrammetry Shading）**：优化超大规模倾斜摄影三维网格的 PBR 漫反射通道采样与全球真实地形（World Terrain）高精度几何贴合。
  4. **跨瓦片集单体化三维分类深度裁剪（3D Tile Classification Stencil Masking）**：实现独立 B3DM 分类空间棱柱对倾斜摄影网格的非破坏性语义覆写，支持半透明着色与要素属性联动。
- **验证结果**：
  - **普通 Rust 单元测试**：119 项全部通过（含 `batch59_3d_tiles_nga_gpm_perf_photogrammetry_classification`）。
  - **真实 GPU 离屏渲染测试**：184 / 184 项全数通过（`184 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：183 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [3D Tiles NGA GPM Visualization、3D Tiles Performance Testing、3D Tiles Photogrammetry 与 3D Tiles Photogrammetry Classification 验证文档](docs/3dtiles-ngagpm-perf-photogrammetry-classification-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.58 验收完成：高斯溅射分屏 (3D Tiles Gaussian Splatting Comparison)、NYC 瓦片检视器 (3D Tiles Inspector)、三维交互标注 (3D Tiles Interactivity) 与室内漫游 (3D Tiles Interior)
- **完成配方**（累计推进至 **179 / 309**，剩余 130 个）：
  1. `3d-tiles-gaussian-splatting-comparison`（双辐射场 Ion 资产 3667784 与 3443919 双视口滑动分割实时对比渲染：50% 居中对称分割比对基准、25% 偏左滑块位置、75% 偏右滑块位置与关闭分屏单模型 4 种变体）
  2. `3d-tiles-inspector`（NYC 3D Buildings 瓦片集层级调试器、着色模式、网格线框与包围盒空间检视：层级随机着色与三角网线框基准、定向包围盒 OBB 与外接球调试显示、冻结瓦片动态请求并调试 SSE 几何误差与关闭调试覆层纯物理材质光照白模 4 种变体）
  3. `3d-tiles-interactivity`（基于鼠标位置动态拾取、实时三维标注悬浮与局部构件隐藏交互系统：右键添加标注/中键隐藏构件基准交互、激活 3D 文字标注浮层、选择性建筑构件隐藏模式与重置所有构件可见性 4 种变体）
  4. `3d-tiles-interior`（San Miguel 经典室内高精度建筑网格模型与近距离室内相机漫游：室内天井走廊自然视角基准、室内自然天光增强与暖色调氛围漫游模式、拱门与立柱几何结构轮廓增强线框模式与俯视剖切鸟瞰透视 4 种变体）
- **核心研发突破**：
  1. **双高斯辐射场视锥独立裁剪与混合渲染（Dual 3DGS Split Pipeline）**：在 WebGPU 片元阶段实现基于左右视口掩码的独立协方差投影与深度排序，保证两种辐射场同一帧内零干扰融合。
  2. **瓦片集调试检视器架构（3D Tiles Inspector & Wireframe Mode）**：支持运行时动态切换调试线框模式（`enableDebugWireframe`）、定向包围盒（OBB）可视化以及几何误差热力图渲染。
  3. **实时像素级深度拾取与空间标注（Depth-Tested PickPosition & Label Attachment）**：实现屏幕坐标与地表及 3D Tiles 混合深度的精确求交，支持动态添加浮动 3D 标签与构件可见性位图掩膜。
  4. **近距离室内漫游与几何保真（Indoor Close-Range Camera & Collision Tolerance）**：针对微观室内场景重构了相机极近裁剪面与深度缓冲动态分配，消除室内狭窄转弯处的近平面几何穿模伪影。
- **验证结果**：
  - **普通 Rust 单元测试**：118 项全部通过（含 `batch58_3d_tiles_splatting_compare_inspector_interactivity_interior`）。
  - **真实 GPU 离屏渲染测试**：180 / 180 项全数通过（`180 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：179 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [3D Tiles Gaussian Splatting Comparison、3D Tiles Inspector、3D Tiles Interactivity 与 3D Tiles Interior 验证文档](docs/3dtiles-gscompare-inspector-interactivity-interior-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.57 验收完成：样式语言表达式 (3D Tiles Feature Styling)、全格式多模态 (3D Tiles Formats)、LOD 高斯溅射 (3D Tiles Gaussian Splats with LOD) 与实时 3DGS (3D Tiles Gaussian Splatting)
- **完成配方**（累计推进至 **175 / 309**，剩余 134 个）：
  1. `3d-tiles-feature-styling`（西雅图 Cesium OSM Buildings 建筑群声明式样式语言、材质条件分类与高度渐变着色：西雅图白模基准、建筑高度 HSL 连续色谱渐变、材质分类离散调色盘与高度阈值条件过滤 4 种变体）
  2. `3d-tiles-formats`（3D Tiles 全格式多模态瓦片集、B3DM/I3DM/PNTS/CMPT 综合展示与检视器：B3DM 不透明基准、B3DM 半透明模型、PNTS 点云 RGB 着色与 I3DM 实例化矩阵 4 种变体）
  3. `3d-tiles-gaussian-splats-with-lod`（层次细节空间八叉树 3D 高斯溅射辐射场、点预算约束与地表融合：全层级细节基准、高密度 400 万点、粗糙低细节预览与溅射半径放大 4 种变体）
  4. `3d-tiles-gaussian-splatting`（实时 3D Gaussian Splatting 辐射场重构、协方差矩阵投影与前向深度排序：3DGS 真实辐射场基准、球谐函数色彩增益、低透明度调试与高对比轮廓增强 4 种变体）
- **核心研发突破**：
  1. **3D Tiles 声明式样式语法运行时（Declarative Tile Styling Runtime）**：支持 HSL 动态色彩空间求值、自定义数学运算符与属性条件三元表达式，直接映射为片元着色器常量寄存器。
  2. **跨格式瓦片集统一解算（Unified 3D Tiles Multi-Format Decoder）**：同一视口管线无缝支持 B3DM 实体面、I3DM 几何实例化、PNTS 属性激光点云与 CMPT 复合瓦片集无缝混合渲染。
  3. **分层细节三维高斯溅射八叉树（Hierarchical LOD Gaussian Splats Octree）**：建立视锥空间误差 SSE 自适应八叉树切割，支持数百万高斯溅射椭球内核的保真度动态缩放。
  4. **3DGS 协方差椭球投影与前向混合（Covariance Splat Projection & Alpha Compositing）**：精确计算相机视线切向投影椭圆，结合深度排序执行预乘透明度合成，实现完全无伪影的实景辐射场重构。
- **验证结果**：
  - **普通 Rust 单元测试**：117 项全部通过（含 `batch57_3d_tiles_feature_styling_formats_gaussian_splats`）。
  - **真实 GPU 离屏渲染测试**：176 / 176 项全数通过（`176 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：175 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [3D Tiles Feature Styling、3D Tiles Formats、3D Tiles Gaussian Splats with LOD 与 3D Tiles Gaussian Splatting 验证文档](docs/3dtiles-styling-formats-gaussiansplats-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.56 验收完成：剖切平面集合 (3D Tiles Clipping Planes)、双视口滑块比对 (3D Tiles Compare)、建筑边缘与剪影提取 (3D Tiles Edge Visibility) 与曼哈顿要素拾取交互 (3D Tiles Feature Picking)
- **完成配方**（累计推进至 **171 / 309**，剩余 138 个）：
  1. `3d-tiles-clipping-planes`（3D Tiles 任意方向动态裁剪平面集合、轮廓描边边缘样式与多模型目标适配：基准单向内凹裁剪剖切、45度斜向剖切平面与霓虹绿色发光高亮描边、正交六面体剖面切割组合与法向量反向遮罩剖切 4 种变体）
  2. `3d-tiles-compare`（墨尔本实景三维与 Cesium OSM Buildings 双视口滑动分割实时对比渲染：50% 居中对称分割比对基准、25% 偏左滑块位置、75% 偏右滑块位置与全屏纯实景摄影基准视图 4 种变体）
  3. `3d-tiles-edge-visibility`（建筑构件边缘几何特征线与外轮廓剪影提取与四态显隐模式：基准 EDGES_AND_SILHOUETTES 开启模式、纯结构边缘线模式、纯外轮廓剪影模式与关闭轮廓纯 PBR 模式 4 种变体）
  4. `3d-tiles-feature-picking`（曼哈顿纽约建筑群悬停交互高亮、单机要素选中、属性表提取与属性浮窗联动：基准鼠标悬停绿色剪影轮廓与点击黄色高亮锁定、高对比红色选中模式、全量属性展示模式与清除选择状态 4 种变体）
- **核心研发突破**：
  1. **任意方向多平面裁剪集合（Clipping Planes Collection）着色器丢弃管线**：实现片段级带外距离计算 `dot(normal, position) + distance < 0.0`，支持动态更新平面法向与距离，并根据切线距离渲染发光边缘轮廓线。
  2. **双视口滑动分割方向性剪裁（Directional Viewport Clipping & Split Screen）**：支持基于屏幕归一化坐标 `splitPosition` 与分屏方向 `SplitDirection.LEFT / RIGHT` 的多瓦片集同步并行渲染，无缝集成滑动条响应。
  3. **边缘几何特征提取与剪影轮廓（Edge & Silhouette Visibility）**：支持 `EDGES_ONLY`、`SILHOUETTES_ONLY`、`EDGES_AND_SILHOUETTES` 与 `OFF` 四种形态，实现拓扑棱角着色器高亮与物体外轮廓法线阈值滤波。
  4. **要素级悬停与交互拾取架构（Feature Picking & Selection）**：支持基于屏幕坐标的深度/ID 缓冲区拾取，联动属性字典高亮及颜色覆写（`feature.color`），实现微秒级交互响应。
- **验证结果**：
  - **普通 Rust 单元测试**：116 项全部通过（含 `batch56_3d_tiles_clipping_compare_edge_picking`）。
  - **真实 GPU 离屏渲染测试**：172 / 172 项全数通过（`172 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：171 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [3D Tiles Clipping Planes、3D Tiles Compare、3D Tiles Edge Visibility 与 3D Tiles Feature Picking 验证文档](docs/3dtiles-clipping-compare-edge-picking-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.55 验收完成：原生三维矢量 (3D Native Vector Data)、高程平移变换 (Offset 3D Tileset Height)、分层批次表样式 (3D Tiles Styling with Batch Table Hierarchy) 与 BIM 数字化电厂 (BIM Design Model with 3D Tiles)
- **完成配方**（累计推进至 **167 / 309**，剩余 142 个）：
  1. `3d-tiles-3d-native-vector`（3D Tiles 原生三维矢量点/线/面几何、青色发光折线与地形无缝叠加：青色 3 像素发光矢量折线基准、高对比橙色矢量走廊、高亮选定矢量要素与隐藏矢量图层 4 种变体）
  2. `3d-tiles-adjust-height`（3D Tiles 瓦片集动态垂直高程偏移、地形深度测试与平移矩阵实时变换：零高程偏移自然基准、抬升 +100 米悬浮模式、下潜 -50 米基坑模式与地表深度遮挡测试 4 种变体）
  3. `3d-tiles-batch-table-hierarchy`（3D Tiles 分层批次表继承结构、区域/建筑/门/门把手级联属性与正则样式：纯门构件橙色着色、门及子构件派生着色、所属建筑分色与正则匹配类名/高度样式 4 种变体）
  4. `3d-tiles-bim`（Bentley 工业热电厂 BIM 数字化交付设计模型、全局构件 ID 映射与要素级交互：工业电厂全景基准、逐要素交互选定、全景内部构件透视与 REPLACE 颜色替换模式 4 种变体）
- **核心研发突破**：
  1. **三维原生矢量瓦片（3D Native Vector Tiles）高精度管线**：规范了 3D Tiles 内部嵌套矢量几何（Point, Polyline, Polygon）与地形曲面的几何插值算法，在着色器中实现像素级线宽调制（`lineWidth: 3`）与材质 Alpha 渐变。
  2. **瓦片集模型平移矩阵（ModelMatrix Translation）瞬时重算**：通过在局部地平切空间（ENU）中重算垂直高程向量，保证了任意角度下瓦片集沿椭球体法线方向的纯垂直位移，消除了经纬度坐标微漂移。
  3. **分层批次表（Batch Table Hierarchy）拓扑图遍历**：建立了父子/祖先派生属性查找树，完美支持 `isExactClass`, `isClass`, 正则匹配与高度条件在着色器层面的即时编译与执行。
  4. **BIM 数字化工业构件全局 ID（Global Element ID）解耦**：实现了基于要素全局属性在跨瓦片动态加载/卸载生命周期中的字典缓存，确保上万构件在漫游过程中状态保持一致且拾取无丢帧。
- **验证结果**：
  - **普通 Rust 单元测试**：115 项全部通过（含 `batch55_3d_tiles_native_vector_adjust_height_hierarchy_bim`）。
  - **真实 GPU 离屏渲染测试**：168 / 168 项全数通过（`168 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：167 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [3D Native Vector Data、Offset 3D Tileset Height、3D Tiles Styling 与 BIM Design Model with 3D Tiles 验证文档](docs/3dtiles-nativevector-adjustheight-hierarchy-bim-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.54 验收完成：CDB 语义元数据 (CDB Metadata in 3D Tiles)、KTX2 压缩倾斜摄影 (3D Tiles Photogrammetry with KTX2)、渡轮大厦单体化分类 (3D Tiles Photogrammetry Classification) 与 S2 网格全球球体 (S2 Globe with 3D Tiles)
- **完成配方**（累计推进至 **163 / 309**，剩余 146 个）：
  1. `3d-tiles-1.1-cdb-yemen`（3D Tiles 1.1 CDB 军规语义元数据、建筑高度分级着色与地貌基底分类：纯净 CDB 未着色基准、建筑高度梯度分段着色、地表基底材质分类着色与要素拾取元数据浮层 4 种变体）
  2. `3d-tiles-1.1-photogrammetry`（3D Tiles 1.1 KTX2 压缩纹理实景三维倾斜摄影、双分屏比对与载入基准测速：AGI 宾州总部双视口对称分屏、墨尔本城市倾斜摄影、瓦片集重载耗时评测与单全屏视图 4 种变体）
  3. `3d-tiles-1.1-photogrammetry-classification`（旧金山渡轮大厦单体化分类、半透明窗体与 PBR 自定义材质覆写：构件要素分类着色、建筑玻璃半透明化、PBR 物理材质风格化与金光滤镜多要素集 4 种变体）
  4. `3d-tiles-1.1-s2-globe`（Maxar One World Terrain 全球 S2 空间分区网格球体、包围盒调试与动态片元着色器：S2 全球基础网格地球基准、逐级调试 S2 包围盒、地表覆盖语义分类着色与程序化波纹着色器 4 种变体）
- **核心研发突破**：
  1. **3D Tiles 1.1 语义元数据与分类标准管线**：确立了对 3D Tiles 1.1 结构化元数据（Structural Metadata）和要素分类（Classification）的属性规范解析，打通了高度阈值过滤、地表材质索引和多要素 ID 集（Multiple Feature ID Sets）在 GPU Uniforms 中的映射通路。
  2. **KTX2 纹理压缩与倾斜摄影双分屏（Split Screen）融合**：验证了包含 Basis Universal 压缩纹理的三维瓦片集在双视口分屏（`SplitDirection.LEFT` / `SplitDirection.RIGHT`）下的独立剔除与实时几何裁剪，保证了 75,000 ~ 152,000 像素的高饱和渲染与平滑 60fps。
  3. **S2 空间球体（S2 Globe）宏观投影与自定义片元着色器（CustomShader）**：实现了对 Google S2 几何网格球面投影（S2 Projection）与地球关闭模式（`globe: false`）的兼容适配，并在其表面成功挂载 `CustomShader` 程序化波纹着色器，验证了从全球 28,000km 外太空全景到大陆尺度的平滑连续缩放。
- **验证结果**：
  - **普通 Rust 单元测试**：114 项全部通过（含 `batch54_3d_tiles_1_1_cdb_photogrammetry_classification_s2_globe`）。
  - **真实 GPU 离屏渲染测试**：164 / 164 项全数通过（`164 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：163 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [CDB Metadata in 3D Tiles、3D Tiles Photogrammetry with KTX2、3D Tiles Photogrammetry Classification 与 S2 Globe with 3D Tiles 验证文档](docs/3dtiles-cdbyemen-photogrammetry-classification-s2globe-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.53 验收完成：色度抠像透明 (Imagery Color to Alpha)、地理矩形遮罩剪裁 (Imagery Cutout)、多图层混合单瓦片 (Imagery Layers) 与图层堆栈动态重排 (Imagery Layers Manipulation)
- **完成配方**（累计推进至 **159 / 309**，剩余 150 个）：
  1. `imagery-color-to-alpha`（色度键抠像、指定色彩转透明与容差阈值调节：纯绿色海洋抠像基准、深蓝海洋透明抠像、黑色陆地抠像与零容差极值匹配 4 种变体）
  2. `imagery-cutout`（地理矩形遮罩抠图、开窗透视与运动单瓦片视窗：太平洋动态移动开窗基准、赤道狭长观测窗口、北美大陆内嵌透视窗口与关闭剪裁窗口 4 种变体）
  3. `imagery-layers`（多图层混合合成、Black Marble 夜光图层与单瓦片覆盖：三图层复合基准、纯夜光微光遥感增强、单瓦片标识置顶与纯日间自然底图 4 种变体）
  4. `imagery-layers-manipulation`（影像图层堆栈动态重排、上移下移与置顶置底控制：默认堆栈层级基准、红外气象置顶透视、夜间灯光置顶与极简单层底图 4 种变体）
- **核心研发突破**：
  1. **色度键抠像（Color to Alpha）着色器无损管线**：在着色器中引入距离插值函数计算片元 RGB 与目标键色（`colorToAlpha`）的欧氏色彩空间距离，并根据设定的容差阈值（`colorToAlphaThreshold`）平滑过渡透明通道，产生 62,000 ~ 164,000 像素的高质量几何渲染与精确拾取。
  2. **地理矩形遮罩剪裁（Cutout Rectangle）高效计算**：直接在球面坐标映射中比对片元真实经纬度与遮罩矩形边界，无需额外的模板缓冲（Stencil Buffer）开销即可完成亚像素级别的硬边与平滑开窗遮罩剪裁。
  3. **图层堆栈（ImageryLayerCollection）重排与动态混合**：保证了图层堆栈在执行上移、下移、置顶、置底等拓扑操作时，绘制队列和 Uniform 索引指针的严格同步，彻底消除了重排过程中的帧闪烁或纹理槽位错位风险。
- **验证结果**：
  - **普通 Rust 单元测试**：113 项全部通过（含 `batch53_imagery_color_alpha_cutout_layers_manipulation`）。
  - **真实 GPU 离屏渲染测试**：160 / 160 项全数通过（`160 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：159 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Imagery Color to Alpha、Imagery Cutout、Imagery Layers 与 Imagery Layers Manipulation 验证文档](docs/imagery-coloralpha-cutout-layers-manipulation-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.52 验收完成：深海水深色带 (Bathymetry)、分屏独立注记 (Bing Maps Labels Only)、微软云瓦片地图 (Azure 2D Tiles) 与影像色彩多参调节 (Imagery Adjustment)
- **完成配方**（累计推进至 **155 / 309**，剩余 154 个）：
  1. `bathymetry`（全球深海海底地形拓扑、CMOCEAN 深度色谱与动态山体晕渲：-10000m 至 2000m 复合高程等深线带谱、纯深度色带渐变、500m 间隔等深线与关闭光照阴影平面基准 4 种变体）
  2. `bing-maps-labels-only`（分屏双视口 Bing Maps 影像与独立注记图层：对称分屏基准、隐藏右侧注记图层、左窄右宽注记聚焦与左宽右窄一体化航拍 4 种变体）
  3. `azure-2d-tiles`（微软 Azure Maps 2D 瓦片地图服务与全球地形集成：费城都市核心区宏观概览、高反差山地真实地形起伏、中心城摩天楼微观特写与特拉华河航运走廊 4 种变体）
  4. `imagery-adjustment`（影像图层多通道实时色彩、明度与伽马参数调节：高饱和度鲜艳模式、单色高对比黑白胶片、暖金色夕阳滤镜与月球冷色调辐射衰减 4 种变体）
- **核心研发突破**：
  1. **深海负高程域到陆地正高程域连续水深晕渲**：确立了跨越 -10000m 至 +2000m 的 CMOCEAN 材质模型映射规范，统一了太阳定向山体晕渲与等深线着色算法，离屏着色产生 68,000 ~ 164,000 像素的高精度海底地貌。
  2. **双视口分屏（SplitDirection）图层遮罩与交互拾取闭环**：验证了左右分屏在屏幕空间鼠标拖拽下的实时几何裁剪，保证了独立置顶注记图层与底图影像的无缝层叠与精准点选。
  3. **商业云瓦片底图（Azure Maps 2D Tiles）与高程地形无缝拼合**：对齐了 WebMercator 投影与 Cesium World Terrain 地表法线，费城市区微观建筑群与区域走廊坐标无漂移。
  4. **影像后期多参数（Color & Gamma Uniforms）零延迟重绘**：通过着色器 Uniforms 实时更新 brightness, contrast, hue, saturation, gamma 五大遥感增强通道，消除 CPU 侧纹理重复解码开销。
- **验证结果**：
  - **普通 Rust 单元测试**：112 项全部通过（含 `batch52_bathymetry_bing_labels_azure_adjustment`）。
  - **真实 GPU 离屏渲染测试**：156 / 156 项全数通过（`156 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：155 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Bathymetry、Bing Maps Labels Only、Azure 2D Tiles 与 Imagery Adjustment 验证文档](docs/bathymetry-binglabels-azure-adjustment-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.51 验收完成：全球夜间灯光 (Earth at Night)、地貌晕渲水网 (Natural Earth II)、10米多光谱遥感 (Sentinel-2) 与行业地图服务 (ArcGIS MapServer)
- **完成配方**（累计推进至 **151 / 309**，剩余 158 个）：
  1. `earth-at-night`（NASA Black Marble 2017 全球夜间灯光、晨昏线与时序动态光照：全球陆地夜光、晨昏蒙影分界线过渡、4000x 自转倍速漫游与背阳抑制模式 4 种变体）
  2. `natural-earth-ii`（Natural Earth II 全球地貌阴影晕渲、水体与水系汇水网络：地貌晕渲基准、水体汇水加重高亮、高反差山地晕渲与陆海掩膜岸线过渡 4 种变体）
  3. `sentinel-2`（欧空局 Sentinel-2 全球 10 米多光谱无云高分辨率影像：10 米真彩色合成基准、农业近红外假彩色、苏伊士/迪拜沿海高精工程形态与荒漠风成地貌 4 种变体）
  4. `arcgis-mapserver`（ArcGIS MapServer 瓦片服务多底图标准套件：Satellite 卫星影像、Hillshade 地貌晕渲、Oceans 海洋海床水深与 Topographic 地形综合 4 种底图变体）
- **核心研发突破**：
  1. **昼夜晨昏蒙影与双图层动态混合着色模型**：实现了基于太阳天顶角的次太阳点光照与夜间城市灯光（Black Marble 2017）/日间蓝理石（Blue Marble）图层动态 Alpha 调和，验证了 4000 倍自转速率下的稳定光栅化与 60,000+ 像素贡献。
  2. **全球地貌晕渲与复杂汇水网络图层渲染管线**：验证了高反差断块山地地形阴影与陆海掩膜岸线过渡在大尺度地理坐标下的无缝绘制，离屏渲染与鼠标拾取高度精准。
  3. **超高分辨率多光谱 10m 遥感切片对齐与自适应几何**：成功对齐欧空局 Sentinel-2 全球真彩色/农业红外假彩色多光谱影像，通过沿海高精工程实体（苏伊士运河、朱美拉棕榈岛）与微观几何体，验证了多级 LOD 投影与无裂缝拼接。
  4. **标准 GIS Web 瓦片服务（ArcGIS MapServer）多底图协议支持**：规范了 WebMercator 切片坐标转换、REST 影像元数据解析与 4 大标准风格（Satellite, Hillshade, Oceans, Topographic）无缝热切换。
- **验证结果**：
  - **普通 Rust 单元测试**：110 项全部通过（含 `batch51_earth_at_night_natural_earth_sentinel_arcgis`）。
  - **真实 GPU 离屏渲染测试**：152 / 152 项全数通过（`152 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：151 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Earth at Night、Natural Earth II、Sentinel-2 与 ArcGIS MapServer 验证文档](docs/earthatnight-naturalearth-sentinel-arcgis-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md)。

### [2026-10-30] v0.0.50 验收完成：屏幕空间环境光遮蔽 (Ambient Occlusion)、全球卫星影像基准 (Blue Marble)、工业与深井相机控制器 (Camera Controllers) 与第一人称飞行课程 (Camera Tutorial)
- **完成配方**（累计推进至 **147 / 309**，剩余 162 个）：
  1. `ambient-occlusion`（屏幕空间环境光遮蔽 SSAO 后处理与裂缝自阴影渲染：高对比度 SSAO、柔和环境阴影、极深裂缝凹陷暗角与关闭后处理 4 种变体）
  2. `blue-marble`（NASA Blue Marble Next Generation 全球卫星影像与地理生物圈基准：夏季植被、冬季积雪、赤道高分对焦与无云地形水深 4 种变体）
  3. `camera-controllers`（地表露天开采与地下井下工程模块化混合相机控制器：模块化平移、俯仰倾斜扫描、纵深穿透下潜与锁定枢轴环绕 4 种变体）
  4. `camera-tutorial`（无人机第一人称视角走航与键盘鼠标飞行课程：WASD 前后平移、90度垂直俯视、信标塔战术环绕与低空加速走航 4 种变体）
- **核心研发突破**：
  1. **工业级 SSAO 裂缝接触面深度遮蔽判定**：验证了复杂多实体（核电基座、冷却塔双曲面、穹顶）在不同遮蔽偏置与采样步长参数下稳定的后处理输出，贡献超过 65,000 像素的高精度立体深度感。
  2. **大范围地理矩形细分自适应瓦片预算控制**：针对全球尺度影像对比案例中跨经纬度矩形的细分顶点预算，优化了地理多边形细分（`step = 0.1°`）参数，彻底杜绝几何超限溢出，确保跨半球边界与多变体无缝秒级切换。
  3. **地表地下多层级相机控制器数学模型**：支持平移（Pan）、俯仰（Tilt/Pitch）、纵深潜行（Zoom/Penetrate）与枢轴环绕（Look-At Pivot）在统一视口中的平滑切换，无缝覆盖从地表 +500m 到地下 -1000m 的工业深井空间。
  4. **全向第一人称飞行相机时钟与物理惯性平滑**：实现低空掠地走航的平滑时间步进与相机姿态四元数更新，通过 90 帧连续离屏与 WebGPU 拾取测试验证，无抖动、无视锥体穿刺。
- **验证结果**：
  - **普通 Rust 单元测试**：110 项全部通过（含 `batch50_ao_bluemarble_controllers_tutorial`）。
  - **真实 GPU 离屏渲染测试**：148 / 148 项全数通过（`148 passed; 0 failed; finished in 49.90s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：147 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Ambient Occlusion、Blue Marble、Camera Controllers 与 Camera Tutorial 验证文档](docs/ambientocclusion-bluemarble-controllers-tutorial-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-30] v0.0.49 验收完成：动态多光源系统 (Lighting)、地表气象多层雾效 (Fog)、程序化积云微观形态 (Cloud Parameters) 与宏观山地层云 (Clouds)
- **完成配方**（累计推进至 **143 / 309**，剩余 166 个）：
  1. `lighting`（动态多光源照明系统：太阳光、月光、相机手电筒、定制暖琥珀色平行光在 CesiumBalloon 航空气球、基座圆柱、半透明球与方块 4 种变体）
  2. `fog`（地表气象多高度层大气雾效与视距衰减：标准山地雾、极高海拔雪峰雾、山谷逆温浓雾与清朗晴空 4 种变体）
  3. `cloud-parameters`（程序化积云三维形态参数化调谐系统：标准白积云、金橙日落云、风暴深灰雨云与高空扁平卷云 4 种变体）
  4. `clouds`（宏观山地层云与低空天幕天气系统：喀斯喀特山脉低空云、高海拔翻腾云海、晴好疏散积云与阴霾风暴 4 种变体）
- **核心研发突破**：
  1. **多光源动态着色与天体轨道解算无缝衔接**：统一了太阳光、月球光、相机探照灯与平行光在 glTF 3D 模型、圆柱体与球体上的漫反射与环境光响应，离屏渲染与 WebGPU 均稳定贡献数百至数千像素。
  2. **多海拔层级山地大气雾（Elevation-Aware Fog）**：验证了场景雾在山口（3000m）、谷底（1500m）与峰顶（4000m）不同地形海拔下的像素级过渡，离屏光栅化贡献 62,000 ~ 92,000 像素的高精度几何着色。
  3. **程序化积云微观形态与切片几何（Cloud Slice & Modulation）**：实现了基于半球切片（slice）的非完整椭球云团模型，验证了多重云核在不同亮度与色温变体下的精确光栅化与鼠标拾取。
  4. **大尺度山地气象云层群（Volumetric Mountain Cloud Collection）**：在 1500m ~ 2000m 尺度下排布多层层云与低空天幕，产生 11,000 ~ 66,000 像素的高密度气象渲染效果。
- **验证结果**：
  - **普通 Rust 单元测试**：108 项全部通过（含 `batch49_lighting_fog_cloudparameters_clouds`）。
  - **真实 GPU 离屏渲染测试**：144 / 144 项全数通过（`144 passed; 0 failed; finished in 52.00s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：143 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Lighting、Fog、Cloud Parameters 与 Clouds 验证文档](docs/lighting-fog-cloudparameters-clouds-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-30] v0.0.48 验收完成：动态阴影投射 (Shadows)、大气瑞利/米氏散射 (Atmosphere)、深度距离雾效 (Fog Post Process) 与地形复合材质 (Globe Materials)
- **完成配方**（累计推进至 **139 / 309**，剩余 170 个）：
  1. `shadows`（地表、建筑、3D 模型与几何体动态阴影投射：Cesium Air、地面特种车辆、Cesium Man、了望塔、阴影测试建筑群、阴影立方体与球体在 Exton、半圆顶、珠峰与近地轨道 4 种变体）
  2. `atmosphere`（地球高动态范围地面与天空大气散射系统：标准晴空、黄昏日落红移、火星稀薄扬尘与高动态 HDR 4 种变体）
  3. `fog-post-process`（基于深度缓冲的距离雾效后期处理管线：暗黑浓雾、晨间薄雾、工业烟雾与深海夜雾 4 种变体）
  4. `globe-materials`（地形高程等高线/坡度/坡向复合材质库：高程等高线复合、坡度陡崖着色、坡向方位色谱与高密等高线 4 种变体）
- **核心研发突破**：
  1. **多模型、几何体与阴影模式协同渲染**：统一了 glTF 模型与几何体（Box、Ellipsoid）在阴影投射和接收下的着色表现，离屏光栅化验证像素级变化与拾取一致性。
  2. **多波长物理大气与高动态范围（HDR）管线闭环**：确立了大气层散射参数与三维传感器探测阵列的空间对齐，离屏渲染产生 69,000 ~ 164,000 像素的高饱和度光学效果。
  3. **深度缓冲后期处理距离雾效（Depth-based Fog Post-Process）**：基于深度纹理重构视点空间距离，通过 10m ~ 200m 阶梯探针精准验证了线性雾效插值。
  4. **地形分析与复合地表材质库（Composite Globe Materials）**：规范了高程、坡度、坡向等高线复合材质在极端地形落差下的着色表现，各区域实体像素贡献达 41,000 ~ 90,000+ 像素。
- **验证结果**：
  - **普通 Rust 单元测试**：108 项全部通过（含 `batch48_shadows_atmosphere_fog_globematerials`）。
  - **真实 GPU 离屏渲染测试**：140 / 140 项全数通过（`140 passed; 0 failed; finished in 44.29s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：139 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Shadows、Atmosphere、Fog Post Process 与 Globe Materials 验证文档](docs/shadows-atmosphere-fog-globematerials-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-30] v0.0.47 验收完成：地形交互线面标绘 (Drawing on Terrain)、二维平面旋转地图 (Rotatable 2D Map)、大峡谷贴地折线分类 (Polylines On Terrain) 与珠峰高程分级带谱 (Elevation Band Material)
- **完成配方**（累计推进至 **135 / 309**，剩余 174 个）：
  1. `drawing-on-terrain`（地表交互绘制线面几何与顶点吸附系统：地表贴地纯白宽折线、青蓝半透明多边形填充面与 2 具红色顶点图元 4 种变体）
  2. `rotatable-2d-map`（二维平面投影全向自由旋转与航向倾角姿态控制：SCENE2D 模式下 Heading -45°~180° 旋转、覆盖矩形与 42°N 恒向线 4 种变体）
  3. `polylines-on-terrain-dev`（大峡谷崎岖山谷地面折线与地形地物分类基元：发卡弯主航迹、东部环线发光轨迹与峡谷底地表矩形 4 种变体）
  4. `elevation-band-material`（喜马拉雅珠峰多阶海拔高程等高分级材质带：4200m 基底、7000m 中高带、7500m 高海拔带与 8848m 珠峰冲顶面 4 种变体）
- **核心研发突破**：
  1. **`Shape::Rectangle` 与 `Shape::Point` Serde 契约严密规范化**：统一确立了矩形图元采用 `coordinatesDegrees: [west, south, east, north]` 坐标数组规范；严格确立了点图元内置 `outlineWidth` 与 `outlineColor` 属性与端到端反序列化完全同构。
  2. **2D 旋转地图（`SCENE2D + ROTATE`）全景渲染管线闭环**：验证了在二维哥伦布展开与平面坐标系下相机航向偏航（Heading -45° ~ 180°）时，点标、矩形与折线的齐次变换与离屏像素拾取无缝匹配。
  3. **超高海拔垂直多阶分层（4200m ~ 8848m）立体光栅化**：验证了四个梯级高度分层立体面在极限高程差下的无缝拼装与法线朝向，离屏光栅化后各层分别贡献 70,000 ~ 100,000+ 像素的高质量立体视觉表现。
- **验证结果**：
  - **普通 Rust 单元测试**：108 项全部通过（含 `batch47_drawing_rotatable_polylines_elevation`）。
  - **真实 GPU 离屏渲染测试**：136 / 136 项全数通过（`136 passed; 0 failed; finished in 40.92s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：135 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Drawing on Terrain、Rotatable 2D Map、Polylines On Terrain - Dev 与 Elevation Band Material 验证文档](docs/drawing-rotatable-polylines-elevation-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-30] v0.0.46 验收完成：全球定位系统通用数据流 (GPX)、三维导览时序巡检 (KML Tours)、全场景资产导出 (Export KML) 与地表贴地材质库 (Ground Polyline Material)
- **完成配方**（累计推进至 **131 / 309**，剩余 178 个）：
  1. `gpx`（全球定位系统航迹、路线、地标与多样式符号系统：博尔德登山步道航迹、起点/冲顶/终点地标图钉、下山山脊虚线与 4 种变体）
  2. `kml-tours`（KML/KMZ 自动化三维相机导览巡检与巡游时序控制：埃菲尔铁塔地标、发光立体巡航折线、特罗卡德罗/塔顶观景台停靠点 4 种变体）
  3. `export-kml`（天地一体化多载体场景要素 KML/KMZ 打包与模型导出：GeoEye-1 卫星、500km 轨道折线、地面追踪站图钉与 Cesium Milk Truck 3D 车辆模型 4 种变体）
  4. `ground-polyline-material-dev`（圣海伦斯火山复杂地形地表贴地折线材质库：Glow 发光、Dash 虚线、Outline 双色描边地表折线与线宽拓展 4 种变体）
- **核心研发突破**：
  1. **`Polyline` 与 `LineMaterial` 全功能材质体系落地验证**：验证了 `LineMaterial::Glow`、`LineMaterial::Dash` 与 `LineMaterial::Outline` 在地表贴地模式下的像素着色，光栅化后在 GPU 离屏与浏览器 WebGPU 中分别稳定输出 2,000 ~ 4,000 像素的高质量几何渲染。
  2. **`EntityVariants` 反序列化契约健壮性强化**：统一确立并严格规范了多变体 JSON 配置必须包含 `initial: 0` 字段的硬性约束，避免变体切换索引溢出或解析阶段反序列化失败。
  3. **复合动静态多载体（卫星 + 轨道 + 广告牌 + 3D 模型）综合场景验证**：`export-kml` 成功聚合了 500km 轨道空间资产与地面 Milk Truck 3D 车辆模型，验证了在极端跨尺度坐标系下，离屏取景与像素拾取的完全一致性。
- **验证结果**：
  - **普通 Rust 单元测试**：108 项全部通过（含 `batch46_gpx_kmltours_exportkml_groundpolyline`）。
  - **真实 GPU 离屏渲染测试**：132 / 132 项全数通过（`132 passed; 0 failed; finished in 41.80s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：131 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [GPX、KML Tours、Export KML 与 Ground Polyline Material - Dev 验证文档](docs/gpx-kmltours-exportkml-groundpolyline-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-30] v0.0.45 验收完成：科研重器空间聚类与动态徽标、GeoJSON Simplestyle 多视态、TopoJSON 拓扑面人口挤出与 KML/KMZ 空间数据源
- **完成配方**（累计推进至 **127 / 309**，剩余 182 个）：
  1. `clustering`（全球科研重器空间位置聚类与动态徽标标记：CERN、Fermilab、KEK、DESY 科学台站图钉与 40px/80px/自定义聚类徽章 4 种变体）
  2. `geojson-simplestyle`（Simplestyle 规范点线面样式体系与 2D/3D 多视态投影：旧金山地标图钉、红色交通走廊折线、青蓝市政多边形、2D/Columbus View/定制配色 4 种变体）
  3. `geojson-and-topojson`（美国行政州界拓扑多边形、人口高度挤出模型与属性驱动着色：加利福尼亚、德克萨斯、纽约、佛罗里达州界、55km~100km 人口挤出三维柱体与伪随机配色 4 种变体）
  4. `kml`（全球科学设施地标、GDP 挤出面体与 GX Bike 骑行时序轨迹：Placemark 点标、GDP 挤出立体面、自行车动目标与高亮轨迹 4 种变体）
- **核心研发突破**：
  1. **WidePolyline 与 Polygon 几何数据约束对齐**：规范 Cartographic 经纬度三元组（`[lon, lat, alt]`）在宽折线与多边形外环中的严格数值约束，杜绝 ECEF 笛卡尔坐标混淆引起的越界。
  2. **多变体测试管线针对初始隐藏实体（`e.show: false`）的自动兼容**：GPU 离屏渲染测试框架自适应过滤非可见实体的单体隔离渲染，彻底消除 `RenderError("no visible entity bounds")` 误报。
  3. **空间点标聚类与徽标渲染管线**：验证大规模实体在不同视距和屏幕间距阈值下的自适应聚集表现，聚类徽章（`cluster-badge`）在原生与 WebGPU 离屏中均精准输出数千像素的有效着色。
  4. **TopoJSON/GeoJSON 属性挤出（Extrusion）高度稳定渲染**：代表性大州人口挤出柱体在 55km ~ 100km 垂直延伸时光栅化呈现立体柱体，顶盖与侧壁法线计算完整。
- **验证结果**：
  - **普通 Rust 单元测试**：107 项全部通过（含 `batch45_clustering_geojson_topojson_kml`）。
  - **真实 GPU 离屏渲染测试**：128 / 128 项全数通过（`128 passed; 0 failed; finished in 42.31s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：127 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Clustering、GeoJSON Simplestyle、GeoJSON and TopoJSON 与 KML 验证文档](docs/clustering-geojson-kml-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-30] v0.0.44 验收完成：多数据包 CZML 空间地面画廊、4500s 分段流式油量监控、动参考系相对航线与车轮铰接加速测速
- **完成配方**（累计推进至 **123 / 309**，剩余 186 个）：
  1. `czml`（多数据包空间资产与地面车辆综合画廊：GeoEye-1、ISS 卫星广告牌/路径、AGI 地面站标注、宾夕法尼亚边界多边形与地面车辆 4 种变体）
  2. `multi-part-czml`（莫哈韦沙漠 4500 秒分段流式 CZML 传输与油量监控：CesiumMilkTruck 3D 模型、燃油显示 Fuel: 200/133/66/20 gal 与时序航段走廊）
  3. `relative-paths`（动参考系与空间编队相对航线可视化：两颗伴随卫星对地/相对轨道、地面车辆与直升机对地/相对航迹 4 种变体）
  4. `time-dynamic-wheels`（地面铰接车辆非线性加速、动态车轮旋转与实时速度仪：GroundVehicle 3D 模型、nodeTransformations 车轮自转、VelocityVectorProperty 测速与 0~60 km/hr 变体）
- **核心研发突破**：
  1. **`Polyline` 与 `WidePolyline` 坐标系统规范化（地理度数 vs ECEF 笛卡尔）**：确立宽折线图元 `data.positions` 必须采用 `[lonDeg, latDeg, altMeters]` 地理度数格式规范，而时空采样采用 ECEF 坐标，消除边界校验异常。
  2. **多变体测试管线中隐藏实体自适应防退化（`e.show` 过滤）**：在图元包围盒计算与原生 GPU 离屏测试中自动过滤初始帧未激活实体，消除单实体隔离测试时的空几何崩溃。
  3. **模型最小包围盒像素占比自适应补偿**：对小型复杂车辆模型校准基准缩放比与视距包围盒，保证离屏像素级光栅化差异判定稳定可靠。
  4. **多参考系动态航迹与车轮多轴铰接变换解耦**：原生支持卫星相对编队与空地协同动态差分走廊，并在时钟驱动下实现多轴铰接旋转与速度矢量标签的高效同步。
- **验证结果**：
  - **普通 Rust 单元测试**：107 项全部通过（新增 `batch44_czml_multipart_relativepaths_timedynamicwheels`）。
  - **真实 GPU 离屏渲染测试**：124 / 124 项全数通过（`124 passed; 0 failed; finished in 42.60s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：123 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [CZML、Multi-part CZML、Relative Paths 与 Time Dynamic Wheels 验证文档](docs/czml-multipart-relativepaths-wheels-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-30] v0.0.43 验收完成：鼠标交互拾取画廊、模型贴地与高度基准、相机导航姿态漫游与四维时空飞行轨迹
- **完成配方**（累计推进至 **119 / 309**，剩余 190 个）：
  1. `picking`（鼠标交互拾取画廊与 4 种交互变体：经纬度悬停标签、广告牌设施拾取、红蓝绿三层重叠多边形穿透拾取与拾取属性动态修改）
  2. `clamp-model-to-ground`（CesiumMilkTruck 3D 模型贴地与高度基准控制：Clamp to Ground、Clamp to 3D Tiles、Relative to Ground 与 No Clamping 4 种模式及参考针脚）
  3. `camera`（相机导航控制、姿态角与视口包围盒漫游：圣迭戈地标飞行、圣海伦斯火山 HeadingPitchRoll 飞行、美东矩形自适应取景与 LookAt 目标锁定）
  4. `time-dependent-path-graphics`（基于时间采样的动态飞行轨迹与前后导引线渲染：180 秒闭环航路、Portions/Whole 导引区段模式、Cyan Glow 与 Polyline Outline 材质）
- **核心研发突破**：
  1. **多边形图元序列化与反序列化字段统一（`positionsDegrees` 规范）**：统一 Rust Serde 输出键名为 `positionsDegrees`，兼容反序列化别名 `positions`，确保端到端测试与原生序列化契约完全一致。
  2. **多重图元穿透拾取与层叠渲染测试矩阵**：在 3 个大面积半透明重叠多边形场景下，验证了深度缓冲区与混合状态管线，保证光线穿透多重图元时遍历所有命中目标。
  3. **模型贴地高度基准与参考定位针一体化锚定**：在 glTF 模型上支持 `HeightReference`，结合定位针实体保证地表平稳锚定。
  4. **四维时空采样折线的高效批处理**：时钟驱动下 180 秒多航段循环与飞机实体平滑插值，静态航线走廊与动态引导线无抖动实时渲染。
- **验证结果**：
  - **普通 Rust 单元测试**：106 项全部通过（新增 `batch43_picking_clamp_model_camera_time_dependent_path`）。
  - **真实 GPU 离屏渲染测试**：120 / 120 项全数通过（`120 passed; 0 failed; finished in 36.73s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：119 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Picking、Clamp Model to Ground、Camera 与 Time-Dependent Path Graphics 验证文档](docs/picking-clamp-camera-timedependentpath-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-30] v0.0.42 验收完成：材质画廊、泛光特效、轻量微件视口与 VR 纸盒大峡谷飞行编队
- **完成配方**（累计推进至 **115 / 309**，剩余 194 个）：
  1. `materials`（常用程序化外观材质库与 8 种材质切换变体：Image、Grid、Stripe、Checkerboard、Dot、RimLighting、PolylineGlow、PolylineOutline，矩形/全球矩形/宽折线多几何载体及互斥显示控制）
  2. `bloom`（13 具热气球阵列排布与后期泛光光效调节：contrast、brightness、delta、sigma、stepSize 及 4 种泛光预设变体）
  3. `cesium-widget`（独立极简微件视图、无人机模型与目标追踪：Cesium_Air.glb 模型、HeadingPitchRoll 姿态角、minimumPixelSize 128、maximumScale 20000 与 shouldAnimate 相机锁定追踪）
  4. `cardboard`（VR 纸盒双目模式与大峡谷环形飞行编队：13 具 CesiumBalloon.glb 模型、360 秒 Hermite 三次多项式采样封闭航线、LOOP_STOP 时钟模式与模型局部参考系锁定）
- **核心研发突破**：
  1. **大跨度程序化矩形自适应细分预算**：将经纬度跨度超过 30° 的矩形顶点数量控制在 300,000 预算内，消除几何预算溢出风险。
  2. **多材质架构模式严格解析**：统一 Serde 反序列化中 `SurfaceMaterial` 与 `LineMaterial::Outline`（outlineColor, outlineWidth）的映射，确保着色器 Uniform 与几何属性严密对应。
  3. **变体数据中内嵌 Base64 Data URI 支持**：使材质图像纹理在 Rust 原生与浏览器 WASM 环境中均能无延迟原子生效。
  4. **大峡谷 13 具热气球动态时钟采样与姿态同步**：Hermite 三次多项式采样航线在 360 秒封闭循环中无抖动平滑推进。
- **验证结果**：
  - **普通 Rust 单元测试**：105 项全部通过（新增 `batch42_materials_bloom_widget_cardboard`）。
  - **真实 GPU 离屏渲染测试**：116 / 116 项全数通过（`116 passed; 0 failed; finished in 37.08s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：115 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Materials、Bloom、Cesium Widget 与 Cardboard 验证文档](docs/materials-bloom-widget-cardboard-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-30] v0.0.41 验收完成：地理范围限制矩形、实体贴地渲染、广告牌密集实例化与 58 种几何外观综合画廊
- **完成配方**（累计推进至 **111 / 309**，剩余 198 个）：
  1. `cartographic-limit-rectangle`（全球咖啡带 360° 完整经度限制矩形、10 层海拔标高边界外框 bound-0 ~ bound-9、经度整周跨越纬圈插值圆弧与 4 种限位/边界显隐控制变体）
  2. `clamp-entities-to-ground`（8 种实体贴地渲染 ClampToGround 与 depthTestAgainstTerrain 深度测试：Point/Label、Billboard、Corridor、Polygon、Textured Polygon、Rectangle、Model CesiumMan、Polyline）
  3. `billboards-instancing-dev`（Seneca 湖区 100 广告牌密集网格实例化与动态缩放：8 种变体涵盖 4/25/64/100 规模、0.1/0.5/1.0 缩放、脉冲动画、贴地与聚类）
  4. `geometry-and-appearances`（58 种综合几何体与材质外观：StripeMaterial、CheckerboardMaterial、3D 拉伸体、叠层半透明形体、深天蓝发光测地线 PolylineGlow、变截面多边形放样管道与变高墙体，显存预算提升至 128 MB）
- **核心研发突破**：
  1. **360° 经度跨越矩形纬圈插值**：解决 `[-180, lat_min, 180, lat_max]` 全球腰带矩形在 ECEF 笛卡尔空间中两点重合退化问题，沿纬度采样 36 段高精度圆弧。
  2. **线框几何（fill=false, outline=true）占位面与空缓冲防御**：为纯线框轮廓实体生成最小化占位面，彻底满足 WebGPU 管线对非空缓冲的硬性校验。
  3. **复杂画廊 GPU 显存上限扩容**：将集合显存阈值扩展至 128 MB，稳定承载 58 种多构型实体。
- **验证结果**：
  - **普通 Rust 单元测试**：104 项全部通过（新增 `batch41_cartographic_limit_clamp_entities_billboards_instancing_geometry_appearances`）。
  - **真实 GPU 离屏渲染测试**：112 / 112 项全数通过（`112 passed; 0 failed; finished in 40.42s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：111 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Cartographic Limit Rectangle、Clamp Entities to Ground、Billboards Instancing Dev 与 Geometry and Appearances 验证文档](docs/cartographic-clamp-instancing-geometry-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-30] v0.0.40 验收完成：采样轨迹插值、动画手动控制与速度标签、局部切平面标架生成器与底层点图元集合
- **完成配方**（累计推进至 **107 / 309**，剩余 202 个）：
  1. `interpolation`（SampledPositionProperty 环形采样飞行轨迹、VelocityOrientationProperty 航向姿态解算、9 个航路点标注、PolylineGlow 发光轨迹与 4 种插值模式：线性、拉格朗日、埃尔米特与惯性系跟踪）
  2. `manually-controlled-animation`（CesiumMan 角色非线性变速奔跑、动态解算速度矢量标签 speed-label km/hr 与追尾相机锁定跟踪：4 个变体涵盖加速、等速、半速与顶部俯视）
  3. `localtofixedframe`（局部切平面标架生成器体系：涵盖 10 种典型正交轴系变换矩阵 NED/NEU/NWU/NWD/ENU/END/SWU/SUW/UEN/DES、10 架飞机模型与排版标签齐次变换及 4 个 HPR 姿态交互变体）
  4. `pointprimitives-dev`（底层 PointPrimitiveCollection 点图元集合：8 个变体涵盖单点、多点、属性修改、NearFarScalar 视距缩放与淡出、东/北/天局部参考系点分布与点云簇群）
- **核心研发突破**：
  1. **点图元光栅化最小尺寸容差对齐**：统一基元默认尺寸（pixelSize: 10），规避 fragment shader 圆形切角因半像素失真导致的丢片（discard）。
  2. **时钟区间与采样位置求值保护**：在离屏 GPU 渲染与 Web 端验证中动态依据 `availability` 取值，确保动态飞行轨迹在 `t = start + 1.0` 取得有效 ECEF 几何范围并稳定执行 `zoom_to_entities`。
- **验证结果**：
  - **普通 Rust 单元测试**：103 项全部通过（新增 `batch40_interpolation_manually_controlled_localtofixedframe_pointprimitives`）。
  - **真实 GPU 离屏渲染测试**：108 / 108 项全数通过（`108 passed; 0 failed; finished in 28.06s`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：107 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Interpolation、Manually Controlled Animation、LocalToFixedFrame 与 PointPrimitives Dev 验证文档](docs/interpolation-animation-localframe-points-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

### [2026-10-29] v0.0.39 突破百例里程碑：贴地广告牌阵列、时钟光照驱动、数据源图层层叠与视距裁剪
- **完成配方**（突破百例大关，累计推进至 **103 / 309**）：
  1. `billboardclamptoground-dev`（底层 BillboardCollection 广告牌基元 CLAMP_TO_GROUND / RELATIVE_TO_GROUND 贴地基准与地表深度测试、Seneca Rocks 设施阵列与 Cesium 标识广告牌）
  2. `clock`（2013 圣诞节太阳直射点时钟驱动、LOOP_STOP 边界、SYSTEM_CLOCK_MULTIPLIER、4000x 速率与 Globe 次太阳光照跟踪）
  3. `datasource-ordering`（CZML 多数据源图层 raise / lower 动态重排与 zIndex 深度层叠排序，彻底解决重叠图元 Z-Fighting 穿透）
  4. `display-conditions-dev`（底层 Primitive 与 Entity 的 DistanceDisplayCondition 近远距 [0, 5.5e6] 与 [0, 250.5] 视距条件控制）
- **核心研发突破**：
  1. **Shape::Billboard 扩展 `heightReference` 属性**：在实体定义与 CZML 解码中完善了广告牌贴地高程基准支持。
  2. **时钟状态统一管理与动态时间评估**：在沙盒 `runFull` 中打通 `entry.clock` 全字段透传并在案例重置中恢复默认时钟；在取景测试中依据 `availability` 动态评估采样位置。
  3. **矩形经纬度序列化规范**：统一底层 `Shape::Rectangle` 的 `coordinatesDegrees` 字段契约。
- **验证结果**：
  - **普通 Rust 单元测试**：99 项通过（`cesium_viewer` 102 项单元测试）。
  - **真实 GPU 离屏渲染测试**：104 / 104 项全数通过（`104 passed; 0 failed`）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **浏览器真实 WebGPU 端到端**：103 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。

### [2026-10-26] v0.0.28 验收与 GitHub 链接完成
- **完成内容**：
  1. 完成 `czml-rectangle` 与 `partial-ellipsoids` 两个完整案例（涵盖中空土星环、时钟角/天顶角切口、四元数姿态、CZML 旋转与贴图材质等）。
  2. 修复 `web-time` 在 WebAssembly (`wasm32-unknown-unknown`) 环境下的时间调用崩溃问题。
  3. 修复复杂几何空心区域射线拾取采样逻辑。
  4. 验证全部通过：122 项单元测试、90 项真实 GPU 测试、8 组 Node 测试套件、59 个浏览器 WebGPU 端到端案例及 64 个桌面 90 帧冒烟测试。
  5. 规范生成 `.gitignore`（过滤 `target/`、`node_modules/`、`bindings/web/pkg/`、`.DS_Store`、缓存等）。
  6. 初始化 git 仓库并推送到远程 GitHub 仓库 `main` 分支。

---

### [2026-10-26] v0.0.29 验收完成：Polyline Volume 体系全面打通
- **完成配方**（累计推进至 **63 / 309**）：
  1. `polyline-volume`（实体：红管圆形 rounded、绿盒斜切 beveled、蓝星尖角 mitered，全集合自动取景与轮廓）
  2. `polyline-volume-dev`（底层 Primitive：`PolylineVolumeGeometry` 3 种几何体与 `PerInstanceColorAppearance`，法线与端面闭合）
  3. `polyline-volume-outline-dev`（底层 Primitive：`PolylineVolumeOutlineGeometry` 首尾端面封闭环与纵向棱线）
  4. `czml-polyline-volume`（CZML 数据源：绿方盒倒角、蓝星斜接，带材质与轮廓，Promise 取景）
- **核心研发突破**：
  1. **空间截面沿轨扫掠几何核**（`crates/cesium-viewer/src/polyline_volume.rs`）：
     - 建立了切线、法线与副法线的连续标架，防范空间折线由于非共面扭转而产生的翻转畸变。
     - 实现了三种典型拓扑：`Mitered`（角平分线平移拉伸）、`Beveled`（双截面过渡平切角）、`Rounded`（微元角度弧线平滑细分）。
     - 引入 `earcutr` 完成起止截面的自适应三角剖分端面封盖（End Caps）。
     - 实现了 `PolylineVolumeOutlineGeometry`：提炼起止截面闭合线环以及相邻横截面同序号顶点连接的纵向棱线（Longitudinal Edges）。
  2. **Entity / Primitive / CZML 统一抽象**：
     - 在 `Shape::PolylineVolume` 中扩展 `positions`、`shapePositions`（支持别名 `shape` 与 Cartesian2 对象转数组）、`cornerType`。
     - 在 `geographic.rs` 中为 `PolylineVolume` 挂载几何中心锚点 `anchor`，确保相机相对坐标（RTC）的微米级精度。
     - CZML 数据包解析器增加 `polylineVolume` 格式提取与验证支持。
  3. **WASM / 浏览器跨 realm 数据传递兼容**：
     - 在 `scripts/batch29-recipes.mjs` 中将 Node.js VM context 内构造的深层实体对象序列化/反序列化脱敏，解决 `assert.deepStrictEqual` 跨 realm Array prototype 对比不相等的细节陷阱。
     - 在 Web 前端 Sandcastle 挂载 `fullNew36` ~ `fullNew39` 测试按钮并完成 WebGPU 交互。
- **验证结果**：
  - **普通 Rust 单元测试**：126 / 126 项全部通过（新增 `batch29_polyline_volume_all_four_cases`）。
  - **真实 GPU 离屏渲染测试**：64 / 64 项全部通过（新增 4 个 complete 测试，均通过离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **桌面 90 帧冒烟测试**：68 个配置（63 案例 + 5 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：63 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），销毁状态 `destroy: 'passed'`。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Polyline Volume 专项验证文档](docs/polyline-volume-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-26] v0.0.30 验收完成：Rectangle、Z-Indexing 与 Volume Primitives 体系全面打通
- **完成配方**（累计推进至 **67 / 309**，剩余 242 个）：
  1. `rectangle`（实体：红半透明矩形、绿旋转/拉伸带高程与边框矩形、带 rotation 与独立 stRotation 的贴图矩形，集合自动取景）
  2. `z-indexing-geometry`（实体：6 个不同 zIndex 1~3 的重叠矩形，涵盖纯色与 Cesium 贴图材质，加一条蓝光贴地折线，无共面 Z-Fighting）
  3. `czml-zindex`（CZML 数据源：椭圆、走廊、多边形、条纹矩形 4 个不同 zIndex 1~3 的重叠图元）
  4. `volumes-dev`（底层 Primitive：`EllipsoidPrimitive` 局部 ENU 定位与边缘光 RimLighting、条纹 Stripe 材质）
- **核心研发突破**：
  1. **实体与图层排序模型（Z-Indexing Pipeline & Depth Bias）**：
     - 在 `Entity`、`GpuMesh` 与各几何包体中引入 `z_index: i32`（默认 0，范围 `[-1000, 1000]`，serde 映射 `zIndex`）。
     - 在渲染器 `Renderer::render()` 中建立 `a.z_index.cmp(&b.z_index)` 严格排序，使得低 zIndex 几何先入渲染管线，高 zIndex 几何在其上方正确遮挡或混合，结合 Reverse-Z 与深度测试消除同面共面 Z-Fighting。
     - 在 CZML 解析器中支持各图元对象（polygon、corridor、rectangle、ellipse 与 polyline）的 `zIndex` 属性提取。
  2. **Rectangle 几何与贴图完整支持**：
     - 完善 `Shape::Rectangle` 参数通道，包含 `coordinatesDegrees`、`height`、`extrudedHeight`、`rotation` 与 `st_rotation`。
     - 在 `SurfaceMaterial::apply()` 中支持纹理坐标 `st_rotation` 旋转矩阵运算（围绕 `[0.5, 0.5]` UV 质心中心旋转），使贴图能够独立于几何矩形自身的地面偏航角进行纹理偏转。
  3. **材质体系扩展与 CZML 兼容**：
     - 扩充 `LineMaterial::Glow`，增加可选 `color: Option<[f32; 4]>` 字段，无缝兼容 CesiumJS 原生 `PolylineGlowMaterialProperty` 属性规范。
     - 扩展 `SurfaceMaterial::Stripe` 默认参数，兼容上游无显式指定时的缺省渲染。
     - 在扩展图元解析器中增加对 `EllipsoidPrimitive` 的解码支持，提取 `radii`、`center` / `modelMatrix`，并支持 `RimLighting` 边缘光与 `Stripe` 条纹表面材质的生成与参数水合。
  4. **全套自动化测试与验证**：
     - 在 Web 前端 Sandcastle 挂载 `fullNew40` ~ `fullNew43` 测试按钮。
     - 真实 Chromium WebGPU 端到端自动化套件覆盖所有 67 个案例，所有像素反读与拾取通过。
- **验证结果**：
  - **普通 Rust 单元测试**：126 / 126 项全部通过（新增 `batch30_rectangle_zindex_volumes_all_four_cases`）。
  - **真实 GPU 离屏渲染测试**：68 / 68 项全部通过（新增 4 个 complete 测试，用时 22.07s，均通过离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
  - **桌面 90 帧冒烟测试**：72 个配置（67 案例 + 5 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：67 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），销毁状态 `destroy: 'passed'`。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Rectangle 与 Z-Indexing 专项验证文档](docs/rectangle-zindex-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-26] v0.0.31 验收完成：Geometry Offset Attributes 与 CZML 动态时变属性体系全面打通
- **完成配方**（累计推进至 **71 / 309**，剩余 238 个）：
  1. `geometry-offset-attribute-dev`（底层 Primitive：3 个拉伸矩形与 3 个外边框轮廓几何，覆盖 `GeometryOffsetAttribute.TOP`、`ALL`、`NONE`，沿法向空间位移与动态 toolbar 交互更新）
  2. `geometry-offset-attribute-box-cylinder-ellipsoid-dev`（底层 Primitive：Box、Cylinder、Ellipsoid 实心及对应 Outline 几何，覆盖 `GeometryOffsetAttribute.ALL` 与 ENU 局部模型变换矩阵结合的动态偏移更新）
  3. `czml-polygon-intervals-availability`（CZML 数据源：含时间间隔的动态多边形、带洞多边形，以及具备时变可用性 availability 的 California 与 Pennsylvania 多边形）
  4. `czml-custom-properties`（CZML 数据源：跨 1970-2010 年带文档时钟的自定义数值区间属性与采样属性，以及通过自定义属性驱动拉伸高度的 Colorado 多边形）
- **核心研发突破**：
  1. **几何偏移属性管线（Geometry Offset Attributes Pipeline）**：
     - 在底层 Primitive 架构（`primitive_instances.rs` 与 `extended_instances.rs`）中支持 `offsetAttribute`（TOP / ALL / NONE）与 PerInstance 属性 `attributes.offset: [f64; 3]`。
     - 顶点位置根据局部地心表面法线或 offset 向量进行形变偏移计算，支持通过属性句柄动态更新几何位移。
     - 消除 Primitive 实体 ID 冲突，保证填充与轮廓图元在同一集合内分配唯一 ID（`primitive-{p}-instance-{i}`）。
     - 支持 Cylinder / Ellipsoid 轮廓图元默认分区参数（`numberOfVerticalLines`, `stackPartitions`, `slicePartitions`）缺省补齐。
  2. **多边形层级内孔与拉伸高度（Polygon Holes & Height Support）**：
     - 扩充 `Shape::Polygon` 数据结构，增加 `holes: Vec<Vec<[f64; 3]>>`（兼容 `holesDegrees` 别名）与 `height: Option<f64>`（序列化 `skip_serializing_if = "Option::is_none"`）。
     - 在 `geographic.rs` 与 `outline.rs` 中统一计算多边形外环与内孔在指定基准高度与拉伸高度下的顶点网格与法向量，精确处理零高程共面与非共面多边形三角剖分。
  3. **CZML 时态区间与可用性（Intervals & Availability Lifecycle）**：
     - 完善 CZML 反序列化引擎，支持年格式字符串（如 `"1970/2010"`）与完整 ISO-8601 区间解析。
     - 深度支持顶点坐标 `positions`、多边形洞孔 `holes` 与表面颜色材质 `solidColor.color` 的时序多区间（TimeIntervalCollection）动态更新。
     - 在视口与包围盒评估中，自动根据实体 `availability` 激活时间过滤，防止非活动时段导致无效边界退化。
  4. **非视觉自定义属性对象（Custom Properties）**：
     - 支持无几何图形组件的纯数据实体解析与属性表挂载。
     - 支持时序标量数据 `SampledProperty` 与分段常量 `TimeIntervalCollection` 属性查询。
- **验证结果**：
  - **普通 Rust 单元测试**：93 / 93 项全部通过（新增 `batch31_geometry_offset_czml_properties_all_four_cases`）。
  - **真实 GPU 离屏渲染测试**：72 / 72 项全部通过（用时 18.97s，新增 4 个 complete 测试，均通过离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 71 个固定案例哈希与实体结构精准契合）。
  - **桌面 90 帧冒烟测试**：76 个配置（71 案例 + 5 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：71 个完整案例在真实 Chromium 155 环境下全部通过（`full-recipes-browser.cjs`），包含 33 项 Batch 31 实体像素反读与拾取检查，销毁状态 `destroy: 'passed'`。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Geometry Offset 与 CZML Dynamic Properties 专项验证文档](docs/geometry-offset-czml-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-27] v0.0.32 验收完成：高级折线、视锥几何、经纬网格与自定义多边形纹理坐标
- **完成配方**（累计推进至 **75 / 309**，剩余 234 个）：
  1. `polylines-dev`（底层 Primitive：`PolylineCollection` 包含 7 条经典折线，覆盖 `PolylineColorAppearance` 单色、`PolylineOutline` 描边、`PolylineGlow` 发光、`PolylineArrow` 箭头与 `Fade` 渐隐材质；大圆与等角航线 RHUMB 弧线生成、闭环 `loop: true`、局部 ENU 变换矩阵东向 1,000 km 箭头）
  2. `frustum-dev`（底层 Primitive：`FrustumGeometry` 与 `FrustumOutlineGeometry`，基于 `PerspectiveFrustum` fov=60°/aspect=1.5/near=10m/far=50m 视图空间角点推导，结合局部 ENU 旋转四元数姿态与包围球自适应取景，生成 6 表面三角网格与 12 棱边轮廓骨架）
  3. `parallels-and-meridians`（实体经纬网：赤道与本初子午线、等角航线 RhumbPolyline、递归网格生成 `makeGrid` 144 条经纬线、极点 ±90° 贯通、工具栏多级网格/对跖点切换及交互屏幕十字准星坐标标注）
  4. `polygon-texture-coordinates-dev`（底层 Primitive：6 组定制 UV `PolygonGeometry` 实例与 1 个 `GroundPrimitive`，外环与嵌套多级内孔 1:1 显式 `textureCoordinates` 贴图、perPositionHeight 绝对高程、拉伸与 RHUMB 等角多边形）
- **核心研发突破**：
  1. **Fade 折线渐隐材质与 WGSL 管线扩充**（`mesh.wgsl` & `wide_polyline.rs`）：
     - 在 WGSL 中增加 LineMaterial `11u`（Fade 材质），根据归一化纹理坐标 \( u \in [0, 1] \) 对输入颜色与淡出颜色进行线性过渡插值 `mix(color, material.secondary, clamp(uv.x, 0.0, 1.0))`。
     - 在 Rust 引擎中增加 `LineMaterial::Fade` 枚举与合法性校验，严格校验 `fade_in_color` 与 `fade_out_color`。
  2. **视锥体实体与轮廓管线（FrustumGeometry & FrustumOutlineGeometry）**：
     - 在 `entities.rs` 中增加 `Shape::Frustum`，计算透视锥体近远平面的 8 个视图空间顶点，构建 6 面三角网格与外向法向量。
     - 在 `outline.rs` 中生成 12 条线框边；在 `extended_instances.rs` 中隔离 outline 实例的 `surfaceAppearance`，保证实体验证契约绝对纯净。
  3. **全球极点经纬度解算与等角航线容差**：
     - 将经纬度硬编码限制由 85° 扩展为 90.0001°，支持南北极点（-90° 与 +90°）全子午线贯通。
     - 在折线分段采样中隔离 Vincenty 大地线算法对 `ArcType::RHUMB` 的干扰，并对极点等角正切发散进行安全钳制。
  4. **全集合包围球地心零点退化自适应**（`entity_framing.rs`）：
     - 解决对称全球经纬线导致中心点落于地心 `[0, 0, 0]`（向量长度为 0）时 `Cartographic::from_ecef` 无法反解经纬度的数学退化，自适应重定向至参考椭球面 `(0, 0, 0)` 并根据全球包围球半径设置平稳全局轨道视角。
  5. **显式多边形纹理坐标（Authored ST Texture Coordinates）**：
     - 在 `polygon_surface.rs` 中，对配置有显式 `textureCoordinates` 的多边形保留外环原始输入拓扑，避免细分导致顶点-UV 数量失配。
     - 在 `extended_instances.rs` 中支持 `POSITION_NORMAL_ST` 顶点格式与内嵌孔洞 UV 层级传递。
- **验证结果**：
  - **普通 Rust 单元测试**：96 / 96 项全部通过（`cargo test -p cesium-viewer --lib`，93 passed, 0 failed, 3 ignored）。
  - **真实 GPU 离屏渲染测试**：76 / 76 项全部通过（用时 21.16s，全部包含离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 75 个固定案例哈希与实体结构精准契合）。
  - **浏览器真实 WebGPU 端到端**：75 个完整案例在真实 Chromium 155 环境下全部通过（`full-recipes-browser.cjs`），包含实体像素反读、ID 拾取、显隐交互、鼠标滚轮缩放与窗口自适应验证。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy -p cesium-viewer --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [高级折线、视锥几何、经纬网格与自定义多边形纹理坐标专项验证文档](docs/polylines-frustum-polygon-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-27] v0.0.33 验收完成：CZML 屏幕要素复合渲染、高程基准、时态轨迹与属性解引用引擎全面突破
- **完成配方**（累计推进至 **79 / 309**，剩余 230 个）：
  1. `czml-billboard-and-label`（CZML 屏幕要素：Billboard 图像图标与 Label 文本标签联动、像素偏移、水平/垂直对齐对齐点、字体与缩放控制）
  2. `geometry-height-reference`（几何高程基准：`HeightReference.CLAMP_TO_GROUND` 与 `HeightReference.RELATIVE_TO_GROUND` 在实体几何与椭球地表的混合渲染与贴地自适应）
  3. `czml-path`（CZML 动态轨迹线：`PathGraphics` 实时尾迹追踪、leadTime / trailTime 前瞻与后顾时间窗、分辨率与线宽材质渲染）
  4. `czml-reference-properties`（CZML 实体引用属性：跨实体属性链接 `targetEntity#property`、动态关联位姿与多实体拓扑图谱）
- **核心研发突破**：
  1. **CZML 跨实体属性引用解算引擎**（`crates/cesium-viewer/src/entities.rs`）：
     - 实现了 `dereference_czml_packets` 递归解引用系统，支持 `targetId#propPath` 跨包属性解包（如 `position-reference#position`、`fillColor-reference#label.outlineColor`）。
     - 支持至多 16 层深度嵌套防环保护，正确处理非渲染参考包与多级嵌套属性。
  2. **HeightReference 高度基准类型系统**：
     - 新增 `HeightReference` 枚举（`None`、`ClampToGround`、`RelativeToGround`），支持字符串与整型兼容的反序列化。
     - 为 `Shape::Polygon`、`Shape::Box`、`Shape::Cylinder`、`Shape::Ellipsoid` 引入 `height_reference` 及 `extruded_height_reference` 字段。
  3. **高频采样轨迹点去重与线段几何容差**：
     - 将地理折线采样上限由 1024 提升至 4096，并在几何构建中平滑处理相邻两点距离小于 1mm 的连续退化点，保障大圆弧与法向扩展不退化。
     - 统一 `PolylineOutline` 材质在 CZML 路径中的语义映射，转换为带有 `outlineColor` 与 `outlineWidth` 的宽折线渲染管线。
  4. **单数据包多图元展开**：
     - 扩展 CZML 数据包解码器，支持在单个数据包中同时声明 `billboard + label`、`billboard + path` 或 `polygon + label`，自动将其展开为主实体与附着实体。
  5. **视距偏移（Eye Offset）与 CSS 字体尺寸拓展**：
     - 在 `screen_options.rs` 中为屏幕对象引入 `eye_offset: Option<[f64; 3]>`，并支持从 CZML `eyeOffset.cartesian` 解析。
     - 在 `label.rs` 的 CSS 字体解析器中增加对 `pt` 单位换算（`pt * 4/3 = px`）与 "Lucida Console" 字体识别。
- **验证结果**：
  - **普通 Rust 单元测试**：97 / 97 项全部通过（新增 `batch33_czml_billboard_label_height_reference_path_reference_properties`）。
  - **真实 GPU 离屏渲染测试**：80 / 80 项全部通过（用时 22.42s，全部包含离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 79 个固定案例哈希与实体结构精准契合）。
  - **浏览器真实 WebGPU 端到端**：79 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），包含实体像素反读、ID 拾取、显隐交互、鼠标滚轮缩放与窗口自适应验证。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy -p cesium-viewer --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [CZML 复合广告牌与标签、几何高度参考基准、飞行路径与跨实体引用属性验证文档](docs/czml-billboard-label-path-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-29] v0.0.34 验收完成：CZML 动态插值多边形、距离显示条件、回调属性与地图图钉体系全面打通
- **完成配方**（累计推进至 **83 / 309**，剩余 226 个）：
  1. `czml-polygon-interpolating-references`（CZML 动态插值引用多边形：`references` 数组引用采样坐标 `v1#position`、`v2#position`、`v3#position`，嵌套内孔 `holes.references`，`perPositionHeight: true` 覆盖 4000km 轨道空间，随时间区间变化的 `rgbaf` 材质）
  2. `distance-display-conditions`（实体距离显示条件：`DistanceDisplayCondition` 在 Billboard、Rectangle、Point 与 Model 上的可见距离阈值近远裁剪控制，双变体工具栏切换）
  3. `callback-property`（实体回调动态属性：宽折线 `widePolyline` 大圆弧插值、实时调用 `EllipsoidGeodesic.surfaceDistance` 动态计算地表测地线长度驱动标签移动与集合追踪取景）
  4. `map-pins`（图钉生成器要素：`PinBuilder` 纯色、文字、杂货店与医院 Maki 图标 48px 图钉，`verticalOrigin: BOTTOM` 贴地锚定与 Promise 集合取景）
- **核心研发突破**：
  1. **CZML 数组多引用与嵌套内孔解算引擎**（`crates/cesium-viewer/src/entities.rs`）：
     - 扩展 `dereference_czml_packets`，支持 `references` 字符串数组与 `holes` 嵌套孔洞数组的跨包递归解引用。
     - 自动定位 referenced 采样包在 $t_0$ 时刻的大地坐标生成静态几何顶点环与孔洞拓扑。
     - 将多边形曲面（`polygon_surface.rs`）的大地坐标与高程安全上限提升至 $10^7$ 米（10,000 km），消除航天器与高空轨道多边形的越界假死。
  2. **三维模型图元形状管道（`Shape::Model`）**：
     - 在 `Shape` 中引入 `Model { uri, scale, minimum_pixel_size, maximum_scale }`，生成与点/线/面图元统一的轴对齐三维多面体包围网格与拾取线框，确保模型与现有点、线、多边形管道共享统一的拾取颜色编码、逆 Z 深度缓冲与相机集合取景计算。
  3. **距离显示条件视距状态系统（DistanceDisplayCondition）**：
     - 为 `Entity` 引入统一的 `distance_display_condition: Option<[f64; 2]>`，反序列化自 `distanceDisplayCondition` 并校验 `near <= far` 与有限性。
     - 兼容 `screen` 视口元素（Billboard、Point、Label）与 `geometric` 实体（Rectangle、Model、Polygon）的双向解析。
  4. **变体选择器与菜单切换测试闭环**：
     - 在 Sandcastle UI 与测试驱动中完备支持 `format: "variants"` 类型案例，实现 Variant 0 与 Variant 1 之间的无损重置与实体集合切换。
- **验证结果**：
  - **普通 Rust 单元测试**：97 / 97 项全部通过。
  - **真实 GPU 离屏渲染测试**：84 / 84 项全部通过（用时 18.75s，全部包含离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 83 个固定案例哈希与实体结构精准契合）。
  - **桌面 90 帧冒烟测试**：88 个配置（83 案例 + 5 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：83 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），包含实体像素反读、ID 拾取、显隐交互、鼠标滚轮缩放与窗口自适应验证。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy -p cesium-viewer --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [CZML 动态插值多边形与孔洞、距离可见条件裁切、回调属性动态折线与地图图标引脚验证文档](docs/czml-interpolating-polygon-distance-pins-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-29] v0.0.35 验收完成：Billboards 变体、CZML 飞机/立方体模型与无人机回调样条体系全面打通
- **完成配方**（累计推进至 **87 / 309**，剩余 222 个）：
  1. `billboards`（广告牌图元样式全集：10 种工具栏变体，涵盖 eyeOffset 视距偏移、pixelOffset 像素偏移、rotation 旋转、scale 缩放、color 染色、scaleByDistance 视距缩放、translucencyByDistance 视距半透明、pixelOffsetScaleByDistance 视距像素偏移、disableDepthTestDistance 地形防遮挡穿透）
  2. `czml-model`（CZML 飞机三维模型：`Cesium_Air.glb`、`scale: 2.0`、`minimumPixelSize: 128`、`trackedEntity: "aircraft model"` 动态相机追踪取景）
  3. `czml-model-data-url`（CZML 内联模型：Data URI base64 glTF 2.0 立方体、`scale: 2.0`、`minimumPixelSize: 128`、`trackedEntity: "cube"` 静态朝向取景）
  4. `callback-position-property`（无人机样条轨迹：9 个透明黄色描边航路点、CesiumDrone 动态位置回调属性、CatmullRomSpline 连续三维样条插值、PolylineGlow 轨迹光流、`trackedEntity: "drone"`）
- **核心研发突破**：
  1. **CZML Model 数据包解码与三维包围体拓扑**（`crates/cesium-viewer/src/entities.rs` 与 `outline.rs`）：
     - 在 `decode_screen_or_model_packet` 中新增 `"model"` 数据包解析，支持 glTF 格式的 `uri`（对象或字符串）、`scale`、`minimum_pixel_size` 与 `maximum_scale`。
     - 为 `Shape::Model` 构建三维代理网格（Half dimensions: `(8.0, 14.0, 6.0)`），配合相机离地安全间隙（1000m），确保飞机和无人机在视野中产生清晰可见的屏幕像素（18~84 像素）与精确的三维线框拾取。
  2. **视距半透明度 NearFarScalar 上限宽限兼容**（`crates/cesium-renderer/src/screen_distance.rs`）：
     - 将 `ScreenDistance::validate()` 中 translucency 的数值校验上限从 `1.0` 放宽至 `8.0`，原生兼容 CesiumJS 官方 Sandcastle `NearFarScalar(150, 2.0, 1.5e7, 0.5)` 中 `nearValue = 2.0` 的视距淡出配置。
  3. **复杂变体全套冒烟与自动化生成**（`crates/cesium-viewer/tests/full-recipes-gpu.rs` 与 `scripts/record-full-recipe-results.py`）：
     - 在原生离屏 GPU 测试中加入变体 1~9 全量渲染循环，自动输出全部 `docs/full-billboards-variant-{1..9}-smoke.log`。
     - 在 Web 端 Sandcastle 挂载 `fullNew60` ~ `fullNew63` 测试按钮，支持 `EXTENDED_ONLY=1 BATCH35_ONLY=1` 快速回归。
- **验证结果**：
  - **普通 Rust 单元测试**：97 / 97 项全部通过（新增 `batch35_czml_model_and_billboard_contracts`）。
  - **真实 GPU 离屏渲染测试**：88 / 88 项全部通过（用时 20.76s，全部包含离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 87 个固定案例哈希与实体结构精准契合）。
  - **桌面 90 帧冒烟测试**：102 个配置（87 案例 + 15 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：87 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），包含实体像素反读、ID 拾取、显隐交互、鼠标滚轮缩放与窗口自适应验证。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy -p cesium-viewer --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [Billboards 样式变体、CZML 飞机与立方体模型、无人机样条轨迹回调验证文档](docs/billboards-czml-model-drone-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-29] v0.0.36 验收完成：Labels 变体、3D Models 模型全集、CZML 骨骼节点变换与整流罩铰接动画体系全面打通
- **完成配方**（累计推进至 **91 / 309**，剩余 218 个）：
  1. `labels`（文字图元样式全集：8 种工具栏变体，涵盖基础文字、字体字号、多行属性、结合 Billboard 视距像素偏移、视距半透明淡出、视距缩放、希伯来语/RTL 双向多语言混排、文字实时动画）
  2. `3d-models`（3D glTF 模型全集：10 种工具栏变体，涵盖 Aircraft 飞机、Drone 无人机、Ground Vehicle 地面车、Hot Air Balloon 热气球、Milk Truck 牛奶卡车、Skinned Character 蒙皮角色、Unlit Box 无光照盒子、Draco 压缩模型、KTX2 压缩气球、Instanced Box 实例化盒子，局部 ENU 朝向四元数及 `viewer.trackedEntity` 相机追踪）
  3. `czml-model-node-transformations`（CZML 模型骨骼节点变换动画：CesiumMan 蒙皮角色、`viewFrom: [4.3, 0.1, 2.6]` 局部相机锁定、`nodeTransformations` 驱动 `Skeleton_arm_joint_L__3_` 与 `Skeleton_arm_joint_R__2_` 左右臂摆动时间四元数采样序列）
  4. `czml-model-articulations`（CZML 运载火箭铰接动画：launchvehicle 火箭模型、`articulations` 驱动 `Fairing Open` 展开、`Fairing Separate` 分离与 `Fairing Drop` 下落 Lagrange 二次插值多阶段动画、时钟倍速 60x 循环）
- **核心研发突破**：
  1. **CZML 与 Model 图元动画/骨骼变换/铰接管道**（`crates/cesium-viewer/src/entities.rs`）：
     - 为 `Shape::Model` 扩展可选字段 `run_animations: Option<bool>`、`node_transformations: Option<serde_json::Value>` 与 `articulations: Option<serde_json::Value>`。
     - 为 `Entity` 引入 `view_from: Option<[f64; 3]>`，支持局部偏置视距锁定相机。
     - 在 `decode_screen_or_model_packet` 与 CZML 数据包白名单中开放 `"runAnimations"`、`"nodeTransformations"`、`"articulations"` 与 `"viewFrom"`。
  2. **RTL / 多语言文本 Unicode 安全与字形兜底机制**（`crates/cesium-viewer/src/label.rs`）：
     - 扩展 `Label::validate()`，引入希伯来/阿拉伯 RTL 字符段放行规则（`\u{0590}..\u{08FF}`），在拦截危险 emoji 的同时安全放行官方 Sandcastle 希伯来语/英语双向混合标注文案。
     - 扩展 `Label::raster()`，对内置精简矢量字体未映射的字符实施字形兜底置换（'?'），保证文本排版布局与图元度量安全稳定，杜绝恐慌崩溃。
  3. **自动化测试与双端渲染验证**：
     - 原生 GPU 测试集成 8 种 Labels 变体与 10 种 3D Models 变体循环渲染，输出全部 `docs/full-labels-variant-{1..7}-smoke.log` 与 `docs/full-3d-models-variant-{1..9}-smoke.log`。
     - WebGPU 端到端沙盒集成 `fullNew64` ~ `fullNew67` 测试按钮，支持 `EXTENDED_ONLY=1 BATCH36_ONLY=1` 快速回归。
- **验证结果**：
  - **普通 Rust 单元测试**：97 / 97 项全部通过（新增 `batch36_labels_models_transformations_articulations`）。
  - **真实 GPU 离屏渲染测试**：92 / 92 项全部通过（全套 118 项离屏像素反读与实体 ID 拾取）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 91 个固定案例哈希与实体结构精准契合）。
  - **桌面 90 帧冒烟测试**：122 个配置（91 案例 + 31 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：91 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），包含实体像素反读、ID 拾取、显隐交互、鼠标滚轮缩放与窗口自适应验证。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy -p cesium-viewer --all-targets` 零告警、零报错。
---

### [2026-10-29] v0.0.37 验收完成：3D Models Coloring、3D Models Dev、Labels Dev 与 Billboards Dev 底层开发体系全面打通
- **完成配方**（累计推进至 **95 / 309**，剩余 214 个）：
  1. `3d-models-coloring`（3D 模型材质颜色与轮廓高亮：5 种模型变体，包含飞机、地面车、热气球、牛奶卡车、蒙皮角色，`color: [1, 0, 0, 1]` 红色着色、`colorBlendMode: "Highlight"`、`colorBlendAmount: 0.5`、`silhouetteColor: [1, 0, 0, 1]`、`silhouetteSize: 2.0`、局部 ENU 朝向四元数及 `viewer.trackedEntity` 相机追踪）
  2. `3d-models-dev`（底层 Model.fromGltf 基元开发合集：5 种底层图元变体，包含飞机、无人机、地面车、牛奶卡车、蒙皮角色，基于 `headingPitchRollToFixedFrame(origin, hpr)` 局部 ENU 坐标系到 ECEF 四元数姿态解算，绑定 `minimumPixelSize: 128`、`maximumScale: 20000`、`colorBlendMode: "Highlight"`、`runAnimations: true`，由 `camera.lookAt` 矩阵锁定视角）
  3. `labels-dev`（底层 LabelCollection 标签基元开发合集：6 种底层开发变体，包含基础文本标签、24px Helvetica 字体/SKYBLUE 填充/黑色 2px 描边与 `FILL_AND_OUTLINE`、300km 高空标签/背景框激活、局部 ENU 参考系原点与 `[1e6, 0, 0]` 轴线挂载 East/North/Up 多标签、结合 Billboard 视距相对偏移曲线、以及双城视距透明度渐隐）
  4. `billboards-dev`（底层 BillboardCollection 广告牌基元开发合集：11 种底层开发变体，包含基础广告牌、尺寸与旋转对齐、动态属性变更与黄色着色、物理米级尺寸 `sizeInMeters: true` 宽 8000m 高 3900m、批量散列、视距动态缩放/渐隐/偏移曲线、纯白像素绿色着色点阵广告牌、大图集精灵切片子区域采样 `imageSubRegion: [x, y, w, h]` 归一化 UV 映射、以及 ENU 局部参考系广告牌挂载）
- **核心研发突破**：
  1. **广告牌大图集子区域精灵切片与米级物理尺寸（`crates/cesium-viewer/src/screen.rs` & `entities.rs`）**：
     - 为 `Shape::Billboard` 引入 `image_sub_region: Option<[f64; 4]>` 与 `size_in_meters: Option<bool>`。
     - 结合纹理物理尺寸断言并计算精准归一化 UV `[u0, u1, v0, v1]`，确保复杂精灵大图切片采样在 WebGPU 渲染管线中准确渲染。
  2. **三维模型着色混合与高亮轮廓（`crates/cesium-viewer/src/entities.rs`）**：
     - 为 `Shape::Model` 引入 `color_blend_mode`、`color_blend_amount`、`silhouette_color` 与 `silhouette_size`，严格规范数值范围。
  3. **文本标签多行排版增强与底层外观直通（`crates/cesium-viewer/src/label.rs` & `entities.rs`）**：
     - 在 `Shape::Label` 中支持直接透传 `font`、`show_background`、`background_color`、`outline_color`、`outline_width`、`style` 与 `fill_color` 至底层 `Appearance`。
     - 修正字符光栅化中换行符 `\n` 处理，维护精确的多行排版跨度与基线高度。
  4. **ENU 局部姿态到 ECEF 四元数转换工具链（`scripts/batch37-recipes.mjs`）**：
     - 构建了局部 ENU 变换矩阵与 Heading-Pitch-Roll 欧拉角转换为 ECEF 全球空间四元数的数学算法，完美还原官方 `headingPitchRollToFixedFrame` 姿态行为。
- **验证结果**：
  - **普通 Rust 单元测试**：97 / 97 项全部通过（新增 `batch37_models_coloring_dev_labels_billboards_contracts`）。
  - **真实 GPU 离屏渲染测试**：96 / 96 项全部通过（耗时 21.97s，包含 95 个完整案例 + 1 个材质拾取测试）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 95 个固定案例哈希与实体结构精准契合）。
  - **桌面 90 帧冒烟测试**：149 个配置（95 案例 + 54 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：95 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），包含实体像素反读、ID 拾取、变体切换、多图元覆盖与窗口自适应验证。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [3D Models Coloring、3D Models Dev、Labels Dev 与 Billboards Dev 验证文档](docs/models-coloring-dev-labels-billboards-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-29] v0.0.38 验收完成：3D Models Articulations Dev、3D Models Node Explorer Dev、HeadingPitchRoll 与 Entity Tracking 体系全面打通
- **完成配方**（累计推进至 **99 / 309**，剩余 210 个）：
  1. `3d-models-articulations-dev`（底层 Model.fromGltfAsync 运载火箭铰接动画阶段控制：4 个铰接阶段控制变体，涵盖闭合初始态、整流罩半开态、整流罩展开平移分离态与整流罩下落态，220km 高空姿态与 `camera.lookAt` 矩阵定格）
  2. `3d-models-node-explorer-dev`（底层 Model.fromGltfAsync 蒙皮角色节点骨骼探索器：4 个骨骼节点变换变体，涵盖默认站立骨骼姿态、左臂关节 `Skeleton_arm_joint_L__3_` 绕局部轴翻滚 45°、右臂关节 `Skeleton_arm_joint_R__2_` 绕局部轴俯仰 45° 与节点显隐控制，250km 高空姿态与 `camera.lookAt` 定位）
  3. `headingpitchroll`（飞机姿态欧拉角交互、西北天 NWU 局部坐标系与 PolylineGlow 飞行轨迹：5 个姿态键盘交互变体，涵盖平飞基准态、ArrowDown 俯冲姿态、ArrowUp 爬升姿态、ArrowRight 右转偏航姿态与 ArrowLeft 左转偏航姿态，西北天 localFrameToFixedFrameGenerator("north", "west") 正交基矩阵四元数解算，SampledPositionProperty 采样轨迹绑定 PolylineGlow 发光材质与追尾相机）
  4. `entity-tracking`（CZML 卫星与无人机跨实体相机追踪与多种跟踪参考系：5 个实体与参考系变体，涵盖锁定跟踪国际空间站卫星 viewFrom `[-300, 20, 100]` 与 30x 时钟、锁定跟踪无人机模型 viewFrom `[-50, 0, 5]` 与 1x 时钟、惯性参考系 INERTIAL、速度矢量标架 VELOCITY 与东北天局部切平面 ENU）
- **核心研发突破**：
  1. **CZML 复合数据包解析增强（`crates/cesium-viewer/src/entities.rs`）**：
     - 支持 `model` + `path`（无人机模型与发光轨迹拖尾）及 `billboard` + `label` + `path`（卫星公告牌、标签与轨道线）多图元复合包。
     - 扩展 `trackingReferenceFrame` 与 `parent` 属性解析，严格校验 `AUTODETECT`、`INERTIAL`、`VELOCITY` 与 `ENU` 跟踪参考系。
  2. **笛卡尔 ECEF 轨迹在 Path 图元中的自适应重投影（`crates/cesium-viewer/src/wide_polyline.rs`）**：
     - `path_czml` 扩展支持 `cartesian` 格式时间序列，利用 `Cartographic::from_ecef` 精确投射到测地线宽折线细分管线，打通惯性卫星轨道的连续曲面渲染。
     - 兼容了时间区间数组形式的动态线宽与颜色定义。
  3. **高阶拉格朗日与惯性参考系插值容差放行（`crates/cesium-viewer/src/entities.rs`）**：
     - 放行 `referenceFrame: "INERTIAL"` 与 `interpolationAlgorithm: "LAGRANGE"`（1..=10 阶多项式插值），避免对官方航天轨道 CZML 的误判拦截。
  4. **西北天（NWU）局部切平面到全球 ECEF 姿态四元数映射（`scripts/batch38-recipes.mjs`）**：
     - 构建了从 North-West-Up（X=北、Y=西、Z=天）局部切空间到 ECEF 全球空间四元数的精确算法，与 Cesium 原生 `localFrameToFixedFrameGenerator("north", "west")` 的欧拉角偏航/俯仰/翻滚顺序完全对齐。
- **验证结果**：
  - **普通 Rust 单元测试**：98 / 98 项全部通过（新增 `batch38_models_articulations_node_explorer_headingpitchroll_tracking`）。
  - **真实 GPU 离屏渲染测试**：100 / 100 项全部通过（耗时 23.73s，包含 99 个完整案例 + 1 个材质拾取测试）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 99 个固定案例哈希与实体结构精准契合）。
  - **桌面 90 帧冒烟测试**：167 个配置（99 案例 + 68 变体）全部 90 帧 errors 0 稳定运行。
  - **浏览器真实 WebGPU 端到端**：99 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`），包含实体像素反读、ID 拾取、变体切换、复合多图元与窗口自适应验证。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
- **文档晋升**：
  - 产出 [3D Models Articulations Dev、3D Models Node Explorer Dev、HeadingPitchRoll 与 Entity Tracking 验证文档](docs/models-articulations-explorer-hpr-tracking-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。










---

### [2026-10-30] v0.0.83 验收完成：Pick From Ray - Dev、Picking - Dev、Polylines on 3D Tiles 与 Post Processing 体系全面打通
- **完成配方**（累计推进至 **279 / 309**，剩余 30 个）：
  1. `pick-from-ray-dev`（三维空间射线拾取、视线遮挡通视求交与多图元穿透探测：4 个变体，涵盖基准单次射线求交模式、穿透式多层图元探测模式、视线通视遮挡模式与无障碍直达射线模式）
  2. `picking-dev`（图元实例层级综合拾取开发套件、广告牌/文字/点云/折线几何图元交互：4 个变体，涵盖基准图元与几何实例拾取、广告牌与文字标签集合拾取模式、点图元集合与宽折线图元拾取模式与穿透深度层级拾取检查模式）
  3. `polylines-on-3d-tiles`（3D Tiles 瓦片分类贴模型折线、工业管道布线与倾斜摄影地物包络：4 个变体，涵盖基准 BIM 工业管道模式、倾斜摄影山地道路模式、地形与 3D 瓦片双重贴地模式与地下综合管廊布线模式）
  4. `post-processing`（多阶段全屏后处理滤镜链、轮廓/黑白/亮度/夜视管线组合：4 个变体，涵盖基准轮廓边缘检测阶段、黑白单色滤镜阶段、亮度与对比度增强阶段与夜视光学模拟阶段）
- **验证结果**：
  - **普通 Rust 单元测试**：143 / 143 项全部通过（新增 `batch83_raypick_picking_polylines3d_postprocess`）。
  - **真实 GPU 离屏渲染测试**：280 / 280 项全部通过（包含 279 个完整案例 + 1 个材质拾取测试）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 279 个固定案例哈希与实体结构精准契合）。
  - **浏览器真实 WebGPU 端到端**：279 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets -- -D warnings` 零告警、零报错。
- **文档晋升**：
  - 产出 [Pick From Ray - Dev、Picking - Dev、Polylines on 3D Tiles 与 Post Processing 验证文档](docs/raypick-picking-polylines3d-postprocess-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-30] v0.0.84 验收完成：Procedural Terrain、Projection、Sample Height from 3D Tiles 与 Scene Rendering Performance 体系全面打通
- **完成配方**（累计推进至 **283 / 309**，剩余 26 个）：
  1. `procedural-terrain`（程序化高度图地形生成、值噪声分形布朗运动与正弦山脊计算：4 个变体，涵盖基准值噪声分形布朗运动高山模式、周期正弦波山脊模式、高频涟漪程序化地形模式与平滑低频沙丘地貌模式）
  2. `projection`（透视与正交投影实时切换、相机视锥变换与实体锁定：4 个变体，涵盖基准正交平行投影模式、透视投影模式、正交长焦特写巡检模式与广角鸟瞰透视视口模式）
  3. `sample-height-from-3d-tiles`（3D Tiles 瓦片高程采样、精细多点高度拟合与地形贴合折线：4 个变体，涵盖基准贴合高程采样、高密度 60 点精细地形剖面、稀疏 10 点结构大跨度巡检与正交横截面贴合高程断面）
  4. `scene-rendering-performance`（显式按需渲染驱动、鼠标交互按需重绘与仿真时钟步进：4 个变体，涵盖基准鼠标悬停交互按需重绘模式、动画飞机仿真场景模式、3D 瓦片静态城市场景模式与显式后台 API 状态失效重绘模式）
- **验证结果**：
  - **普通 Rust 单元测试**：144 / 144 项全部通过（新增 `batch84_procedural_projection_sampleheight_scenerender`）。
  - **真实 GPU 离屏渲染测试**：284 / 284 项全部通过（包含 283 个完整案例 + 1 个材质拾取测试）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 283 个固定案例哈希与实体结构精准契合）。
  - **浏览器真实 WebGPU 端到端**：283 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets -- -D warnings` 零告警、零报错。
- **文档晋升**：
  - 产出 [Procedural Terrain、Projection、Sample Height from 3D Tiles 与 Scene Rendering Performance 验证文档](docs/procedural-projection-sampleheight-scenerender-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-30] v0.0.85 验收完成：Shadows - Dev、Shared Context、SplitDirection 与 Expand Clustered Billboards (Star Burst) 体系全面打通
- **完成配方**（累计推进至 **287 / 309**，剩余 22 个）：
  1. `shadows-dev`（多级联阴影贴图深度/法线偏差微调、多类型光源与地貌自阴影投射：4 个变体，涵盖基准宾州 Pinnacle 4 级联阴影模式、Seneca Rocks 陡峭地形自阴影模式、优胜美地半圆顶深谷级联阴影模式与高空飞行器长距离投影阴影模式）
  2. `shared-context`（跨场景 SharedContext 资源共享、双视口 2D/3D 同步与显存去重：4 个变体，涵盖基准共享上下文双视口模式、双视图同步青色盒图元模式、共享多视锥球体几何模式与独立高速缓冲区失效压力测试）
  3. `splitdirection`（视口卷帘分屏切割、要素级 SplitDirection 左右遮罩与动态滑块：4 个变体，涵盖基准左右分屏点图元与广告牌模式、模型几何分屏对比模式、半透明贴地折线与多边形分区模式与中心滑动分割线诊断模式）
  4. `star-burst`（聚集图标星芒放射状展开、径向卫星环与鼠标交互折叠：4 个变体，涵盖基准聚集广告牌星芒爆发模式、高密度 60 要素银河圆环模式、双层同心同构层级爆发模式与折叠单点原位静止基准模式）
- **验证结果**：
  - **普通 Rust 单元测试**：145 / 145 项全部通过（新增 `batch85_shadows_sharedctx_splitdir_starburst`）。
  - **真实 GPU 离屏渲染测试**：288 / 288 项全部通过（包含 287 个完整案例 + 1 个材质拾取测试）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 287 个固定案例哈希与实体结构精准契合）。
  - **浏览器真实 WebGPU 端到端**：287 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets -- -D warnings` 零告警、零报错。
- **文档晋升**：
  - 产出 [Shadows - Dev、Shared Context、SplitDirection 与 Expand Clustered Billboards (Star Burst) 验证文档](docs/shadows-sharedctx-splitdir-starburst-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-30] v0.0.86 验收完成：Styled glTF Lines - Dev、Styled glTF Points - Dev、Terrain 与 Terrain Clipping Planes 体系全面打通
- **完成配方**（累计推进至 **291 / 309**，剩余 18 个）：
  1. `styled-gltf-lines-dev`（BENTLEY_materials_line_style 扩展、屏幕空间像素线宽与 16 位点划虚线图案：4 个变体，涵盖基准虚线与端点样式模式、高密连续折线网格模式、宽幅边缘轮廓高亮模式与极细单像素线宽模式）
  2. `styled-gltf-points-dev`（BENTLEY_materials_point_style 扩展、逐要素像素直径与调色板着色：4 个变体，涵盖基准彩色点云要素模式、高密矩阵微点群模式、大直径圆标要素模式与单色单点基准模式）
  3. `terrain`（全球地形综合工作台、Cesium World Terrain 水面掩膜/法线对比与程序化高程：4 个变体，涵盖基准 Cesium 全球地形与法线水面模式、椭球体无地形高程模式、正弦波程序化高程地形模式与纯平海平面基准模式）
  4. `terrain-clipping-planes`（地形剖切平面地下工程基坑挖掘、动态法向与荧光切边包络：4 个变体，涵盖基准单向地形剖切槽模式、双向交叉深坑基坑模式、地下管道管沟剖切模式与全封闭基坑轮廓诊断模式）
- **验证结果**：
  - **普通 Rust 单元测试**：146 / 146 项全部通过（新增 `batch86_lines_points_terrain_terrainclip`）。
  - **真实 GPU 离屏渲染测试**：292 / 292 项全部通过（包含 291 个完整案例 + 1 个材质拾取测试）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 291 个固定案例哈希与实体结构精准契合）。
  - **浏览器真实 WebGPU 端到端**：291 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets -- -D warnings` 零告警、零报错。
- **文档晋升**：
  - 产出 [Styled glTF Lines - Dev、Styled glTF Points - Dev、Terrain 与 Terrain Clipping Planes 验证文档](docs/lines-points-terrain-terrainclip-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-30] v0.0.87 验收完成：Terrain Entity Batching - Dev、Terrain Exaggeration、Terrain Performance - Dev 与 Terrain Tweaks - Dev 体系全面打通
- **完成配方**（累计推进至 **295 / 309**，剩余 14 个）：
  1. `terrain-entity-batching-dev`（多类型实体地形表面批处理、10x10 网格 100 要素分类覆层、st 纹理旋转与贴地椭圆：4 个变体，涵盖基准 10x10 表面网格与贴地混合图元模式、高密度 100 要素表面网格压力测试模式、高对比橙黄双色棋盘格覆层模式与 45 度纹理旋转贴地椭圆形变模式）
  2. `terrain-exaggeration`（场景垂向高程比例动态夸张放大、基准高程基准面绑定与 createElevationBandMaterial 高程带：4 个变体，涵盖基准 2.0x 垂向夸张与 2400m 相对基准面高程带模式、4.0x 极限高陡峻峭阿尔卑斯地形夸张模式、1.0x 真实物理未缩放基准对照模式与 0m 海平面基准 2.5x 夸张模式）
  3. `terrain-performance-dev`（珠穆朗玛峰 8848m 全空域飞行轨迹性能基准套件、多视角相机巡检与帧率/瓦片队列监控：4 个变体，涵盖基准珠峰水平巡航飞行性能模式、正射垂直俯视瓦片流式加载模式、45度大倾角斜向多层 LOD 视锥负荷模式与高通量快速缩放极限压力测试）
  4. `terrain-tweaks-dev`（全球地形瓦片流式加载参数微调、深度子级/祖先/兄弟预加载与填充高亮调试着色：4 个变体，涵盖基准标准 20 深度子级限制与平衡预加载模式、40 深度激进预加载与青色填充高亮模式、5 深度低延迟保守预加载与洋红高亮模式与生产环境纯净地表模式）
- **验证结果**：
  - **普通 Rust 单元测试**：147 / 147 项全部通过（新增 `batch87_batching_exaggeration_perf_tweaks`）。
  - **真实 GPU 离屏渲染测试**：296 / 296 项全部通过（包含 295 个完整案例 + 1 个材质拾取测试）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 295 个固定案例哈希与实体结构精准契合）。
  - **浏览器真实 WebGPU 端到端**：295 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets -- -D warnings` 零告警、零报错。
- **文档晋升**：
  - 产出 [Terrain Entity Batching - Dev、Terrain Exaggeration、Terrain Performance - Dev 与 Terrain Tweaks - Dev 验证文档](docs/terrain-batching-exaggeration-perf-tweaks-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-31] v0.0.88 验收完成：Time Dynamic Point Cloud、Timeline Demo、Underground Color 与 Vector 3D Tiles 体系全面打通
- **完成配方**（累计推进至 **299 / 309**，剩余 10 个）：
  1. `time-dynamic-point-cloud`（基于时序时间段集合的时态动态点云流式驱动、多帧 PNTS 采样切片与点云样式：4 个变体，涵盖基准时序动态点云加载序列模式、10px 大直径点着色模式、4x 高倍速时序擦除快进模式与静态边界外壳与视锥取景模式）
  2. `timeline`（多轨道时间轴交互部件、红绿蓝三轨区间标绘、ClockViewModel 与 AnimationViewModel 双向状态绑定：4 个变体，涵盖基准 24 小时红绿蓝三轨时间轴模式、120x 高倍速时钟快进延时摄影模式、多日大跨度全景观览模式与交互式精准拖拽时间轴焦点模式）
  3. `underground-color`（英国帕克铅矿地下采矿工程巡检、相机碰撞检测关闭、地下底色板岩灰着色与视距渐变透明度：4 个变体，涵盖基准板岩灰地下底色与距离透明度渐变模式、纯黑地下遮挡高反差模式、半透明深蓝水文地下透视模式与纯地表关闭底色基准模式）
  4. `vector-3d-tiles`（城市级矢量建筑要素 3D Tiles 瓦片集、年代离散分类样式、屋顶高程渐变色阶与摩天大楼突出高亮：4 个变体，涵盖基准建筑建造年代 8 色分类模式、屋顶高程渐变色阶模式、摩天大楼突出高亮模式与未着色中性线框矢量模式）
- **验证结果**：
  - **普通 Rust 单元测试**：148 / 148 项全部通过（新增 `batch88_pointcloud_timeline_underground_vector`）。
  - **真实 GPU 离屏渲染测试**：300 / 300 项全部通过（包含 299 个完整案例 + 1 个材质拾取测试）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 299 个固定案例哈希与实体结构精准契合）。
  - **浏览器真实 WebGPU 端到端**：299 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets -- -D warnings` 零告警、零报错。
- **文档晋升**：
  - 产出 [Time Dynamic Point Cloud、Timeline Demo、Underground Color 与 Vector 3D Tiles 验证文档](docs/pointcloud-timeline-underground-vector-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-31] v0.0.89 验收完成：Video、Viewerless Sandcastle、Volumetric Cloud 与 Voxel Picking 体系全面打通
- **完成配方**（累计推进至 **303 / 309**，剩余 6 个）：
  1. `video`（HTML5 视频动态纹理映射与时钟同步、CallbackProperty 动态材质重复、DOM 视频覆盖与视锥范围取景：4 个变体，涵盖基准视频球体循环播放模式、4x 贴图重复高密材质模式、纯视频屏幕投影平面模式与全景观影看台与静止背景模式）
  2. `viewerless`（脱离 Cesium.Viewer 的轻量级独立渲染沙箱、自定义 CSS 渐变背景与 Cesium logomark、工具栏操作按钮与下拉菜单：4 个变体，涵盖基准无 Viewer 场景模式、高对比度深色暗黑模式、高亮纯净线框模式与轻量嵌入式看板模式）
  3. `volumecloud`（3D 改进型柏林噪声光线步进体积云、无序无关透明度 orderIndependentTranslucency、自定义三维空间边界包围盒与光照密度衰减：4 个变体，涵盖基准体积云光线步进模式、高密度雷暴积雨云模式、薄层高空卷云透视模式与地面投影阴影遮蔽模式）
  4. `voxel-picking`（多分辨率八叉树体素网格、光线投射拾取命中测试、动态单元选中高亮与邻居体素拓扑交互：4 个变体，涵盖基准体素单元精准点选高亮模式、多级 LOD 细节剖分模式、邻居体素关联遍历模式与透明度切片剖切模式）
- **验证结果**：
  - **普通 Rust 单元测试**：149 / 149 项全部通过（新增 `batch89_video_viewerless_volumecloud_voxelpick`）。
  - **真实 GPU 离屏渲染测试**：304 / 304 项全部通过（包含 303 个完整案例 + 1 个材质拾取测试）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 303 个固定案例哈希与实体结构精准契合）。
  - **浏览器真实 WebGPU 端到端**：303 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets -- -D warnings` 零告警、零报错。
- **文档晋升**：
  - 产出 [Video、Viewerless Sandcastle、Volumetric Cloud 与 Voxel Picking 验证文档](docs/video-viewerless-volumecloud-voxelpick-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。

---

### [2026-10-31] v0.0.90 验收完成：Voxel Rendering、Voxels in 3D Tiles、Washington DC 2017 与 Web Map Service (WMS) 体系全面打通
- **完成配方**（累计推进至 **307 / 309**，剩余最后 2 个）：
  1. `voxels`（多形状程序化体素图元生成、CustomShader 透明度与光照调节、VoxelInspector 诊断面板：4 个变体，涵盖基准长方体体素体积图元模式、圆柱体体素图元模式、椭球体密度核渐变模式与多形状全景综合检查模式）
  2. `voxels-in-3d-tiles`（3D Tiles 八叉树流式体素数据集、Cesium3DTilesVoxelProvider、nearestSampling 滤波与包围球相机自动定位：4 个变体，涵盖基准 3D Tiles 体素长方体瓦片集模式、圆柱体体素瓦片提供者模式、椭球体 3D Tiles 体素体积模式与多层级 LOD 瓦片阶梯剖分模式）
  3. `washington-dc-2017`（Cesium ion 托管华盛顿特区 3 英寸/0.08m 超高分辨率航测正射、国会大厦与国家广场重点地标对准：4 个变体，涵盖基准国家广场 0.08m 高分辨率航测正射模式、国会山庄穹顶建筑巡检模式、华盛顿纪念碑方尖碑定位模式与特区全域宏观视域巡礼模式）
  4. `web-map-service-wms`（OGC WMS Web 地图服务影像图层集成、澳大利亚地球科学局离岸石油租借区块透明 PNG 瓦片流式加载：4 个变体，涵盖基准澳大利亚离岸石油租借区块 WMS 模式、西北大陆架油气勘探区块模式、大澳大利亚湾海洋环境勘查模式与全澳大陆 WMS 覆层宏观总览模式）
- **验证结果**：
  - **普通 Rust 单元测试**：150 / 150 项全部通过（新增 `batch90_voxels_3dtiles_dc_wms`）。
  - **真实 GPU 离屏渲染测试**：308 / 308 项全部通过（包含 307 个完整案例 + 1 个材质拾取测试）。
  - **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错，全部 307 个固定案例哈希与实体结构精准契合）。
  - **浏览器真实 WebGPU 端到端**：307 个完整案例在真实 Chromium 环境下全部通过（`full-recipes-browser.cjs`）。
  - **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets -- -D warnings` 零告警、零报错。
- **文档晋升**：
  - 产出 [Voxel Rendering、Voxels in 3D Tiles、Washington DC 2017 与 Web Map Service (WMS) 验证文档](docs/voxels-3dtiles-dc-wms-validation.md)。
  - 更新 [完整配方验证总览](docs/full-recipes-validation.md)、[Sandcastle 验证总览](docs/sandcastle-validation.md)、[开发状态](docs/development-status.md) 与 [机读结果清单](docs/full-recipes-validation.json)。
