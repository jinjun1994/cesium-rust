# Sandcastle 剩余 252 项工作清单（2026-10-21）

本清单由现有 309 项 catalog 减去 57 项 functional-passed 得到，按主工作包人工归类；每项只归属一次，交叉依赖见各包说明。归类是排期入口，未完成案例尚未逐行作穷尽行为审计，开发前必须阅读固定源、HTML、资源及所有菜单分支。不得按名称推断通过。

[主路线、wgpu 优化及验收规则](roadmap-2026-10-21.md)

历史基线：`b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`（Cesium 1.146）。当前本地上游：`77d30241863d30f299e7271a22f95b7d589c1c75`。本文所有表格勾选均为待办；不修改机器验收清单。

## 汇总

| 工作包 | 计划阶段 | 待完成案例 |
| --- | --- | ---: |
| G01 几何、材质与 Primitive | M1 / P1 | 18 |
| G02 屏幕集合、标签与聚类 | M1 / P1 | 11 |
| G03 实体、时间与数据源 | M2 / P1 | 23 |
| G04 相机、投影与场景模式 | M2 / P1 | 10 |
| G05 地形、地表材质与贴地 | M3 / P1 | 20 |
| G06 影像图层、provider 与全景 | M3 / P1 | 21 |
| G07 模型、glTF、动画与 PBR | M4 / P1 | 19 |
| G08 3D Tiles、点云、元数据与样式 | M4 / P1 | 22 |
| G09 裁剪、分类、查询与吸附 | M5 / P1 | 22 |
| G10 光照、后处理、自定义 shader 与视频 | M5 / P1 | 26 |
| G11 高斯、体素、矢量、I3S 与粒子 | M6 / P2 | 18 |
| G12 Viewer、控件、Inspector 与宿主 | 贯穿 M0–M7 / P1 | 7 |
| G13 外部服务与组合展示 | M6 / P2 | 29 |
| G14 调度、批处理与性能案例 | 贯穿 M0–M7 / P0→P1 | 6 |
| **合计** | | **252** |

57 + 252 = 309。当前本地新增 1 项另列末尾，迁移后是 310 项；两个同名案例的新分支不额外增加案例 ID。G12/G14 是贯穿任务，不意味着要等其他功能完成才启动。

## 近期顺序

1. M0 / 0.0.27：固定基线、设备恢复、性能采样、4096 slots 分页、批量 uniform、按需渲染。案例完成数保持 57。
2. 0.0.28：czml-rectangle、partial-ellipsoids，全部通过后 59 / 309；rectangle 含动态回调和分类，移至 G03/G09 基础具备后。
3. 0.0.29：四个 polyline-volume 家族案例，全部通过后 63 / 309。
4. 0.0.30：polygon-texture-coordinates-dev，全部通过后 64 / 309；同时建设 atlas，但不据此提升其他案例。
5. 此后按通用依赖推进各包；复杂功能不得按固定每批 6 项强行切分。

## G01 几何、材质与 Primitive（18 项）

阶段：M1 / P1。前置/交叉依赖：M0；材质通用路径；动态属性部分依赖 G03。

公共验收：几何顶点/索引/法线/ST、填充轮廓、全部参数和异常；与上游独立几何参考对照。

| 状态 | 案例 ID / 固定源码 | 上游标题 | 专项说明 |
| --- | --- | --- | --- |
| 待办 | [rectangle](../tests/sandcastle/sources/rectangle/main.js) | Rectangle | 延期至 G03/G09 基础具备：原源含 CallbackProperty 动态 rotation/ST 与 TERRAIN 分类 |
| 待办 | [czml-rectangle](../tests/sandcastle/sources/czml-rectangle/main.js) | CZML Rectangle | 0.0.28：保留原 CZML 全包与加载/取景关系 |
| 待办 | [partial-ellipsoids](../tests/sandcastle/sources/partial-ellipsoids/main.js) | Partial Ellipsoids | 0.0.28：innerRadii、cone/clock 裁切、封边与轮廓 |
| 待办 | [polygon-texture-coordinates-dev](../tests/sandcastle/sources/polygon-texture-coordinates-dev/main.js) | Polygon Texture Coordinates - Dev | 0.0.30：显式 ST、嵌套孔洞、图片采样 |
| 待办 | [polyline-volume](../tests/sandcastle/sources/polyline-volume/main.js) | Polyline Volume | 0.0.29：圆/矩形/星形截面与全部 cornerType |
| 待办 | [polyline-volume-dev](../tests/sandcastle/sources/polyline-volume-dev/main.js) | Polyline Volume - Dev | 0.0.29：原 Primitive 实例与外观参数 |
| 待办 | [polyline-volume-outline-dev](../tests/sandcastle/sources/polyline-volume-outline-dev/main.js) | Polyline Volume Outline - Dev | 0.0.29：扫掠体轮廓拓扑与拐角 |
| 待办 | [czml-polyline-volume](../tests/sandcastle/sources/czml-polyline-volume/main.js) | CZML Polyline Volume | 0.0.29：原 CZML 与通用扫掠内核 |
| 待办 | [geometry-and-appearances](../tests/sandcastle/sources/geometry-and-appearances/main.js) | Geometry and Appearances | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [geometry-and-appearances-dev](../tests/sandcastle/sources/geometry-and-appearances-dev/main.js) | Geometry and Appearances - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [geometry-offset-attribute-box-cylinder-ellipsoid-dev](../tests/sandcastle/sources/geometry-offset-attribute-box-cylinder-ellipsoid-dev/main.js) | Geometry Offset Attribute box cylinder ellipsoid - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [geometry-offset-attribute-dev](../tests/sandcastle/sources/geometry-offset-attribute-dev/main.js) | Geometry Offset Attribute - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [materials](../tests/sandcastle/sources/materials/main.js) | Materials | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [parallels-and-meridians](../tests/sandcastle/sources/parallels-and-meridians/main.js) | Parallels and Meridians | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [polylines-dev](../tests/sandcastle/sources/polylines-dev/main.js) | Polylines - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [frustum-dev](../tests/sandcastle/sources/frustum-dev/main.js) | Frustum - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [z-indexing-geometry](../tests/sandcastle/sources/z-indexing-geometry/main.js) | Z-Indexing Geometry | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [volumes-dev](../tests/sandcastle/sources/volumes-dev/main.js) | Volumes - Dev | EllipsoidPrimitive + RimLighting/Stripe 与 hover pick，不是体素案例 |

