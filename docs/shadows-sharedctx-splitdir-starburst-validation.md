# Batch 85 (v0.0.85) 案例验证报告：Shadows - Dev、Shared Context、SplitDirection 与 Expand Clustered Billboards (Star Burst)

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖高级阴影贴图深度偏差校准与多光源级联投射 (Shadows - Dev)、跨双视口 WebGL/WebGPU 显存上下文与几何资源共享 (Shared Context)、屏幕空间左右分屏滑动对比与图元裁剪 (SplitDirection)、以及聚集点广告牌星芒放射状展开与交互折叠 (Expand Clustered Billboards / Star Burst) 的配方，累计完成 **287 / 309**（剩余 22 个）：

1. **`shadows-dev`（多级联阴影贴图深度/法线偏差微调、多类型光源与地貌自阴影投射）**
   - 官方地址：`https://sandcastle.cesium.com/?id=shadows-dev`
   - 变体全集（4 种阴影深度偏差与复杂地形投射变体）：
     - `Baseline: Pinnacle PA 4-Cascade Shadow Mapping (Shadow Tester Model, Bias Tuning)`（基准宾州 Pinnacle 4 级联阴影模式：Shadow Tester 测试模型与偏差调优）
     - `Seneca Rocks Steep Terrain Self-Shadowing Mode (High-Angle Horizon Occlusion)`（Seneca Rocks 陡峭地形自阴影模式：高角度地平线自遮挡阴影）
     - `Half Dome Deep Valley Cascade Horizon Mode (Soft Shadow Edge Filtering)`（优胜美地半圆顶深谷级联阴影模式：软阴影边缘滤波与大跨度深度偏差）
     - `High-Altitude Airborne Cesium Air Shadow Projection Mode (Long-Range Cast)`（高空飞行器长距离投影阴影模式：CesiumAir 飞行模型超远投射）
   - 核心要素：
     - 构建 Shadow Tester 校准体量盒（`Shape::Box`）、方向光视锥光学矢量柱（`Shape::Cylinder`）、阴影深度偏差探针球（`Shape::Ellipsoid`）与 Pinnacle PA 阴影接收地面矩形（`Shape::Rectangle`）。
     - 验证了 `polygonOffset`、`polygonOffsetFactor`、`normalOffsetScale`、`normalShading`、`depthBias` 参数动态解算、4 级联瀑布阴影映射（CSM）、以及在地形、地球椭球面和 3D Tiles 混合场景下的阴影闭塞精度。

2. **`shared-context`（跨场景 SharedContext 资源共享、双视口 2D/3D 同步与显存去重）**
   - 官方地址：`https://sandcastle.cesium.com/?id=shared-context`
   - 变体全集（4 种多视口上下文显存共享变体）：
     - `Baseline: Shared Context Dual-Viewport (Green Semi-Transparent Cylinder, Black Outline)`（基准共享上下文双视口模式：半透明绿色圆柱与黑色外轮廓）
     - `Dual-Viewer Synchronized Cyan Box Primitive Mode (Cross-Context Texture Reuse)`（双视图同步青色盒图元模式：跨上下文纹理与片元管线复用）
     - `Shared Resource Multi-Frustum Sphere Geometry Mode (Shared Mesh Buffer)`（共享多视锥球体几何模式：单份顶点缓冲区跨多相机驱动）
     - `Independent High-Speed Buffer Invalidation Stress Test (Dynamic Pipeline Cycle)`（独立高速缓冲区失效压力测试：高频管线重置下的显存稳定性）
   - 核心要素：
     - 构建绿色圆柱（`Shape::Cylinder`，length 400km，radius 200km）、共享几何盒体（`Shape::Box`）、共享网格球体（`Shape::Ellipsoid`）与双视口共享舞台矩形（`Shape::Rectangle`）。
     - 验证了 `Cesium.SharedContext` 引擎支持、双 Viewer 独立相机生命周期绑定共享底层管线、避免重复分配 GPU 顶点/索引缓冲区以及零内存泄露。

