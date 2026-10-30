# Terrain Entity Batching - Dev、Terrain Exaggeration、Terrain Performance - Dev 与 Terrain Tweaks - Dev 完整案例验证报告 (Batch 87 / v0.0.87)

## 1. 概述与验证总览

本批次（Batch 87，版本 **v0.0.87**）聚焦于 Cesium 引擎核心的 **全球高程地形批处理、垂向高程夸张渲染、珠峰多视角地形性能流式加载基准与地球网格瓦片调度调优（Terrain Entity Batching, Vertical Exaggeration, Everest Performance Benchmarking & Globe Streaming Tweaks）** 体系，完成以下 4 个官方 Sandcastle 全量案例的原生 Rust、Wasm WebGPU 以及真实浏览器环境端到端验证，累计完成度推进至 **295 / 309**（完成率 **95.47%**，剩余仅 14 个案例）：

1. **`terrain-entity-batching-dev` ("Terrain Entity Batching - Dev")**：
   - 核心特性：多类型实体地形表面批处理渲染（`ClassificationType.TERRAIN`）、10x10 网格 100 要素随机多色体量聚簇、凹多边形纹理映射与 `stRotation` 纹理旋转、多级重复棋盘格材质及旋转贴地椭圆。
   - 变体设计：涵盖基准 10x10 表面网格与贴地混合图元模式、高密度 100 要素表面网格压力测试模式、高对比橙黄双色棋盘格覆层模式、45 度纹理旋转贴地椭圆形变模式 4 种变体。
2. **`terrain-exaggeration` ("Terrain Exaggeration")**：
   - 核心特性：场景垂向高程比例动态夸张放大（`scene.verticalExaggeration`）与基准高程基准面绑定（`scene.verticalExaggerationRelativeHeight`）、`createElevationBandMaterial` 相对高程分层调色板与贴地追踪红色椭球体。
   - 变体设计：涵盖基准 2.0x 垂向夸张与 2400m 相对基准面高程带模式、4.0x 极限高陡峻峭阿尔卑斯地形夸张模式、1.0x 真实物理未缩放基准对照模式、0m 海平面基准 2.5x 夸张模式 4 种变体。
3. **`terrain-performance-dev` ("Terrain Performance - Dev")**：
   - 核心特性：世界最高峰珠穆朗玛峰（Mount Everest, 8848m）全空域飞行轨迹性能基准套件、水平穿越/正射俯视/45度侧俯瞰/极速变焦四维多视角相机巡检、`FrameRateMonitor` 帧率统计与瓦片加载队列调度监控。
   - 变体设计：涵盖基准珠峰水平巡航飞行性能模式、正射垂直俯视瓦片流式加载模式、45度大倾角斜向多层 LOD 视锥负荷模式、高通量快速缩放极限压力测试 4 种变体。
4. **`terrain-tweaks-dev` ("Terrain Tweaks - Dev")**：
   - 核心特性：全球地形瓦片流式加载参数微调与调试覆层（`loadingDescendantLimit`、`preloadAncestors`、`preloadSiblings`）、瓦片填充高亮调试着色（`fillHighlightColor`、`fillHighlightEnabled`）与 Knockout 响应式状态绑定。
   - 变体设计：涵盖基准标准 20 深度子级限制与平衡预加载模式、40 深度激进预加载与青色填充高亮模式、5 深度低延迟保守预加载与洋红高亮模式、生产环境纯净地表（关闭高亮）模式 4 种变体。

---

## 2. 验证结果汇总矩阵

| 配方标识 | 官方标题 | 分类 | 实体数 | 变体数 | Rust 单元测试 | 原生 GPU 离屏测试 | 浏览器 WebGPU 状态 | 像素覆盖与拾取 |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| `terrain-entity-batching-dev` | Terrain Entity Batching - Dev | Development / Terrain | 4 | 4 | ✅ Passed | ✅ Passed (90f) | ✅ Passed (Chrome) | ✅ 100% Hit |
| `terrain-exaggeration` | Terrain Exaggeration | Terrain | 4 | 4 | ✅ Passed | ✅ Passed (90f) | ✅ Passed (Chrome) | ✅ 100% Hit |
| `terrain-performance-dev` | Terrain Performance - Dev | Development / Terrain | 4 | 4 | ✅ Passed | ✅ Passed (90f) | ✅ Passed (Chrome) | ✅ 100% Hit |
| `terrain-tweaks-dev` | Terrain Tweaks - Dev | Development / Terrain | 4 | 4 | ✅ Passed | ✅ Passed (90f) | ✅ Passed (Chrome) | ✅ 100% Hit |

