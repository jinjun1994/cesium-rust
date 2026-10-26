# 六个完整案例（0.0.25）

固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。本批完成 Polygon、Polygon - Dev、Coplanar Polygon - Dev、CZML Polygon、Points、Offline，累计 **51 / 309**，其余 258 个继续开发。

| 案例 | 全部案例行为 |
| --- | --- |
| Polygon | 全部 8 个原图形：地表红面、开放顶底的绿色挤出面、两种带原 JPEG/自定义 ST 的逐顶点高度面（含孔洞）、橙色逐顶点挤出面、四层蓝色孔洞/岛屿、青色竖直面、紫色 RHUMB 挤出面；保留材料、透明度、轮廓和全集合取景 |
| Polygon - Dev | 全部 6 个实例，原 Primitive 分组、实例字节颜色、closed/translucent 选项、嵌套孔洞、逐顶点高度及挤出、两组原始纹理坐标与 JPEG；保留原默认视角 |
| Coplanar Polygon - Dev | 全部 3 个实例，任意共面 3D 面、孔洞内岛屿、蓝/红颜色及原 Checkerboard 默认值、-20° ST 旋转、原材质外观及默认视角 |
| CZML Polygon | 全部 5 个原 packet：红面、红/青棋盘格、开放顶底绿面、橙色逐顶点挤出和轮廓、带两个洞的蓝面；保留真实原始 CZML 输入、颜色字节与异步完成后取景 |
| Points | 全部 6 个原菜单：添加点、创建时属性、创建后修改、多个点、按距离缩放、按距离透明；每次切换执行清空/重置；保留原始位置、尺寸、轮廓、SKYBLUE/YELLOW 等颜色、alpha 和 NearFarScalar |
| Offline | 原本地 TMS Natural Earth II、关闭底图选择器和地理编码器；随包提供完整 0–2 层 42 张 JPEG；Web 从本地服务器加载，原生从本地文件加载，无 ion token 或外网依赖 |

## 实现

新增 `polygon_surface.rs`：有界递归层级、偶数深度岛屿、带孔 earcut、共面主轴投影、原始 ST 层级校验、逐顶点 ECEF 面、静态挤出及开放顶底盖。地表面按 WGS84 / RHUMB 细分，曲面顶底盖采用约 20 km 目标边长及递归深度上限；顶点共享和曲面平滑法线保持原 32 MiB 集合预算。共面面使用协方差 Jacobi 主轴，并在孔洞内岛屿复用外层 ST 坐标域；图像行翻转适配原 ST 的底部原点，保持 Logo 正向；逐顶点面的轮廓保持直 ECEF 线段。RHUMB 轮廓按等方纬度插值，保持恒纬边。

Entity / Primitive 原纹理图片通过统一的有界 PNG/JPEG 资源加载器解析，支持原 Primitive 相对图片路径、重复图片去重、并发限制、旧请求取消及原集合保留；不扫描任意元数据字段。SDK 在解析影像模板 URL 后恢复 `{z}` / `{x}` / `{reverseY}` 占位符，避免 URL 编码导致全部本地请求指向错误路径。

新增 `entity_variants.rs` 为 Points 的全部静态菜单结果提供严格校验和原生选择；Web 下拉菜单与重置按钮复现原脚本各回调的结果。它不等于通用 Cesium CallbackProperty 或任意运行时 JavaScript 执行器。

原 `Polygon - Dev` 中 Primitive 构造参数上的 `materialSupport` / `translucent` 字段不控制原 Cesium Primitive 外观；输入仍保留这些原构造参数，渲染状态来自实际 appearance。原始脚本、原始 `.primitive.json` / `.czml`、独立预期 `.json` 和全部 Points `.variants.json` 均随包保留。

## 独立参考与资源来源

`build-polygon-reference.mjs` 使用离线测试专用 `@cesium/core@0.1.0`。先核对 PolygonGeometry / CoplanarPolygonGeometry 的 Git blob SHA-1 与固定 commit 的源文件一致，再生成 `polygon-reference.json`。Rust 对全部 20 个共面参考顶点核对世界位置（<0.1 m）及自动/旋转 ST（<1e-5）；另外验证自定义 ST 的逐顶点保留、错误层级拒绝和开放顶底的拓扑变化。参考工具不参与运行时渲染。

