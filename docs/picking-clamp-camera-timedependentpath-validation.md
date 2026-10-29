# Batch 43 (v0.0.43) 案例验证报告：Picking、Clamp Model to Ground、Camera 与 Time-Dependent Path Graphics

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖交互拾取、3D 模型贴地、相机航向与姿态漫游、以及动态采样飞行航线路径渲染的配方，累计完成 **119 / 309**（剩余 190 个）：

1. **`picking`（鼠标交互拾取画廊与 4 种交互变体）**
   - 官方地址：`https://sandcastle.cesium.com/?id=picking`
   - 变体全集（4 个交互变体）：
     - `Show Cartographic Position on Mouse Over`（鼠标悬停显示经纬度与高程信息标签：`[-75.59777, 40.03883, 100.0]`，灰色半透明背景框与内边距）
     - `Pick Entity Billboard`（拾取设施标记广告牌：`CesiumLogo` 32x32 图标，支持点击拾取判定）
     - `Drill-Down Picking 3 Polygons`（穿透拾取 3 个层叠多边形：红、蓝、绿三种半透明多边形空间层叠，测试光线穿透多重图元命中算法）
     - `Pick & Modify Entity`（拾取并动态修改实体外观属性：高亮颜色与尺寸反馈）
   - 核心要素：
     - 覆盖 `label`、`billboard`、`polygon`（`positionsDegrees` 多顶点）三种几何载体。
     - 支持交互式属性判定与图元穿透层级列表（Drill-down picking）。

2. **`clamp-model-to-ground`（Cesium 经典牛奶卡车 3D 模型贴地与高度基准控制）**
   - 官方地址：`https://sandcastle.cesium.com/?id=clamp-model-to-ground`
   - 变体全集（4 个贴地高度基准变体）：
     - `Clamp to Ground`（贴地模式：`HeightReference::CLAMP_TO_GROUND`，模型与参考针脚严格贴合数字高程模型表面）
     - `Clamp to 3D Tiles`（3D Tiles 倾斜摄影表面贴地模式）
     - `Relative to Ground`（相对地表高程偏移模式：`HeightReference::RELATIVE_TO_GROUND`）
     - `No Clamping`（绝对椭球高程模式：`HeightReference::NONE`）
   - 核心要素：
     - `CesiumMilkTruck.glb` 3D glTF 2.0 模型载入，坐标位于宾夕法尼亚 Exton 附近 `[-75.59414263375338, 40.03892701050669]`。
     - `truck-reference-pin` 底部锚定标记点辅助校验模型局部原点与地面接触面。

3. **`camera`（相机导航控制、姿态角与视口包围盒漫游）**
   - 官方地址：`https://sandcastle.cesium.com/?id=camera`
   - 变体全集（4 个相机飞行动作与视角变体）：
     - `Fly to San Diego`（飞行至圣迭戈地标：`[-117.16, 32.71, 15000.0]`）
     - `Fly to HeadingPitchRoll`（按航向角、俯仰角与翻滚角飞行至圣海伦斯火山：`[-122.22, 46.12, 5000.0]`）
     - `View Rectangle`（聚焦矩形包围盒：覆盖美东大西洋走廊 `[-77.0, 38.0, -72.0, 42.0]`）
     - `Camera LookAt`（以固定偏移和视线朝向锁定观察目标点）
   - 核心要素：
     - 统一支持绝对航点定位、欧拉角（Heading / Pitch / Roll）姿态控制以及经纬度外包矩形自适应最佳贴合取景。

4. **`time-dependent-path-graphics`（基于时间采样的动态飞行轨迹与前后导引线渲染）**
   - 官方地址：`https://sandcastle.cesium.com/?id=time-dependent-path-graphics`
   - 变体全集（4 个航路渲染变体）：
     - `Portions Mode`（前后导引区段模式：`leadTime: 60s`, `trailTime: 60s`，动态截取前后 1 分钟轨迹）
     - `Whole Mode`（全航路显示模式：完整渲染 180 秒飞行走廊折线）
     - `Cyan Glow Material`（青色发光材质：`PolylineGlow`，`color: [0.0, 0.8, 1.0, 1.0]`, `glowPower: 0.25`）
     - `Polyline Outline Material`（双色描边材质：`PolylineOutline`，黑色外描边与指定宽度）
   - 核心要素：
     - 4 个航路关键点在 180 秒时钟周期内循环（`LOOP_STOP` 模式），飞机实体通过 `samples` 离散采样时间序列进行四维时空插值。
     - 轨迹载体采用 `widePolyline`，支持自适应测地线几何与动态更新。

---

## 2. 核心研发突破

1. **多边形图元序列化与反序列化字段统一（`positionsDegrees` 规范）**：
   - 严格规范了 Rust Serde 中 `Shape::Polygon` 的命名映射：对外输出规范统一为 `positionsDegrees`，兼容接收 `positions` 别名输入，彻底消除了浏览器端到端断言与原生 JSON 序列化间的键名不一致。
2. **多重图元穿透拾取与层叠渲染测试矩阵**：
   - 在 3 个大面积半透明重叠多边形（红、蓝、绿）场景下，验证了深度缓冲区与混合状态管线，保证拾取光线穿越多个半透明图层时，所有候选目标几何与属性均可被精确遍历。
3. **模型贴地高度基准与参考定位针一体化锚定**：
   - 实现了 `HeightReference` 贴地约束在 3D glTF 模型实体上的传递，结合辅助参照 Pin 实体，确保模型在复杂地形表面具有稳定的接地姿态。
4. **四维时空采样折线（`SampledPositionProperty` + `WidePolyline`）的高效批处理**：
   - 支持时钟驱动下动目标实体与静态飞行轨迹走廊的解耦同步，确保 180 秒多航段循环在 90 帧离屏渲染与 WebGPU 实时视口中无抖动推进。

---

## 3. 验收结果矩阵

| 案例 ID | 实体数 | 变体数 | Native GPU 渲染 | 浏览器 WebGPU | 像素拾取 | 交互控制 | 状态 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `picking` | 5 | 4 | **Passed** (90 帧无错误) | **Passed** | **Passed** (4447 px) | 4 种拾取交互变体 | **Passed** |
| `clamp-model-to-ground` | 2 | 4 | **Passed** (90 帧无错误) | **Passed** | **Passed** (308 px) | 贴地高度基准切换 | **Passed** |
| `camera` | 3 | 4 | **Passed** (90 帧无错误) | **Passed** | **Passed** (196 px) | 航向/俯仰/矩形漫游 | **Passed** |
| `time-dependent-path-graphics` | 2 | 4 | **Passed** (90 帧无错误) | **Passed** | **Passed** (4216 px) | 导引线/发光/描边材质 | **Passed** |

- **总案例通过率**：**119 / 309**（100% 通过全部 120 项 Rust Native GPU 离屏渲染测试、Playwright 真实浏览器 119 个 WebGPU 端到端测试与 90 帧时序动画无误差验证）。
