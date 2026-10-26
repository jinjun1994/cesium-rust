# cesium-rs · wgpu 基础开发版

依据 `docs/implementation-plan.md` 从零实现的 Rust / WGSL 工程。当前版本 **0.0.26**，落实 P0 及部分 P1/P2 基础链路，**尚未达到方案的 V0.1 完成定义**。所有支持范围以 `docs/capabilities.json` 为准。

## 0.0.26：六个折线完整案例

新增 **Polyline、Polyline - Dev、Polyline Color - Dev、Polyline Material - Dev、Polyline Dash、CZML Polyline**，累计 **57 / 309**，剩余 252 项。实现像素线宽、WGS84 测地线／恒向线／直 ECEF、逐段／逐顶点颜色，以及辉光/taper、箭头、轮廓、16 位屏幕像素虚线和透明拾取。测试 UI 增加六个完整案例按钮，保留全部 23 个原始实例。完整范围与证据见 [折线验证](docs/polyline-validation.md)。

## 启动测试 UI

源码包已包含生成的 WASM，无需先编译 Rust 即可打开工作台：

```bash
python3 scripts/start-test-ui.py
# 或 npm start
```

启动后访问 http://127.0.0.1:8000/ 。默认展示 94 个主要入口；勾选“点云子树”可搜索全部 3,721 个 tileset JSON。选择样本后点击“加载选中样本”，或轮测筛选结果。界面提供地球渐进影像、ion 航拍影像、GeoJSON、缩放、文件加载和拾取测试，错误与验证范围直接显示。WebGPU 初始化失败时，仍可浏览目录、报告和原生 GPU 内容预览。

```bash
cargo build --locked -p asset-inspector
python3 scripts/validate-samples.py
# 可选：原生 GPU 单个内容预览，需 Vulkan / Metal / DX12
cargo build --locked -p cesium-viewer --bin sample-render-check
python3 scripts/render-samples.py --max-samples 16
npm ci
npm test
```

所有样本 URL 固定到清单中的仓库 commit。批量报告测试所有目录 JSON，并有限量检查主要入口的直接内容；不会把解析 / 解码通过标记为整个 tileset 视觉验收。详细结果、覆盖范围和复测说明见 `docs/sample-validation.md`。

## 本批新增（0.0.25）：六个完整多边形、点与离线案例

新增 **Polygon、Polygon - Dev、Coplanar Polygon - Dev、CZML Polygon、Points、Offline**。支持嵌套孔洞与岛屿、逐顶点高度和挤出、开放顶底盖、共面多边形及自定义纹理坐标，Points 提供原始六个菜单和重置；Offline 随包包含 42 张 Natural Earth II 瓦片，运行时可使用本地影像。累计 **51 / 309** 个完整案例通过，剩余 258 个。范围和验证见 [本批案例验证](docs/polygon-points-offline-validation.md)。

## 本批新增（0.0.24）：六个完整案例

新增 **CZML Corridor、CZML Position Definitions、Show or Hide Entities、HTML Overlays、Resolution Scaling、Hello World**。补齐 CZML 走廊与弧度坐标、三组父级显隐、随相机更新的 HTML 图片、分辨率与高 DPI 控制；Hello World 将已有地球基础功能纳入完整案例验收。累计 **45 / 309** 个完整案例通过，剩余 264 个。

启动 `npm start`，进入 `http://127.0.0.1:8000/bindings/web/sandcastle.html`，点击对应“完整”按钮。分组案例提供三个切换按钮，覆盖层显示原 Cesium 图片，分辨率案例提供滑块、文本框和推荐分辨率开关。完整范围与验证见 [六个案例验证](docs/scene-czml-groups-validation.md)。

## 本批新增（0.0.23）：四个完整圆形与走廊案例

新增 **Circle - Dev、Corridor、Corridor - Dev、Corridor Outline - Dev**。实现三种走廊转角与端帽、按米宽度、高度/挤出、填充和轮廓，保留原始实例颜色、外观与相机行为。测试 UI 增加四个可直接运行的入口；累计 **39 / 309** 个完整功能案例通过。完整范围和独立几何参考见 [圆形与走廊案例验证](docs/corridor-circle-validation.md)。

## 本批新增（0.0.22）：十一个完整几何、材质和 CZML 案例

