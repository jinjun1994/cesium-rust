# 六个 Primitive 完整案例（0.0.20）

本批完整移植固定 Cesium 1.146 的六个 Sandcastle 开发案例：**Box Outline - Dev、Cylinder Outline - Dev、Sphere Outline - Dev、Ellipsoid Outline - Dev、Rectangle Outline - Dev、Wall Outline - Dev**。commit 为 `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`，属于已收录的 309 个真实 gallery 案例。累计完整功能通过数为 **18 / 309**，其余 291 个仍待完成。

| 案例 | 原脚本的全部几何、实例和外观 |
| --- | --- |
| Box Outline - Dev | 400 km 立方体；-106°/45° 的地面 ENU 矩阵乘局部 z=200 km 平移；BoxOutlineGeometry.fromDimensions；白色实例颜色 |
| Cylinder Outline - Dev | 长 400 km、上下半径 150 km；-100°/40° ENU + z=200 km；16 条竖线，128 段上下圆环 |
| Sphere Outline - Dev | 半径 300 km；-98°/45° ENU + z=300 km；stackPartitions=8、slicePartitions=12 |
| Ellipsoid Outline - Dev | radii=[200,200,300] km；-102°/45° ENU + z=300 km；stackPartitions=16、slicePartitions=8 |
| Rectangle Outline - Dev | [-100°,30°,-90°,40°] 地表矩形；世界坐标，无模型变换 |
| Wall Outline - Dev | -120°/60° 至 -90°/60° 的 WGS84 曲面墙，底高 0、顶高 500 km；WallOutlineGeometry.fromConstantHeights |

六例均保留 GeometryInstance、白色 ColorGeometryInstanceAttribute、flat PerInstanceColorAppearance，以及 `Math.min(2, maximumAliasedLineWidth)`。本后端公开的 maximumAliasedLineWidth 为 **1**，因此原表达式结果为一像素。实际绘制和整数拾取使用 LineList，网格之间没有填充面。

这些脚本没有 zoomTo 或自定义相机命令。工作台载入时使用统一北美宽视角（眼点 -100°/40°、17,500 km，向下观察），不会自动近距离取景；“自动取景全部实体”是可选测试操作。应用默认视角和默认底图与官方 Viewer 不逐像素匹配，不能把手动取景登记为原脚本自动取景。

## 启动与测试 UI

包内已包含生成的 WASM：

```bash
python3 scripts/start-test-ui.py
# http://127.0.0.1:8000/bindings/web/sandcastle.html
```

“完整案例移植”新增六个 Outline 按钮，也可从目录选择上述开发案例后点击“运行当前完整案例”。编辑器展示实际传入 Rust 的 `.primitive.json`，可用“应用 JSON / Primitive”重新加载。显隐、定位、删除、轮廓拾取、滚轮缩放和全部取景仍可测试。

```js
await viewer.loadPrimitiveInstances('/tests/sandcastle/full/box-outline-dev.primitive.json');
console.log(viewer.maximumAliasedLineWidth); // 1
viewer.zoomToEntities(); // 可选测试操作
```

```bash
CESIUM_OFFLINE=1 CESIUM_PRIMITIVES=tests/sandcastle/full/sphere-outline-dev.primitive.json \
  cargo run --locked -p cesium-viewer --bin desktop-viewer
```

桌面加载 Primitive 配置同样使用宽视角；可加 `CESIUM_ZOOM_ENTITIES=1` 近距离观察。CESIUM_ENTITIES / CESIUM_CZML / CESIUM_PRIMITIVES 互斥。

## Rust 接口与实现范围

`Scene::set_primitive_instances(renderer, bytes)` 严格解析原几何实例配置并编译为共享驻留绘制集合，原模型矩阵直接进入渲染器；支持单实例或实例数组、可选显式 id、颜色、上述六类静态轮廓和限定外观。解析/几何生成/上传完成后才替换当前集合，失败保留旧集合。Web SDK 支持对象或 URL、8 MiB 上限、共享最多四个导入任务、取消/过期结果丢弃和 destroy 保护。此接口替换共享集合，不是独立的完整 Cesium PrimitiveCollection API，也不保留原始 JS 对象句柄。

