# 六个实体 Primitive 完整案例（0.0.21）

本批完整移植固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1` 的 **Box - Dev、Cylinder - Dev、Sphere - Dev、Ellipsoid - Dev、Wall - Dev、Ellipse - Dev**。累计完整功能通过数为 **24 / 309**，其余 285 项待完成。

| 案例 | 完整保留的原始行为 |
| --- | --- |
| Box - Dev | 400×300×500 km 箱体；-105°/45° 地面 ENU 乘局部 z=250 km；红色 alpha=0.5 经实例颜色字节量化为 128/255；closed=true，默认 translucent=true |
| Cylinder - Dev | 两个实例；-100°/40° 的 CSS GREEN 圆柱及 -105°/40° 红锥，长 400 km、底半径 200 km，顶半径分别 200/0 km；各自 ENU+z=200 km；closed=true、translucent=false |
| Sphere - Dev | -100°/40° ENU+z=300 km；红色 300 km 半径球；位置/法线顶点格式；closed=true、translucent=false |
| Ellipsoid - Dev | -100°/40° ENU+z=300 km；蓝色 radii=[200,200,300] km 椭球；closed=true、translucent=false |
| Wall - Dev | 按原顺序添加蓝色十一站锯齿上下界、红色 -115°/44° 至 -90°/44° 的 100–200 km 长墙、绿色 100 km 闭合墙；WGS84 曲面路径；translucent=false，缺省 closed=false |
| Ellipse - Dev | 红色地表椭圆 400×300 km；绿色 400×200 km、45° 旋转、200 km 高；蓝色 300×200 km、90° 旋转、0–200 km 挤出；原中心位置、颜色、closed=true 与 translucent=false |

六份源码均只有 Viewer、几何实例和 Primitive 添加，没有案例特定相机调用或额外工具栏。运行时实际读取原 `.primitive.json` 并由 Rust 编译；独立 `.json` 快照仅用于核对，绘制不使用 CesiumJS。

## 运行和测试 UI

源码包已包含重建的 WASM 和绑定，无需先安装 Rust：

```bash
python3 scripts/start-test-ui.py
# http://127.0.0.1:8000/bindings/web/sandcastle.html
```

“完整案例移植”新增六个实体 Primitive 按钮，也可从目录选择对应 Dev 案例，点击“运行当前完整案例”。编辑器显示原始 Primitive 配置，可通过“应用 JSON / Primitive”重新加载。支持显隐、删除、定位、拾取、滚轮缩放和全部取景。

这些脚本没有 zoomTo；工作台沿用统一北美宽视角，默认没有自动近景取景。细小表面和垂直墙在远景中可能小于一个像素，可点击“自动取景全部实体”观察细节。浏览器分别核对默认相机/画面与手动取景后的各实体颜色、拾取；不把手动取景登记为原脚本行为。

```js
await viewer.loadPrimitiveInstances('/tests/sandcastle/full/ellipse-dev.primitive.json');
viewer.zoomToEntities(); // 可选测试操作
```

```bash
CESIUM_OFFLINE=1 CESIUM_PRIMITIVES=tests/sandcastle/full/box-dev.primitive.json \
  cargo run --locked -p cesium-viewer --bin desktop-viewer
