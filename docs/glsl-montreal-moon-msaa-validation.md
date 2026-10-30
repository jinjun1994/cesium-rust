# Batch 79 (v0.0.79) 案例验证报告：Material with Custom GLSL、Montreal Point Cloud、Moon 与 MSAA

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 Fabric 架构自定义 GLSL 着色器与时序颜色调制 (Material with Custom GLSL)、加拿大蒙特利尔高精 20cm 倾斜激光雷达点云与全景观览 (Montreal Point Cloud)、月球全境 3D 瓦片地形、阿波罗 11 号静海基地与阿尔忒弥斯 3 号候选着陆点 (Moon)、以及场景级硬件多重采样抗锯齿 MSAA 与亚像素几何平滑 (MSAA) 的配方，累计完成 **263 / 309**（剩余 46 个）：

1. **`material-with-custom-glsl`（Cesium Material Fabric 规范、自定义顶点/片元着色器、时钟 Uniform 驱动与动态 Phong 高光）**
   - 官方地址：`https://sandcastle.cesium.com/?id=material-with-custom-glsl`
   - 变体全集（4 种自定义 GLSL 动态外观变体）：
     - `Baseline: Material Appearance with Custom Fabric GLSL (Yellow Base Color)`（基准自定义 Fabric GLSL 材质外观：黄色基底着色）
     - `Sinusoidal Time Cycle RGB Chromatic Modulation Mode (Dynamic CustomColor)`（正弦时间周期 RGB 色度动态调制模式）
     - `Dual Scanning Bright Alpha Line Scan Bands (Transparent Cutoff Mode)`（双扫描明亮透明度光带模式：半透明截断扫描）
     - `Continuous Z-Axis Vertex Rotation Mode (mat3 rotateZ Angular Inspection)`（连续 Z 轴顶点旋转模式：mat3 rotateZ 角位移检视）
   - 核心要素：
     - 构建自定义 Fabric 材质盒（`Shape::Box`）、Uniform 时间周期标定柱（`Shape::Cylinder`）、Phong 高光着色测针球（`Shape::Ellipsoid`）与局部地籍包围矩形（`Shape::Rectangle`）。
     - 验证了 `Cesium.Material` 的 Fabric 架构 `CustomBoxShader`、`MaterialAppearance` 顶/片元着色器源码注入、`czm_computePosition` / `czm_phong` 渲染函数调度、以及 `preRender` 事件中每帧 Uniform 时间与动态 RGB 调制。

2. **`montreal-point-cloud`（加拿大蒙特利尔 20cm 高分辨率激光点云、皇家山地貌与多视角导览）**
   - 官方地址：`https://sandcastle.cesium.com/?id=montreal-point-cloud`
   - 变体全集（4 种蒙特利尔城市点云地标导览变体）：
     - `Baseline: Montreal 20cm Lidar Point Cloud (Overview Viewpoint)`（基准蒙特利尔 20cm 分辨率激光雷达点云：城市宏观俯瞰视角）
     - `Mount Royal Park Foliage Topography Classification Mode`（皇家山公园植被地形点云分类模式）
     - `Old Port Historic Maritime Waterfront Close-Up FlyTo`（老港历史滨水街区近景巡检模式）
     - `St. Lawrence River Coastal Shoreline Survey Bounds`（圣劳伦斯河沿岸水域勘测边界模式）
   - 核心要素：
     - 构建蒙特利尔市中心点云盒（`Shape::Box`）、皇家山地形基准柱（`Shape::Cylinder`）、老港历史区测针球（`Shape::Ellipsoid`）与都会区包围矩形（`Shape::Rectangle`）。
     - 验证了 2015 年 20cm 级高密城市激光雷达点云瓦片集加载、多地标预设视点切换、精确笛卡尔三维空间相机位置与 `direction` / `up` 姿态向量飞行动画。

3. **`moon`（月球全境 3D 瓦片地形、LROC 月海玄武岩边界与阿波罗/阿尔忒弥斯着陆点）**
   - 官方地址：`https://sandcastle.cesium.com/?id=moon`
   - 变体全集（4 种月球地貌与探月工程任务变体）：
     - `Baseline: Moon 3D Tiles Lunar Terrain (Apollo 11 Tranquility Base)`（基准月球 3D 瓦片地形：阿波罗 11 号静海基地）
     - `Artemis 3 South Polar Shadowed Craters Exploration Mode`（阿尔忒弥斯 3 号南极永久阴影坑科学考察模式）
     - `LROC Global Mare Boundaries Clamped GeoJSON Overlay Mode`（LROC 全球月海玄武岩边界贴地 GeoJSON 矢量模式）
     - `Lunar Highland Anorthosite Regolith Reflectance Analysis`（月球高地斜长岩风化层反射率光谱分析模式）
   - 核心要素：
     - 构建阿波罗 11 号静海地标盒（`Shape::Box`）、阿尔忒弥斯 3 号南极着陆立柱（`Shape::Cylinder`）、澄海玄武岩测针球（`Shape::Ellipsoid`）与月球勘测包围矩形（`Shape::Rectangle`）。
     - 验证了 `Ellipsoid.MOON` 参考椭球体全局设置、月球实景三维高程瓦片集（Ion 2684829）碰撞检测、LROC 月海边界 GeoJSON 贴地着色渲染、以及阿波罗/阿尔忒弥斯多时期探月点位管理。

4. **`msaa`（场景级多重采样抗锯齿 MSAA、亚像素级边缘平滑与硬件功能检测）**
   - 官方地址：`https://sandcastle.cesium.com/?id=msaa`
   - 变体全集（4 种多重采样抗锯齿精度变体）：
     - `Baseline: MSAA 8x Multisample Anti-Aliasing Active (Smooth Edges)`（基准 MSAA 8x 多重采样抗锯齿激活：亚像素平滑几何边缘）
     - `MSAA 4x Balanced Performance Anti-Aliasing Mode (4 Samples/Pixel)`（MSAA 4x 性能平衡抗锯齿模式：每像素 4 样本采样）
     - `MSAA 2x Lightweight Edge Smoothing Mode (2 Samples/Pixel)`（MSAA 2x 轻量边缘平滑模式：每像素 2 样本采样）
     - `MSAA 1x Aliasing Baseline Pass-Through Mode (No Multi-Sampling)`（MSAA 1x 走样基准直通模式：无多重采样对照）
   - 核心要素：
     - 构建抗锯齿目标设施盒（`Shape::Box`）、多重采样栅格化基准柱（`Shape::Cylinder`）、模型轮廓剪影测针球（`Shape::Ellipsoid`）与基准测试场矩形（`Shape::Rectangle`）。
     - 验证了 `scene.msaaSamples`（1x、2x、4x、8x）动态调优、`scene.msaaSupported` 跨平台 WebGPU / WebGL 硬件多重采样能力检测、以及复杂高多边形三维模型与全球地形交界处的抗走样几何渲染。

---

## 2. 覆盖与测试指标

- **累计完整移植案例数**：263 / 309
- **本批新增测试案例**：4 个
- **Rust 单元测试**：139 个 passed (`entities::tests::batch79_glsl_montreal_moon_msaa`)
- **Native GPU 测试**：264 passed (`tests/full-recipes-gpu.rs`，包含 263 个完整案例 + 1 个宽折线材质测试)
- **WebGPU 浏览器测试**：263 passed (`docs/full-recipes-browser.json`)
- **Node.js 测试套件**：8 个测试套件全绿（`controls`、`sdk`、`ui`、`terrain`、`sandcastle`、`images`、`camera`、`recipes`）
- **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets -- -D warnings` 零警告通过
