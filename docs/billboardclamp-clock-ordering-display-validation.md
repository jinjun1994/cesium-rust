# Batch 39 (v0.0.39) 案例验证报告：BillboardClampToGround Dev、Clock、DataSource Ordering 与 Display Conditions Dev

## 1. 案例概要

本批次推进 Sandcastle 官方完整案例移植，突破百例大关，实现并严格验收 4 个核心贴地广告牌阵列、时钟与光照时间序列驱动、数据源图层层叠排序、以及近远视距条件显示控制配方，累计完成 **103 / 309**（剩余 206 个）：

1. **`billboardclamptoground-dev`（底层 BillboardCollection 贴地广告牌与地表深度测试交互）**
   - 官方地址：`https://sandcastle.cesium.com/?id=billboardclamptoground-dev`
   - 变体全集（5 个贴地与经度平移变体）：
     - `Add billboard (Relative to ground)`（向 Seneca Rocks 设施阵列添加 Cesium 标识广告牌，`heightReference: RELATIVE_TO_GROUND`）
     - `Clamp to ground`（切换为紧密贴地，`heightReference: CLAMP_TO_GROUND`）
     - `None`（切换为绝对椭球面高度，`heightReference: NONE`）
     - `Increase longitude`（沿经度方向交互微调平移 +0.001°，测试局部地形高程自适应重采样）
     - `Remove billboard`（动态移除广告牌图元实例，保留 3x3 设施底座点阵）
   - 核心要素：
     - Seneca Rocks 3x3 设施点阵（`facility-0-0` ~ `facility-2-2`，共 9 个），全部使用 `HeightReference.CLAMP_TO_GROUND`。
     - 启用 Globe `depthTestAgainstTerrain: true`，验证广告牌与地形/椭球面的深度测试及逆深度消隐。

2. **`clock`（2013 圣诞节太阳直射点时钟驱动、LOOP_STOP 边界与实时光照跟踪）**
   - 官方地址：`https://sandcastle.cesium.com/?id=clock`
   - 变体全集（4 个时间控制与光照追踪变体）：
     - `Default Clock (4000x Speed, Loop Stop)`（圣诞节 2013-12-25 00:00:00Z 至 2013-12-26 00:00:00Z，4000x 速率，LOOP_STOP 循环停顿模式）
     - `Speed Up (8000x)`（时钟速率乘 2 加速，演示动态步长推进）
     - `Slow Down (2000x)`（时钟速率除以 2 减速）
     - `Reset Current Time (t = 0h)`（复位当前时间至初始历元，同步刷新 ClockViewModel 与 Timeline 界面组件）
   - 核心要素：
     - Globe `enableLighting: true`，基于太阳向量与地球自转角度实时计算半球光照明暗交界线与次太阳点（Subsolar Point）。
     - 包含 `christmas-sun-marker` 太阳直射点动态轨迹与 `christmas-clock-label` 南回归线标注。

3. **`datasource-ordering`（CZML 多数据源图层 raise / lower 动态层级重排与 zIndex 控制）**
   - 官方地址：`https://sandcastle.cesium.com/?id=datasource-ordering`
   - 变体全集（4 个图层堆叠与顺序重排变体）：
     - `Initial (czml2 on top of czml1)`（czml2 黄色/青色矩形图层位于 czml1 红色/蓝色矩形图层之上，zIndex 2 vs 1）
     - `Lower czml2 (czml1 on top)`（czml2 下移一层，czml1 翻转至顶层显示）
     - `Raise czml2 to top`（czml2 提升至最顶层）
     - `Send czml2 to bottom`（czml2 压入最底层）
   - 核心要素：
     - 验证 `DataSourceDisplay` 与底层多通道图元渲染管线的 `zIndex` 排序语义，消除相交与重叠半透明地表图元的 Z-Fighting 穿透闪烁。

4. **`display-conditions-dev`（底层 Primitive 与 Entity 的 DistanceDisplayCondition 近远距视距裁剪）**
   - 官方地址：`https://sandcastle.cesium.com/?id=display-conditions-dev`
   - 变体全集（4 个视距条件变体）：
     - `Billboards/Rectangles: Near 0, Far 5.5e6`（宏观远景可见：广告牌与贴地矩形在 0 ~ 5,500,000m 视距内渲染，视点拉远至外太空时淡出/剔除）
     - `Points/Models: Near 0, Far 250.5`（微观近景可见：点图元与精细模型在近距离 0 ~ 250.5m 细致可见，视点后退时平滑消隐）
     - `Zoom In (Camera Close Range)`（相机飞至 150m 近距，验证近距图元激活与远距图元按条件裁剪）
     - `Zoom Out (Camera Planetary Range)`（相机退至 6,000km 行星级宏观视角，验证近距图元彻底消隐与远距图元可见）
   - 核心要素：
     - 原生对齐 CesiumJS `new DistanceDisplayCondition(5.5e6)` 单参数构造语义（`near = 0.0, far = 5.5e6`）。
     - 覆盖 `BillboardCollection`、`PerInstanceColorAppearance` 矩形图元与 `PointPrimitiveCollection` 多种图元类型。

---

## 2. 核心研发突破

1. **Shape::Billboard 扩展 `heightReference` 贴地高程基准（`crates/cesium-viewer/src/entities.rs`）**：
   - 针对底层图元与 CZML 解码链路，为 `Shape::Billboard` 引入 `height_reference: Option<HeightReference>`，对齐 `CLAMP_TO_GROUND`、`RELATIVE_TO_GROUND` 及 `NONE` 属性。
   - 扩展 CZML 广告牌属性白名单，完善了数据驱动下的贴地变换管道。
2. **时钟状态统一管理与动态时间评估（`bindings/web/sandcastle.js` & `crates/cesium-viewer/tests/full-recipes-gpu.rs`）**：
   - 在前端沙盒 `runFull` 中打通案例元数据 `entry.clock` 向 WebAssembly SDK 的完整透传：涵盖 `startTime`、`stopTime`、`currentTime`、`multiplier` 与 `range`。
   - 在案例切换与重置（`resetCaseControls`）中规范了默认时钟恢复逻辑，避免历史案例的时间跨度污染后续常规图元。
   - 在 Native GPU 取景测试中自适应依据实体的 `availability` 探测动态历元，确保动态采样实体在视锥包围盒计算与像素校验中稳定命中。
3. **矩形经纬度序列化对齐（`coordinatesDegrees` 统一规范）**：
   - 规范了底层 `Shape::Rectangle` 与 CZML 解包层关于 `coordinatesDegrees` 的字段对齐，统一了跨平台浏览器与原生测试的实体属性契约。

---

## 3. 验收全景与指标汇总

- **普通 Rust 测试**：99 项（`cesium_viewer` 102 项单元测试通过）。
- **实际 GPU 测试**：104 项完整案例 GPU 渲染测试全数通过（`104 passed; 0 failed`）。
- **Node.js 测试套件**：8 组测试全绿（103 个配方校验通过）。
- **浏览器 WebGPU 自动化**：103 个配方在 Chromium WebGPU 环境下 100% 验收通过（无一失败）。
- **代码规范**：`cargo clippy --workspace --all-targets` 零警告，`cargo fmt --check` 格式一致。