## G02 屏幕集合、标签与聚类（11 项）

阶段：M1 / P1。前置/交叉依赖：M0、atlas/实例；贴地分支依赖 G05，时变分支依赖 G03。

公共验收：全部屏幕参数、距离曲线、字体/alpha/遮挡/拾取、动态修改；大集合批处理。

| 状态 | 案例 ID / 固定源码 | 上游标题 | 专项说明 |
| --- | --- | --- | --- |
| 待办 | [billboards](../tests/sandcastle/sources/billboards/main.js) | Billboards | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [billboards-dev](../tests/sandcastle/sources/billboards-dev/main.js) | Billboards - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [billboards-instancing-dev](../tests/sandcastle/sources/billboards-instancing-dev/main.js) | Billboards Instancing - Dev | 共享 quad/纹理、实例属性与独立拾取 |
| 待办 | [labels](../tests/sandcastle/sources/labels/main.js) | Labels | 需字体/样式/动态内容及原菜单全分支，不止现有静态字体 |
| 待办 | [labels-dev](../tests/sandcastle/sources/labels-dev/main.js) | Labels - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [distance-display-conditions](../tests/sandcastle/sources/distance-display-conditions/main.js) | Distance Display Conditions | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [display-conditions-dev](../tests/sandcastle/sources/display-conditions-dev/main.js) | Display Conditions - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [pointprimitives-dev](../tests/sandcastle/sources/pointprimitives-dev/main.js) | PointPrimitives - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [map-pins](../tests/sandcastle/sources/map-pins/main.js) | Map Pins | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [clustering](../tests/sandcastle/sources/clustering/main.js) | Clustering | 聚类事件、成员、像素范围与相机变化；不能只画少量合并图标 |
| 待办 | [star-burst](../tests/sandcastle/sources/star-burst/main.js) | Expand Clustered Billboards | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |

## G03 实体、时间与数据源（23 项）

阶段：M2 / P1。前置/交叉依赖：M0/G01/G02；模型/地形分支按需 G07/G05。

公共验收：时间区间/参考/插值、加载顺序/事件/删除、所有原数据与分支；动态状态可重放。

| 状态 | 案例 ID / 固定源码 | 上游标题 | 专项说明 |
| --- | --- | --- | --- |
| 待办 | [callback-position-property](../tests/sandcastle/sources/callback-position-property/main.js) | Callback Position Property | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [callback-property](../tests/sandcastle/sources/callback-property/main.js) | Callback Property | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [clock](../tests/sandcastle/sources/clock/main.js) | Clock | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [custom-datasource](../tests/sandcastle/sources/custom-datasource/main.js) | Custom DataSource | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [czml](../tests/sandcastle/sources/czml/main.js) | CZML | 原加载资源可能跨多种属性，先审计全部数据包 |
| 待办 | [czml-billboard-and-label](../tests/sandcastle/sources/czml-billboard-and-label/main.js) | CZML Billboard and Label | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [czml-custom-properties](../tests/sandcastle/sources/czml-custom-properties/main.js) | CZML Custom Properties | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [czml-path](../tests/sandcastle/sources/czml-path/main.js) | CZML Path | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [czml-polygon-interpolating-references](../tests/sandcastle/sources/czml-polygon-interpolating-references/main.js) | CZML Polygon - Interpolating References | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [czml-polygon-intervals-availability](../tests/sandcastle/sources/czml-polygon-intervals-availability/main.js) | CZML Polygon - Intervals, Availability | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [czml-reference-properties](../tests/sandcastle/sources/czml-reference-properties/main.js) | CZML Reference Properties | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [czml-zindex](../tests/sandcastle/sources/czml-zindex/main.js) | CZML ZIndex | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [datasource-ordering](../tests/sandcastle/sources/datasource-ordering/main.js) | DataSource Ordering | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [export-kml](../tests/sandcastle/sources/export-kml/main.js) | Export KML | 导出再导入的语义与资源 round-trip |
| 待办 | [geojson-and-topojson](../tests/sandcastle/sources/geojson-and-topojson/main.js) | GeoJSON and TopoJSON | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [geojson-simplestyle](../tests/sandcastle/sources/geojson-simplestyle/main.js) | GeoJSON simplestyle | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [gpx](../tests/sandcastle/sources/gpx/main.js) | GPX | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [interpolation](../tests/sandcastle/sources/interpolation/main.js) | Interpolation | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [kml](../tests/sandcastle/sources/kml/main.js) | KML | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [kml-tours](../tests/sandcastle/sources/kml-tours/main.js) | KML Tours | KML/KMZ 资源基础 + 相机/时间/播放事件 |
| 待办 | [multi-part-czml](../tests/sandcastle/sources/multi-part-czml/main.js) | Multi-part CZML | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [relative-paths](../tests/sandcastle/sources/relative-paths/main.js) | Relative Paths | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [time-dependent-path-graphics](../tests/sandcastle/sources/time-dependent-path-graphics/main.js) | Time Dependent Path Graphics | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |

