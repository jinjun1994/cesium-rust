# 四个新增完整案例（0.0.30）：Rectangle、Z-Indexing 与 Volume Primitives

固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。本批完整实现并验收 **Rectangle**、**Z-Indexing Geometry**、**CZML Z-Index** 与 **Volumes - Dev** 四个完整源码案例，覆盖 16 个原始实例。累计完整功能案例通过数 **67 / 309**，剩余 242 个待完整验收。

| 案例 | 保留的完整案例行为 |
| --- | --- |
| Rectangle | 三个原始矩形实体：半透明红色矩形（Alpha=0.5）、绿色旋转拉伸矩形（height=100000, extrudedHeight=300000, rotation=45°）、带纹理贴图矩形（Image material 引用 `Cesium_Logo_Color.jpg`，同时指定 rotation 与独立的 stRotation 纹理旋转角）；全集合 `viewer.zoomTo` 自动取景 |
| Z-Indexing Geometry | 全部七个重叠几何实例：6 个重叠同高度矩形分布于 zIndex 1、2、3 层级（包含纯色红色/绿色/蓝色与贴图矩形），以及一条地形蓝色 Glow 发光折线（zIndex 2）；验证多层几何同高度重叠时的深度层叠与无 Z-Fighting 渲染 |
| CZML Z-Index | 四个带 zIndex 的 CZML 几何实体：紫色椭圆圆盘（zIndex 3）、黄色发光走廊（zIndex 3）、深蓝多边形（zIndex 1）、条纹矩形（zIndex 2）；验证 CZML 协议中 zIndex 的完整反序列化与场景排布 |
| Volumes - Dev | 两个体图元（EllipsoidPrimitive）实例：局部半径椭球（RimLighting 边缘发光材质）与带模型变换矩阵的条纹椭球（Stripe 材质、modelMatrix/center 本地 ENU 定位）；验证底层 Volume 图元加载与专属着色器参数绑定 |

## 实现与测试 UI

1. **Z-Indexing 多层渲染排序管线**：
   - 在 `Entity`、`GpuMesh` 与各几何包体中引入 `z_index: i32` 属性，严格限制范围并在反序列化中支持默认值为 0。
   - 在渲染器 `Renderer::render()` 的网格排序阶段，加入 `a.z_index.cmp(&b.z_index)` 稳定排序，确保低 zIndex 几何先入渲染管线，高 zIndex 几何在其上方正确遮挡或混合，结合 Reverse-Z 与深度测试消除同面共面 Z-Fighting。

2. **矩形与纹理旋转（stRotation）**：
   - 完善 `Shape::Rectangle` 参数通道，包含 `coordinatesDegrees`、`height`、`extrudedHeight`、`rotation` 与 `st_rotation`。
   - 在 `SurfaceMaterial::apply()` 中支持纹理坐标 `st_rotation` 旋转矩阵运算（围绕 `[0.5, 0.5]` UV 质心中心旋转），使贴图能够独立于几何矩形自身的地面偏航角进行纹理偏转。

3. **CZML Z-Index 与发光材质兼容**：
   - 全面支持在 CZML packet 中解析 `zIndex`，覆盖 `polygon`、`corridor`、`rectangle`、`ellipse` 与 `polyline`。
   - 扩充 `LineMaterial::Glow`，增加可选 `color: Option<[f32; 4]>` 字段，无缝兼容 CesiumJS 原生 `PolylineGlowMaterialProperty` 属性规范。

4. **体图元（EllipsoidPrimitive）支持**：
   - 在扩展图元解析器中增加对 `EllipsoidPrimitive` 的解码支持，提取 `radii`、`center` / `modelMatrix`。
   - 支持 `RimLighting` 边缘光与 `Stripe` 条纹表面材质的生成与参数水合。

5. **测试 UI**：
   - 运行 `python3 scripts/start-test-ui.py`，访问 `http://127.0.0.1:8000/bindings/web/sandcastle.html`。
   - 新增“完整 Rectangle”、“完整 Z-Indexing Geometry”、“完整 CZML Z-Index”与“完整 Volumes - Dev”测试按钮，支持在 309 案例检索框内快速检索与交互。

## 验收证据

1. **源码契约与哈希校验**：
   - 固定源码经执行提取真实结构，4 个案例的 SHA-256 严密绑定至 manifest 索引。
2. **普通 Rust 单元测试**：
   - 全工作区 126 项单元测试通过（包含 `entities::tests::batch30_rectangle_zindex_volumes_all_four_cases`）。
3. **真实 GPU 离屏渲染测试**：
   - `crates/cesium-viewer/tests/full-recipes-gpu.rs` 全部 68 项完整案例测试通过（用时 22.07s，包含 `complete_rectangle`、`complete_z_indexing_geometry`、`complete_czml_zindex` 与 `complete_volumes_dev`）。
4. **桌面 90 帧冒烟测试**：
   - 原生运行 90 帧，无任何 GPU 或渲染错误，日志记录于 `docs/full-rectangle-smoke.log`、`docs/full-z-indexing-geometry-smoke.log`、`docs/full-czml-zindex-smoke.log` 与 `docs/full-volumes-dev-smoke.log`。
5. **浏览器 WebGPU 端到端测试**：
   - 在真实 Chromium 155 环境下运行 `scripts/full-recipes-browser.cjs`，所有 67 个完整案例全部通过画布颜色反读、实体 ID 拾取、显隐切换、滚轮缩放与窗口自适应检查，销毁状态正常（`destroy: 'passed'`）。
6. **Node 测试套件**：
   - 8 组 Node 测试套件全部通过（`npm test`）。