本批新增 Entity 的可选 `modelMatrix`（列主序 ECEF 4×4）。它与 positionDegrees / sampled position 三选一；仅支持局部几何，要求有限仿射矩阵、非奇异线性部分、基向量长度 1e-6..100，平移长度 1..1e8 m。世界 Rectangle/Wall 目前只接受缺省或单位矩阵。JSON 浮点解析启用 float_roundtrip，保留原 ECEF 双精度输入；有精确位值回归测试。

当前 Primitive 外观要求 flat=true、PerInstanceColorAppearance 与 lineWidth=1；Cylinder 的 numberOfVerticalLines 限 16，Sphere/Ellipsoid 分段沿用 3..128 及现有 128 点曲线采样。未知字段、动态属性、其他外观、非支持线宽或几何明确拒绝。原脚本没有给 instance 设置 id，运行时生成诊断 ID `primitive-0-instance-0` 供工作台管理和拾取检查，不伪造原脚本 id。

仍沿用共享引擎的几何预算、经纬度边界与地表偏移：地理轮廓约 10.5 m，局部 Box/Cylinder/Sphere/Ellipsoid 使用原矩阵。没有实现宽线、异步 Worker 生成、Primitive 增量属性修改、通用 MaterialAppearance 或一般 CesiumJS API 兼容。以上六个原脚本的几何/实例/外观行为完整移植，不声明逐像素一致或全部 Viewer 控件实现。

## 验收证据

- 源码记录器执行全部十八份固定 main.js 并校验 SHA-256，六个新案例的每个构造字段均有白名单；同时保存原始 Primitive 配置与独立预期实体快照。运行时实际加载原 `.primitive.json`，并由 Rust 编译，不加载预转换快照作渲染输入。
- 六项新增普通 Rust 测试覆盖全部参数、ECEF/ENU 与局部平移、精确 f64、仿射验证与非均匀缩放、实例数组/ID/重复拒绝、线段拓扑、曲面墙高程与路径，以及未知字段/颜色/预算错误。
- 十八项专项实际 GPU 测试通过。新案例核对默认宽视角可见、失败替换原子性、横竖画幅手动取景、白色轮廓像素与整数 ID，以及轮廓间隙穿透。预览为 `full-<id>-preview.png`，显示手动取景后的实际 GPU 图像。
- 十八个桌面原配置分别运行 90 帧，均为 errors 0；六个新配置通过 CESIUM_PRIMITIVES 读取原始输入，使用默认宽视角。
- 十八个 Chromium 155 / WASM WebGPU 案例核对原参数（包括完整模型矩阵）、真实 canvas 像素、轮廓/填充拾取、实际滚轮高度变化、窗口尺寸变化与重取景、目录入口及 destroy。报告绑定源文件、实际输入与预期快照 SHA-256。
- 全工作区 **90 项普通 Rust 测试、45 项实际 GPU 测试、八组 Node 测试通过**；fmt/clippy、原生构建和 WASM 构建/绑定通过。

浏览器验收使用显式 Vulkan/SwiftShader 与固定本地 256×256 中性影像（避免大图作为每个瓦片造成缓存压力），未验证实时影像服务或所有驱动。默认浏览器后端 DeviceLost 与完整恢复仍待解决。

复测命令：

```bash
npm ci
npm test
cargo test --locked --workspace
cargo test --locked --workspace -- --ignored --test-threads=1
npm run test:recipes:web # 可通过 CHROMIUM_PATH 指定 Chromium
python3 scripts/record-full-recipe-results.py
```

最新证据：`full-recipes-validation.json`、`full-recipes-browser.json`、`full-recipes-*-validation.log` 和每例 smoke 日志。历史版本文档保留原计数。

官方参考：[Primitive](https://cesium.com/learn/cesiumjs/ref-doc/Primitive.html)、[GeometryInstance](https://cesium.com/learn/cesiumjs/ref-doc/GeometryInstance.html)、[Geometry 与 Appearance](https://cesium.com/learn/cesiumjs-learn/cesiumjs-geometry-appearances/)。
