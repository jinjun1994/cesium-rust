# Batch 81 (v0.0.81) 案例验证报告：Panorama、Particle System、Particle System Fireworks 与 Particle System Tails

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 360 度等距圆柱投影与六面成立方体天空盒全景 (Panorama)、基于物理引力与运动载具体系粒子系统 (Particle System)、多重随机散布爆炸烟花与径向速度截止 (Particle System Fireworks)、以及超音速飞行器双模动态尾迹粒子带与大地螺旋发散 (Particle System Tails) 的配方，累计完成 **271 / 309**（剩余 38 个）：

1. **`panorama`（360 度全景沉浸式球体/立方体贴图漫游、相机 lookAt 姿态锁定与鼠标滚轮视场 FOV 缩放）**
   - 官方地址：`https://sandcastle.cesium.com/?id=panorama`
   - 变体全集（4 种全景投影与视场交互变体）：
     - `Baseline: Equirectangular 360 Panorama (Laon Cathedral Interior, Standard 60° FOV)`（基准等距长方圆柱 360 全景：法国拉昂大教堂内部，标准 60° 视场）
     - `Cube Map Skybox Panorama Mode (Phnom Penh Central Market, 6-Sided Cubic Frame)`（六面成立方体天空盒全景模式：金边中央市场，6 面立方体框架）
     - `Narrow Telephoto Optical Zoom Mode (FOV 25°, High-Precision Architectural Detail)`（望远长焦光学变焦模式：25° 狭窄视场，高精建筑细部透视）
     - `Ultra-Wide Immersive Panorama Mode (FOV 95°, Expanded Environmental Peripheral View)`（超广角沉浸式全景模式：95° 宽广视场，扩展环境外围视野）
   - 核心要素：
     - 构建等距圆柱全景投影球（`Shape::Ellipsoid`）、六面成立方体天空盒体量（`Shape::Box`）、全景相机中心方位基准柱（`Shape::Cylinder`）与全景交互勘测包围矩形（`Shape::Rectangle`）。
     - 验证了 `Cesium.EquirectangularPanorama` 与 `Cesium.CubeMapPanorama` 全景图元添加与动态替换、`Transforms.eastNorthUpToFixedFrame` 局部定向变换矩阵、`camera.lookAt` 原点环顾姿态解算、以及滚轮平滑 FOV 视场调谐（20° ~ 100°）。

2. **`particle-system`（运奶车运动轨迹跟踪、Circle/Sphere/Cone 多型粒子发射器与局部引力扰动）**
   - 官方地址：`https://sandcastle.cesium.com/?id=particle-system`
   - 变体全集（4 种物理动力学粒子发射变体）：
     - `Baseline: Circle Emitter Smoke Plume (Standard Light Sea Green to White Exhaust)`（基准圆形发射器尾气烟流：浅海绿至白色半透明渐变）
     - `Volumetric Sphere Emitter Mode (Dense Omni-Directional 25.0 Rate Emission)`（球形发射器全向膨胀模式：25.0 高发射率高密度烟雾）
     - `High-Velocity Directional Cone Jet (Narrow 45° Cone, Maximum Speed 8.0 m/s)`（高速方向性锥体喷流：45° 聚束锥体，8.0 m/s 尾喷速度）
     - `Gravitational Deflection Plume Mode (Downwards Vector Gravity -9.8 m/s²)`（重力偏转下沉烟羽模式：地心局部向下 -9.8 m/s² 引力加速度）
   - 核心要素：
     - 构建运奶车模型包围盒（`Shape::Box`）、圆形发射器物理喷口截面柱（`Shape::Cylinder`）、多阶段脉冲膨胀烟团球（`Shape::Ellipsoid`）与时态采样运动走廊包围矩形（`Shape::Rectangle`）。
     - 验证了 `Cesium.ParticleSystem` 结合 `CircleEmitter` / `SphereEmitter` / `ConeEmitter` / `BoxEmitter` 多类型发射几何、时序 `ParticleBurst` 脉冲迸发、`SampledPositionProperty` 运动实体实时跟踪、以及基于地心坐标单位向量归一化的 local gravity `updateCallback`。