可选重建：

```bash
npm install --prefix .reference-tools @cesium/core@0.1.0 mersenne-twister earcut
node scripts/build-polygon-reference.mjs
```

JPEG 来自同一 commit 的 `packages/sandcastle/public/images/Cesium_Logo_Color.jpg` 和 `packages/engine/Source/Assets/Textures/NaturalEarthII/`。43 项资源的路径、大小和 SHA-256 见 `tests/sandcastle/batch25-assets.json`。上游许可保留在 `tests/sandcastle/UPSTREAM-LICENSE.md` 与 `reference-license.md`。原 Logo 仅用于复现该示例。

## 测试 UI

```bash
npm start
```

打开 `http://127.0.0.1:8000/bindings/web/sandcastle.html`，点击新增六个“完整”按钮，也可从 309 项目录选择后点“运行当前完整案例”。

- 多边形：查看全部面、孔洞、竖直面、纹理与轮廓；拖拽、滚轮、点击选取；默认视角的 Dev 案例可使用“自动取景全部实体”。
- Points：下拉菜单逐项切换六个示例；重置不会累积实体。缩放观察点大小或透明度随距离变化。
- Offline：加载本地 Natural Earth II，断开外网仍可缩放；切回其他完整案例会恢复原影像提供器并隐藏旧菜单。
- 原生 Points 可用 `CESIUM_VARIANTS=.../points.variants.json` 与 `CESIUM_VARIANT_INDEX=0..5` 选择各项；`CESIUM_SCENE=.../offline.scene.json` 使用本地影像。

交付包已更新 WASM；`build-web.py` 兼容自定义 `CARGO_TARGET_DIR`。重新构建需要 wasm32 目标和 wasm-bindgen-cli 0.2.129。

## 验证

109 项普通 Rust、78 项实际 GPU、8 组 Node 测试；51 个案例在同一 Chromium WebGPU 主会话验证，另有原 DPR=2 上下文；51 个原生案例及 Points 另 5 个选项共 56 个配置，各运行 90 帧，要求 `errors 0`。

GPU 对每个原图形检查真实可见像素、显隐差异与整数拾取；Points 验证全部菜单，Offline 解码并上传真实本地 JPEG。浏览器核对原参数、全部实例像素/拾取、透明、缩放和窗口变化；对蓝色多边形分别探测外环、孔洞、岛屿和岛中孔洞。Points 除逐项像素/拾取外，还测量 150 m 与 15000 km 的实际点面积和透明能量差。

Offline 浏览器验证期间阻断公网，要求本地瓦片请求、丰富的实际地球颜色及滚轮响应；专项实测 34 个本地瓦片请求、20323 种颜色、0 个新增外部请求。原生 Offline 实测 33 张瓦片 ready、90 帧 errors 0。最终报告记录实际数值。其余案例仍使用确定性中性影像，方便稳定核对几何。真实 ion/Bing 服务不在本批新增范围。

证据见 `full-recipes-validation.json`、`full-recipes-*-validation.log`、`full-recipes-browser.json`、每个 `full-<id>-preview.png` / `full-<id>-smoke.log`，以及 Points 额外 5 份 variant smoke 日志。

## 边界

验收针对六份固定源码的完整案例行为，不声明全 Cesium API、任意地形贴合或逐像素一致。PolygonSurface 限深度 8、1024 个源顶点、每层最多 32 个孔，纬度 ±85°、高度 ±1000 km；共面挤出与地表模式下的自定义 ST 尚不支持。共面原顶点保持原 ECEF；地理面沿用 10 m 填充 / 10.5 m 轮廓深度偏移。三角化、曲面网格密度、光照与官方渲染可能不同。

Points 原菜单及 Offline 地球的功能在浏览器完整验证；原生按配置/索引加载，不声称提供完整 Cesium Viewer 控件。前批 HTML/DPR 控件仍在浏览器验证，相关原生 smoke 仅检查共享内核。测试采用显式 Vulkan / SwiftShader；默认浏览器设备丢失及完整恢复仍待完成。
