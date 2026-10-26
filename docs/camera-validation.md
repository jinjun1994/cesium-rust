# 五项相机控制验证（0.0.16）

本批完成五项有明确边界的相机功能，原生与 WebAssembly 共用 Rust Scene：

| 功能 | 已实现行为 | 验证 |
| --- | --- | --- |
| setView | WGS84 相机眼点与 heading/pitch/roll；ENU 姿态转换 | 极点正交矩阵、HPR 往返、画面与中心拾取 |
| flyTo | 定时大圆路径、平滑高度抬升、四元数姿态插值、精确终点 | 跨日期变更线短弧、对跖点、时钟暂停、零时长与终点拾取 |
| cancelFlight | 冻结当前姿态、可重复取消、替换飞行、手动接管 | 取消后逐像素相同；错误请求保留旧飞行；拖拽/缩放终止飞行 |
| trackEntity | 按实体时钟跟随当前 ECEF 位置，保持当地 HeadingPitchRange | 移动 30 秒后眼点移动超过 30 km，实体仍居中可拾取；隐藏/删除/不可用/清空自动停止 |
| lookAt | 固定地理目标及 HeadingPitchRange，后续可手动绕目标旋转 | 眼点到目标距离、中心拾取、时间与实体移动不改变固定目标 |

## 启动与操作

```bash
python3 scripts/start-test-ui.py
# http://127.0.0.1:8000/bindings/web/sandcastle.html
```

新增“setView 姿态定位”“flyTo 五秒飞行”“取消飞行”“跟随移动实体”“停止跟随”“lookAt 固定目标”按钮。飞行演示从 109.3°E 飞至 109.7°E，暂停实体时钟仍会继续；跟随演示加载已有动态 CZML，支持暂停、倒放和时间轴。状态区显示模式、飞行进度及跟随 ID。

拖拽、缩放、定位实体/地球/GeoJSON 或设置新的相机目标会接管相机。跟随时拖拽停止跟随并保留当前目标的轨道。显式 setView/飞行姿态下拖拽调整眼点处的 heading/pitch，缩放沿视线移动。实体全量替换停止跟随；upsert 同一 ID 在下一次 tick 使用新位置；隐藏、删除、清空立即停止，时间不可用在下一次 tick 停止，不自动恢复。

```js
const view = {
  destinationDegrees: [109.7, 25.1, 96000], // 相机眼点：经度、纬度、椭球高（米）
  heading: 0, pitch: -Math.PI / 2, roll: 0 // 弧度
};
viewer.setView(view);
const flightId = viewer.flyTo(view, {duration: 3});
viewer.cancelFlight(); // 返回是否有正在执行的飞行
viewer.lookAt([109.7, 25.1, 16000], {heading: 0.6, pitch: -0.8, range: 80000});
viewer.trackEntity('point', {heading: 0, pitch: -1.2, range: 40000});
viewer.stopTracking(); // 返回是否正在跟随
console.log(viewer.cameraState); // mode/view/flightId/flightStatus/flightProgress/trackedEntity
```

setView/flyTo 的默认 heading/roll 为 0，pitch 为 -π/2；destinationDegrees 必填。lookAt/trackEntity 的默认 offset 为 heading=0、pitch=-π/4、range=100000。flyTo 默认 duration=3。相机使用 ECEF f64，投影仍为透视 60°。heading 从北向东为正，pitch 向下为负。垂直俯视的 heading/roll 分解不唯一，应比较实际姿态矩阵。

## 原生配置

```bash
CESIUM_OFFLINE=1 CESIUM_ENTITIES=tests/entities/screen-external.json \
  CESIUM_CAMERA=tests/camera/fly-to.json cargo run --locked -p cesium-viewer --bin desktop-viewer
CESIUM_OFFLINE=1 CESIUM_CZML=tests/entities/screen-motion.czml \
  CESIUM_CAMERA=tests/camera/track-entity.json cargo run --locked -p cesium-viewer --bin desktop-viewer
```

`CESIUM_CAMERA` 仅读取本地 JSON 数组，最多 16 条命令 / 16 KiB；五组配置见 tests/camera。命令在实体加载后按顺序执行，不含等待指令；取消配置验证立即取消，飞行中途取消由 GPU 测试验证。C 取消飞行，U 停止跟随。后设置的 CESIUM_VIEW_LOCATION 会覆盖相机配置。

## 边界

- 自定义 SDK 接口与 JSON 命令，不是 CesiumJS API 的直接替换；没有完整 flyTo 选项、Promise/回调事件、boundingSphere、矩形目标、flyHome、lookAtTransform、2D/Columbus、正交相机或惯性控制。
- destination 经度 -180..180°、纬度 -90..90°、眼点高度 10..30,000,000 m；姿态有限且绝对值 <=10000 rad，pitch ±π/2；duration 0..120 s。非法参数先验证，不打断现有控制。
- lookAt/跟随 pitch 为 -π/2..0，range 10..30,000,000 m；目标高度 -10000..100,000,000 m（跟随沿用实体位置校验），创建的眼点须高于椭球 10 m。近水平/垂直轨道用 1e-6 rad 避免退化。手动轨道继承既有 pitch 限制。
- 飞行沿经纬度单位法向量的大圆插值；高度线性加正弦平方抬升，姿态在世界 ECEF 中 slerp。这是有界基线，不复刻 Cesium 的椭球最短测地线、飞行自动俯仰或地形路径规划。地形仅基于已加载缓存避让，未加载高地不作保证。
- 飞行用每帧真实秒数，独立于实体 clock.multiplier/shouldAnimate；Web 对后台帧间隔截断为 1 秒。Scene tick 单次有效间隔 0..60 秒；无有效 tick 不推进。跟随依赖实体时钟，不采用实体姿态/速度朝向。
- Web/原生正常输入通过 Scene.camera_drag/camera_zoom 接管生命周期。直接写公开 Orbit 参数属于低层接口，不触发取消状态。
- 移动相机会使旧异步拾取失效；移动期间的拾取可能被下一帧丢弃，停止后可正常重新拾取。尚未实现带相机快照的动态拾取。

## 已执行验证

全工作区 **71 个普通 Rust 测试、27 个实际 GPU 测试、七组 Node 测试通过**；fmt/clippy、原生编译、WASM 发布构建和绑定生成通过。五个桌面相机配置分别运行 90 帧，错误均为 0，日志为 `docs/camera-*-smoke.log`。浏览器脚本已加入相机按钮断言，但本轮因 Chromium 缺失未能启动；历史 DeviceLost(Destroyed) 仍未解决。

测试源码在 `src/camera_control.rs`、`tests/camera-gpu.rs`、`scripts/camera-sdk-test.mjs` 与扩展的 `scripts/sandcastle-test.mjs`。GPU 导出图为 `docs/camera-preview.png`。Node 测试替换 GPU 边界，GPU 测试使用真实 wgpu/Mesa，均不代替浏览器 WebGPU 验收。

Sandcastle 目录仍为固定 Cesium 1.146 的 309 项；保持 19 项已有关联子集，完整上游配方通过数仍为 0。本批相机演示独立提供，不把 API 同名计为完整上游配方兼容。

官方接口参考：[Camera](https://cesium.com/learn/cesiumjs/ref-doc/Camera.html)、[Viewer.trackedEntity](https://cesium.com/learn/cesiumjs/ref-doc/Viewer.html#trackedEntity)。