## G04 相机、投影与场景模式（10 项）

阶段：M2 / P1。前置/交叉依赖：M0；跟随需 G03，碰撞需 G05。

公共验收：全部相机按钮、投影/模式切换、飞行中断、坐标转换、同步与输入事件。

| 状态 | 案例 ID / 固定源码 | 上游标题 | 专项说明 |
| --- | --- | --- | --- |
| 待办 | [camera](../tests/sandcastle/sources/camera/main.js) | Camera | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [camera-controllers](../tests/sandcastle/sources/camera-controllers/main.js) | Camera Controllers for Asset Inspection | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [camera-tutorial](../tests/sandcastle/sources/camera-tutorial/main.js) | Camera Tutorial | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [cardboard](../tests/sandcastle/sources/cardboard/main.js) | Cardboard | 双目/stereo/交互；目标宿主能力单独验证 |
| 待办 | [entity-tracking](../tests/sandcastle/sources/entity-tracking/main.js) | Entity tracking | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [headingpitchroll](../tests/sandcastle/sources/headingpitchroll/main.js) | HeadingPitchRoll | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [localtofixedframe](../tests/sandcastle/sources/localtofixedframe/main.js) | LocalToFixedFrame | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [multiple-synced-views](../tests/sandcastle/sources/multiple-synced-views/main.js) | Multiple Synced Views | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [projection](../tests/sandcastle/sources/projection/main.js) | Projection | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [rotatable-2d-map](../tests/sandcastle/sources/rotatable-2d-map/main.js) | Rotatable 2D Map | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |

## G05 地形、地表材质与贴地（20 项）

阶段：M3 / P1。前置/交叉依赖：M0/G01；分类 G09、模型 G07、Tiles terrain G08 为交叉依赖。

公共验收：真实地形高度/接缝/LOD/夸张、地下、贴地重采样；provider 网络/取消/缓存。

| 状态 | 案例 ID / 固定源码 | 上游标题 | 专项说明 |
| --- | --- | --- | --- |
| 待办 | [arcgis-tiled-elevation-terrain](../tests/sandcastle/sources/arcgis-tiled-elevation-terrain/main.js) | ArcGIS Tiled Elevation Terrain | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [billboardclamptoground-dev](../tests/sandcastle/sources/billboardclamptoground-dev/main.js) | BillboardClampToGround - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [clamp-entities-to-ground](../tests/sandcastle/sources/clamp-entities-to-ground/main.js) | Clamp Entities to Ground | 真实 terrain 与 Tiles 分支均需验收，椭球贴合不替代 |
| 待办 | [clamp-model-to-ground](../tests/sandcastle/sources/clamp-model-to-ground/main.js) | Clamp Model to Ground | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [drawing-on-terrain](../tests/sandcastle/sources/drawing-on-terrain/main.js) | Drawing on Terrain | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [elevation-band-material](../tests/sandcastle/sources/elevation-band-material/main.js) | Elevation Band Material | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [geometry-height-reference](../tests/sandcastle/sources/geometry-height-reference/main.js) | Geometry Height Reference | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [globe-materials](../tests/sandcastle/sources/globe-materials/main.js) | Globe Materials | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [globe-materials-3d-tiles-terrain](../tests/sandcastle/sources/globe-materials-3d-tiles-terrain/main.js) | Globe Materials – 3D Tiles Terrain | 依赖 G08 Tiles terrain provider，不能按普通 quantized-mesh 提前结项 |
| 待办 | [globe-materials-water-mask-elevation-map](../tests/sandcastle/sources/globe-materials-water-mask-elevation-map/main.js) | Globe Materials – Water Mask Elevation Map | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [globe-translucency](../tests/sandcastle/sources/globe-translucency/main.js) | Globe Translucency | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [ground-polyline-material-dev](../tests/sandcastle/sources/ground-polyline-material-dev/main.js) | Ground Polyline Material - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [ground-primitive-dev](../tests/sandcastle/sources/ground-primitive-dev/main.js) | Ground Primitive - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [ground-primitive-materials-dev](../tests/sandcastle/sources/ground-primitive-materials-dev/main.js) | Ground Primitive Materials - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [polylines-on-terrain-dev](../tests/sandcastle/sources/polylines-on-terrain-dev/main.js) | Polylines On Terrain - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [procedural-terrain](../tests/sandcastle/sources/procedural-terrain/main.js) | Procedural Terrain | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [terrain](../tests/sandcastle/sources/terrain/main.js) | Terrain | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [terrain-exaggeration](../tests/sandcastle/sources/terrain-exaggeration/main.js) | Terrain Exaggeration | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [terrain-tweaks-dev](../tests/sandcastle/sources/terrain-tweaks-dev/main.js) | Terrain Tweaks - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [underground-color](../tests/sandcastle/sources/underground-color/main.js) | Underground Color | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |

## G06 影像图层、provider 与全景（21 项）

阶段：M3 / P1。前置/交叉依赖：M0；时间 G03、投影 G04、多源资源调度 G14。

公共验收：所有图层控件/投影/过滤/alpha/split/时间；协议、pickFeatures、attribution 与外部服务。

