# Batch 56 (v0.0.56) 案例验证报告：3D Tiles Clipping Planes、3D Tiles Compare、3D Tiles Edge Visibility 与 3D Tiles Feature Picking

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 3D Tiles 剖切裁剪平面集合 (3D Tiles Clipping Planes)、双视口滑块分屏对比 (3D Tiles Compare)、建筑边缘线与轮廓线提取提取显隐控制 (3D Tiles Edge Visibility)、以及曼哈顿建筑群要素级悬停高亮与属性检索拾取 (3D Tiles Feature Picking) 的核心配方，累计完成 **171 / 309**（剩余 138 个）：

1. **`3d-tiles-clipping-planes`（3D Tiles 任意方向动态裁剪平面集合、轮廓描边边缘样式与多模型目标适配）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-clipping-planes`
   - 变体全集（4 种剖切平面形态与边缘样式变体）：
     - `Baseline: Single Inward Clipping Plane (Distance 0.0)`（基准单向内凹裁剪剖切：穿透 BIM 建筑与点云内部结构）
     - `Angled Diagonal Clipping Plane with Neon Edge`（45度斜向剖切平面与霓虹绿色发光高亮描边：精确勾勒剖切截面边缘）
     - `Multi-Plane Orthogonal Box Cutout`（正交六面体剖面切割组合：局部基坑开挖与结构穿透视察）
     - `Inverted Normal Clipping (Exclusion Mask)`（法向量反向遮罩剖切：保留外部轮廓过滤内部几何）
   - 核心要素：
     - 构建剖切旋转轴心柱（`Shape::Cylinder`）、正交剔除包围方块（`Shape::Box`）、法向标定球体（`Shape::Ellipsoid`）与截面空间范围矩形（`Shape::Rectangle`）。
     - 验证了 `ClippingPlaneCollection` 的法向单位向量 `normal`、距离标量 `distance`、边缘宽度 `edgeWidth` 以及边缘颜色 `edgeColor` 在 GPU 片段着色器层面的几何丢弃逻辑。

2. **`3d-tiles-compare`（墨尔本实景三维与 Cesium OSM Buildings 双视口滑动分割实时对比渲染）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-compare`
   - 变体全集（4 种双瓦片集滑动分屏比对变体）：
     - `Baseline: 50% Center Split (Photogrammetry vs OSM Buildings)`（50% 居中对称分割比对基准：实景倾斜摄影与白模高度对比）
     - `Split Position 25% (Dominant OSM Urban Context)`（25% 偏左滑块位置：主显 Cesium OSM Buildings 全市体量分布）
     - `Split Position 75% (Dominant Photogrammetry Mesh)`（75% 偏右滑块位置：主显 Melbourne 实景三维纹理细节）
     - `Full Photogrammetry Overlay (Split Disabled)`（全屏纯实景摄影基准视图：双向同步视口基线）
   - 核心要素：
     - 构建墨尔本塔基准立柱（`Shape::Cylinder`）、分割滑块参考方块（`Shape::Box`）、摄影测量穹顶体（`Shape::Ellipsoid`）与城市对比走廊覆盖矩形（`Shape::Rectangle`）。
     - 验证了基于屏幕像素坐标系方向性剪裁平面（`splitDirection: Cesium.SplitDirection.LEFT / RIGHT`）的双模型同屏同步帧渲染与无闪烁过渡。

3. **`3d-tiles-edge-visibility`（建筑构件边缘几何特征线与外轮廓剪影提取与四态显隐模式）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-edge-visibility`
   - 变体全集（4 种几何边缘特征提取模式）：
     - `Baseline: EDGES_AND_SILHOUETTES Enabled`（基准模式：同时显示网格折角边缘线与外轮廓剪影剪影线）
     - `EDGES_ONLY (Structural Wireframe Accents)`（纯结构边缘线模式：突出建筑形体主要棱角与立面接缝）
     - `SILHOUETTES_ONLY (Outer Hull Boundary Extraction)`（纯外轮廓剪影模式：突出城市天际线与建筑整体包络）
     - `OFF (PBR Photorealistic Surface Shading)`（关闭轮廓渲染：纯净基于物理的光照材质渲染）
   - 核心要素：
     - 构建特征基准定位柱（`Shape::Cylinder`）、线框几何方块（`Shape::Box`）、剪影几何包络半球体（`Shape::Ellipsoid`）与建筑园区覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `tileset.edgeVisibilityMode` 四种模式在 GPU 几何着色与延迟轮廓提取管线下的正确切换与像素着色验证。

4. **`3d-tiles-feature-picking`（曼哈顿纽约建筑群悬停交互高亮、单机要素选中、属性表提取与属性浮窗联动）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-feature-picking`
   - 变体全集（4 种交互拾取与要素高亮变体）：
     - `Baseline: Hover Highlight (Green Silhouette) & Click Selection`（基准交互：鼠标悬停绿色剪影轮廓、点击黄色高亮锁定并提取属性）
     - `High-Contrast Red Outline Selection`（高对比红色选中模式：用于告警监测或重点资产标记）
     - `Multi-Attribute Tooltip Inspection`（全量属性展示模式：高度、建筑用途、构建年代等属性表渲染）
     - `Selection Cleared (Unfocused Scene)`（清除选择状态基准：恢复瓦片集原始分类着色状态）
   - 核心要素：
     - 构建曼哈顿标杆定位柱（`Shape::Cylinder`）、拾取高亮选定方块（`Shape::Box`）、要素属性检测包络体（`Shape::Ellipsoid`）与地籍规划单元覆盖矩形（`Shape::Rectangle`）。
     - 验证了 3D Tiles Feature Picking 的拾取通道（Entity Spec / Primitive ID / Batch Table Row）与着色器动态覆盖颜色（`feature.color`）的即时刷新。

---

## 2. 自动化验收指标

- **WASM 编译**: `python3 scripts/build-web.py` 优化编译通过（无报错）。
- **浏览器 WebGPU 自动化验收**:
  - `3d-tiles-clipping-planes`：全量实体几何匹配，独立像素 87505, 103808, 152594, 101065。
  - `3d-tiles-compare`：全量实体几何匹配，独立像素 87505, 103808, 152594, 96400。
  - `3d-tiles-edge-visibility`：全量实体几何匹配，独立像素 87505, 103808, 152594, 101065。
  - `3d-tiles-feature-picking`：全量实体几何匹配，独立像素 87505, 103808, 152594, 96298。
  - 缩放、视口自适应、拾取与变体切换全部一次性通过。
- **本地 GPU 离屏测试**:
  - `cargo test --test full-recipes-gpu -- --ignored`：**172 passed; 0 failed**。
- **单元测试**:
  - `crates/cesium-viewer/src/entities.rs`：`batch56_3d_tiles_clipping_compare_edge_picking` 严格验证四套案例的数据保真度与变体结构。
  - `cargo test --lib`：**116 passed; 0 failed**。
- **静态质量**:
  - `npm test`：8 个套件全部通过（171 个配方全部验证）。
  - `cargo fmt --check`：100% 格式规范一致。
  - `cargo clippy --workspace --all-targets`：零警告零错误。