新增 Ellipse/Circle/Polygon/Coplanar Polygon Outline - Dev、Rectangle - Dev、Per Instance Color - Dev、Simple Polyline - Dev、Material - Dev、Ellipsoid Surface - Dev、CZML Colors、CZML Point - Time Dynamic。补齐层级轮廓、旋转挤出矩形、逐段/逐顶点颜色、Dot 材质与默认材质参数、动态点自动时间轴及地球遮挡。累计 **35 / 309** 个完整功能案例通过验收。启动后可直接点击新增按钮测试。完整范围和证据见 [本轮案例验证](docs/geometry-material-validation.md)。

## 本批新增（0.0.21）：六个完整实体 Primitive 案例

新增 **Box - Dev、Cylinder - Dev、Sphere - Dev、Ellipsoid - Dev、Wall - Dev、Ellipse - Dev**。保留原始实例矩阵、几何、颜色及外观默认值，补齐填充光照、闭合/透明渲染状态和圆柱侧面平滑法线。新增六个完整案例按钮；累计 **24 / 309** 个案例通过源码、实际 GPU、桌面和浏览器验证。范围与证据见 [实体 Primitive 完整案例](docs/primitive-solid-validation.md)。

## 上批新增（0.0.20）：六个完整 Primitive 轮廓案例

新增 **Box Outline - Dev、Cylinder Outline - Dev、Sphere Outline - Dev、Ellipsoid Outline - Dev、Rectangle Outline - Dev、Wall Outline - Dev**。完整保留原几何实例、ECEF 模型矩阵、白色实例颜色、分段与线宽限制；新增 Rust/Web/桌面的 Primitive 配置加载入口，GPU 使用实际一像素 LineList。测试页提供六个新增按钮及原配置编辑器；累计 **18 / 309** 个完整案例通过验收。详见 [Primitive 轮廓完整案例](docs/primitive-outline-validation.md)。

## 上批新增（0.0.19）：再完成六个完整案例

新增 **CZML Box、Wall、CZML Wall、Circles and Ellipses、CZML Circles and Ellipses、CZML Point**。补齐长墙 WGS84 曲面路径、大尺度椭圆、图形高度独立于实体位置，以及原始 CZML 加载后的自动取景。测试页提供六个新增“完整”按钮；累计 **12 / 309** 个案例通过源码、实际 GPU、桌面和浏览器 WebGPU 验收。详见 [墙体、椭圆与 Point 完整案例](docs/wall-ellipse-point-validation.md)。

## 上批新增（0.0.18）：再完成三个完整案例

新增 Spheres and Ellipsoids、CZML Spheres and Ellipsoids、CZML Cones and Cylinders，保留原始分段、颜色编码、透明度与异步取景。补齐椭球可配置分段/轮廓、平滑法线、CZML rgbaf 和真实原 CZML 输入路径。新增三个完整案例按钮；累计 **6 / 309** 通过源码、原生 GPU、桌面与浏览器 WebGPU 专项验收。范围及证据见 [椭球与 CZML 完整案例](docs/ellipsoid-czml-validation.md)。

## 上批新增（0.0.17）：三个完整 Sandcastle 案例

Box、Cylinders and Cones、Plane 已按固定原始源码完整移植：原始位置/尺寸、颜色/透明度、黑色及黄色轮廓、仅轮廓实体和整组 zoomTo 均保留。补齐 Plane 法向/距离和按画幅自动取景；测试页有三个“完整案例”按钮。三个案例已通过原生 GPU、90 帧桌面与真实浏览器 WebGPU 专项验证；完整功能案例通过数为 **3 / 309**，不声明与官方逐像素一致。详见 [完整案例验收](docs/full-recipes-validation.md)。

## 上批新增（0.0.16）：五项相机控制

新增 setView 经纬度眼点与 HPR 姿态、定时 flyTo、取消飞行、移动 Entity 跟随和固定目标 lookAt。测试面板提供演示按钮、进度与跟随状态，拖拽/缩放/定位自动接管。双端共用 Rust Scene；用法、参数边界和验证见 [相机验证](docs/camera-validation.md)。

## 上批新增（0.0.15）：五项屏幕效果

新增 Label 背景与 padding、FILL/OUTLINE/FILL_AND_OUTLINE 描边样式、Billboard 旋转、随距离缩放像素偏移，以及按距离关闭屏幕图形深度测试。五个独立配方和组合 CZML 可在 UI 测试；详见 [屏幕效果验证](docs/screen-effects-validation.md)。

