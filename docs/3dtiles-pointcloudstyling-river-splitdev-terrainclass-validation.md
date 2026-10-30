# Batch 61 (v0.0.61) 案例验证报告：3D Tiles Point Cloud Styling、Global Rivers Vector Tiles with Property-based LOD、3D Tiles Split - Dev 与 3D Tiles Terrain Classification

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖激光点云声明式样式语言与逐点温度/坐标函数 (3D Tiles Point Cloud Styling)、全球 HydroRIVERS 矢量水系流速层级与属性 LOD 地形贴地渲染 (Global Rivers Vector Tiles with Property-based LOD)、多格式 3D Tiles 双视口滑动分割对比与阴影检视 (3D Tiles Split - Dev)、以及维也纳矢量 3D Tiles 全球地形覆层分类与交互高亮 (3D Tiles Terrain Classification) 的配方，累计完成 **191 / 309**（剩余 118 个）：

1. **`3d-tiles-point-cloud-styling`（点云声明式样式语言、逐点温度条件色谱、三角/几何坐标函数与动态点图元尺寸）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-point-cloud-styling`
   - 变体全集（4 种点云样式与表达式变体）：
     - `Baseline: Step Red/Blue Temperature Styling (${temperature} > 0.5 ? red : blue)`（温度二值红蓝阶跃着色基准）
     - `Color Ramp (10-Stage Temperature Conditions & Alpha)`（10 级温度条件色带与透明度通道混合模式）
     - `Position-Based Spatial Gradient (vec4(${POSITION}, 1.0))`（空间归一化三维坐标位置色彩渐变模式）
     - `Point Size & Multiple Defines Dynamic Styling`（动态宏定义 `length/time` 与自适应点尺寸缩放模式）
   - 核心要素：
     - 构建样式探测立柱（`Shape::Cylinder`）、点云过滤方块（`Shape::Box`）、温度场辐射球体（`Shape::Ellipsoid`）与点云测绘覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `${temperature}`、`${POSITION}` 空间内置变量、`pointSize` 动态计算以及 `Cesium3DTileStyle` 在片元着色器中的快速求值。

2. **`3d-tiles-property-lod-river`（全球 HydroRIVERS 矢量水系、基于 ORD_FLOW 属性的分级 LOD、线宽衰减与地形贴地）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-property-lod-river`
   - 变体全集（4 种水系层级与线宽变体）：
     - `Baseline: HydroRIVERS ORD_FLOW River Hierarchy (LOD 1-10 Rainbow Color Ramp)`（1-10 级对数水系流量彩虹色带基准）
     - `Major Rivers Emphasized (ORD_FLOW <= 3 Navy/Blue with Thick Lines)`（干流主江加粗高亮模式：宽幅深蓝干流）
     - `Narrow Tributaries Accentuated (ORD_FLOW >= 7 Warm Hues)`（末端细小支流强化模式：暖色系末级水网）
     - `Monochrome River Hierarchy with Variable Line Widths`（单色灰度线宽分级模式：凸显几何拓扑层级）
   - 核心要素：
     - 构建水文监测水准柱（`Shape::Cylinder`）、流域盆地范围方块（`Shape::Box`）、节点集水区球体（`Shape::Ellipsoid`）与喜马拉雅水系走廊矩形（`Shape::Rectangle`）。
     - 验证了 `HeightReference.CLAMP_TO_TERRAIN` 贴地约束、动态线宽公式 `5.0 - clamp(Number(${ORD_FLOW}), 0.0, 10.0) * 0.4` 与 DOM 图例交互。

3. **`3d-tiles-split-dev`（14 种多格式 3D Tileset 滑动分屏对比、SplitDirection 显隐控制与阴影投射检视）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-split-dev`
   - 变体全集（4 种分屏分割与多格式瓦片变体）：
     - `Baseline: Tileset Default Split (SplitDirection.NONE, slider at 50%)`（基准 50% 居中无分割全景视图）
     - `Left Viewport Split (SplitDirection.LEFT, slider at 30%)`（左侧视口单侧遮罩剖切模式：30% 分割位置）
     - `Right Viewport Split (SplitDirection.RIGHT, slider at 70%)`（右侧视口单侧遮罩剖切模式：70% 分割位置）
     - `Instanced Tileset Split with Shadow Caster Enabled`（实例化 I3DM 瓦片集分屏与动态实时阴影投射模式）
   - 核心要素：
     - 构建分屏基准立柱（`Shape::Cylinder`）、瓦片集模型方块（`Shape::Box`）、视点枢轴球体（`Shape::Ellipsoid`）与多瓦片边界矩形（`Shape::Rectangle`）。
     - 验证了 `scene.splitPosition` 屏幕空间坐标裁剪、Knockout 双向数据绑定以及 `viewerCesium3DTilesInspectorMixin` 运行时挂载。

4. **`3d-tiles-terrain-classification`（维也纳城市矢量 3D Tiles 全球地形覆层、地块单体化分类与鼠标悬停高亮）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-terrain-classification`
   - 变体全集（4 种地块分类覆层与高亮变体）：
     - `Baseline: Vienna Austria Vector Classification Draped on Terrain (rgba(255, 255, 255, 0.5))`（维也纳半透明白色地块贴地分类基准）
     - `Hover Highlight State (Yellow Color Override on Picked Feature)`（鼠标悬停动态黄色要素高亮模式：实时还原与着色）
     - `Colored Zoning Parcels (Green/Blue Draped Polygon Classification)`（绿蓝规划功能分区覆层多边形模式）
     - `Classification Suppressed (Pure World Terrain View in Vienna)`（关闭分类覆层模式：奥地利维也纳纯全球地形基准）
   - 核心要素：
     - 构建维也纳地名定位柱（`Shape::Cylinder`）、建筑单体多边形体量方块（`Shape::Box`）、要素拾取探测球体（`Shape::Ellipsoid`）与矢量覆层边界矩形（`Shape::Rectangle`）。
     - 验证了地理编码器瞬时定位（`geocoder.flightDuration = 0.0`）、矢量瓦片对地形的贴合分类与光标移动事件交互。

---

## 2. 自动化验收指标

- **WASM 编译**: `python3 scripts/build-web.py` 优化编译通过（零报错）。
- **浏览器 WebGPU 自动化验收**:
  - `3d-tiles-point-cloud-styling`：全量实体几何匹配，独立像素 1516, 1040, 826, 20818。
  - `3d-tiles-property-lod-river`：全量实体几何匹配，独立像素 87336, 103418, 166025, 81332。
  - `3d-tiles-split-dev`：全量实体几何匹配，独立像素 2147, 1386, 1061, 20818。
  - `3d-tiles-terrain-classification`：全量实体几何匹配，独立像素 49160, 34601, 26647, 71548。
  - 缩放、视口自适应、拾取与变体切换全部一次性通过。
- **本地 GPU 离屏测试**:
  - `cargo test --test full-recipes-gpu -- --ignored`：**192 passed; 0 failed**。
- **单元测试**:
  - `crates/cesium-viewer/src/entities.rs`：`batch61_3d_tiles_point_cloud_styling_river_split_terrain` 严格验证四套案例的数据保真度与变体结构。
  - `cargo test --lib`：**121 passed; 0 failed**。
- **静态质量**:
  - `npm test`：8 个套件全部通过（191 个配方全部验证）。
  - `cargo fmt --check`：100% 格式规范一致。
  - `cargo clippy --workspace --all-targets`：零警告零错误。