| 状态 | 案例 ID / 固定源码 | 上游标题 | 专项说明 |
| --- | --- | --- | --- |
| 待办 | [arcgis-mapserver](../tests/sandcastle/sources/arcgis-mapserver/main.js) | ArcGIS MapServer | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [azure-2d-tiles](../tests/sandcastle/sources/azure-2d-tiles/main.js) | Azure 2D Tiles | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [bing-maps-labels-only](../tests/sandcastle/sources/bing-maps-labels-only/main.js) | Bing Maps Labels Only | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [cartographic-limit-rectangle](../tests/sandcastle/sources/cartographic-limit-rectangle/main.js) | Cartographic Limit Rectangle | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [globe-interior](../tests/sandcastle/sources/globe-interior/main.js) | Globe Interior | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [google-2d-tiles](../tests/sandcastle/sources/google-2d-tiles/main.js) | Google 2D Tiles | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [google-2d-tiles-with-custom-styles](../tests/sandcastle/sources/google-2d-tiles-with-custom-styles/main.js) | Google 2D Tiles with Custom Styles | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [google-earth-enterprise](../tests/sandcastle/sources/google-earth-enterprise/main.js) | Google Earth Enterprise | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [google-streetview-panorama](../tests/sandcastle/sources/google-streetview-panorama/main.js) | Google Street View Panorama | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [imagery-adjustment](../tests/sandcastle/sources/imagery-adjustment/main.js) | Imagery Adjustment | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [imagery-color-to-alpha](../tests/sandcastle/sources/imagery-color-to-alpha/main.js) | Imagery Color To Alpha | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [imagery-cutout](../tests/sandcastle/sources/imagery-cutout/main.js) | Imagery Cutout | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [imagery-layers](../tests/sandcastle/sources/imagery-layers/main.js) | Imagery Layers | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [imagery-layers-manipulation](../tests/sandcastle/sources/imagery-layers-manipulation/main.js) | Imagery Layers Manipulation | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [imagery-layers-split](../tests/sandcastle/sources/imagery-layers-split/main.js) | Imagery Layers Split | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [imagery-layers-texture-filters](../tests/sandcastle/sources/imagery-layers-texture-filters/main.js) | Imagery Layers Texture Filters | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [panorama](../tests/sandcastle/sources/panorama/main.js) | Panorama | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [splitdirection](../tests/sandcastle/sources/splitdirection/main.js) | SplitDirection | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [web-map-service-wms](../tests/sandcastle/sources/web-map-service-wms/main.js) | Web Map Service (WMS) | 版本/CRS/轴序、时间参数、错误响应与 pickFeatures |
| 待办 | [web-map-tile-service-picking](../tests/sandcastle/sources/web-map-tile-service-picking/main.js) | Web Map Tile Service Picking | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [web-map-tile-service-with-time](../tests/sandcastle/sources/web-map-tile-service-with-time/main.js) | Web Map Tile Service with Time | G03 时间 + WMTS 维度切换/缓存失效 |

## G07 模型、glTF、动画与 PBR（19 项）

阶段：M4 / P1。前置/交叉依赖：M0/G03；IBL/高级光照与 G10 联合交付。

公共验收：原模型全选项及纹理/动画/skin/morph/实例/扩展；节点/材质与动态拾取。

| 状态 | 案例 ID / 固定源码 | 上游标题 | 专项说明 |
| --- | --- | --- | --- |
| 待办 | [3d-models](../tests/sandcastle/sources/3d-models/main.js) | glTF Models | 另验本地增量：glTF 2.1 Unlit Box；旧源与当前源双基线 |
| 待办 | [3d-models-articulations-dev](../tests/sandcastle/sources/3d-models-articulations-dev/main.js) | 3D Models Articulations - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-models-coloring](../tests/sandcastle/sources/3d-models-coloring/main.js) | Coloring and Styling glTF Models | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-models-dev](../tests/sandcastle/sources/3d-models-dev/main.js) | glTF Models — Development | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-models-node-explorer-dev](../tests/sandcastle/sources/3d-models-node-explorer-dev/main.js) | 3D Models Node Explorer - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [czml-model](../tests/sandcastle/sources/czml-model/main.js) | CZML Model | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [czml-model-articulations](../tests/sandcastle/sources/czml-model-articulations/main.js) | CZML Model Articulations | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [czml-model-data-url](../tests/sandcastle/sources/czml-model-data-url/main.js) | CZML Model Data URL | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [czml-model-node-transformations](../tests/sandcastle/sources/czml-model-node-transformations/main.js) | CZML Model - Node Transformations | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [gltf-pbr-extensions](../tests/sandcastle/sources/gltf-pbr-extensions/main.js) | glTF PBR Next | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [gltf-planar-fill-dev](../tests/sandcastle/sources/gltf-planar-fill-dev/main.js) | glTF Planar Fill - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [image-based-lighting](../tests/sandcastle/sources/image-based-lighting/main.js) | Image-Based Lighting | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [khr-mesh-primitive-restart-dev](../tests/sandcastle/sources/khr-mesh-primitive-restart-dev/main.js) | KHR_mesh_primitive_restart - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [manually-controlled-animation](../tests/sandcastle/sources/manually-controlled-animation/main.js) | Manually Controlled Animation | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [pbr-lighting-dev](../tests/sandcastle/sources/pbr-lighting-dev/main.js) | PBR Lighting - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [physically-based-materials](../tests/sandcastle/sources/physically-based-materials/main.js) | Physically-Based Materials | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [styled-gltf-lines-dev](../tests/sandcastle/sources/styled-gltf-lines-dev/main.js) | Styled glTF Lines - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [styled-gltf-points-dev](../tests/sandcastle/sources/styled-gltf-points-dev/main.js) | Styled glTF Points - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [time-dynamic-wheels](../tests/sandcastle/sources/time-dynamic-wheels/main.js) | Time Dynamic Wheels | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |

## G08 3D Tiles、点云、元数据与样式（22 项）

阶段：M4 / P1。前置/交叉依赖：M0/G07；时间 G03；遍历/缓存 G14。

公共验收：内容真实渲染、metadata/style/picking、LOD/coverage、交互和动态内容，不能仅解析通过。