## 上批新增（0.0.14）：五项功能

新增中英文 Label、距离显示范围、随距离缩放、随距离透明度变化，以及 Billboard/Label 水平/垂直锚点与 Label 基线。五个独立配方和组合 CZML 可在测试 UI 运行，近/中/远按钮帮助比较。支持范围与测试证据见 [Label 与距离验证](docs/label-distance-validation.md)。

## 上批新增（0.0.13）

新增屏幕像素 Point 圆点/轮廓、PNG/JPEG Billboard、尺寸/scale/pixelOffset 与共享透明拾取。接入 CZML 静态图形及线性采样位置，并加入三个测试 UI 配方。支持范围和限制见 [屏幕实体验证](docs/screen-validation.md)。label、距离属性、贴地和完整 Billboard API 尚未实现。

## 上批新增（0.0.12）

新增 Entity/CZML PNG/JPEG 图片材质、重复纹理与透明拾取；支持一像素几何轮廓和 `fill=false`，并提供图片/轮廓 JSON 与 CZML 测试按钮。外部图片通过 `loadEntities` / `loadCzmlData` 有限并发加载，支持取消和过期结果丢弃；桌面支持相对文件资源。

支持范围及证据见 [图片与轮廓验证](docs/image-outline-validation.md)。这仍是 Sandcastle 功能子集。

## 上批新增（0.0.11）

透明混合、网格级排序与透明片元拾取已加入共享 wgpu 渲染器；GLB 支持 BLEND。Entity/CZML 新增条纹、棋盘格和网格材质。测试页提供透明/材质、CZML 材质及原始透明墙体配方。48 项普通测试、9 项实际 GPU 测试通过。

范围、用法与限制见 [材质验证](docs/material-validation.md)。仍未完成全部 Sandcastle 功能，复杂相交透明物体和浏览器 WebGPU 尚待完善与验收。

## 上批新增（0.0.10）

地理折线、多边形孔洞/常高挤出、跨日期变更线矩形、静态椭圆与墙体已加入 Rust Entity；CZML 新增 polygon、rectangle、ellipse、wall。测试页提供实体定位、显隐、删除和按 ID 添加/更新，并关联 10 个案例的功能子集。修正了经纬度定位偏差、浮点闭合环误拒绝和宽折线曲面细分。

操作方法、边界与测试证据见 [地理实体验证](docs/entity-geography-validation.md)。仍未完成全部 Sandcastle 功能；完整案例验收状态继续单独记录。

## Sandcastle 全案例验收台

工作台顶部“Sandcastle 全案例”打开 `bindings/web/sandcastle.html`，包含固定 Cesium 1.146 的全部 309 个案例目录、参考源码/API 和本机验收记录。新增 Rust Entity 箱体、椭球、圆柱/圆锥、平面与严格 CZML 子集，以及播放/暂停/倍速/倒放/时间轴和实体拾取。点击“五种实体几何”或“CZML 动态箱体”直接测试，无需 token。

**目录收录不代表功能全部实现**。本版本二十四个完整案例的特定脚本行为已通过验收，其余仍待开发；剩余范围与 SDK/桌面用法见 [Sandcastle 验证与缺口](docs/sandcastle-validation.md)。

## 全球地形

工作台和桌面默认加载公开 Terrarium 全球高程，无需 token；约 ±85.05° 以外使用椭球回退。工作台“全球地形”区域可启用 Cesium World Terrain（使用上方您自己的 ion token）、关闭地形、加载 layer.json / Terrarium 模板、调整高度倍率和细化精度、定位珠峰、倾斜观察并查询已加载高程。离线合成样例用于断网测试，不是真实高程。

```bash
# 桌面：公开地形 + 全球影像
cargo run --locked -p cesium-viewer --bin desktop-viewer
# 桌面：使用您自己的 ion token，同时用于航拍与地形
CESIUM_ION_TOKEN=YOUR_TOKEN cargo run --locked -p cesium-viewer --bin desktop-viewer
# 桌面：自定义 quantized-mesh layer.json，或含 {z}/{x}/{y} 的 Terrarium 模板
CESIUM_TERRAIN_URL=/path/to/layer.json cargo run --locked -p cesium-viewer --bin desktop-viewer
# 禁用地形：CESIUM_TERRAIN=0；离线：CESIUM_OFFLINE=1
# 初始视角：CESIUM_VIEW_LOCATION=86.92528,27.9881,20000
```

