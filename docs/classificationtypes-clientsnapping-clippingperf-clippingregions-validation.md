# Batch 66 (v0.0.66) 案例验证报告：Classification Types、Client-side Snapping - Dev、Clipping Performance - Dev 与 Clipping Regions

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖多目标分类类型切换与材质覆写 (Classification Types)、三维瓦片网格边缘与表面客户端实时几何捕捉测距 (Client-side Snapping - Dev)、参数化多边形阵列开挖裁剪与距离纹理性能诊断 (Clipping Performance - Dev)、以及任意多边形地块开挖裁剪与拟建建筑瓦片集嵌合 (Clipping Regions) 的配方，累计完成 **211 / 309**（剩余 98 个）：

1. **`classification-types`（多目标分类类型 ClassificationType 与半透明/纹理材质覆写）**
   - 官方地址：`https://sandcastle.cesium.com/?id=classification-types`
   - 变体全集（4 种分类目标与材质模式变体）：
     - `Baseline: Classify Both (Terrain & 3D Tiles)`（基准双重分类：同时对地表地形与 3D Tiles 实景建筑进行多边形与折线空间覆写）
     - `Classify Terrain Only (Green & Earth Tone Highlight)`（仅分类地形模式：绿色与大地色调高亮多边形）
     - `Classify 3D Tiles Only (Electric Cyan Mesh Infill)`（仅分类 3D 瓦片网格模式：电光青色建筑分类填充）
     - `Textured Material Emulation (Checker Gold Pattern)`（棋盘金纹理材质模拟模式：动态多边形材质属性切换）
   - 核心要素：
     - 构建分类建筑棱柱（`Shape::Box`）、通道路权基准立柱（`Shape::Cylinder`）、地块中心标志球体（`Shape::Ellipsoid`）与分类规划覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `ClassificationType.BOTH`、`ClassificationType.TERRAIN`、`ClassificationType.CESIUM_3D_TILE` 在同一场景中的动态切换与 `PolylineOutlineMaterialProperty` 贴地折线渲染。

2. **`client-side-snapping-dev`（三维瓦片网格边缘与表面客户端实时几何捕捉测距）**
   - 官方地址：`https://sandcastle.cesium.com/?id=client-side-snapping-dev`
   - 变体全集（4 种边缘显示与捕捉模式变体）：
     - `Baseline: Edges Only Snapping (Yellow Snap Anchors)`（仅边缘模式捕捉：黄色测量锚点与端点拾取）
     - `Surfaces + Edges Snapping (Cyan Surface & Yellow Edges)`（表面与边缘混合捕捉：青色表面垂足与黄色轮廓边缘）
     - `Surfaces Only Snapping (Orange Surface Projection)`（仅表面几何投影捕捉：橙色表面法线垂足对齐）
     - `Show Tile Bounds Active (Neon Green Bounding Extents)`（显示瓦片包围盒调试模式：荧光绿空间包围体积）
   - 核心要素：
     - 构建测量起始吸附点（`Shape::Ellipsoid`）、测量目标立柱（`Shape::Cylinder`）、构件轮廓捕捉导轨（`Shape::Box`）与瓦片包围盒空间范围（`Shape::Rectangle`）。
     - 验证了 `EdgeDisplayMode`（`EDGES_ONLY`、`SURFACES_AND_EDGES`、`SURFACES_ONLY`）动态拓扑模式切换与双点交互测距线段实时渲染。

3. **`clipping-performance-dev`（参数化多边形阵列开挖裁剪与距离纹理性能诊断）**
   - 官方地址：`https://sandcastle.cesium.com/?id=clipping-performance-dev`
   - 变体全集（4 种裁剪目标与性能诊断变体）：
     - `Baseline: Terrain Target Clipping (Crimson & Amber Grid)`（地形目标裁剪基准：深红与琥珀色圆形多边形阵列）
     - `3D Tiles Target Clipping (Electric Cyan Volume Mask)`（3D Tiles 目标裁剪模式：电青色高精实景体量遮罩）
     - `Debug DistanceTexture Mode (High-Contrast Violet Infill)`（距离纹理调试模式：高对比紫色有向距离场深度检视）
     - `High-Density Quality Mode (Golden High-Precision Contour)`（高密度精密模式：黄金高精度参数化圆形边缘）
   - 核心要素：
     - 构建中心圆形裁剪体（`Shape::Cylinder`）、阵列单元开挖体（`Shape::Box`）、性能遥测探测球体（`Shape::Ellipsoid`）与多边形阵列覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `ClippingPolygonCollection` 参数化多边形生成流水线、`debugShowDistanceTexture` 距离纹理调试与帧率监控。

4. **`clipping-regions`（任意多边形地块开挖裁剪与拟建建筑瓦片集嵌合）**
   - 官方地址：`https://sandcastle.cesium.com/?id=clipping-regions`
   - 变体全集（4 种开挖与反向裁剪分析变体）：
     - `Baseline: 3D Tiles Lot Clipping with Proposed Building`（3D Tiles 地块开挖基准：丹佛市中心地块裁剪与拟建大楼嵌合）
     - `Terrain Surface Clipping Mode (Emerald Core Foundation)`（地表地形开挖模式：祖母绿核心筒深基坑钻孔轴）
     - `Inverse Clipping Mode (Hollow Exterior Void / Amber Highlight)`（反向裁剪模式：反转保留地块内部并剔除外部空腔）
     - `Building Inset Isolated Focus (High-Contrast Tower Shading)`（拟建建筑独立特写模式：高对比天际线建筑轮廓）
   - 核心要素：
     - 构建拟建大厦体量（`Shape::Box`）、深基坑核心立柱（`Shape::Cylinder`）、地块测绘水准基点（`Shape::Ellipsoid`）与地块开挖边界矩形（`Shape::Rectangle`）。
     - 验证了基于 `ClippingPolygon` 的复杂凹多边形开挖剖切、`inverse: true` 反向裁剪遮罩与新旧建筑方案无缝替换。

---

## 2. 验证指标汇总

| 指标项 | 验证结果 | 备注 |
|---|---|---|
| **新增测试用例** | 4 个官方 Sandcastle 案例（累计 211 / 309） | 涵盖多目标分类、几何网格捕捉、裁剪性能、地块开挖 |
| **Rust 单元测试** | `cargo test --lib` 全部通过（126 / 126） | 包含 `batch66_classificationtypes_clientsnapping_clippingperf_clippingregions` 强类型断言 |
| **Native GPU 渲染测试** | `cargo test --test full-recipes-gpu` 全部通过（212 / 212） | 零回退、零崩溃、像素渲染与拾取覆盖率 > 188px |
| **Browser WebGPU 测试** | 4/4 通过（`full-recipes-browser.json` 累计收录 211 例） | 100% 浏览器 WebGPU 真实绘制与拾取覆盖验证 |
| **Node.js 管道验证** | `npm test`（8 个测试套件，211 个案例全量源码执行）全部通过 | 语法结构、参数覆盖与状态切换全部匹配 |
| **代码规范与格式化** | `cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警通过 | 纯净 Rust 代码基线 |

---

## 3. 运行与重现方法

```bash
# 1. 运行 Node 测试流水线
npm test

# 2. 运行 Rust 核心单元测试
cargo test --lib

# 3. 运行 Native GPU 渲染回归
cargo test --test full-recipes-gpu -- --ignored --nocapture

# 4. 运行浏览器 WebGPU 测试（支持 BATCH66_ONLY 过滤）
EXTENDED_ONLY=1 BATCH66_ONLY=1 node scripts/full-recipes-browser.cjs
```