| 状态 | 案例 ID / 固定源码 | 上游标题 | 专项说明 |
| --- | --- | --- | --- |
| 待办 | [3d-tiles-1.1-cdb-yemen](../tests/sandcastle/sources/3d-tiles-1.1-cdb-yemen/main.js) | CDB Metadata in 3D Tiles | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-1.1-photogrammetry](../tests/sandcastle/sources/3d-tiles-1.1-photogrammetry/main.js) | 3D Tiles Photogrammetry with KTX2 | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-1.1-s2-globe](../tests/sandcastle/sources/3d-tiles-1.1-s2-globe/main.js) | S2 Globe with 3D Tiles | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-adjust-height](../tests/sandcastle/sources/3d-tiles-adjust-height/main.js) | Offset 3D Tileset Height | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-batch-table-hierarchy](../tests/sandcastle/sources/3d-tiles-batch-table-hierarchy/main.js) | 3D Tiles Styling | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-bim](../tests/sandcastle/sources/3d-tiles-bim/main.js) | BIM Design Model with 3D Tiles | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-compare](../tests/sandcastle/sources/3d-tiles-compare/main.js) | 3D Tiles Compare | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-feature-picking](../tests/sandcastle/sources/3d-tiles-feature-picking/main.js) | 3D Tiles Feature Picking | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-feature-styling](../tests/sandcastle/sources/3d-tiles-feature-styling/main.js) | 3D Tiles Feature Styling | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-formats](../tests/sandcastle/sources/3d-tiles-formats/main.js) | 3D Tiles Formats | 逐内容类型实际解码/渲染/拾取，不以 header 通过代替 |
| 待办 | [3d-tiles-interactivity](../tests/sandcastle/sources/3d-tiles-interactivity/main.js) | 3D Tiles Interactivity | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-interior](../tests/sandcastle/sources/3d-tiles-interior/main.js) | 3D Tiles Interior | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-nga-gpm-visualization](../tests/sandcastle/sources/3d-tiles-nga-gpm-visualization/main.js) | 3D Tiles NGA GPM Visualization | GPM 扩展/属性纹理与原可视化参数，需独立规范审计 |
| 待办 | [3d-tiles-photogrammetry](../tests/sandcastle/sources/3d-tiles-photogrammetry/main.js) | 3D Tiles Photogrammetry | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-picking-dev](../tests/sandcastle/sources/3d-tiles-picking-dev/main.js) | 3D Tiles Picking - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-point-cloud](../tests/sandcastle/sources/3d-tiles-point-cloud/main.js) | 3D Tiles Point Cloud | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-point-cloud-shading](../tests/sandcastle/sources/3d-tiles-point-cloud-shading/main.js) | 3D Tiles Point Cloud Shading | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-point-cloud-styling](../tests/sandcastle/sources/3d-tiles-point-cloud-styling/main.js) | 3D Tiles Point Cloud Styling | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-split-dev](../tests/sandcastle/sources/3d-tiles-split-dev/main.js) | 3D Tiles Split - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-vertical-exaggeration](../tests/sandcastle/sources/3d-tiles-vertical-exaggeration/main.js) | Vertical Exaggeration with 3D Tiles | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [czml-3d-tiles](../tests/sandcastle/sources/czml-3d-tiles/main.js) | CZML 3D Tiles | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [time-dynamic-point-cloud](../tests/sandcastle/sources/time-dynamic-point-cloud/main.js) | Time Dynamic Point Cloud | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |

## G09 裁剪、分类、查询与吸附（22 项）

阶段：M5 / P1。前置/交叉依赖：G01/G04/G05/G07/G08；深度与 stencil/查询 pass。

公共验收：颜色/深度/ID 共用裁剪，分类目标正确；ray/drill/height/mostDetailed、吸附与异步失效。

| 状态 | 案例 ID / 固定源码 | 上游标题 | 专项说明 |
| --- | --- | --- | --- |
| 待办 | [3d-tiles-1.1-photogrammetry-classification](../tests/sandcastle/sources/3d-tiles-1.1-photogrammetry-classification/main.js) | 3D Tiles Photogrammetry Classification | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-clipping-planes](../tests/sandcastle/sources/3d-tiles-clipping-planes/main.js) | 3D Tiles Clipping Planes | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-edge-visibility](../tests/sandcastle/sources/3d-tiles-edge-visibility/main.js) | 3D Tiles Edge Visibility | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-photogrammetry-classification](../tests/sandcastle/sources/3d-tiles-photogrammetry-classification/main.js) | 3D Tiles Photogrammetry Classification | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-point-cloud-classification](../tests/sandcastle/sources/3d-tiles-point-cloud-classification/main.js) | 3D Tiles Point Cloud Classification | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-terrain-classification](../tests/sandcastle/sources/3d-tiles-terrain-classification/main.js) | 3D Tiles Terrain Classification | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [classification](../tests/sandcastle/sources/classification/main.js) | Classification | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [classification-types](../tests/sandcastle/sources/classification-types/main.js) | Classification Types | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [clamp-to-3d-model](../tests/sandcastle/sources/clamp-to-3d-model/main.js) | Clamp to 3D Model | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [client-side-snapping-dev](../tests/sandcastle/sources/client-side-snapping-dev/main.js) | Client-side Snapping - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [clipping-regions](../tests/sandcastle/sources/clipping-regions/main.js) | Clipping Regions | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [edge-visibility-dev](../tests/sandcastle/sources/edge-visibility-dev/main.js) | Edge Visibility - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [hybrid-snapping-dev](../tests/sandcastle/sources/hybrid-snapping-dev/main.js) | Hybrid Snapping - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [ion-snapping-dev](../tests/sandcastle/sources/ion-snapping-dev/main.js) | Ion Snapping - Dev | 本地算法和远端 snapping 能力分开验收，凭据外部阻塞单列 |
| 待办 | [many-clipping-planes-dev](../tests/sandcastle/sources/many-clipping-planes-dev/main.js) | Many Clipping Planes - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [multifrustum-snapping-dev](../tests/sandcastle/sources/multifrustum-snapping-dev/main.js) | Multifrustum Snapping - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [pick-from-ray-dev](../tests/sandcastle/sources/pick-from-ray-dev/main.js) | Pick From Ray - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [picking](../tests/sandcastle/sources/picking/main.js) | Picking | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [picking-dev](../tests/sandcastle/sources/picking-dev/main.js) | Picking - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [polylines-on-3d-tiles](../tests/sandcastle/sources/polylines-on-3d-tiles/main.js) | Polylines on 3D Tiles | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [sample-height-from-3d-tiles](../tests/sandcastle/sources/sample-height-from-3d-tiles/main.js) | Sample Height from 3D Tiles | 最详细级别异步查询、取消/缓存与错误状态 |
| 待办 | [terrain-clipping-planes](../tests/sandcastle/sources/terrain-clipping-planes/main.js) | Terrain Clipping Planes | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |

## G10 光照、后处理、自定义 shader 与视频（26 项）

阶段：M5 / P1。前置/交叉依赖：M0/G07；feature metadata G08；frame plan、深度/透明基础。

公共验收：全部效果开关/参数/时序；Cesium 图像对照，shader 适配，跨后端与性能降级。

| 状态 | 案例 ID / 固定源码 | 上游标题 | 专项说明 |
| --- | --- | --- | --- |
| 待办 | [ambient-occlusion](../tests/sandcastle/sources/ambient-occlusion/main.js) | Ambient Occlusion | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [atmosphere](../tests/sandcastle/sources/atmosphere/main.js) | Atmosphere | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [bloom](../tests/sandcastle/sources/bloom/main.js) | Bloom | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [cloud-parameters](../tests/sandcastle/sources/cloud-parameters/main.js) | Cloud Parameters | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [clouds](../tests/sandcastle/sources/clouds/main.js) | Clouds | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [custom-per-feature-post-process](../tests/sandcastle/sources/custom-per-feature-post-process/main.js) | Custom Per-Feature Post Process | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [custom-post-process](../tests/sandcastle/sources/custom-post-process/main.js) | Custom Post Process | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [custom-primitive-dev](../tests/sandcastle/sources/custom-primitive-dev/main.js) | Custom Primitive - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [custom-shaders-3d-tiles](../tests/sandcastle/sources/custom-shaders-3d-tiles/main.js) | Custom Shaders 3D Tiles | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [custom-shaders-models](../tests/sandcastle/sources/custom-shaders-models/main.js) | Custom Shaders Models | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [custom-shaders-property-textures](../tests/sandcastle/sources/custom-shaders-property-textures/main.js) | Custom Shaders Property Textures | G08 结构化元数据/属性纹理 + shader 接口 |
| 待办 | [depth-of-field](../tests/sandcastle/sources/depth-of-field/main.js) | Depth of Field | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [fog](../tests/sandcastle/sources/fog/main.js) | Fog | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [fog-post-process](../tests/sandcastle/sources/fog-post-process/main.js) | Fog Post Process | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [fxaa](../tests/sandcastle/sources/fxaa/main.js) | FXAA | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [high-dynamic-range](../tests/sandcastle/sources/high-dynamic-range/main.js) | High Dynamic Range | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [lensflare](../tests/sandcastle/sources/lensflare/main.js) | LensFlare | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [lighting](../tests/sandcastle/sources/lighting/main.js) | Lighting | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [material-with-custom-glsl](../tests/sandcastle/sources/material-with-custom-glsl/main.js) | Material with Custom GLSL | GLSL/Cesium 内建函数适配；只接受 WGSL 不算原 API 兼容 |
| 待办 | [msaa](../tests/sandcastle/sources/msaa/main.js) | MSAA | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [multiple-shadows-dev](../tests/sandcastle/sources/multiple-shadows-dev/main.js) | Multiple Shadows - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [per-feature-post-processing](../tests/sandcastle/sources/per-feature-post-processing/main.js) | Per-Feature Post Processing | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [post-processing](../tests/sandcastle/sources/post-processing/main.js) | Post Processing | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [shadows](../tests/sandcastle/sources/shadows/main.js) | Shadows | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [shadows-dev](../tests/sandcastle/sources/shadows-dev/main.js) | Shadows - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [video](../tests/sandcastle/sources/video/main.js) | Video | 视频时间、更新、色彩、跨源、生命周期；Native/Web 解码适配分开 |

## G11 高斯、体素、矢量、I3S 与粒子（18 项）

阶段：M6 / P2。前置/交叉依赖：G03/G07/G08/G09/G10/G14 按内容选择依赖。

公共验收：真实内容/属性/拾取/层级、排序与透明、时变行为、受控 CPU/GPU 缓存。