地形采用父级回退、整组子瓦片替换、裙边、RTC 坐标和已加载地形的相机避地；影像重投影贴地。GeoJSON 和 3D Tiles 保留其原始高度，不会自动贴地。高程保留数据源原始基准，未进行大地水准面修正。实时 ion 地形需要您有权限的 token；本次未实测其在线服务。详见 `docs/terrain-validation.md`。

## 快速运行

需要 Rust 1.99.0；`rust-toolchain.toml` 固定工具链、rustfmt、clippy 与 WASM target，`Cargo.lock` 固定依赖。桌面需要可用的 Vulkan、Metal 或 DX12 adapter；Linux 示例启用 X11。

```bash
cargo run --locked -p cesium-viewer --bin desktop-viewer
cargo run --locked -p cesium-viewer --bin desktop-viewer -- tests/fixtures/tileset.json
cargo run --locked -p cesium-viewer --bin desktop-viewer -- /path/to/model.glb
cargo run --locked -p cesium-viewer --bin desktop-viewer -- https://your-server/tileset.json
```

无输入时显示完整的 WGS84 地球，先显示 NASA Blue Marble，再按视角逐级加载 Cesium Natural Earth II 瓦片（0–2 级）。配置 `CESIUM_ION_TOKEN` 后改为 Cesium ion 默认全球航拍影像（asset 2，Bing Maps Aerial）。默认航拍需要用户自己的 ion token；工程没有内嵌第三方演示凭据。初始相机位于地心外 2.65 个赤道半径，朝向亚洲。左键拖动旋转、滚轮缩放；按 `+` / `-` 放大缩小。地球缩放按离地高度计算，最近约 10 米，并按 WGS84 椭球局部半径限制相机进入地表，近景旋转速度同步降低。Home 回到全球视角，Esc 退出。

此前看到的三角形来自 `tests/fixtures/triangle.glb` / `triangle.b3dm`，由工程生成，用于格式解析、变换和拾取验证；旧默认视角对准该瓦片附近，无法观察全地球。现在只有显式运行 `--demo-tile` 才显示这个近景测试：

```bash
cargo run --locked -p cesium-viewer --bin desktop-viewer -- --demo-tile
```

瓦片模式仍可点击拾取、高亮，并在终端显示 Batch Table 属性。标题栏统计地球开关、驻留内容、几何/底图估算字节、实际绘制调用与错误数。NASA 回退底图无网络依赖；Natural Earth II 无需 token，默认航拍需要 ion token。用 `CESIUM_OFFLINE=1` 完全关闭在线影像。

## 全球渐进影像

```bash
# Cesium 默认全球航拍：先在环境中配置自己的 token，再启动。
export CESIUM_ION_TOKEN='YOUR_ION_ACCESS_TOKEN'
cargo run --locked -p cesium-viewer --bin desktop-viewer
# 完全离线底图
CESIUM_OFFLINE=1 cargo run --locked -p cesium-viewer --bin desktop-viewer
```

流程与 Cesium 默认 world imagery 一致：请求 ion asset 2 endpoint，从 BING options 获取服务/密钥，再请求 Bing metadata，使用返回的模板、subdomains、zoomMax 与版权信息生成 QuadKey URL。遇到不支持的 ion externalType 会明确报错并保留 NASA 底图，不把其他数据源冒充默认航拍。

影像模块独立于 3D Tiles：地平线/视锥剔除、屏幕误差细化（最多 384 个区域）、低级祖先优先请求、4 个下载/解码并发、每帧最多提交 1 张影像、未就绪区域使用最近已加载祖先的子区域 UV。父子覆盖区域互斥，避免叠层 Z-fighting；不同网格精度的边缘用 skirt 闭合。Mercator 的 ±85.051°之外用 NASA 极区填充。

桌面 HTTPS 使用 rustls 和系统信任库校验证书，支持系统已配置的企业代理 CA。

GPU 影像缓存上限 64 MiB，其中 16 MiB 预留覆盖网格；可见祖先被保留。缓存不足时继续显示祖先并拒绝新上传。JPEG/PNG 输入限 4 MiB、图像最长边 1024、解码分配限 8 MiB。失败最多 3 次尝试，按帧延后重试（非 HTTP Retry-After 策略）。请求失败保留回退覆盖，token 和带凭据 URL 不进入错误日志。Web 切换场景会 abort 并用 epoch 丢弃旧请求。