# 可加 CESIUM_ZOOM_ENTITIES=1 近距离观察
```

## 引擎补齐

- 严格编译器新增 BoxGeometry、CylinderGeometry、SphereGeometry、EllipsoidGeometry、WallGeometry（含 fromConstantHeights 与逐位置高度数组）、EllipseGeometry；保留原实例矩阵、实例数组顺序、颜色和 POSITION_NORMAL 顶点格式。局部几何使用原 ECEF 模型矩阵，世界墙/椭圆使用地理坐标；球/椭球沿用默认 64×64 分段。
- 填充 Primitive 的 PerInstanceColorAppearance 支持 flat（默认 false）、closed（默认 false）和 translucent（默认 true）。派生的 faceForward 为 !closed。flat=true 不受光照影响；本批原案例均使用带光照表面。
- 闭合不透明几何剔除背面；闭合透明几何先绘制背面，再绘制正面，颜色与整数 ID 通道共用几何/剔除规则。开放墙体双面显示，并按观察方向翻转背面法线。translucent=false 不按实例 alpha 混合；默认 translucent=true 即使颜色 alpha=1 也保持指定渲染状态。
- 颜色实例属性按 Cesium 的 floatToByte 规则量化后归一化；源脚本的 alpha=0.5 精确保留为 128/255，而非当成 Entity 颜色直接使用 0.5。
- 椭球使用解析平滑法线；填充圆柱/圆锥侧面按半径和锥度计算平滑法线，端盖保持各自法线；椭圆顶底盖与挤出侧面保留朝外绕序。
- 共享材质增加可控 faceForward、闭合透明两遍绘制和漫反射着色路径。原 glTF、Entity、轮廓、Label 和屏幕图形维持原默认行为；回归覆盖这些路径。

光照采用共享固定定向光、漫反射和 0.2 环境分量的近似，不声明与 Cesium Phong、太阳天文位置、默认底图或最终像素一致。仍未实现 OIT、任意相交透明三角形排序、一般 MaterialAppearance、任意自定义 shader/renderState 或完整 Viewer 通用控件。

## API 边界

使用现有 `Scene::set_primitive_instances`、Web `loadPrimitiveInstances` 和桌面 CESIUM_PRIMITIVES。输入最多 8 MiB、256 个 Primitive/实例，替换共享驻留集合，失败保留旧集合；不是独立完整 PrimitiveCollection API。未提供实例 id 时使用诊断 id，原始 JS 对象句柄不保留。

填充输入 vertexFormat 限 POSITION_NORMAL；目前支持上述六种填充几何，不接受其他几何、动态属性、显式 faceForward 或填充自定义 renderState。常量墙高与逐点高度数组不可混用。Entity 的可选 surfaceAppearance 仅适用于实心非屏幕、无额外材质的几何；模型矩阵、地理坐标、几何预算及地表偏移沿用前版边界。现有轮廓案例仍使用一像素 LineList。

## 验证

1. 全部二十四份固定 main.js 完整执行并核对 SHA-256；未知构造字段拒绝遗漏。六个新案例同时保存实际 Primitive 输入与独立预期实体快照，核对默认外观、颜色量化、模型矩阵和全部原始参数。
2. 六项新增普通 Rust 测试覆盖整个实例图、外观默认值与 GPU 材质标志、量化颜色、圆柱/锥体平滑法线和绕序、椭圆旋转/高度/挤出及端盖绕序、opaque alpha 与 flat 覆盖，以及未知/冲突输入拒绝。
3. 二十四项专项真实 GPU 测试通过。新案例验证实际填充拾取、横竖画幅取景、光照开关造成可见变化、不透明实例 alpha 改变不改变混合结果、半透明与 alpha=1 的画面差异。内部视点验证闭合不透明球剔除、开放后可拾取，以及闭合透明箱体背面通道可拾取。预览为 full-<id>-preview.png，显示手动取景后的实际 GPU 图像。
4. 二十四组桌面原配置各运行 90 帧，全部 errors 0。Primitive 案例实际使用 CESIUM_PRIMITIVES 原输入与默认宽视角。
5. 二十四个 Chromium 155 + WASM WebGPU 案例通过：原始完整参数（含 surfaceAppearance）、默认宽视角、手动取景后的逐类颜色、每个实例拾取、实际滚轮高度变化、窗口变化后取景、目录入口和 destroy。默认远景颜色统计与取景后统计分别记录，报告绑定源码/运行输入/预期快照哈希。
6. 全工作区 **96 项普通 Rust 测试、51 项实际 GPU 测试、八组 Node 测试通过**；fmt/clippy、原生/WASM 构建与绑定生成通过。测试预览采用编码后同步、原子替换，避免损坏原预览。

浏览器使用显式 Vulkan/SwiftShader 和固定本地 256×256 中性影像，未验证实时影像服务或所有设备驱动。默认后端 DeviceLost 与通用恢复仍待修复。

```bash
npm ci
npm test
cargo test --locked --workspace
cargo test --locked --workspace -- --ignored --test-threads=1
npm run test:recipes:web # 可用 CHROMIUM_PATH 指定 Chromium
python3 scripts/record-full-recipe-results.py
```

最新证据：full-recipes-validation.json、full-recipes-browser.json、full-recipes-*-validation.log 和逐例 smoke 日志。历史文档保留原计数。

官方参考：[PerInstanceColorAppearance](https://cesium.com/learn/cesiumjs/ref-doc/PerInstanceColorAppearance.html)、[ColorGeometryInstanceAttribute](https://cesium.com/learn/cesiumjs/ref-doc/ColorGeometryInstanceAttribute.html)、[CylinderGeometry](https://cesium.com/learn/cesiumjs/ref-doc/CylinderGeometry.html)。
