# Batch 42 (v0.0.42) 案例验证报告：Materials、Bloom、Cesium Widget 与 Cardboard

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个核心程序化材质库、后期泛光特效、轻量微件视口与虚拟现实纸盒漫游配方，累计完成 **115 / 309**（剩余 194 个）：

1. **`materials`（常用程序化外观材质库与 8 种材质切换变体）**
   - 官方地址：`https://sandcastle.cesium.com/?id=materials`
   - 变体全集（8 个交互式材质变体）：
     - `Image Material (Default)`（贴图材质：加载官方 Cesium 标志 JPEG 图像纹理）
     - `Grid Material`（网格材质：绿色网格线、cellAlpha: 0.1、lineThickness: 2.0）
     - `Stripe Material`（条纹材质：水平红白相间条纹、repeat: 16）
     - `Checkerboard Material`（棋盘格材质：红绿相间国际象棋盘方格、repeat: 4x4）
     - `Dot Material`（圆点材质：黄绿深浅圆点阵列、repeat: 10x10）
     - `RimLighting Material (World)`（边缘光照/轮廓发光材质：深蓝基础色与高光边缘）
     - `Polyline Glow Material`（折线发光材质：深天蓝发光管道、glowPower: 0.25）
     - `Polyline Outline Material`（折线双色描边材质：黑色外轮廓与指定线宽）
   - 核心要素：
     - 覆盖 `mat-rectangle`、`mat-world-rectangle` 和 `mat-polyline` 三种主要几何载体。
     - 控制材质切换与实体 `show` 互斥隐藏显示，保证多材质切换时图元状态的一致性与独立渲染。

2. **`bloom`（13 具热气球阵列排布与后期泛光光效调节）**
   - 官方地址：`https://sandcastle.cesium.com/?id=bloom`
   - 变体全集（4 个泛光调节变体）：
     - `Bloom Default (Enabled)`（默认泛光：contrast: 128, brightness: -0.3, delta: 1.0, sigma: 3.78, stepSize: 5.0）
     - `Bloom Subtle`（柔和微弱泛光：contrast: 110, brightness: -0.2, sigma: 2.0）
     - `Bloom Intense`（强光耀斑模式：contrast: 150, brightness: -0.4, delta: 1.5, sigma: 5.0）
     - `Bloom Disabled`（关闭泛光对比视图）
   - 核心要素：
     - 13 具 `CesiumBalloon.glb` 模型在纬度 44.0503706° 沿经度每隔 0.00025° 排成一字队形。
     - 相机以固定目标点偏置视角（`lookAt`）俯瞰气球队列与地平线光效。

3. **`cesium-widget`（独立极简微件视图、无人机模型与目标追踪）**
   - 官方地址：`https://sandcastle.cesium.com/?id=cesium-widget`
   - 变体全集（4 个微件模式变体）：
     - `Default (Widget Mode, Tracking Drone)`（轻量微件模式并锁定追踪飞行器）
     - `Free Camera View`（解除追踪，自由环绕观察视角）
     - `High Altitude Overview`（高空俯瞰视角）
     - `Chase View (Behind Drone)`（机尾跟随追踪视角）
   - 核心要素：
     - 轻量级无额外工具栏的纯净视口。
     - `Cesium_Air.glb` 飞行器模型，设置 `HeadingPitchRoll` 姿态角（heading 1.5707963267948966）、`minimumPixelSize: 128`、`maximumScale: 20000`。
     - 开启 `shouldAnimate: true`，相机设置 `widget.trackedEntity = entity` 持续聚焦移动飞行器。

4. **`cardboard`（VR 纸盒双目模式与大峡谷环形飞行编队）**
   - 官方地址：`https://sandcastle.cesium.com/?id=cardboard`
   - 变体全集（4 个 VR 飞行变体）：
     - `Default (Grand Canyon Formation Flight)`（大峡谷上空 13 具热气球编队环飞）
     - `Single Balloon Lead View`（头机单一视点跟随）
     - `5 Balloons Wedge Formation`（5 具热气球楔形编队飞行）
     - `Overhead Canyon Panorama`（峡谷鸟瞰全景飞行模式）
   - 核心要素：
     - 13 具 `CesiumBalloon.glb` 模型在科罗拉多大峡谷（中心经纬度 `[-112.110693, 36.0994841]`）上空编队飞行。
     - 采用 Hermite 三次多项式采样插值生成 360 秒封闭环形循环飞行航线（`LOOP_STOP` 时钟模式）。
     - 支持模型局部坐标系锁定（`modelMatrix` 变换）与 VR Cardboard 双目分离投影。

---

## 2. 核心研发突破

1. **大跨度程序化矩形自适应三角剖分预算（`GeoJSON tessellation budget`）**：
   - 针对 `[-100, 20, -70, 40]` 与 `[-20, -10, 20, 10]` 大面积矩形，细分算法在保持测地线弯曲精度的同时，将三角面片和顶点数量严格控制在 300,000 顶点预算内，兼顾椭球贴合度与渲染帧率。
2. **多材质架构模式严格解析（`SurfaceMaterial` & `LineMaterial`）**：
   - 统一梳理了表面程序化材质（`Image`, `RimLighting`, `Stripe`, `Checkerboard`, `Dot`, `Grid`）与宽线材质（`PolylineGlow`, `PolylineOutline`）在 Serde 反序列化与 WebGPU 着色器 Uniform 缓冲中的字段映射，确保轮廓描边与填充材质互斥性完全符合引擎契约。
3. **变体数据中图像材质直接解析（`Base64 Data URI`）**：
   - 在变体集合反序列化路径中，支持直接解析内嵌 Base64 编码的 JPEG/PNG 纹理，使变体切换在 Rust 原生与浏览器 WASM 环境中均能无延迟原子生效。
4. **大峡谷 13 具热气球动态时钟采样与姿态同步**：
   - 支持多实体 360 秒连续时钟循环与离散轨迹插值，各热气球沿相位偏置轨道平滑巡航，完成 90 帧时间序列原生渲染与无误差验收。

---

## 3. 验收结果矩阵

| 案例 ID | 实例数 | 变体数 | Native GPU 渲染 | 浏览器 WebGPU | 像素拾取 | 取景与交互 | 状态 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `materials` | 3 | 8 | **Passed** (90 帧无错误) | **Passed** | **Passed** | 8 种材质切换 | **Passed** |
| `bloom` | 13 | 4 | **Passed** (90 帧无错误) | **Passed** | **Passed** | 泛光预设与队列观察 | **Passed** |
| `cesium-widget` | 1 | 4 | **Passed** (90 帧无错误) | **Passed** | **Passed** | 飞行器追踪微件 | **Passed** |
| `cardboard` | 13 | 4 | **Passed** (90 帧无错误) | **Passed** | **Passed** | 大峡谷编队飞行 | **Passed** |

- **总案例通过率**：**115 / 309**（100% 通过全部 Rust Native GPU 离屏渲染测试、Playwright 真实浏览器 WebGPU 测试与 90 帧时序动画无误差验证）。