其他 XYZ/TMS 服务也可通过环境配置：`CESIUM_IMAGERY_URL` 支持 `{z}`、`{x}`、`{y}`、`{reverseY}`、`{quadkey}`；`CESIUM_IMAGERY_PROJECTION=geographic` 为 2×1 地理坐标 TMS，否则为 Web Mercator（内部 z 从 1 开始）；`CESIUM_IMAGERY_MAX_LEVEL` 指定上限。不要把普通 Web Mercator z=0 单瓦片服务的零级假设套到 Bing level 0 上：Cesium Bing level 0 对应此模块 z=1。

Natural Earth II 是 Cesium 的公开示例底图，不是 ion 默认全球航拍；最高 2 级不能提供街道级细节。已实际验证 Natural Earth 下载、拼接和桌面流式加载；尚未提供用户 token，真实 ion/Bing 联网验证未完成。当前实现不含 ion token 自动续期、设备恢复、物理地形或完整 imagery provider 类型集。

实际影像瓦片 GPU 导出见 `docs/imagery-preview.png`，原 NASA 底图导出见 `docs/earth-preview.png`。

实际 GPU 渲染预览见 `docs/earth-preview.png`。可用同一场景和渲染器重现：

```bash
cargo run --locked -p cesium-viewer --example render-earth -- earth-preview.png
```

该命令读回真实 GPU 颜色目标，检查非空球体区域和色彩数，并导出 PNG；不是生成式图片或另一套绘图实现。

HTTP 与本地文件都先检查输入字节限制。子 URI 按 tileset URL 解析，不自动继承 query token；HTTP 不跟随重定向。ion 影像凭据由独立 provider 处理，3D Tiles 网络路径仍不实现通用鉴权。不要把该示例当作生产网络客户端。

## 三维模型材质与外部 tileset

```bash
cargo run --locked -p cesium-viewer --bin desktop-viewer -- tests/fixtures/textured.glb
cargo run --locked -p cesium-viewer --bin desktop-viewer -- tests/fixtures/external-tileset.json
```

GLB / b3dm 支持 BIN bufferView 和 base64 data URI 内嵌 PNG/JPEG 图像。实现 glTF 核心五种纹理：baseColor、metallicRoughness、normal、occlusion、emissive。baseColor/emissive 按 sRGB 采样，其余为 linear；金属度使用 B、粗糙度使用 G、遮蔽使用 R。支持 TEXCOORD_0/1、COLOR_0、TANGENT、normal scale、occlusion strength、顶点颜色相乘、sampler wrap/mag/min/mipmap 过滤和完整 mip 链。颜色 mip 在 linear light 下平均。OPAQUE 忽略纹理 alpha，MASK 在颜色和 ID 拾取中使用相同采样与阈值。

法线使用提供的切线；缺失切线时采用片元导数估算 TBN，尚未实现 MikkTSpace。光照仍是方向光 GGX + ambient 近似，未包含 IBL、透明混合、物理曝光或所有 glTF 扩展。外部 glTF 缓冲和图像 URI 仍明确拒绝。

图像默认最多 256 个、最长边 4096，受共享 decoded byte limit 约束。CPU 共享同一 decoded image 的 Arc；GPU 每个 mesh 的纹理和 mip 字节计入驻留预算，预算不足不会继续上传。尚未实现跨 mesh GPU 纹理去重和完整峰值/staging 记账。

3D Tiles 的 content URI 可以指向另一个 tileset JSON。加载器将外部根原子接入，继承引用节点变换和省略的 refine，资源相对外部 JSON 的 URL 解析。检测祖先文档 URL 循环、跨文档最大深度和总节点上限；外部引用节点不能已有 children。尚未实现 tilesetVersion/完整结构化 metadata、多 content、implicit subtree 或所有地理 bounds 类型的紧致剔除。

## GeoJSON 图层