| 状态 | 案例 ID / 固定源码 | 上游标题 | 专项说明 |
| --- | --- | --- | --- |
| 待办 | [3d-tiles-3d-native-vector](../tests/sandcastle/sources/3d-tiles-3d-native-vector/main.js) | 3D Native Vector Data | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-gaussian-splats-with-lod](../tests/sandcastle/sources/3d-tiles-gaussian-splats-with-lod/main.js) | 3D Tiles Gaussian Splats – Multiple LODs | LOD/排序/overdraw 与质量一致的性能对照 |
| 待办 | [3d-tiles-gaussian-splatting](../tests/sandcastle/sources/3d-tiles-gaussian-splatting/main.js) | 3D Tiles Gaussian Splats | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-gaussian-splatting-comparison](../tests/sandcastle/sources/3d-tiles-gaussian-splatting-comparison/main.js) | 3D Tiles Gaussian Splats – Mesh Comparison | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [3d-tiles-property-lod-river](../tests/sandcastle/sources/3d-tiles-property-lod-river/main.js) | Global Rivers Vector Tiles with Property-based LOD | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [i3s-3d-object-layer](../tests/sandcastle/sources/i3s-3d-object-layer/main.js) | I3S Consumption Cesium Demo | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [i3s-building-scene-layer](../tests/sandcastle/sources/i3s-building-scene-layer/main.js) | I3S Building Scene Cesium Demo | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [i3s-feature-picking](../tests/sandcastle/sources/i3s-feature-picking/main.js) | I3S Feature Picking Cesium Demo | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [i3s-integratedmesh-layer](../tests/sandcastle/sources/i3s-integratedmesh-layer/main.js) | I3S Consumption Cesium Demo | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [particle-system](../tests/sandcastle/sources/particle-system/main.js) | Particle System | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [particle-system-fireworks](../tests/sandcastle/sources/particle-system-fireworks/main.js) | Particle System Fireworks | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [particle-system-tails](../tests/sandcastle/sources/particle-system-tails/main.js) | Particle System Tails | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [particle-system-weather](../tests/sandcastle/sources/particle-system-weather/main.js) | Particle System Weather | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [vector-3d-tiles](../tests/sandcastle/sources/vector-3d-tiles/main.js) | Vector 3D Tiles | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [volumecloud](../tests/sandcastle/sources/volumecloud/main.js) | Volumetric Cloud | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [voxel-picking](../tests/sandcastle/sources/voxel-picking/main.js) | Voxel Picking | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [voxels](../tests/sandcastle/sources/voxels/main.js) | Voxel Rendering | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [voxels-in-3d-tiles](../tests/sandcastle/sources/voxels-in-3d-tiles/main.js) | Voxels in 3D Tiles | 另验本地增量：3D Tiles 2.0 fromGltf 体素选项 |

## G12 Viewer、控件、Inspector 与宿主（7 项）

阶段：贯穿 M0–M7 / P1。前置/交叉依赖：M0 先建生命周期；控件依赖其被检查的功能包。

公共验收：全部按钮/面板/事件、创建销毁、多视图、API 和 DOM；shared-context 单列兼容性风险。

| 状态 | 案例 ID / 固定源码 | 上游标题 | 专项说明 |
| --- | --- | --- | --- |
| 待办 | [3d-tiles-inspector](../tests/sandcastle/sources/3d-tiles-inspector/main.js) | 3D Tiles Inspector | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [cesium-inspector](../tests/sandcastle/sources/cesium-inspector/main.js) | Cesium Inspector | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [cesium-widget](../tests/sandcastle/sources/cesium-widget/main.js) | Cesium Widget | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [custom-geocoder](../tests/sandcastle/sources/custom-geocoder/main.js) | Custom Geocoder | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [shared-context](../tests/sandcastle/sources/shared-context/main.js) | Shared Context | M0 原型：WebGL 与 WebGPU 无直接 context 等价；桥接不能隐瞒深度差异 |
| 待办 | [timeline](../tests/sandcastle/sources/timeline/main.js) | Timeline Demo | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [viewerless](../tests/sandcastle/sources/viewerless/main.js) | Viewerless Sandcastle | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |

## G13 外部服务与组合展示（29 项）

阶段：M6 / P2。前置/交叉依赖：G05–G11 按资产依赖；鉴权、网络与授权测试资源。

公共验收：原资产/全部交互/查询/attribution；离线协议测试与授权实网验收分别留证据。

| 状态 | 案例 ID / 固定源码 | 上游标题 | 专项说明 |
| --- | --- | --- | --- |
| 待办 | [aec-architectural-design](../tests/sandcastle/sources/aec-architectural-design/main.js) | Design Model – Layer Control and Metadata Picking | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [aec-clipping](../tests/sandcastle/sources/aec-clipping/main.js) | Design Model – Terrain Inset | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [aec-elements-api-query](../tests/sandcastle/sources/aec-elements-api-query/main.js) | Design Model – Elements API Query | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [aec-isolate-by-category](../tests/sandcastle/sources/aec-isolate-by-category/main.js) | Design Model – Category Filtering | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [aec-metadata-styling](../tests/sandcastle/sources/aec-metadata-styling/main.js) | Design Model – Metadata Styling | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [aec-snapping](../tests/sandcastle/sources/aec-snapping/main.js) | AEC Snapping | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [aec-tileset-metadata-filtering](../tests/sandcastle/sources/aec-tileset-metadata-filtering/main.js) | Design Model – Tileset Metadata Filtering | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [aerometrex-san-francisco](../tests/sandcastle/sources/aerometrex-san-francisco/main.js) | Aerometrex San Francisco | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [arcticdem](../tests/sandcastle/sources/arcticdem/main.js) | ArcticDEM | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [bathymetry](../tests/sandcastle/sources/bathymetry/main.js) | Global Bathymetry | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [blue-marble](../tests/sandcastle/sources/blue-marble/main.js) | Blue Marble | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [cesium-osm-buildings](../tests/sandcastle/sources/cesium-osm-buildings/main.js) | Cesium OSM Buildings | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [cesium-world-terrain](../tests/sandcastle/sources/cesium-world-terrain/main.js) | Cesium World Terrain | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [drape-imagery-on-3d-tiles](../tests/sandcastle/sources/drape-imagery-on-3d-tiles/main.js) | Overlay Imagery on 3D Tiles | 模型 UV/投影/多影像合成、Tiles 遮挡与服务资产 |
| 待办 | [earth-at-night](../tests/sandcastle/sources/earth-at-night/main.js) | Earth at Night | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [google-photorealistic-3d-tiles](../tests/sandcastle/sources/google-photorealistic-3d-tiles/main.js) | Google Photorealistic 3D Tiles | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [google-photorealistic-3d-tiles-with-building-insert](../tests/sandcastle/sources/google-photorealistic-3d-tiles-with-building-insert/main.js) | Google Photorealistic 3D Tiles with Building Insert | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [imagery-assets-available-from-ion](../tests/sandcastle/sources/imagery-assets-available-from-ion/main.js) | Imagery Assets available from ion | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [imodel-mesh-export-service](../tests/sandcastle/sources/imodel-mesh-export-service/main.js) | iModel Mesh Export Service | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [itwin-feature-service](../tests/sandcastle/sources/itwin-feature-service/main.js) | iTwin Feature Service | 服务查询、classification/metadata 与鉴权错误/续期 |
| 待办 | [itwin-gaussian-splats](../tests/sandcastle/sources/itwin-gaussian-splats/main.js) | iTwin Gaussian Splats | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [japan-buildings](../tests/sandcastle/sources/japan-buildings/main.js) | Japan Buildings | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [mars](../tests/sandcastle/sources/mars/main.js) | Mars | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [montreal-point-cloud](../tests/sandcastle/sources/montreal-point-cloud/main.js) | Montreal Point Cloud | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [moon](../tests/sandcastle/sources/moon/main.js) | Moon | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [natural-earth-ii](../tests/sandcastle/sources/natural-earth-ii/main.js) | Natural Earth II | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [pamap-terrain](../tests/sandcastle/sources/pamap-terrain/main.js) | PAMAP Terrain | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [sentinel-2](../tests/sandcastle/sources/sentinel-2/main.js) | Sentinel-2 | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |
| 待办 | [washington-dc-2017](../tests/sandcastle/sources/washington-dc-2017/main.js) | Washington DC 2017 | 原服务/资产与全部交互；无凭据时 blocked-external，不用本地替代资产冒充通过 |

