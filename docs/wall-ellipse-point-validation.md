# 六个新增完整案例（0.0.19）

本批完整移植 **CZML Box、Wall、CZML Wall、Circles and Ellipses、CZML Circles and Ellipses、CZML Point**。上游固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。累计通过 **12 / 309** 个完整案例，剩余 297 个仍待完成。

| 案例 | 保留的全部案例行为 |
| --- | --- |
| CZML Box | 原始 document 与三个箱体 packet；原位置、400×300×500 km 尺寸；蓝色实心、红色 128/255 透明度加黑边、黄色仅轮廓；异步 load/add 后对原 Promise 目标取景 |
| Wall | 红色长墙沿 -115° 至 -90°、44°N 的曲面路径，高度 100–200 km；绿色闭合 100 km 墙及黑边；蓝色十一站锯齿上下界、alpha=0.5 与黑边；全部实体取景 |
| CZML Wall | 原始 document 与 wall packet；50°N 的十一站，顶高在 1,500 km / 500 km 间交替，底高默认 0；红色 alpha=150/255；异步加载后取景 |
| Circles and Ellipses | 300 km 绿色圆、200 km 图形高度及黑边；400×250 km 红色表面椭圆、alpha=0.5；300×150 km 蓝色椭圆、45° 旋转、0–200 km 挤出及黑边；保留原实体位置 150/100 km，图形高度独立计算；整组取景 |
| CZML Circles and Ellipses | 全部原 CZML packet 和上述几何；保留 CZML 绿色 255、红/蓝 alpha=127/255、红色椭圆白色轮廓、原始 rotation=0.78539；异步加载后取景 |
| CZML Point | 原 packet `point 1`、名称 point、位置 [-111,40,0]；白色 20 px 圆点、红色 4 px 轮廓；异步加载后取景 |

这些原始脚本没有额外自定义工具栏或外部数据依赖。这里的完整功能指上述固定 `main.js` 的全部案例实体、图形参数、材质、轮廓与初始取景行为，渲染由共享 Rust/wgpu 与 WASM 执行。不会加载官方 CesiumJS 代替本地引擎。完整通用 Viewer 控件、一般 Promise/DataSource API 和官方逐像素一致不在本次声明范围内。

## 直接运行

源码包已包含重建的 WASM 和绑定，无需安装 Rust：

```bash
python3 scripts/start-test-ui.py
# http://127.0.0.1:8000/bindings/web/sandcastle.html
```

在“完整案例”区域点击六个新增按钮；也可以从全部 309 个案例目录选择对应项，再点“运行当前完整案例”。编辑器保留原 CZML 输入。实体列表可定位、显隐和删除，画布支持拖拽、滚轮缩放与全部取景。

```js
await viewer.loadEntities('/tests/sandcastle/full/wall.json');
viewer.zoomToEntities();
await viewer.loadCzmlData('/tests/sandcastle/full/czml-point.czml');
viewer.zoomToEntities();
```

桌面也使用相同的原始配置：

```bash
CESIUM_OFFLINE=1 CESIUM_CZML=tests/sandcastle/full/czml-wall.czml \
  CESIUM_ZOOM_ENTITIES=1 cargo run --locked -p cesium-viewer --bin desktop-viewer
# Entity JSON 使用 CESIUM_ENTITIES=tests/sandcastle/full/wall.json
```

## 引擎补齐与适用范围