```bash
# 加载地球与 GeoJSON，自动定位数据范围
cargo run --locked -p cesium-viewer --bin desktop-viewer -- tests/fixtures/geojson/demo.geojson
# 同时加载三维瓦片和地理图层；保留瓦片原有视角，Home 回到全球
cargo run --locked -p cesium-viewer --bin desktop-viewer -- tests/fixtures/tileset.json --geojson=tests/fixtures/geojson/demo.geojson
# 导出实际 GPU 叠加效果
CESIUM_TEST_GEOJSON=tests/fixtures/geojson/demo.geojson cargo run --locked -p cesium-viewer --example render-earth -- geojson-preview.png
```

支持 FeatureCollection、Feature、Point、LineString、Polygon、MultiPoint、MultiLineString、MultiPolygon 和 GeometryCollection；Feature 空 geometry、id 和原 properties 均保留。坐标使用 WGS84 经度/纬度/可选椭球高度，拒绝 CRS 覆盖和无效坐标。Polygon 支持孔洞、经度跨日界线展开、自交/孔洞重叠检查，以及经纬度三角面细分后映射到椭球。顶点保存为图层局部 f32，地理原点通过 f64 model transform 提交，避免直接在大 ECEF 坐标上存小标记。每个几何批次都带 feature ID，可点击查询 `{id, properties}`。

Web 页面提供“叠加 GeoJSON 示例”“叠加 GeoJSON URL”“定位图层”“清除图层”。SDK 用法：

```javascript
const id = await viewer.loadGeoJson('/data/roads.geojson', {
  point_radius: 5000, line_width: 2000, surface_offset: 100,
  point_color: [1, 0.15, 0.02, 1], line_color: [1, 0.7, 0.03, 1],
  fill_color: [0.02, 0.65, 0.25, 1], segment_degrees: 0.5,
});
viewer.focusGeoJson(id);
// viewer.addGeoJson(objectOrUint8Array, style) 同步添加；返回图层 id。
viewer.removeGeoJson(id);
viewer.clearGeoJson();
```

图层添加不替换现有底图、影像或 tileset。SDK 清空/切换会中止旧请求并拒绝迟到结果，移除图层会使待完成的拾取失效；主源加载完成时也再次切换请求 epoch，避免等待期间发出的旧瓦片被投到新场景。最多 4 个并发 GeoJSON fetch。

范围限制：点和线宽均为米，不是屏幕像素；点采用抬升的 ENU 圆盘。折线分段形成 ribbon，没有完整 miter/round join、dash 或贴地高度采样。面在经纬度平面 triangulate，支持跨度不超过半球且位于 ±85°内的简单环；极区点/线可使用。没有文字标签、挤出、透明混合、terrain clamp 或按属性计算样式。图层使用不受方向光影响的 unlit 色彩。颜色为 opaque linear RGBA；默认 surface_offset=100 米，避免曲面近似造成遮挡。

预算：每次输入 8 MiB、10000 features、100000 输入 positions、300000 输出 vertices、32 MiB 解码几何，GeometryCollection 深度 32；每个 polygon ring 至多 1024 顶点，交叉检测总比较数至多 400 万。最多 32 个图层，GeoJSON GPU 几何总预算 64 MiB。解码和细分仍在主线程/调用线程，尚无 Worker 或全应用内存峰值保证。实际 GPU 导出见 `docs/geojson-preview.png`。

## 浏览器

**验证限制：WASM 编译和 JS 绑定生成已通过，但本环境的 Chromium 151 + 软件 WebGPU 在初始化后报告 DeviceLost(Destroyed)，点击端到端测试未通过。尚未确定根因；浏览器运行不作为本次已验收能力。**

源码包包含已生成的 `bindings/web/pkg`，可直接启动本地 HTTP 服务：

```bash
python3 -m http.server 8000
```

打开 <http://localhost:8000/bindings/web/>。需要支持 WebGPU 的浏览器；远程部署需要 HTTPS。页面右下角有放大、缩小和全球视角按钮；支持滚轮、触控双指缩放，点击画布后也可用 `+` / `-` / Home。页面可载入 tileset URL 或 GLB/b3dm URL，也可打开本地 GLB/b3dm 文件。跨域服务需要 CORS。

重新构建：

```bash
cargo install wasm-bindgen-cli --version 0.2.129 --locked
python3 scripts/build-web.py
```

SDK 用法：