3. **`particle-system-fireworks`（夜空多级爆裂烟花、随机三维空间偏移与径向速度衰减截止）**
   - 官方地址：`https://sandcastle.cesium.com/?id=particle-system-fireworks`
   - 变体全集（4 种高空多色彩礼炮爆裂变体）：
     - `Baseline: Multi-Stage Pyrotechnic Salvo (20 Fireworks with Golden and Magenta Bursts)`（基准多级烟花齐射：20 组金黄与洋红混合爆裂）
     - `High-Altitude Emerald & Cyan Starbursts (Z-Axis +150m, Cool Luminescent Hues)`（高空翡翠与青蓝星芒：Z 轴 +150m 升空，冷色发光色调）
     - `Rapid-Fire Staccato Finale Mode (Burst Size 800, Short 4.0s Lifetime, Maximum Density)`（极速连发压轴华章模式：800 粒子单发、4.0s 短寿命高密度）
     - `Ground-Level Heavy Mortar Single Bloom (Massive 120m Explosion Radius)`（近地超重型礼炮单团绽放：120m 巨型爆心半径）
   - 核心要素：
     - 构建多阶段金色烟花核心爆心球（`Shape::Ellipsoid`）、高空翡翠青蓝齐射集簇盒（`Shape::Box`）、地面礼炮发射炮管基座柱（`Shape::Cylinder`）与烟花安全隔离包围矩形（`Shape::Rectangle`）。
     - 验证了 20 组粒子系统随机三维位置偏移分配（`xMin/xMax`, `yMin/yMax`, `zMin/zMax`）、动态颜色采样、以及当粒子飞离爆炸半径时将速度强制归零（`magnitudeSquared >= size * size`）的截止力函数。

4. **`particle-system-tails`（CesiumAir 超音速客机尾迹、彗星冰蓝冰晶流与火箭热能喷注双模切换）**
   - 官方地址：`https://sandcastle.cesium.com/?id=particle-system-tails`
   - 变体全集（4 种超音速飞行尾流变体）：
     - `Baseline: Comet Tail Cryogenic Particle Ribbon (100 Icy Blue / Silver Systems)`（基准彗尾深冷粒子飘带：100 组冰蓝与银灰低速弥散粒子系统）
     - `Rocket Thruster Thermal Afterburner Mode (50 Fiery Gold / Crimson High-Temp Systems)`（火箭助推热力加力模式：50 组炽热金红高温高喷射率粒子系统）
     - `Dual Wingtip Vortex Contrails Mode (Symmetric Aerodynamic Condensation Lines)`（双翼尖对称空气动力学涡流凝结尾迹模式）
     - `Low-Velocity Atmospheric Smoke Ribbon Mode (Broad Dispersion Drift Trail)`（低速大气烟带宽幅漂移弥散模式）
   - 核心要素：
     - 构建 Cesium_Air 飞机实体模型包络盒（`Shape::Box`）、彗尾冷凝粒子流截面柱（`Shape::Cylinder`）、火箭热力加力膨胀火球（`Shape::Ellipsoid`）与超音速巡航走廊包围矩形（`Shape::Rectangle`）。
     - 验证了 大数量（50 ~ 100 组）微型粒子系统并联架构、经纬度极坐标螺旋微积分扰动函数（Cartographic Step Integration）、以及机体模型矩阵与尾喷口偏移矩阵的级联变换绑定。

---

## 2. 覆盖与测试指标

- **累计完整移植案例数**：271 / 309
- **本批新增测试案例**：4 个
- **Rust 单元测试**：141 个 passed (`entities::tests::batch81_panorama_particles_fireworks_tails`)
- **Native GPU 测试**：272 passed (`tests/full-recipes-gpu.rs`，包含 271 个完整案例 + 1 个宽折线材质测试)
- **WebGPU 浏览器测试**：271 passed (`docs/full-recipes-browser.json`)
- **Node.js 测试套件**：8 个测试套件全绿（`controls`、`sdk`、`ui`、`terrain`、`sandcastle`、`images`、`camera`、`recipes`）
- **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets -- -D warnings` 零警告通过
