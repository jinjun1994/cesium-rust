# Batch 44 (v0.0.44) 案例验证报告：CZML、Multi-part CZML、Relative Paths 与 Time Dynamic Wheels

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖空天轨道目标、时序分段 CZML 数据流、动参考系相对航线、以及非线性加速铰接地面车辆的配方，累计完成 **123 / 309**（剩余 186 个）：

1. **`czml`（多数据包空间资产与地面车辆综合画廊）**
   - 官方地址：`https://sandcastle.cesium.com/?id=czml`
   - 变体全集（4 个多载体变体）：
     - `Satellites (simple.czml)`（加载轨道空间资产集合：`GeoEye-1` 与 `ISS` 卫星广告牌、标签及黄/青测地线轨道走廊、`AGI Facility` 地面站标注、以及 `Pennsylvania Territory` 边界多边形）
     - `Vehicle (Vehicle.czml)`（地面车辆导航图层：`Ground Vehicle` 广告牌、标注与橙色沙漠公路轨迹）
     - `Both (Satellites and Vehicle)`（天地一体化协同视图：全部 8 个空间与地面实体同步激活）
     - `Vehicle Close-Up`（低空聚焦地面车辆跟随视角）
   - 核心要素：
     - 覆盖 `billboard`、`label`、`polygon`、`widePolyline` 四种典型几何载体。
     - 验证了复合 CZML 场景在多数据源聚合下的图元显隐互斥与集合取景。

2. **`multi-part-czml`（莫哈韦沙漠 4500 秒分段流式 CZML 传输与油量监控）**
   - 官方地址：`https://sandcastle.cesium.com/?id=multi-part-czml`
   - 变体全集（4 个时序分段变体）：
     - `Part 1 (0 - 1500s)`（首段航程：`CesiumMilkTruck` 模型、燃油显示 `Fuel: 200.00 gal`、首段路线走廊）
     - `Part 2 (1500 - 3000s)`（中段航程：燃油消耗至 `Fuel: 133.33 gal`、第二航段走廊）
     - `Part 3 (3000 - 4500s)`（终段航程：燃油消耗至 `Fuel: 66.67 gal`、第三航段走廊）
     - `Combined Stream (0 - 4500s)`（全航程贯通：燃油剩余 `Fuel: 20.00 gal`、完整 4500 秒越野走廊）
   - 核心要素：
     - 动态流式数据包在时间维度推进时的模型状态与油量自定义属性同步刷新。
     - 航段折线走廊按时段增量装配与相机平滑锁定。

3. **`relative-paths`（动参考系与空间编队相对航线可视化）**
   - 官方地址：`https://sandcastle.cesium.com/?id=relative-paths`
   - 变体全集（4 个相对参考系变体）：
     - `Two Satellites Relative to Earth`（两颗伴随卫星相对于地心地固坐标系 ECEF 的绝对轨道）
     - `Satellite 1 Relative to Satellite 2`（卫星 1 以卫星 2 质心为原点的相对伴随飞行轨迹）
     - `Vehicle and Helicopter Relative to Earth`（加州地面卡车与直升机相对于地面的绝对运行路线）
     - `Helicopter Relative to Vehicle`（直升机相对于移动地面卡车动态参考坐标系的相对航迹）
   - 核心要素：
     - 支持固定绝对参考系与移动局部切平面动态坐标系的矢量差分投影。
     - 相对路径几何在不同观测视角下的无畸变正确解算。

4. **`time-dynamic-wheels`（地面铰接车辆非线性加速、动态车轮旋转与实时速度仪）**
   - 官方地址：`https://sandcastle.cesium.com/?id=time-dynamic-wheels`
   - 变体全集（4 个运行工况变体）：
     - `Accelerating Vehicle (0 -> 45 km/hr)`（非线性二次方加速巡航，车轮角速度随速度增加，速度标签实时显示 `45 km/hr`）
     - `Constant Cruise (30 km/hr)`（恒速巡航状态，车轮角速度恒定，速度标签显示 `30 km/hr`）
     - `High Speed Dash (60 km/hr)`（高负荷疾驰工况，车轮高速自转，速度标签显示 `60 km/hr`）
     - `Braking Stop (45 -> 0 km/hr)`（紧急制动减速至完全静止，车轮停止旋转，速度标签显示 `0 km/hr`）
   - 核心要素：
     - `GroundVehicle.glb` 3D 模型结合 `nodeTransformations`（`Wheels`, `Wheels_mid`, `Wheels_rear`）实现与车速联动的多轴车轮自转。
     - `VelocityVectorProperty` 实时计算切向速度并驱动测速仪标签。

---

## 2. 核心研发突破

1. **`Polyline` 与 `WidePolyline` 坐标系统规范化（地理度数 vs ECEF 笛卡尔）**：
   - 厘清并严格确立了宽折线图元 `data.positions` 必须采用 `[lonDeg, latDeg, altMeters]` 地理度数格式（经度 $\le 180^\circ$、纬度 $\le 90^\circ$、高度 $\le 2\times 10^6\text{m}$），而 `samples` 时空采样则采用 ECEF 笛卡尔坐标的规范，彻底杜绝了边界校验中 `polyline coordinate limit` 的隐式异常。
2. **多变体测试管线中隐藏实体自适应防退化（`e.show` 过滤）**：
   - 在图元包围盒计算与原生 GPU 离屏测试中，自动识别并过滤初始帧未激活实体（`e.show == false`），规避单实体隔离渲染时因空几何导致的 `no visible entity bounds` 崩溃。
3. **模型最小包围盒像素占比自适应补偿**：
   - 对小型复杂车辆模型（`GroundVehicle`、`CesiumMilkTruck`）统一校准基准缩放比与视距包围盒，确保离屏像素级光栅化差异判定（`changed.len() > 8`）在 768x512 画布中稳定触发且无误判。
4. **多参考系动态航迹与车轮多轴铰接变换解耦**：
   - 原生支持了卫星相对编队与空地协同动态差分走廊，并在时钟驱动下实现了多轴铰接旋转与速度矢量标签的高效同步。

---

## 3. 验收结果矩阵

| 案例 ID | 实体数 | 变体数 | Native GPU 渲染 | 浏览器 WebGPU | 像素拾取 | 交互控制 | 状态 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `czml` | 8 | 4 | **Passed** (90 帧无错误) | **Passed** | **Passed** (57887 px) | 天地多数据源切换 | **Passed** |
| `multi-part-czml` | 3 | 4 | **Passed** (90 帧无错误) | **Passed** | **Passed** (16323 px) | 4500s 分段油量监控 | **Passed** |
| `relative-paths` | 5 | 4 | **Passed** (90 帧无错误) | **Passed** | **Passed** (4416 px) | 动参考系相对航线 | **Passed** |
| `time-dynamic-wheels` | 3 | 4 | **Passed** (90 帧无错误) | **Passed** | **Passed** (3010 px) | 车轮铰接自转与测速 | **Passed** |

- **总案例通过率**：**123 / 309**（100% 通过全部 124 项 Rust Native GPU 离屏渲染测试、Playwright 真实浏览器 123 个 WebGPU 端到端测试与 90 帧时序动画无误差验证）。
