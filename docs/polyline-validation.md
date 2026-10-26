# 六个折线完整案例（0.0.26）

固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。本批实现六个完整源码案例，覆盖 23 个原始折线实例。

| 案例 | 保留的完整案例行为 |
| --- | --- |
| Polyline | 红色贴地测地线、绿色恒向线、带 taper 的蓝色辉光、带黑色轮廓的橙色高空线、NONE 直 ECEF 紫色箭头、青色虚线；全集合自动取景 |
| Polyline - Dev | 两个 PolylineGeometry；红色透明测地线与蓝色直线，像素宽度、原始位置、ColorGeometryInstanceAttribute、PolylineColorAppearance；原默认视角 |
| Polyline Color - Dev | 全部 24 个原始位置和颜色；第一条逐段颜色不混合，第二条逐顶点连续插值；原默认视角 |
| Polyline Material - Dev | Glow、Arrow、Outline 三条折线，默认蓝色 Glow、白色 Arrow、白线红色 Outline；原位置与 10 像素线宽，PolylineMaterialAppearance 的各材质默认值 |
| Polyline Dash | 红色默认虚线、30 像素蓝线配黄色间隙、8 像素周期橙线、两种青色／黄色 16 位掩码；全部原始高度与取景 |
| CZML Polyline | 全部五个原始包；贴地、Glow、Outline、NONE Arrow、Dash；CZML 常量颜色与材质参数、全集合自动取景 |

## 实现与测试 UI

新增 `wide_polyline.rs` 和独立的 GPU 折线三角带上传模式。顶点保存前后相邻位置，由 WGSL 投影后计算屏幕法线与有界 miter，宽度为渲染目标像素，不随相机距离改变。路径支持 WGS84 测地线、等方纬度恒向线、直 ECEF；高度插值、跨日期变更线、逐段色断点、逐顶点色渐变均保留。取景使用实际曲线位置。

辉光／taper、箭头、轮廓和虚线的主要方程对照固定上游原 GLSL；四份参考文件与 SHA-256 见 `tests/sandcastle/batch26-shaders.json`，材质默认值另从固定 `Scene/Material.js` 提取并保留源文件哈希（`reference-shaders/material-defaults.json`）；回归测试分别核对 Material 默认蓝色辉光、默认红色边框与 Property 的显式黑色边框。上游 Apache-2.0 许可见 `tests/sandcastle/reference-license.md`。颜色与整数 ID 拾取共用材质透明覆盖，透明虚线间隙和箭头外部可穿透选取。已有 GeoJSON `polyline` 的米宽带状面接口继续保留，新接口为 `widePolyline`。

`batch26-recipes.mjs` 执行每份完整固定源码，并严格核对所有构造字段。原始 `.primitive.json`／`.czml`、独立预期 `.json` 及来源源码均随包提供。Polyline Color 原源码未设随机种子；为了可重复验收，记录器固定随机流，两种颜色模式和全部 24 次随机调用均保留；不声称颜色与任意一次上游随机运行相同。

启动 `npm start`，访问 `http://127.0.0.1:8000/bindings/web/sandcastle.html`。新增六个“完整”按钮；也可在 309 项目录中搜索名称，点击“运行当前完整案例”。Dev 案例首先保留默认宽视角，可点击“自动取景全部实体”。支持滚轮缩放、拖拽、按实体显隐、点击拾取、编辑全部折线 JSON、切换案例及窗口缩放。

原生可使用 `CESIUM_ENTITIES=tests/sandcastle/full/polyline.json`、`CESIUM_PRIMITIVES=tests/sandcastle/full/polyline-dev.primitive.json` 或 `CESIUM_CZML=tests/sandcastle/full/czml-polyline.czml`；自动取景加 `CESIUM_ZOOM_ENTITIES=1`。

## 边界

这六个源案例默认使用椭球地形，贴地线在该椭球表面生成；本批没有新增动态真实地形折线贴合、3D Tiles 分类贴合、深度失败材质、回调属性或任意 Cesium API。输入每条 2–1024 点、1–256 像素、采样不超过 50,000 点，保留原 32 MiB 集合预算；错误加载不替换已显示集合。折线材质使用单样本透明混合和现有深度排序，不声称与 Cesium 逐像素一致或所有 Viewer 控件等价。默认浏览器后端的既有 DeviceLost 问题仍未解决，自动化使用明确配置的 Vulkan / SwiftShader。

## 验收

证据与计数汇总见 `full-recipes-validation.json`，逐案例浏览器证据见 `full-recipes-browser.json`。普通 Rust 检查六份原始输入与全部预期实体字段、颜色/插值、恒向线与日期变更线、非法宽度/颜色/材质和重复位置。GPU 对每个实例做真实像素差及 ID 拾取；额外测试 18 像素折线在 200 km 与 400 km 观察距离保持宽度，透明虚线间隙不返回 ID、非透明绿色间隙返回 ID，且 0 掩码完全不可选。浏览器覆盖全部原参数、每实例真实像素/拾取、显隐、滚轮、窗口变化与重新取景。桌面输入各运行 90 帧。

最终通过：115 项普通 Rust、85 项实际 GPU（其中 57 个完整案例 + 1 个定量折线测试）、8 组 Node、57 个浏览器案例（最终 WASM SHA-256 绑定）、57 个桌面案例及 5 个额外 Points 配置共 62 个 90 帧检查。累计 **57 / 309**，剩余 **252**。