3. **`splitdirection`（视口卷帘分屏切割、要素级 SplitDirection 左右遮罩与动态滑块）**
   - 官方地址：`https://sandcastle.cesium.com/?id=splitdirection`
   - 变体全集（4 种左右分屏要素裁剪变体）：
     - `Baseline: Left/Right Split Screen Points & Billboards (SplitDirection Left vs Right)`（基准左右分屏点图元与广告牌模式：天蓝左侧与红色右侧分区）
     - `Model Geometry Split Comparison Mode (Cesium Air Split Left, Cesium Man Right)`（模型几何分屏对比模式：左侧飞机模型与右侧人物模型同步）
     - `Translucent Clamped Polyline & Polygon Partition Mode (Left Cyan vs Right Magenta)`（半透明贴地折线与多边形分区模式：青色与洋红色卷帘左右对比）
     - `Central Swipe Boundary Diagnostic Mode (Active Splitter Position Tracking)`（中心滑动分割线诊断模式：动态跟踪屏幕像素滑动切分轴）
   - 核心要素：
     - 构建左侧分区天蓝点探针球（`Shape::Ellipsoid`）、右侧红色分区立柱（`Shape::Cylinder`）、中央分屏滑动界线盒（`Shape::Box`）与视口分割参考矩形（`Shape::Rectangle`）。
     - 验证了 `SplitDirection.LEFT`、`SplitDirection.RIGHT` 与 `SplitDirection.NONE` 片元着色器屏幕空间 X 坐标边界丢弃测试、以及点集与广告牌在分屏滑动时的精确切割。

4. **`star-burst`（聚集图标星芒放射状展开、径向卫星环与鼠标交互折叠）**
   - 官方地址：`https://sandcastle.cesium.com/?id=star-burst`
   - 变体全集（4 种星芒爆发展开变体）：
     - `Baseline: Clustered Billboards Star-Burst Mode (30 Icons Radial Orbit Expansion)`（基准聚集广告牌星芒爆发模式：30 个图标向外径向轨道环展开）
     - `High-Density 60 Entity Expanding Galaxy Ring Mode (Wide Dispersion Arc)`（高密度 60 要素银河圆环模式：大弥散弧长扩展）
     - `Dual-Tier Concentric Shells Hierarchical Burst Mode (Inner & Outer Rings)`（双层同心同构层级爆发模式：内外双同心圆星芒分级排布）
     - `Collapsed Single-Point Anchor Resting Baseline Mode (Resting Pre-Burst)`（折叠单点原位静止基准模式：爆发前聚集紧缩态）
   - 核心要素：
     - 构建聚集中心原点探针球（`Shape::Ellipsoid`）、径向扩展轨迹柱（`Shape::Cylinder`）、爆发展开卫星外包盒（`Shape::Box`）与设施园区包围矩形（`Shape::Rectangle`）。
     - 验证了 `ScreenSpaceEventHandler` 鼠标左键点击碰撞检测、动态星芒几何算法（从中心聚焦点插值平滑辐射至外围环形多边形节点）、文字标签同步显隐以及再次点击平滑回弹收拢。

---

## 2. 覆盖与测试指标

- **累计完整移植案例数**：287 / 309
- **本批新增测试案例**：4 个
- **Rust 单元测试**：145 个 passed (`entities::tests::batch85_shadows_sharedctx_splitdir_starburst`)
- **Native GPU 测试**：288 passed (`tests/full-recipes-gpu.rs`，包含 287 个完整案例 + 1 个宽折线材质测试)
- **WebGPU 浏览器测试**：287 passed (`docs/full-recipes-browser.json`)
- **Node.js 测试套件**：8 个测试套件全绿（`controls`、`sdk`、`ui`、`terrain`、`sandcastle`、`images`、`camera`、`recipes`）
