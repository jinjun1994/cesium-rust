# 四个新增完整案例（0.0.32）：高级折线图元、视锥几何、经纬网格与自定义多边形纹理坐标

固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。本批完整实现并验收 **Polylines - Dev**、**Frustum - Dev**、**Parallels and Meridians** 与 **Polygon Texture Coordinates - Dev** 四个完整源码案例，覆盖 160 个原始实例（7 + 2 + 144 + 7）。累计完整功能案例通过数 **75 / 309**，剩余 234 个待完整验收。

| 案例 | 保留的完整案例行为 |
| --- | --- |
| Polylines - Dev | 7 条独立宽折线几何图元（PolylineCollection 原语）：包含 PolylineColorAppearance 单色、PolylineOutline 描边折线、PolylineGlow 辉光渐变、PolylineArrow 方向箭头、Fade 渐隐材质；支持 GEODESIC 大圆弧度、RHUMB 等角航线插值与 loop: true 首尾闭合；支持局部 ENU 站心参考系 `modelMatrix`（向东延伸 1,000 公里箭头） |
| Frustum - Dev | 透视视锥实体与轮廓（FrustumGeometry 与 FrustumOutlineGeometry）：由 `PerspectiveFrustum`（fov = 60°, aspectRatio = 1.5, near = 10m, far = 50m）构建，在 WGS-84 地球表面位置 `[-105.0, 45.0, 20.0]` 通过 ENU 本地旋转四元数定向摆放；包含红色半透明填充面与黑色轮廓骨架（12 条棱边），支持精确包围球自动对焦 |
| Parallels and Meridians | 全球经纬网格系统（144 个图元与实体）：包含赤道与本初子午线等角折线（Rhumb Lines）、支持 10°、1°、0.1° 等多级递归细分经纬网格（makeGrid）；支持极点 ±90° 完整子午线贯通；支持赤道/子午线/经纬网/对跖点工具栏切换及屏幕交互点击十字准星高程坐标标注 |
| Polygon Texture Coordinates - Dev | 6 个定制贴图纹理坐标的多边形实例（PolygonGeometry）及 1 个 GroundPrimitive 贴图多边形：支持外环与多层嵌套内孔（nested holes）1:1 对应的显式 `textureCoordinates` UV 层级树；支持零高程共面多边形、perPositionHeight 绝对高程多边形、拉伸多边形与 RHUMB 航线多边形；映射 Cesium 官方彩色标识纹理 |

## 核心实现技术

1. **Fade 折线渐变材质与 WGSL 片元着色管线**：
   - 在 `crates/cesium-renderer/src/shaders/mesh.wgsl` 中实现 LineMaterial `11u`（Fade 渐变）：根据归一化折线纹理坐标 \( u \in [0, 1] \) 对输入颜色与淡出目标色进行线性插值 `mix(color, material.secondary, clamp(uv.x, 0.0, 1.0))`。
   - 在 `crates/cesium-viewer/src/wide_polyline.rs` 中增加 `LineMaterial::Fade` 枚举项，严格校验 `fade_in_color` 与 `fade_out_color` 浮点 RGBA 合法性，输出对应着色器材质属性。

2. **视锥几何（Frustum Geometry）与轮廓线框解析**：
   - 在 `entities.rs` 中新增 `Shape::Frustum { fov, aspect_ratio, near, far }`，基于透视截头锥体公式推导近截面与远截面的 8 个视图空间局部角点，构建 6 个表面（12 个三角面片）及其外向法向量。
   - 在 `outline.rs` 中对应生成 12 条线框边（近截面 4 条、远截面 4 条、连接 4 射线）。
   - 在 `extended_instances.rs` 中解析 `FrustumGeometry` 与 `FrustumOutlineGeometry`，将本地四元数旋转矩阵通过 `model()` 姿态变换至目标 ECEF 世界坐标系；对于 outline 实例自动隔离 `surfaceAppearance` 以满足严格实体验证契约。

3. **全球极点经纬度解算与等角航线容差**：
   - 在 `wide_polyline.rs` 与 `geographic.rs` 中将纬度硬编码限制由 85° 扩展为 90.0001°，支持南北极点（-90° 与 +90°）全子午线贯通。
   - 优化 `rhumb` 等角航线计算，对极点正切发散进行有效边界钳制，并在段采样时隔离大地线 Vincenty 算法调用，避免在极点等角折线插值时发生极点奇异退化。

4. **实体包围盒退化自适应与全集合自动取景**：
   - 针对全球对称经纬网（中心点落于地心 `[0, 0, 0]`、范数小于 1.0 的情况），在 `entity_framing.rs` 中引入地心零点安全规避策略，将视角中心回退至参考椭球面 `(0, 0, 0)`（Null Island），并根据全球包围球半径自适应配置全局轨道观察视角，消除了零向量无法转换为大地经纬度的数学退化。

5. **显式多边形纹理坐标（Authored ST Texture Coordinates）**：
   - 完善 `polygon_surface.rs`：当多边形配置有定制 `textureCoordinates` 属性时，外边框环路保留与作者输入完全对应的顶点拓扑结构，避免测地线细分导致顶点-UV 数量失配。
   - 在 `extended_instances.rs` 中解析多边形材质外观与 `POSITION_NORMAL_ST` 顶点格式，与内嵌内孔层级统一传递至底层几何管线。

## 验收证据

1. **Rust 全工作区单元测试**：
   - `cargo test -p cesium-viewer --lib` 96 项单元测试全部通过（93 passed, 0 failed, 3 ignored）。
2. **真实 GPU 离屏渲染测试**：
   - `cargo test -p cesium-viewer --test full-recipes-gpu -- --ignored` 76 个测试全部通过（包含 `complete_polylines_dev`、`complete_frustum_dev`、`complete_parallels_and_meridians` 与 `complete_polygon_texture_coordinates_dev`），离屏耗时 21.16 秒。
3. **真实浏览器 WebGPU 端到端测试**：
   - 在原生 Chromium 环境下执行 `scripts/full-recipes-browser.cjs`，所有 75 个完整案例通过实体像素颜色反读、ID 拾取、显隐交互与窗口自适应验证。
4. **Node 契约校验套件**：
   - `npm test` 8 组契约测试全部通过，全部 75 个固定案例哈希与实体结构精准契合。
