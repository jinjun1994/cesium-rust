# 地理 Entity 与 CZML（0.0.10）

本批继续推进 Sandcastle 的实体几何主线。新增功能通过共享 Rust Scene 与 wgpu 执行，桌面和 WASM 使用相同网格；未替换为 CesiumJS。

## 测试入口

```bash
python3 scripts/start-test-ui.py
# http://127.0.0.1:8000/bindings/web/sandcastle.html
```

点击“地理几何 / 挤出”或“CZML 地理几何”。下拉选择实体后可定位、隐藏、显示或删除；JSON 编辑器的“按 ID 添加 / 更新”保留未涉及的实体。添加/更新需要完整 Entity 描述，不是属性级 patch。

10 个相关案例（polygon、rectangle、wall、circles-and-ellipses、polyline、geometry-and-appearances 及四个 CZML 案例）现在关联可运行的本地配方。清单 `tests/sandcastle/native-recipes.json` 使用 `subset-only` 标识，不改变上游完整案例的 `not-verified` 状态。

## 新增几何

| 类型 | 自定义 Entity JSON | 实现范围 |
| --- | --- | --- |
| polyline | positionsDegrees、widthMetres | WGS84 地理折线、米宽带状面；宽线同时横向细分；没有屏幕像素宽度、圆角、虚线或贴地 |
| polygon | positionsDegrees、holes、extrudedHeight | 自动闭合轮廓、简单孔洞、曲面细分、常高挤出及内外侧壁；自交、重叠孔洞和超预算输入拒绝 |
| rectangle | coordinatesDegrees: [西,南,东,北]、height、extrudedHeight | 曲面矩形、跨日期变更线、挤出；不支持旋转或纹理 |
| ellipse | positionDegrees、semiMajorAxis、semiMinorAxis、rotation、extrudedHeight | 静态中心、切平面采样后投影到椭球高度面；rotation 为从北逆时针的弧度；轴范围 1–200,000 米 |
| wall | positionsDegrees、minimumHeights、maximumHeights | 地理线段的高度插值墙体；最低高度默认 0，最高默认每个点的高度；高度数组必须和点数相同 |

几何采用经纬度线性细分的近似曲面，不是 Cesium 全部 ArcType 的精确替代；折线拐角是分段带状面，复杂拐角可能存在连接差异。表面施加 **10 米显示偏移**并采用 0.1° 细分，减少曲面三角形落入椭球和深度冲突；不会读取地形，也不修改输入描述中的高度。GeoJSON 自有 style 仍决定其偏移。

所有新图形仅支持不透明纯色，使用 unlit 颜色；没有轮廓、纹理坐标材质、阴影、贴地、zIndex 或动态图形属性。地理坐标图形不能同时指定 entity.position/samples；椭圆需要静态 positionDegrees。输入经度 ±180°、纬度 ±85°；矩形最大 180° 宽，多边形受半球及简单环限制。每个轮廓最多 1,024 点，最多 32 个孔洞；多边形/矩形/椭圆沿用 GeoJSON 高度 ±1,000,000 米限制，墙体输入上下高度 ±2,000,000 米。整体仍受 8 MiB 输入、256 个实体、32 MiB 几何和 100,000 个总位置动画样本限制。

## CZML 扩展

新增 polygon、rectangle、ellipse、wall；支持静态 positions 的 cartographicDegrees / cartesian、polygon.holes、height、perPositionHeight、常高 extrudedHeight、rectangle.wsenDegrees / wsen（弧度）、wall.minimumHeights / maximumHeights 和 graphics.show。数值、布尔支持常量或 `{number: ...}` / `{boolean: ...}`；高度列表支持数组或 `{array: [...]}`。

矩形/多边形/墙体的位置放在图形字段，不要求顶层 position。椭圆仍需要顶层静态 position，ellipse.height 默认 0；polygon.perPositionHeight 默认 false 时，图形 height 决定高度。变化的 perPositionHeight 多边形可以绘制表面；挤出只允许所有点的高度相同。

`outline`、`fill`、`closeTop/closeBottom`、`heightReference`、`granularity`、采样图形属性、引用、多时段图形数据及非 solidColor 材质继续明确拒绝。CZML polyline 的 width 是像素单位，本版本没有把它错误解释为米；可用自定义 Entity polyline 测试米宽折线。

新增 SDK：

```javascript
viewer.upsertEntities([completeEntity]);  // 原子地按 ID 添加或更新
const entities = viewer.entities;        // 当前描述的快照
viewer.showEntity('id', false);
viewer.showEntity('id', true);
```

更新的实体重新生成网格，其余实体保留已有 GPU mesh ID。解码、预算或上传失败时，原集合保持可用。隐藏实体从绘制和拾取中退出，恢复显示不重新上传几何。

桌面示例：

```bash
CESIUM_OFFLINE=1 CESIUM_ENTITIES=tests/entities/geographic.json CESIUM_VIEW_LOCATION=110,25.25,230000 cargo run --locked -p cesium-viewer --bin desktop-viewer
CESIUM_OFFLINE=1 CESIUM_CZML=tests/entities/geographic.czml CESIUM_VIEW_LOCATION=110,25.25,230000 cargo run --locked -p cesium-viewer --bin desktop-viewer
```

## 修复与验证

- 修正 focusLocation 和 GeoJSON 定位混用地理纬度与地心纬度，新增实际中心射线经纬度断言；小孔洞拾取不再受到视角偏差影响。
- 修正 GeoJSON 经度展开后浮点舍入导致合法闭合环被拒绝的问题；支持输入首尾数字采用整数/浮点混合的等值坐标。
- 宽折线横向细分，验证 100 千米宽带的三角形内部仍在椭球外。
- 46 个普通 Rust 测试、7 个实际 Mesa Vulkan GPU 测试；新增 GPU 验证孔洞穿透、图形拾取、墙体、显隐、无效 upsert 保留原集合、无关 mesh ID 稳定及 CZML 几何加载。
- 实际 GPU 预览：`docs/geographic-entities-preview.png`。正上方观察时墙体显得较薄，定位墙体后使用倾斜视角可看到立面。
- 5 组 Node/DOM/SDK 测试覆盖新配方、显隐、定位、删除、upsert、关联案例入口及快速切换配方时丢弃旧加载结果；原有影像、地形、3D Tiles 回归保留。
- 两组新增地理 Entity / CZML 桌面配方各运行 90 帧，Entities 6 / 4，错误数均为 0。
- WASM 编译、绑定生成、fmt/clippy 验证。本轮浏览器 WebGPU 端到端因缺少 Chromium 可执行文件未能启动，不能用原生 GPU 或 DOM 测试替代。

## 后续主线

先补透明渲染、轮廓与常用材质，让当前几何案例更接近完整配方；随后推进点/billboard/label、像素折线、完整 CZML 引用/增量数据、相机飞行与追踪，再扩展 glTF 动画、3D Tiles 高级格式、2D 与后处理等。所有完整案例仍需逐项数据、视觉和交互验收。