- **墙体路径**：独立 Wall 填充和轮廓共用 WGS84 Vincenty inverse/direct 路径。按地表距离细分，步长至多约 0.1° 对应的赤道弧长；上下界沿每个输入段线性插值。闭合墙保留原闭合站点。重复连续点、近对跖或不收敛路径、超过单段 1,800 步的输入明确拒绝。经度 ±180°、纬度 ±85°、高度 ±2,000 km，仍受总几何预算限制。
- **大椭圆与旋转**：半轴扩展至 1..1,000,000 m，短轴不大于长轴。边界遵循固定 Core EllipseGeometryLibrary 的地心单位向量、east/north 旋转轴与 quaternion 构造；填充和轮廓共用边界，顶底盖改用径向网格，避免大椭圆走一般多边形细分造成内存膨胀。边界至少 192 段并按尺度增加；挤出轮廓使用 16 条竖线。
- **图形高度与位置**：Entity JSON 椭圆新增可选 `shape.height`；省略时保持此前使用 `positionDegrees[2]` 的兼容行为。CZML 椭圆默认图形高度为 0，显式 height 独立于实体原位置，解析不再覆盖 position。原配方蓝色椭圆 position 高度 100 km，但图形底高为 0；绿色圆 position 为 150 km，但图形 height 为 200 km。
- **几何与绘制限制**：径向网格内部采用经纬度插值，细分密度和官方内部拓扑不同；现有填充约 10 m、轮廓约 10.5 m 的表面偏移保留。不声明贴地、地形采样、动态几何、宽线或逐像素匹配。每实体仍限 300,000 顶点、1,800,000 索引与 32 MiB 几何预算。
- **CZML 与 Point**：四个新增 CZML 案例实际加载原 `.czml`，独立 JSON 仅作预期参数核对。Point 沿用屏幕图形管线，20 px 填充加两侧各 4 px 轮廓，总直径 28 px；颜色覆盖与整数拾取共用片元判定。其宽度在相机远近变化时保持不变。

## 验证

1. 构建记录器执行全部十二份固定 `main.js`，核对源码 SHA-256；所有实体及图形字段均被映射，未知字段拒绝遗漏。CZML 原 packet 全量保留，并核对 load Promise 被 add 与 zoomTo 使用。
2. 六项新增普通 Rust 测试覆盖 WGS84 距离/跨日界线/路径反向、长墙弯曲与上下高度、椭圆轴向/旋转/大尺度边界、图形高度和原始位置独立、CZML Point 图形参数。
3. 十二项完整案例真实 GPU 测试核对横/竖画幅自动取景、实际填充拾取、黑/白/黄轮廓像素与 ID、仅轮廓穿透和透明混合。地理几何使用投影三角形内点检查，而非假定实体原点一定落在墙面上。Point 在 1 km / 10 km 观察距离分别验证白色中心、红色轮廓、轮廓外拾取穿透及固定像素宽度。
4. 十二个桌面配置逐个运行 90 帧，全部 `errors 0`；所有 CZML 案例使用原始 `.czml`。日志为 `full-<id>-smoke.log`，对应预览为 `full-<id>-preview.png`。
5. 十二个真实 Chromium 155 + WASM WebGPU 案例通过：原始参数及颜色、canvas 像素读回、每个填充实体拾取、滚轮前后实际相机高度变化、窗口尺寸变化后重新取景、目录入口和销毁。浏览器报告绑定源码、预期 JSON 与实际运行输入的 SHA-256。
6. 全工作区 **84 项普通 Rust 测试、39 项实际 GPU 测试、八组 Node 测试通过**；fmt/clippy、原生构建、WASM 构建和绑定生成通过。

浏览器验收使用显式 Vulkan/SwiftShader 和固定本地影像，未验证实时影像服务或所有设备驱动；默认浏览器后端 DeviceLost 与完整设备恢复仍待解决。

复测：

```bash
npm ci
npm test
cargo test --locked --workspace
cargo test --locked --workspace -- --ignored --test-threads=1 # 需可用 GPU
cargo test --locked -p cesium-viewer --test full-recipes-gpu -- --ignored --test-threads=1
npm run test:recipes:web # 需 Chromium，可用 CHROMIUM_PATH 指定
python3 scripts/record-full-recipe-results.py # 匹配证据后才提升状态
```

最新证据为 `full-recipes-validation.json`、`full-recipes-browser.json` 与 `full-recipes-*-validation.log`。历史文档的原版本计数保留。

官方参考：[WallGeometry](https://cesium.com/learn/cesiumjs/ref-doc/WallGeometry.html)、[EllipseGeometry](https://cesium.com/learn/cesiumjs/ref-doc/EllipseGeometry.html)、[PointGraphics](https://cesium.com/learn/cesiumjs/ref-doc/PointGraphics.html)、[固定 EllipseGeometryLibrary 源码](https://github.com/CesiumGS/cesium/blob/b8d3a36fe98a3e432eb89253c95d5f20e605e0f1/packages/core/Source/Core/EllipseGeometryLibrary.js)。
