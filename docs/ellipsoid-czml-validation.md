# 三个新增完整案例（0.0.18）

本批完整移植 **Spheres and Ellipsoids、CZML Spheres and Ellipsoids、CZML Cones and Cylinders**。上游仍固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。累计完整功能案例通过数 **6 / 309**，其他 303 项待完整验收。

| 案例 | 全部原始行为 | 本批关键补齐 |
| --- | --- | --- |
| Spheres and Ellipsoids | 原位置与半径；蓝色实心椭球；红色 alpha=0.5 球体加黑边；黄色仅轮廓椭球，slicePartitions=24、stackPartitions=36；全集合 zoomTo | 可配置分段、独立轮廓网格、平滑法线、极点退化三角形处理 |
| CZML Spheres and Ellipsoids | 原始 document 1.0 及三个 packet；红球 alpha=100/255；黑色 outlineColor 使用 rgbaf；黄色 24×36 分区；CzmlDataSource.load/add 后对 Promise 目标取景 | 严格 rgbaf、CZML 分段参数、真实 CZML 解析与异步加载完成后取景 |
| CZML Cones and Cylinders | 原始 document 1.0 及两个 packet；绿色 [0,255,0,128] 圆柱与黑边；红色零顶半径圆锥；异步加载后全集合取景 | 原始 CZML 直接加载、128/255 透明度、完整原参数与几何/拾取验收 |

三份源码没有额外工具栏或远程数据依赖。所有案例特定实体、属性与初始取景均已移植；没有把它们改成通用演示或把官方页面嵌入本地。绘制仍由共享 Rust/wgpu 与 WASM 执行。

## 启动

```bash
python3 scripts/start-test-ui.py
# http://127.0.0.1:8000/bindings/web/sandcastle.html
```

新增三个“完整”按钮。也可从目录选择对应案例，再点“运行当前完整案例”。编辑器保留原 CZML；暂停、实体定位、显隐、删除、缩放和全部自动取景仍可用。

```js
await viewer.loadEntities('/tests/sandcastle/full/spheres-and-ellipsoids.json');
viewer.zoomToEntities();
await viewer.loadCzmlData('/tests/sandcastle/full/czml-spheres-and-ellipsoids.czml');
viewer.zoomToEntities();
await viewer.loadCzmlData('/tests/sandcastle/full/czml-cones-and-cylinders.czml');
viewer.zoomToEntities();
```

```bash
CESIUM_OFFLINE=1 CESIUM_CZML=tests/sandcastle/full/czml-spheres-and-ellipsoids.czml \
  CESIUM_ZOOM_ENTITIES=1 cargo run --locked -p cesium-viewer --bin desktop-viewer
```

## 引擎变更与边界

- Entity `shape.type=ellipsoid` 新增可选 `stackPartitions` / `slicePartitions`。目前只接受 3..128 的整数；CZML 同样接受整数或 `{number:...}` 常量，动态值、引用与更高分段明确拒绝。
- 未提供参数时，填充采用 64×64，轮廓采用 Core EllipsoidOutlineGeometry 的 10 / 8 默认值。显式参数同时作用于填充和轮廓；轮廓每条曲线使用 128 个采样点 / 127 段，保留上游的首尾经线接缝。极点的零长度线不绘制。
- 椭球法线按局部 `position / radii²` 归一化；极点三角形按精确极点生成，避免 sin(π) 残值产生退化面。仍使用已有一像素轮廓，不实现宽轮廓、空心/部分椭球或动态几何参数。
- CZML RGBA 颜色现在支持二选一的 `{rgba:[字节]}` 与 `{rgbaf:[0..1浮点]}`。同时提供两种编码、长度错误、越界或动态数组均拒绝。原始字节透明度不会四舍五入为 0.5。
- CZML 未指定材质时改为白色，匹配上游默认；此前的橙色演示默认不再用于 CZML。自定义 Entity JSON 的旧默认颜色保持不变。
- UI 的 CZML 完整案例使用 `loadCzmlData`，而不是把预转换 JSON 当运行输入。构建时同时保留原始 CZML 和独立预期实体 JSON，用于核对真实解析结果。异步导入仍具备失败原子性、取消、过期结果丢弃等已有保护。
- 这些原始 CZML 脚本只有单个数据源。此批实现其全部加载/添加/取景行为，不声明完整 DataSourceCollection 的多源、增量 process/delete、引用或一般 Promise API 兼容。

## 验证证据

1. 构建时在记录环境执行全部六份固定 main.js，校验各自 SHA-256；CZML 记录器还核对 load 返回的 Promise 确实被 add 并传入 zoomTo。原始 packet 全量保留，未知实体或图形字段拒绝遗漏。
2. 四项新增普通 Rust 测试覆盖椭球表面方程、分段/轮廓拓扑、法线、旧输入默认值、范围拒绝、CZML 包装数值与 rgbaf 的严格编码。
3. `tests/full-recipes-gpu.rs` 共六项真实 GPU 测试。新 CZML 案例直接走 Rust CZML 解析；所有顶点在横/竖画幅内、黑/黄轮廓像素与 ID、网格空隙穿透、实心拾取、半透明与不透明画面的差异均验证。图片为 `docs/full-spheres-and-ellipsoids-preview.png` 等。
4. 六组桌面原配置各运行 90 帧，Entities 为 3 / 2 / 3 / 3 / 3 / 2，errors 全为 0；两个新增 CZML 配置使用原始 `.czml` 文件。
5. 六个完整案例均通过真实 Chromium 155 + WASM WebGPU 测试：原始参数、实际 canvas 颜色、实体拾取、滚轮缩放、窗口变化后重新取景、目录入口及 destroy。报告同时绑定源文件、预期 JSON 和实际运行输入（CZML）的 SHA-256。
6. 全工作区 **78 项普通 Rust 测试、33 项实际 GPU 测试、八组 Node 测试通过**；原生/WASM 构建、绑定、fmt/clippy 通过。

浏览器测试使用显式 Vulkan/SwiftShader，并将外部影像请求替换为本地固定底图；不验证实时影像服务或所有设备驱动。默认浏览器后端仍可能丢失设备，完整设备恢复未实现。此处“完整”指固定脚本的案例功能，不代表逐像素一致、全部 Viewer 通用控件或 CesiumJS 全 API 兼容。

专项运行命令：

```bash
npm test
cargo test --locked --workspace
cargo test --locked -p cesium-viewer --test full-recipes-gpu -- --ignored --test-threads=1
npm run test:recipes:web  # 需可用 Chromium；可通过 CHROMIUM_PATH 指定
python3 scripts/record-full-recipe-results.py
```

共享最新证据为 `docs/full-recipes-validation.json`、`docs/full-recipes-browser.json`、`docs/full-recipes-*-validation.log`；0.0.17 文档保留当批历史状态。

官方参考：[EllipsoidGeometry](https://cesium.com/learn/cesiumjs/ref-doc/EllipsoidGeometry.html)、[EllipsoidOutlineGeometry](https://cesium.com/learn/cesiumjs/ref-doc/EllipsoidOutlineGeometry.html)。