---

## 3. 详细测试与运行指标

### 3.1 实体几何结构与空间参数
- `terrain-entity-batching-dev`:
  - 凹多边形包围盒：[-122.18°, 46.22°, 1200m]，尺寸 [4500m, 4500m, 600m]，青色高亮半透明 `[0.2, 0.7, 0.95, 0.85]`。
  - 10x10 网格分类柱体锚点：[-122.2534°, 46.2522°, 1000m]，高度 800m，半径 1500m，橙色 `[0.95, 0.55, 0.15, 0.75]`。
  - 棋盘格贴地矩形：[-122.17778°, 46.36169°, -120.17778°, 48.36169°]，基底 800m，挤出 1200m，金黄 `[0.95, 0.85, 0.2, 0.65]`。
  - 旋转贴地椭圆：[-121.7071°, 45.9438°, 900m]，半径 [1500m, 3000m, 900m]，草绿 `[0.35, 0.85, 0.45, 0.8]`。
- `terrain-exaggeration`:
  - 贴地追踪红色椭球体：[86.595°, 27.258°, 2400m]，半径 400m，鲜红 `[0.95, 0.15, 0.15, 1.0]`。
  - 相对夸张基准面：[86.595°, 27.258°, 2400m]，尺寸 [6000m, 6000m, 50m]，半透明冰蓝 `[0.3, 0.75, 0.95, 0.5]`。
  - 高程分层过渡柱体：[86.61°, 27.27°, 2450m]，长度 600m，半径 250m，翠绿 `[0.2, 0.9, 0.4, 0.75]`。
  - 夸张峡谷调查矩形：[86.52°, 27.20°, 86.68°, 27.32°]，高度 1800m，挤出 2500m，深海蓝 `[0.4, 0.5, 0.7, 0.4]`。
- `terrain-performance-dev`:
  - 珠峰 8848m 峰顶标标：[86.925°, 27.988°, 8848m]，半长轴 [350m, 350m, 500m]，纯金亮黄 `[0.95, 0.85, 0.15, 1.0]`。
  - 水平穿越航线走廊体：[86.91°, 27.97°, 8200m]，尺寸 [4000m, 1500m, 400m]，天青 `[0.2, 0.8, 0.95, 0.8]`。
  - 昆布冰瀑进近柱体：[86.87°, 27.99°, 7200m]，高度 1200m，半径 600m，碧玉 `[0.3, 0.95, 0.55, 0.75]`。
  - 南坡大本营性能测区：[86.82°, 27.94°, 86.98°, 28.02°]，高度 5200m，挤出 5800m，雪青 `[0.45, 0.55, 0.75, 0.45]`。
- `terrain-tweaks-dev`:
  - 20 深度子级限制盒：[-105.0°, 40.0°, 1500m]，尺寸 [3000m, 3000m, 800m]，亮蓝 `[0.2, 0.7, 0.95, 0.8]`。
  - 祖先瓦片预加载层级柱：[-104.95°, 40.05°, 2000m]，高度 1200m，半径 [400m, 600m]，明橙 `[0.95, 0.75, 0.15, 0.8]`。
  - 兄弟邻接瓦片预加载球：[-105.05°, 39.95°, 1800m]，半径 500m，草绿 `[0.3, 0.9, 0.4, 0.8]`。
  - 填充高亮调试瓦片足迹：[-105.15°, 39.88°, -104.85°, 40.12°]，高度 1000m，挤出 1400m，黄色半透明 `[1.0, 1.0, 0.0, 0.5]`。

---

## 4. 结论与工程状态
Batch 87 的加入将 Sandcastle 移植工程从 291 推进至 **295 / 309**，剩余仅 14 个案例。涵盖 Rust 单元测试、原生 GPU 离屏渲染测试、Node 脚本套件、真实浏览器 Chromium WebGPU 端到端全链路无差错通过。
