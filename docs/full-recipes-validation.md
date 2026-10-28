# 最新完整案例进度：0.0.33

新增 CZML Billboard and Label、Geometry Height Reference、CZML Path、CZML Reference Properties 四个完整案例，累计 **79 / 309**，剩余 230 个。127 项普通 Rust、80 项实际 GPU（全套 110 项）、8 组 Node、79 个浏览器 WebGPU 案例及 84 个桌面 90 帧配置通过。验证包含 CZML 广告牌与标签复合渲染、HeightReference 高度基准（RELATIVE_TO_GROUND 与 CLAMP_TO_GROUND）、飞行轨迹采样路径与外描边折线材质（CZML Path 与 PolylineOutline）、以及跨实体引用属性解算（reference properties 递归解包）。详见 [本批验证](czml-billboard-label-path-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.32

新增 Polylines - Dev、Frustum - Dev、Parallels and Meridians、Polygon Texture Coordinates - Dev 四个完整案例，累计 **75 / 309**，剩余 234 个。126 项普通 Rust、76 项实际 GPU（全套 106 项）、8 组 Node、75 个浏览器 WebGPU 案例及 80 个桌面 90 帧配置通过。验证包含 Fade 渐隐折线材质管线（LineMaterial 11u 与 WGSL 插值）、透视视锥实体与线框几何（FrustumGeometry / FrustumOutlineGeometry 与局部 ENU 姿态矩阵）、全球经纬度解算与等角航线容差（±90° 极点贯通与地心零点安全规避），以及显式多边形纹理坐标与嵌套内孔贴图（textureCoordinates 与 POSITION_NORMAL_ST 格式）。详见 [本批验证](polylines-frustum-polygon-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.31

新增 Geometry Offset Attribute - Dev、Geometry Offset Attribute (Box, Cylinder, Ellipsoid) - Dev、CZML Polygon Intervals and Availability、CZML Custom Properties 四个完整案例，累计 **71 / 309**，剩余 238 个。126 项普通 Rust、72 项实际 GPU（全套 102 项）、8 组 Node、71 个浏览器 WebGPU 案例及 76 个桌面 90 帧配置通过。验证包含几何法向偏移管线（GeometryOffsetAttribute TOP/ALL/NONE 与 attributes.offset）、多边形层级内孔与拉伸高度（holes 与 height）、CZML 时间区间/有效性动态生命周期（intervals 与 availability）、以及非视觉自定义属性对象（custom properties 与采样时间序列）。详见 [本批验证](geometry-offset-czml-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.30

新增 Rectangle、Z-Indexing Geometry、CZML Z-Index、Volumes - Dev 四个完整案例，累计 **67 / 309**，剩余 242 个。126 项普通 Rust、68 项实际 GPU（全套 98 项）、8 组 Node、67 个浏览器 WebGPU 案例及 72 个桌面 90 帧配置通过。验证包含多层几何 Z-Indexing 稳定层叠与 Reverse-Z 深度测试无 Z-Fighting、矩形拉伸/高度/旋转与独立纹理旋转角（stRotation）、CZML zIndex 解析与 Glow 发光线材质扩充，以及体图元（EllipsoidPrimitive）解码与 RimLighting/Stripe 材质绑定。详见 [本批验证](rectangle-zindex-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.29

新增 Polyline Volume、Polyline Volume - Dev、Polyline Volume Outline - Dev、CZML Polyline Volume 四个完整案例，累计 **63 / 309**，剩余 246 个。126 项普通 Rust、64 项实际 GPU、8 组 Node、63 个浏览器 WebGPU 案例及 68 个桌面 90 帧配置通过。验证包含三维空间截面沿轨扫掠几何核、Frenet/Darboux 局部标架、Rounded/Beveled/Mitered 拐角拓扑、首尾截面 earcutr 端面封盖、纵向棱线与首尾环线框提取，以及 Entity/Primitive/CZML 统一支持。详见 [本批验证](polyline-volume-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.28

新增 CZML Rectangle、Partial Ellipsoids 两个完整案例，累计 **59 / 309**，剩余 250 个。122 项普通 Rust、60 项实际 GPU、8 组 Node、59 个浏览器 WebGPU 案例及 64 个桌面 90 帧配置通过。验证包含内径中空壳体（innerRadii）、时钟与天顶切口（clock/cone angles）、四元数姿态、CZML 拉伸与旋转、条纹/图像材质绑定、中空表面精确三角面拾取以及 WebAssembly 跨平台时间抽象。详见 [本批验证](czml-rectangle-partial-ellipsoids-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.26

新增 Polyline、Polyline - Dev、Polyline Color - Dev、Polyline Material - Dev、Polyline Dash、CZML Polyline 六个完整案例，累计 **57 / 309**，剩余 252 个。115 项普通 Rust、85 项实际 GPU、8 组 Node、57 个浏览器案例及 62 个桌面 90 帧配置通过。验证包含像素线宽、全部原材质参数、Material 默认值与 Property 参数的区别、逐实体拾取和透明虚线空隙。详见 [本批验证](polyline-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.25

新增 Polygon、Polygon - Dev、Coplanar Polygon - Dev、CZML Polygon、Points、Offline 六个完整案例，累计 **51 / 309**，剩余 258 个。109 项普通 Rust、78 项实际 GPU、8 组 Node、51 个浏览器案例及 56 个桌面 90 帧配置通过。详见 [本批验证](polygon-points-offline-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.24

本批新增 CZML Corridor、CZML Position Definitions、Show or Hide Entities、HTML Overlays、Resolution Scaling、Hello World 六个完整案例，累计 **45 / 309**，其余 264 项待完成。105 项普通 Rust、72 项实际 GPU、8 组 Node、45 项浏览器 WebGPU 与 45 项桌面 90 帧检查通过。HTML / DPR 控件在浏览器验收，三个场景案例的桌面检查仅覆盖共享地球与投影内核。详见 [本批验证](scene-czml-groups-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.23

本批新增 Circle - Dev、Corridor、Corridor - Dev、Corridor Outline - Dev 四个完整案例，累计 **39 / 309**，其余 270 项待完成。103 项普通 Rust、66 项实际 GPU、8 组 Node、39 项同会话浏览器 WebGPU 和 39 项桌面 90 帧验证通过。范围与独立几何参考见 [本轮验证](corridor-circle-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.22

本批新增 11 个完整几何、材质及 CZML 案例，累计 **35 / 309**，其余 274 项待完成。100 项普通 Rust、62 项实际 GPU、8 组 Node、35 项同会话浏览器 WebGPU 与 35 项桌面 90 帧验证通过。范围与证据见 [本轮验证](geometry-material-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.21

本批新增 Box、Cylinder、Sphere、Ellipsoid、Wall、Ellipse - Dev 六个完整实体 Primitive 案例，累计 **24 / 309**，其余 285 项待完成。最新范围与证据见 [实体 Primitive 完整案例](primitive-solid-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.20

本批完整移植六个 Primitive 轮廓开发案例（Box、Cylinder、Sphere、Ellipsoid、Rectangle、Wall Outline - Dev），累计 **18 / 309**，其余 291 个待完成。最新范围与证据见 [Primitive 完整案例](primitive-outline-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.19

本批完整移植六个案例：CZML Box、Wall、CZML Wall、Circles and Ellipses、CZML Circles and Ellipses、CZML Point。累计 **12 / 309**，其余 297 项待完整验收。最新范围与证据见 [墙体、椭圆与 Point 完整案例](wall-ellipse-point-validation.md)。以下保留各历史版本记录，最新共享报告以 0.0.19 为准。

# 最新进度：0.0.18

累计六个完整功能案例通过；本批新增三个案例及证据见 [椭球与 CZML 完整案例](ellipsoid-czml-validation.md)。共享验证报告以最新版本为准。以下为 0.0.17 当批记录。

# 三个完整 Sandcastle 案例（0.0.17）

本批完整移植并验收 **Box、Cylinders and Cones、Plane** 三个案例，固定上游 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。验收对象是三个 `main.js` 中的全部案例行为，不再用通用几何演示替代原案例。

| 原案例 | 原始内容全部保留 | 验收 |
| --- | --- | --- |
| Box | -114° / -107° / -100°，40°N，300 km；400×300×500 km 三个箱体；蓝色实心、红色 0.5 alpha 加黑边、黄色仅轮廓；全集合 zoomTo | 源码参数、GPU、桌面 90 帧、浏览器 WebGPU 通过 |
| Cylinders and Cones | -100° / -105°，40°N，200 km；400 km 长、200 km 半径；半透明 CSS GREEN 圆柱加黑边、红色零顶半径圆锥；全集合 zoomTo | 同上 |
| Plane | -114° / -107° / -100°，40°N，300 km；400×300 km；UNIT_X / UNIT_Y / UNIT_Z 三种平面；蓝色实心、红色 0.5 alpha 加黑边、黄色仅轮廓；全集合 zoomTo | 同上 |

三个脚本没有额外工具栏、动态数据源、外部模型或自定义时间事件。它们要求的实体、材质、轮廓与初始自动取景都已实现。渲染由共享 Rust/wgpu 执行，浏览器使用生成的 WASM，不嵌入 CesiumJS 渲染器。

## 直接测试

```bash
python3 scripts/start-test-ui.py
# http://127.0.0.1:8000/bindings/web/sandcastle.html
```

“完整案例移植”区域有三个独立按钮及“自动取景全部实体”。也可以在左侧目录搜索 Box、Cylinders and Cones 或 Plane，然后点击“运行当前完整案例”。实体选择、定位、显隐、删除、编辑、拾取和滚轮缩放均可继续使用。普通功能子集配方保持单独记录。

```js
await viewer.loadEntities('/tests/sandcastle/full/box.json');
viewer.zoomToEntities();                  // 全部当前可见且可用的实体
viewer.zoomToEntities(['box-1','box-2']);  // 可选 ID 子集
```

新增 Plane JSON 字段 `normal`、`distance`，位于 `shape`，遵循实体 ENU 坐标系中的 `normal·position + distance = 0`。normal 默认为 [0,0,1]，必须为有限单位向量；distance 默认为 0，范围 ±1,000,000 m。填充和轮廓共用与固定上游 `PlaneGeometryUpdater.createPrimitiveMatrix` 一致的切向基，旧水平 Plane 配方继续兼容。圆柱/圆锥面分段增至默认 128，轮廓圆环 128 段、16 条竖线。

集合取景按实际几何顶点的 ECEF 范围计算保守包围球，包含 fill=false 的轮廓实体；依据当前画布宽高比选择水平/垂直较窄视角并加余量，避免硬编码视距裁掉模型。隐藏和时间不可用实体不参与；显式未知 ID、空可见集合或非法视口原子报错，保留旧相机。该 API 是当前自定义 SDK 的同步接口，不声明兼容 CesiumJS zoomTo 的全部输入类型或 Promise 生命周期。它会终止旧飞行/跟随并使旧拾取失效。

桌面：

```bash
CESIUM_OFFLINE=1 CESIUM_ENTITIES=tests/sandcastle/full/plane.json \
  CESIUM_ZOOM_ENTITIES=1 cargo run --locked -p cesium-viewer --bin desktop-viewer
```

另两案例替换 JSON 文件名即可。CESIUM_ZOOM_ENTITIES 在实体加载与其他初始相机设置之后执行。

## 源码和证据

- `scripts/build-full-recipes.mjs` 在 Node VM 中执行三个固定 main.js 的全部语句，记录 Viewer、entities.add 与 zoomTo；没有省略分支或用另一个演示替代。仅移除 ES import，记录器只允许三个案例使用的类和字段。脚本校验目录中的 sourceSha256，生成完整实体 JSON；不是通用 CesiumJS 兼容层。
- `scripts/full-recipes-test.mjs` 独立重新执行原始脚本，将全部转换结果与交付 JSON 逐字段比较。名称、位置、尺寸、RGB/alpha、fill、outline、outlineColor、Plane 法向与距离、集合取景调用均有证据。
- `crates/cesium-viewer/tests/full-recipes-gpu.rs` 为每个案例执行独立真实 wgpu/Mesa 测试。横屏与竖屏投影检查所有顶点处于画幅；填充几何返回实体 ID；黄色轮廓内侧穿透；黑/黄色边线的实际像素与拾取正确；将 alpha=0.5 改为 1 后超过 100 个像素改变。导出 `docs/full-*-preview.png`。
- `docs/full-*-smoke.log`：三个桌面案例各 90 帧，Entities 3 / 2 / 3，errors 0。
- `scripts/full-recipes-browser.cjs` 使用真实 Chromium 155、Rust WASM 和 WebGPU。验证全部原始实体参数、实际 canvas 像素颜色、实体点击拾取、滚轮缩放、resize 后取景、目录中的完整案例入口与 destroy。报告 `docs/full-recipes-browser.json` 关联源文件和 JSON 的 SHA-256。
- 浏览器使用显式 **Vulkan / SwiftShader**，并将外部影像响应固定为本地 NASA 底图纹理，避免网络改变结果。该测试不验证实时影像服务、用户设备驱动或所有浏览器。默认后端仍可复现设备丢失；保留 Instance/Adapter 句柄是生命周期保护，尚不等于完成设备自动恢复。
- `scripts/record-full-recipe-results.py` 只有在源/数据哈希、GPU、浏览器与桌面证据匹配时，才将 `full-recipes.json` 对应条目提升为 functional-passed。笔记不会改变通过数；重新生成配方会回到待验收状态，需重新执行验证。

全工作区 **74 个普通 Rust 测试、30 个实际 GPU 测试、八组 Node 测试通过**；原生/WASM 构建、绑定生成、fmt/clippy 通过。浏览器专项三个案例通过；历史通用 `test:web` 脚本不计为本次全部通过。

## “完整”的边界

完整功能案例通过数从 **0 增至 3**；目录仍为 309 个，其他 306 个尚未完成完整案例验收。19 个历史功能子集关联继续单独保留。

本次完成三个固定脚本的全部案例行为；不声明与官方像素完全一致，不包含完整 Viewer 通用控件（例如地名搜索、所有底图选择）、CesiumJS API 二进制/对象兼容或全部渲染器特性。光照、底图清晰度、抗锯齿和透明排序仍可能与官方有视觉差异；复杂相交透明体、OIT、设备恢复等缺口仍存在。图形案例完成不代表其他使用相同 API 的复杂案例也完成。

原始源码与官方入口：[Box](https://sandcastle.cesium.com/?id=box)、[Cylinders and Cones](https://sandcastle.cesium.com/?id=cylinders-and-cones)、[Plane](https://sandcastle.cesium.com/?id=plane)。