## G14 调度、批处理与性能案例（6 项）

阶段：贯穿 M0–M7 / P0→P1。前置/交叉依赖：M0 先建测量；其余依赖 G01/G05/G08/G09。

公共验收：固定资产/轨迹/质量、原 UI 参数语义、p95/p99/内存/覆盖；证明优化收益与正确性。

| 状态 | 案例 ID / 固定源码 | 上游标题 | 专项说明 |
| --- | --- | --- | --- |
| 待办 | [3d-tiles-performance-testing-dev](../tests/sandcastle/sources/3d-tiles-performance-testing-dev/main.js) | Streaming Performance Testing - Dev | 基准资产、原参数、SSE/coverage、真实性能 |
| 待办 | [clipping-performance-dev](../tests/sandcastle/sources/clipping-performance-dev/main.js) | Clipping Performance - Dev | 裁剪数量规模测试 + 无误裁/拾取对照 |
| 待办 | [constant-lod-dev](../tests/sandcastle/sources/constant-lod-dev/main.js) | Constant LOD - Dev | 保留固定 LOD 语义，不能让自动质量调节改变案例 |
| 待办 | [scene-rendering-performance](../tests/sandcastle/sources/scene-rendering-performance/main.js) | Scene Rendering Performance | M0 先做按需渲染底座，再验原案例全部控件 |
| 待办 | [terrain-entity-batching-dev](../tests/sandcastle/sources/terrain-entity-batching-dev/main.js) | Terrain Entity Batching - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |
| 待办 | [terrain-performance-dev](../tests/sandcastle/sources/terrain-performance-dev/main.js) | Terrain Performance - Dev | 按本包公共验收 + 原源码全部分支；外部资源如有则单列实网证据 |

## 当前本地上游增量（不计入上述 252 项）

| 增量 | 来源 | 依赖与验收 |
| --- | --- | --- |
| 新增案例 `3d-tiles-bounding-box-tests-dev` | [当前源码](../../cesium/packages/sandcastle/gallery/3d-tiles-bounding-box-tests-dev/main.js) | G07/G08/G12；Cesium3DTileset.fromGltf、bounding volume、八个菜单资源、ENU 放置、相机 zoomTo 与切换清理 |
| `3d-models` 新菜单 | [当前源码](../../cesium/packages/sandcastle/gallery/3d-models/main.js) | glTF 2.1 Unlit Box；重新哈希与生成输入，保留历史原源回归 |
| `voxels-in-3d-tiles` 新菜单 | [当前源码](../../cesium/packages/sandcastle/gallery/voxels-in-3d-tiles/main.js) | Cesium3DTilesVoxelProvider.fromGltf / 3D Tiles 2.0 体素；数据、availability、metadata 与交互 |

两个上游提交还包含 `packages/gltf` 抽取、Tiles/subtree/model 等实现变化。完整本地源码复刻需生成 API/Specs 差异清单，不能只完成这三行就声称已吸收全部增量。

## 每项开工与结项记录模板

```text
case_id / upstream_commit / main_sha / html_sha / assets_sha
work_package / API 与格式依赖 / 平台 / 外部服务
原脚本语句、菜单、按钮、事件、时序和失败行为清单
通用 Rust 实现位置 / SDK 映射 / shader 与 GPU 能力路径
独立 Cesium 参考 / 容差 / 视觉与语义差异
Native + Web 运行证据 / 二进制哈希 / 设备与后端
性能轨迹/指标 / 恢复与取消 / 已知限制
状态、完成日期、复核者；blocked-external 必须保留原因
```

### 清单完整性检查

本次生成时检查：252 个唯一 ID，均属于 catalog，均不在 57 个通过集合；无重复/遗漏；每个固定源链接存在。状态提升仍必须由运行证据驱动。

后续变更先更新权威机器清单，再更新本文，且保留主路线中“历史基线 / 当前本地基线”的区别。
