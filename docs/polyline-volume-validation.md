# 四个新增完整案例（0.0.29）

固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。本批完整实现并验收 **Polyline Volume** 系列四个完整源码案例：`polyline-volume`、`polyline-volume-dev`、`polyline-volume-outline-dev` 与 `czml-polyline-volume`，覆盖 10 个原始实例。累计完整功能案例通过数 **63 / 309**，剩余 246 个待完整验收。

| 案例 | 保留的完整案例行为 |
| --- | --- |
| Polyline Volume | 三个原始三维多段线管体截面扫掠实例：圆形管体（rounded 圆角拓扑）、矩形管体（beveled 斜切角拓扑）、星形管体（mitered 尖角拓扑）；支持填充色、透明度与外轮廓线（outline / outlineColor）；全集合 `viewer.zoomTo` 自动取景 |
| Polyline Volume - Dev | 三个底层 Primitive `PolylineVolumeGeometry` 几何实例：圆形管体、矩形管体、星形管体；支持 `POSITION_NORMAL` 顶点格式、`PerInstanceColorAppearance` 独立实例着色、法线计算与端面封盖；保留上游默认宽视角观察与手动取景 |
| Polyline Volume Outline - Dev | 两个底层 Primitive `PolylineVolumeOutlineGeometry` 几何实例：矩形盒体轮廓与星形管体轮廓；支持 `mitered` 与 `rounded` 拐角拓扑；包含起止截面封闭环与沿多段线方向的纵向棱线（longitudinal edges）；单像素物理轮廓线 |
| CZML Polyline Volume | 两个原始 CZML `polylineVolume` 数据包实例：绿色斜切角矩形（beveled box）与蓝色尖角星形（mitered star）；支持 `positions` WGS84 笛卡尔坐标、`shape` 二维多边形截面、`cornerType` 拐角拓扑定义、`solidColor` 颜色透明度与轮廓；`CzmlDataSource.load` Promise 异步解析与 `viewer.zoomTo` 自动取景 |

## 核心实现与架构设计

1. **三维沿轨截面扫掠几何核（Sweep Geometry Kernel）**：
   - 在 `crates/cesium-viewer/src/polyline_volume.rs` 中完整实现沿 ECEF 空间折线的 2D 截面扫掠算法。
   - 自动计算沿轨迹切线、局部法线与副法线的局部坐标标架（Frenet / Darboux 标架平滑过渡，防止非共面转角时的轴扭转奇异性）。
   - 实现三类拐角几何拓扑（Corner Types）：
     - `Mitered`：根据相邻线段角平分线计算切面截面，按 `1 / sin(half_angle)` 缩放顶点，形成尖角过渡。
     - `Beveled`：在转折点处根据转折角分别生成两道截面，通过斜切平面平滑过渡。
     - `Rounded`：在转折弧线上按微元角度细分插入过渡截面片，形成圆润弯头。
   - 端面封盖（End Caps）：起点与终点截面使用 `earcutr` 三角剖分填充，法线反向对齐首尾切向量。

2. **管体轮廓线提取（PolylineVolume Outline）**：
   - 实现 `polyline_volume::outline` 与 `PolylineVolumeOutlineGeometry`。
   - 生成起点截面封闭环、终点截面封闭环，以及连接相邻切片对应顶点的纵向棱线，保持内部空腔与清晰的线框结构。

3. **Entity、Primitive 与 CZML 体系全面打通**：
   - `entities.rs`：新增 `Shape::PolylineVolume`，支持 `shapePositions`（及别名 `shape`）序列化与反序列化，校验切片数量及截面多边形顶点数。
   - `geographic.rs`：为 `PolylineVolume` 计算几何中心锚点 `anchor`，支持大尺度地理坐标相对于相机的高精度浮点（RTC）渲染。
   - `extended_instances.rs`：支持 `PolylineVolumeGeometry` 与 `PolylineVolumeOutlineGeometry`，兼容 Cartesian2 对象数组 `{x, y}` 与标准元组数组 `[x, y]`。
   - `czml()`：支持 CZML `polylineVolume` 格式解析，提取 WGS84 ECEF 顶点数组与截面顶点。

4. **WebAssembly / 浏览器 WebGPU 平台支持**：
   - 更新 `bindings/web/sandcastle.html` 与 `bindings/web/sandcastle.js`，挂载 `fullNew36` ~ `fullNew39` 四个专用测试按钮。
   - 更新 `scripts/extended-recipes-browser.cjs` 与 `scripts/batch29-buttons.json`，支持浮点截面顶点的高精度近似比对与拾取。

## 验收证据

1. **源码契约与哈希校验**：
   - 提取自固定上游 Cesium Sandcastle 源码，SHA-256 签名严密记录。
2. **普通 Rust 单元测试**：
   - 全工作区单元测试通过（包含 `polyline_volume::tests::batch29_polyline_volume_all_four_cases`）。
3. **真实 GPU 离屏渲染测试**：
   - `crates/cesium-viewer/tests/full-recipes-gpu.rs` 全部 64 项测试通过（包含 `complete_polyline_volume`、`complete_polyline_volume_dev`、`complete_polyline_volume_outline_dev` 与 `complete_czml_polyline_volume`）。
4. **桌面 90 帧冒烟测试**：
   - 原生运行 90 帧，无任何 GPU 或渲染错误，日志记录于 `docs/full-polyline-volume-smoke.log`、`docs/full-polyline-volume-dev-smoke.log`、`docs/full-polyline-volume-outline-dev-smoke.log` 与 `docs/full-czml-polyline-volume-smoke.log`。
5. **浏览器 WebGPU 端到端测试**：
   - 在 Chromium 环境下运行 `scripts/full-recipes-browser.cjs`，所有 63 个完整案例全部通过画布颜色反读、实体 ID 拾取、显隐切换、滚轮缩放与窗口自适应检查，销毁状态正常（`destroy: 'passed'`）。
6. **Node 测试套件**：
   - 8 组 Node 测试套件全部通过（`npm test`）。