```javascript
import {attachCanvas} from './bindings/web/sdk.js';
const viewer = await attachCanvas(canvas, {
  onPick: result => console.log(result),
  onError: message => console.error(message),
  onCredits: credits => { /* 将服务版权显示在可见区域；HTML 用 DOMParser/textContent 安全展示 */ },
  // ionToken: YOUR_ION_ACCESS_TOKEN, // 可选，初始化默认全球航拍影像
});
await viewer.loadWorldImagery(YOUR_ION_ACCESS_TOKEN);
// viewer.showGlobe() 返回无 token 的 Natural Earth II 示例影像。
// await viewer.load('/assets/tileset.json');
// 卸载时释放 RAF、DOM 监听、ResizeObserver、请求、WASM 与 GPU 引用。
viewer.destroy();
```

Web 路径用 4 个受限 fetch 和 JS epoch 检查旧结果；**解码仍在主线程**，尚未实现 Worker。CPU/WASM/浏览器总内存、设备恢复和大资源分段上传尚未达到发布方案要求。

## 工程模块

| crate | 已实现职责 |
| --- | --- |
| cesium-math | WGS84 正反算、ENU、射线、保守缩放、视锥、SSE、相机相对变换、Reverse-Z |
| cesium-formats | 安全 GLB/b3dm 拆包、静态三角形、标准 sparse/stride accessor、节点变换、五类核心材质纹理及 sampler、Batch Table JSON、基础 _BATCHID |
| cesium-tiles | 显式树、refine 继承、父子变换、sphere/box、保守 region、空节点、ADD/REPLACE 事务选择 |
| cesium-streaming | 预算预留/调整/释放、scene/device/generation token、带 pin 的事务式 LRU 淘汰 |
| cesium-renderer | wgpu 30.0.1、显式三组布局、动态对象 uniform、WGSL GGX 核心 PBR 纹理材质、Reverse-Z、R32Uint 异步拾取、高亮 |
| cesium-viewer | 桌面与 Web 入口、场景协调、相机操作、原生有界下载/解码、驻留内容提交、全球渐进影像 provider/覆盖/缓存 |
| asset-inspector | validate / inspect / capabilities 命令 |

```bash
cargo run --locked -p asset-inspector -- validate tests/fixtures/triangle.b3dm
cargo run --locked -p asset-inspector -- inspect tests/fixtures/tileset.json
cargo run --locked -p asset-inspector -- capabilities
```

## 验证

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo build --locked -p cesium-viewer --lib --target wasm32-unknown-unknown
# 在有 GPU 或 Mesa Vulkan 软件 adapter 的环境显式执行：
cargo test --locked -p cesium-renderer offscreen_render_and_pick -- --ignored
# 浏览器端到端测试：
npm ci
npx playwright install chromium
npm run test:web
```

离屏测试等待 GPU 仅用于测试；正常渲染和拾取不阻塞等待 GPU。测试资产为本工程生成的小型合成资产，不足以替代真实城市资产与长时漫游验收。验证记录见 `docs/validation.md`，后续开发任务见 `docs/development-status.md`。

## 当前限制

外链 glTF、压缩扩展、透明 BLEND、动画、pnts/i3dm/cmpt、隐式瓦片、多内容、viewerRequestVolume、CZML、Worker、设备丢失恢复与细化滞回尚未实现。已支持核心 PBR 纹理、外部 tileset、全球影像和 GeoJSON 基础子集。已检测到的不支持资产明确报错；不会声称完整兼容 CesiumJS 或 3D Tiles 1.1。

`region` 使用 WGS84 区间包围球，不应用 tile.transform；小区域可以正确定位相机，跨日界线和极区保持保守包含。背景椭球使用 256×128 的 f32 ECEF 网格，仅用于背景，不满足近地表高精度地形用途。CPU/GPU 预算当前覆盖输出几何与任务预留，**不等于总进程/浏览器/驱动内存上限**；JSON、解码临时数组、uniform、附件、驱动分配尚未统一记账。

原生每帧最多提交一个内容，尚未分段复制大型几何。HTTP 临时失败目前按终止失败处理，父回退仍保留；重试、取消策略、拓扑扩展、缓存协议和可观测性属于下一阶段工作。纯整数拾取与 MASK 使用相同 discard；最多一个在途拾取，resize 使旧结果失效。无效结果目前与空命中都返回空值，后续需暴露明确状态。

这是独立实现的项目工作名，与 Cesium 官方没有隶属关系。依赖与许可证说明见 `docs/third-party.md`。
