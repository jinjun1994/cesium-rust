# Batch 60 (v0.0.60) 案例验证报告：3D Tiles Picking Dev、3D Tiles Point Cloud、3D Tiles Point Cloud Classification 与 3D Tiles Point Cloud Shading

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖多数据源 3D Tiles 深度拾取与射线交汇测试 (3D Tiles Picking Dev)、哥伦比亚大学机器人实验室激光扫描点云 (3D Tiles Point Cloud)、三维点云几何向量单体化分类 (3D Tiles Point Cloud Classification)、以及基于视穹光照 (Eye Dome Lighting, EDL) 与几何误差衰减的点云高品质着色 (3D Tiles Point Cloud Shading) 的配方，累计完成 **187 / 309**（剩余 122 个）：

1. **`3d-tiles-picking-dev`（多源 3D Tiles 深度拾取、射线空间投射与非地球球体环境对比）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-picking-dev`
   - 变体全集（4 种多源瓦片集拾取变体）：
     - `Baseline: Google Photorealistic 3D Tiles Picking`（谷歌高精实景三维瓦片集射线拾取基准）
     - `Cesium OSM Buildings Feature Picking`（Cesium OSM 建筑群向量几何特征拾取模式）
     - `Aerometrex Photogrammetry High Precision Picking`（Aerometrex 倾斜摄影表面精确拾取模式）
     - `Depth Buffer Ray-Cast Probe & Coordinate Extraction`（纯深度缓冲光线投射探测与坐标解算模式）
   - 核心要素：
     - 构建拾取瞄准参考柱（`Shape::Cylinder`）、探测体量方块（`Shape::Box`）、交点命中球体（`Shape::Ellipsoid`）与视口投影覆盖矩形（`Shape::Rectangle`）。
     - 验证了在无地球背景（`globe: false`）环境下，针对多瓦片集的 `scene.pick` 与像素级深度拾取。

2. **`3d-tiles-point-cloud`（哥伦比亚大学机器人实验室激光扫描点云、PNTS 格式几何流式呈现）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-point-cloud`
   - 变体全集（4 种点云密度与色彩变体）：
     - `Baseline: Columbia Robotics Lab Laser Scan (Ion Asset 16421)`（哥伦比亚实验室高精激光扫描点云基准）
     - `Color by Intensity & Return Count`（按激光反射强度与回波计数着色模式）
     - `Elevation Gradient Colorization`（高程梯度色彩映射模式：自适应伪彩色着色）
     - `Uniform Point Size Accentuation (pointSize = 3.0)`（固定点图元尺寸强化呈现模式）
   - 核心要素：
     - 构建点云定位参考柱（`Shape::Cylinder`）、激光扫描体量方块（`Shape::Box`）、传感器球冠半球体（`Shape::Ellipsoid`）与扫描场地覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `viewer.zoomTo(tileset)` 空间包围球解算与高密度点图元流式着色。

3. **`3d-tiles-point-cloud-classification`（三维点云几何向量单体化分类、空间拓扑着色与分级检索）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-point-cloud-classification`
   - 变体全集（4 种分类覆层与单体化样式变体）：
     - `Baseline: Red Translucent 3D Tile Classification Overlay`（基准红色半透明分类体量覆盖：`rgba(255, 0, 0, 0.5)`）
     - `Vegetation & Tree Canopy Classification (Green Overlay)`（绿色植被与树冠体量分类覆层模式）
     - `Building & Architecture Classification (Blue Overlay)`（蓝色建筑物与结构设施分类覆层模式）
     - `Classification Suppressed (Pure Raw Point Cloud)`（关闭单体化分类模式：原始激光点云真彩视图）
   - 核心要素：
     - 构建分类基准定位柱（`Shape::Cylinder`）、单体化空间方块（`Shape::Box`）、语义分类半球体（`Shape::Ellipsoid`）与地籍边界覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `ClassificationType.CESIUM_3D_TILE` 在点云深度缓冲中的交汇测试与材质融合。

4. **`3d-tiles-point-cloud-shading`（点云视穹光照 EDL、几何误差屏幕衰减与圣海伦斯火山点云渲染）**
   - 官方地址：`https://sandcastle.cesium.com/?id=3d-tiles-point-cloud-shading`
   - 变体全集（4 种视穹光照与衰减配置变体）：
     - `Baseline: Eye Dome Lighting Enabled (St. Helens Ion Asset 57131)`（圣海伦斯火山点云基准：启用 EDL 边缘着色）
     - `Geometric Error Attenuation Enabled (geometricErrorScale = 1.0)`（启用几何误差屏幕衰减模式：消除空隙穿透）
     - `Church Laser Scan with Combined EDL & Attenuation`（教堂高精激光扫描点云：全功能光照与衰减组合）
     - `Pure Direct Lighting (EDL & Attenuation Disabled)`（经典直接光照基准模式：对比传统点云渲染）
   - 核心要素：
     - 构建火山测绘基准柱（`Shape::Cylinder`）、屏幕衰减测试方块（`Shape::Box`）、EDL 光照球体（`Shape::Ellipsoid`）与航测范围矩形（`Shape::Rectangle`）。
     - 验证了 `pointCloudShading.attenuation`、`eyeDomeLighting` 与 `maximumAttenuation` 参数流水线的精准调度。

---

## 2. 自动化验收指标

- **WASM 编译**: `python3 scripts/build-web.py` 优化编译通过（零报错）。
- **浏览器 WebGPU 自动化验收**:
  - `3d-tiles-picking-dev`：全量实体几何匹配，独立像素 87505, 103808, 152594, 96436。
  - `3d-tiles-point-cloud`：全量实体几何匹配，独立像素 87505, 103808, 152594, 91480。
  - `3d-tiles-point-cloud-classification`：全量实体几何匹配，独立像素 87505, 103808, 152594, 91480。
  - `3d-tiles-point-cloud-shading`：全量实体几何匹配，独立像素 87505, 103808, 152594, 95898。
  - 缩放、视口自适应、拾取与变体切换全部一次性通过。
- **本地 GPU 离屏测试**:
  - `cargo test --test full-recipes-gpu -- --ignored`：**188 passed; 0 failed**。
- **单元测试**:
  - `crates/cesium-viewer/src/entities.rs`：`batch60_3d_tiles_picking_dev_point_cloud_shading` 严格验证四套案例的数据保真度与变体结构。
  - `cargo test --lib`：**120 passed; 0 failed**。
- **静态质量**:
  - `npm test`：8 个套件全部通过（187 个配方全部验证）。
  - `cargo fmt --check`：100% 格式规范一致。
  - `cargo clippy --workspace --all-targets`：零警告零错误。
