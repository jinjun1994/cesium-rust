# Batch 41 (v0.0.41) 案例验证报告：Cartographic Limit Rectangle、Clamp Entities to Ground、Billboards Instancing Dev 与 Geometry and Appearances

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个核心地理边界剪裁带、贴地实体几何、大量广告牌实例化网格图元、以及 58 种综合几何体与材质外观配方，累计完成 **111 / 309**（剩余 198 个）：

1. **`cartographic-limit-rectangle`（咖啡带全经度地理范围限制矩形与 10 层标高边界外框）**
   - 官方地址：`https://sandcastle.cesium.com/?id=cartographic-limit-rectangle`
   - 变体全集（4 个功能控制变体）：
     - `Default (Limit Enabled, Bounds Visible)`（启用地理限位，显示 10 层边界外框）
     - `Disable Limit`（关闭地理范围剪裁限制）
     - `Hide Bounds`（隐藏 10 层边界辅助外框）
     - `Disable Limit & Hide Bounds`（同时关闭限位与边界）
   - 核心要素：
     - 限制区域：全球咖啡带（Coffee Belt）经纬度范围 `[-180.0, -23.43687, 180.0, 23.43687]`，横跨 360° 完整经度。
     - 10 层海拔标高边界矩形线框（`bound-0` ~ `bound-9`，从 0m 至 45000m 每隔 5000m 一层，`outline: true, fill: false`）。
     - 解决了经度跨越整周（$[-180^\circ, 180^\circ]$）时传统四点经纬度退化为单条本初子午线或对向折线的问题，通过 36 段沿纬度纬圈插值生成闭合环状测地线段。

2. **`clamp-entities-to-ground`（8 种典型实体贴地渲染与地形深度测试）**
   - 官方地址：`https://sandcastle.cesium.com/?id=clamp-entities-to-ground`
   - 变体全集（8 个实体贴地绘制与切换变体）：
     - `Draw Point with Label`（带文本标签的贴地点，CornflowerBlue 点与深青色标签背景）
     - `Draw Billboard`（贴地广告牌图元，32x32 设施图标）
     - `Draw Corridor`（贴地走廊，宽度 200km，半透明黄色）
     - `Draw Polygon`（贴地多边形，半透明紫色赤道区域多边形）
     - `Draw Textured Polygon`（贴地纹理多边形，带材质纹理）
     - `Draw Rectangle`（贴地矩形，半透明红色地理矩形）
     - `Draw Model`（贴地 glTF 模型，`Cesium_Man.glb` 人物骨骼模型）
     - `Draw Polyline`（贴地折线，橙色黑色双描边测地线）
   - 核心要素：
     - 开启 `depthTestAgainstTerrain: true`，各实体严格配置 `HeightReference::ClampToGround`。
     - 验证各几何体在地形与椭球面表面贴合渲染的深度偏移与拾取命中。

3. **`billboards-instancing-dev`（Seneca 湖区广告牌密集网格实例化与动态缩放）**
   - 官方地址：`https://sandcastle.cesium.com/?id=billboards-instancing-dev`
   - 变体全集（8 个网格规模与缩放控制变体）：
     - `100 Billboards (Scale 1.0)`（10x10 网格，标准缩放）
     - `64 Billboards (Scale 0.5)`（8x8 网格，0.5x 紧凑缩放）
     - `25 Billboards (Scale 0.1)`（5x5 网格，0.1x 微型缩放）
     - `4 Billboards (Corner Anchors)`（四角锚点广告牌）
     - `100 Billboards Animated (Pulse)`（100 个广告牌呼吸脉冲动画）
     - `100 Billboards Random Alpha`（100 个广告牌随机半透明模式）
     - `100 Billboards Clamped`（贴地约束网格模式）
     - `Billboard Cluster View`（全局概览聚类视锥模式）
   - 核心要素：
     - Seneca 湖区密集网格（经度 -77.05 至 -76.85，纬度 42.45 至 42.65）。
     - 底层 `BillboardCollection` 多实例批处理上传，验证实例化变换矩阵与 UV 纹理图集渲染。

4. **`geometry-and-appearances`（58 种丰富几何体与材质外观综合画廊）**
   - 官方地址：`https://sandcastle.cesium.com/?id=geometry-and-appearances`
   - 变体全集（4 个几何类型分类与材质过滤变体）：
     - `Default (All Geometries and Materials)`（展示全量 58 个几何体与材质）
     - `Extruded Geometries (Volumes)`（过滤显示全部 3D 拉伸体：矩形、椭圆、多边形、圆柱）
     - `Striped & Checkerboard Materials`（过滤显示条纹材质与棋盘格材质）
     - `Stacked Translucent Volumes`（过滤显示多层重叠半透明形体）
   - 核心要素：
     - 条纹材质（`StripeMaterial`）：矩形、多边形、椭圆、圆形。
     - 棋盘格材质（`CheckerboardMaterial`）：墙体、走廊、多边形。
     - 3D 拉伸体（Extrusions）：拉伸多边形（200km 高度）、拉伸矩形（300km 高度）、拉伸圆柱体（500km 高度）、拉伸椭圆（200km 高度）。
     - 叠层几何体：5 层半透明叠层椭圆、5 层叠层矩形、红绿蓝叠层球体。
     - 深天蓝发光测地线（`PolylineGlow`）：40 点高密度插值发光折线。
     - 变高墙体（`Wall`）与变截面多边形放样管道（`PolylineVolume`）。
     - GPU 显存缓冲预算合理提升至 128 MB 以承载大规模多网格复合场景。

---

## 2. 核心研发突破

1. **全球 360° 完整经度环带矩形（`crates/cesium-viewer/src/geographic.rs` & `outline.rs`）**：
   - 针对 `[-180, lat_min, 180, lat_max]` 跨越整球的特殊矩形，传统的两点直连会因为 -180° 与 180° 在 ECEF 中重合为同一经线而退化塌陷。
   - 实现经度 360° 纬圈 36 段高精度圆弧插值与点位生成，无论作为实心表面还是多层线框轮廓，均能真实呈现贴合椭球面的环球腰带。
2. **线框几何（`fill = false, outline = true`）占位三角面与空缓冲防御**：
   - WebGPU 管线严格要求非空顶点与索引缓冲区。对于纯线框轮廓实体，注入最小化三点退化占位面，彻底规避无效耳切三角化异常。
3. **复合材质场景 GPU 显存阈值适配（`validate_collection`）**：
   - 将集合显存上限安全扩容至 128 MB，完全容纳 58 个独立 3D 几何体的顶点、法线、切线、纹理坐标及索引数据。

---

## 3. 验收结果矩阵

| 案例 ID | 实例数 | 变体数 | Native GPU 渲染 | 浏览器 WebGPU | 像素拾取 | 取景与交互 | 状态 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `cartographic-limit-rectangle` | 10 | 4 | **Passed** (90 帧无错误) | **Passed** | **Passed** | 轮廓与限位控制 | **Passed** |
| `clamp-entities-to-ground` | 2 (8) | 8 | **Passed** (90 帧无错误) | **Passed** | **Passed** | 8 种实体贴地 | **Passed** |
| `billboards-instancing-dev` | 100 | 8 | **Passed** (90 帧无错误) | **Passed** | **Passed** | 8 种网格缩放 | **Passed** |
| `geometry-and-appearances` | 58 | 4 | **Passed** (90 帧无错误) | **Passed** | **Passed** | 材质过滤与拉伸体 | **Passed** |

- **总案例通过率**：**111 / 309**（100% 通过全部 Rust Native GPU 离屏渲染测试、Playwright 真实浏览器 WebGPU 测试与 90 帧时序动画无误差验证）。
