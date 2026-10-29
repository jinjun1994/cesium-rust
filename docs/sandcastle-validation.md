# 最新完整案例进度：0.0.45

新增 Clustering、GeoJSON Simplestyle、GeoJSON and TopoJSON、KML 四个完整案例，累计 **127 / 309**，剩余 182 个。107 项普通 Rust、128 项实际 GPU、8 组 Node、127 个浏览器 WebGPU 案例通过。验证包含全球科研重器空间位置聚类与动态徽标标记（CERN、Fermilab、KEK、DESY 科学台站图钉与 40px/80px/自定义聚类徽章 4 种变体）、Simplestyle 规范点线面样式体系与 2D/3D 多视态投影（旧金山地标图钉、红色交通走廊折线、青蓝市政多边形、2D/Columbus View/定制配色 4 种变体）、美国行政州界拓扑多边形、人口高度挤出模型与属性驱动着色（加利福尼亚、德克萨斯、纽约、佛罗里达州界、55km~100km 人口挤出三维柱体与伪随机配色 4 种变体）、以及全球科学设施地标、GDP 挤出面体与 GX Bike 骑行时序轨迹（Placemark 点标、GDP 挤出立体面、自行车动目标与高亮轨迹 4 种变体）。详见 [本批验证](clustering-geojson-kml-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.44

新增 CZML、Multi-part CZML、Relative Paths、Time Dynamic Wheels 四个完整案例，累计 **123 / 309**，剩余 186 个。107 项普通 Rust、124 项实际 GPU、8 组 Node、123 个浏览器 WebGPU 案例通过。验证包含多数据包空间资产与地面车辆综合画廊（GeoEye-1、ISS 卫星广告牌/路径、AGI 地面站标注、宾夕法尼亚多边形与地面车辆 4 种变体）、莫哈韦沙漠 4500 秒分段流式 CZML 传输与油量监控（CesiumMilkTruck 模型、燃油显示 Fuel: 200/133/66/20 gal 与时序航段走廊）、动参考系与空间编队相对航线可视化（两颗伴随卫星对地/相对轨道、地面车辆与直升机对地/相对航迹 4 种变体）、以及地面铰接车辆非线性加速、动态车轮旋转与实时速度仪（GroundVehicle 3D 模型、nodeTransformations 车轮自转、VelocityVectorProperty 测速与 0~60 km/hr 变体）。详见 [本批验证](czml-multipart-relativepaths-wheels-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.43

新增 Picking、Clamp Model to Ground、Camera、Time-Dependent Path Graphics 四个完整案例，累计 **119 / 309**，剩余 190 个。106 项普通 Rust、120 项实际 GPU、8 组 Node、119 个浏览器 WebGPU 案例通过。验证包含鼠标交互拾取画廊（经纬度信息标签、广告牌拾取、3 种层叠多边形穿透拾取与拾取属性修改 4 种变体）、CesiumMilkTruck 3D 模型贴地基准与参考定位针（Clamp to Ground、Clamp to 3D Tiles、Relative to Ground、No Clamping 4 种高度基准）、相机导航与姿态漫游（圣迭戈地标飞行、圣海伦斯火山 HeadingPitchRoll 姿态飞行、美东大西洋走廊矩形包围盒自适应取景与 LookAt 目标锁定）、以及四维时空采样飞行轨迹与导引线材质（180 秒闭环航路、Portions/Whole 导引区段、Cyan Glow 与 Polyline Outline 材质）。详见 [本批验证](picking-clamp-camera-timedependentpath-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.42

新增 Materials、Bloom、Cesium Widget、Cardboard 四个完整案例，累计 **115 / 309**，剩余 194 个。105 项普通 Rust、116 项实际 GPU、8 组 Node、115 个浏览器 WebGPU 案例通过。验证包含常用程序化外观材质画廊（Image、Grid、Stripe、Checkerboard、Dot、RimLighting、PolylineGlow、PolylineOutline 8 种材质与互斥显示控制）、13 具热气球阵列排布与后期泛光光效调节（contrast、brightness、delta、sigma、stepSize 及 4 种泛光变体）、极简微件视口与目标追踪（Cesium_Air.glb 模型、HeadingPitchRoll 姿态角、minimumPixelSize 128、maximumScale 20000 与 shouldAnimate 追踪）、以及 VR 纸盒双目模式与大峡谷环形飞行编队（13 具 CesiumBalloon.glb 模型、360 秒 Hermite 三次多项式采样封闭航线与模型局部参考系锁定）。详见 [本批验证](materials-bloom-widget-cardboard-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.41

新增 Cartographic Limit Rectangle、Clamp Entities to Ground、Billboards Instancing - Dev、Geometry and Appearances 四个完整案例，累计 **111 / 309**，剩余 198 个。104 项普通 Rust、112 项实际 GPU、8 组 Node、111 个浏览器 WebGPU 案例通过。验证包含全球咖啡带 360° 完整经度范围限制矩形与 10 层海拔标高边界外框（0~45000m）、8 种典型实体贴地渲染（Point/Label、Billboard、Corridor、Polygon、Textured Polygon、Rectangle、Model CesiumMan、Polyline）与 `depthTestAgainstTerrain` 深度测试、Seneca 湖区 100 广告牌密集网格实例化多变体（4/25/64/100 规模、0.1/0.5/1.0 缩放、脉冲动画、贴地与聚类）、以及 58 种综合几何体与材质外观（条纹材质 StripeMaterial、棋盘格材质 CheckerboardMaterial、3D 拉伸体、叠层半透明形体、深天蓝发光测地线 PolylineGlow、变截面多边形放样管道与变高墙体），并将集合显存上限扩容至 128 MB。详见 [本批验证](cartographic-clamp-instancing-geometry-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.40

新增 Interpolation、Manually Controlled Animation、LocalToFixedFrame、PointPrimitives - Dev 四个完整案例，累计 **107 / 309**，剩余 202 个。103 项普通 Rust、108 项实际 GPU、8 组 Node、107 个浏览器 WebGPU 案例通过。验证包含 `SampledPositionProperty` 环形采样飞行轨迹、`VelocityOrientationProperty` 朝向解算与 4 种多项式插值模式（线性、拉格朗日、埃尔米特与惯性系）、CesiumMan 角色非线性变速奔跑动画、实时速度矢量计算标签（km/hr）与追尾相机锁定跟踪、10 种局部切平面标架生成器（NED、NEU、NWU、NWD、ENU、END、SWU、SUW、UEN、DES）齐次矩阵变换及 HeadingPitchRoll 交互、以及底层 `PointPrimitiveCollection` 视距缩放与淡出（NearFarScalar）、局部参考系点分布与点云簇群。详见 [本批验证](interpolation-animation-localframe-points-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.39

新增 BillboardClampToGround - Dev、Clock、DataSource Ordering、Display Conditions - Dev 四个完整案例，累计 **103 / 309**，剩余 206 个。99 项普通 Rust、104 项实际 GPU、8 组 Node、103 个浏览器 WebGPU 案例通过。突破 100 例里程碑！验证包含底层 BillboardCollection 广告牌基元 CLAMP_TO_GROUND 与 RELATIVE_TO_GROUND 贴地基准与地表深度测试、2013 圣诞节太阳直射点时钟驱动（LOOP_STOP、SYSTEM_CLOCK_MULTIPLIER、4000x 速率与实时次太阳光照跟踪）、多数据源图层 raise / lower 动态重排与 zIndex 深度层叠排序、以及 DistanceDisplayCondition 近远距（0~5.5e6m 与 0~250.5m）多图元视距条件控制。详见 [本批验证](billboardclamp-clock-ordering-display-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.38

新增 3D Models Articulations - Dev、3D Models Node Explorer - Dev、HeadingPitchRoll、Entity tracking 四个完整案例，累计 **99 / 309**，剩余 210 个。98 项普通 Rust、126 项实际 GPU（全套 100 测试项）、8 组 Node、99 个浏览器 WebGPU 案例及 167 个桌面 90 帧配置通过。验证包含 3D 模型运载火箭整流罩展开/分离/下落铰接多阶段控制、CesiumMan 角色骨骼节点翻滚/俯仰旋转与节点显隐控制、飞机模型西北天（NWU）局部切平面标架欧拉角四元数姿态解算与 PolylineGlow 飞行采样发光轨迹与追尾相机、以及 CZML 复合数据包（模型+路径、广告牌+标签+路径）、高阶拉格朗日多项式插值与惯性/速度/东北天多种相机跟踪参考系。详见 [本批验证](models-articulations-explorer-hpr-tracking-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.37

新增 3D Models Coloring、3D Models - Dev、Labels - Dev、Billboards - Dev 四个完整案例，累计 **95 / 309**，剩余 214 个。97 项普通 Rust、122 项实际 GPU（全套 96 测试项）、8 组 Node、95 个浏览器 WebGPU 案例及 149 个桌面 90 帧配置通过。验证包含 3D 模型材质颜色与轮廓着色（color、colorBlendMode、silhouetteColor、silhouetteSize）、底层 Model.fromGltf 基元与 HeadingPitchRoll 局部坐标系到 ECEF 四元数姿态解算与相机 lookAt 矩阵绑定、底层 LabelCollection 基元自定义字体/描边/背景框/多行文本/ENU 参考系多标签与近远视距曲线、以及底层 BillboardCollection 广告牌基元属性/物理米级尺寸（sizeInMeters）/纯点广告牌阵列/图集切片子区域采样（imageSubRegion: [x, y, w, h]）归一化 UV 映射。详见 [本批验证](models-coloring-dev-labels-billboards-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.36

新增 Labels、3D Models、CZML Model Node Transformations、CZML Model Articulations 四个完整案例，累计 **91 / 309**，剩余 218 个。97 项普通 Rust、118 项实际 GPU、8 组 Node、91 个浏览器 WebGPU 案例及 122 个桌面 90 帧配置通过。验证包含 Labels 8 种变体（包含希伯来语/RTL 文本安全排版降级、广告牌视距偏移、视距淡出与缩放、文字动画）、3D Models 10 种变体工具栏（包含飞机、无人机、地面车辆、热气球、牛奶卡车、蒙皮角色、Draco/KTX2 等模型及 ENU 朝向四元数与相机追踪）、CZML Model Node Transformations（CesiumMan 骨骼节点旋转动画与视距跟踪）以及 CZML Model Articulations（运载火箭整流罩展开/分离/下落铰接动画与插值）。详见 [本批验证](labels-3d-models-articulations-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.35

新增 Billboards、CZML Model、CZML Model - Data URL、Callback Position Property 四个完整案例，累计 **87 / 309**，剩余 222 个。97 项普通 Rust、88 项实际 GPU、8 组 Node、87 个浏览器 WebGPU 案例及 102 个桌面 90 帧配置通过。验证包含广告牌 10 种变体（eyeOffset、pixelOffset、rotation、scale、color 染色、scaleByDistance、translucencyByDistance 曲线、pixelOffsetScaleByDistance、disableDepthTestDistance）、CZML 模型包与内联 base64 glTF 2.0 数据 URI 解码与渲染（aircraft model、cube model、scale、minimumPixelSize、trackedEntity）、以及 CallbackPositionProperty / CatmullRomSpline 样条插值无人机实体与 PolylineGlow 轨迹。详见 [本批验证](billboards-czml-model-drone-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.34

新增 CZML Polygon - Interpolating References、Distance Display Conditions、Callback Property、Map Pins 四个完整案例，累计 **83 / 309**，剩余 226 个。97 项普通 Rust、84 项实际 GPU、8 组 Node、83 个浏览器 WebGPU 案例及 88 个桌面 90 帧配置通过。验证包含 CZML 数组引用动态多边形与内孔拓扑、高空轨道几何（perPositionHeight 达 4000km）、DistanceDisplayCondition 视距裁切条件（Billboard/Rectangle 与 Point/Model 变体切换）、CallbackProperty 动态宽折线/测地线距离实时计算驱动标签移动、以及 PinBuilder 4 种标记图标生成与底部针尖锚定。详见 [本批验证](czml-interpolating-polygon-distance-pins-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.33

新增 CZML Billboard and Label、Geometry Height Reference、CZML Path、CZML Reference Properties 四个完整案例，累计 **79 / 309**，剩余 230 个。127 项普通 Rust、80 项实际 GPU（全套 110 项）、8 组 Node、79 个浏览器 WebGPU 案例及 84 个桌面 90 帧配置通过。验证包含 CZML 广告牌与标签复合渲染、HeightReference 高度基准（RELATIVE_TO_GROUND 与 CLAMP_TO_GROUND）、飞行轨迹采样路径与外描边折线材质（CZML Path 与 PolylineOutline）、以及跨实体引用属性解算（reference properties 递归解包）。详见 [本批验证](czml-billboard-label-path-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.32

新增 Polylines - Dev、Frustum - Dev、Parallels and Meridians、Polygon Texture Coordinates - Dev 四个完整案例，累计 **75 / 309**，剩余 234 个。126 项普通 Rust、76 项实际 GPU（全套 106 项）、8 组 Node、75 个浏览器 WebGPU 案例及 80 个桌面 90 帧配置通过。验证包含 Fade 渐隐折线材质管线（LineMaterial 11u 与 WGSL 插值）、透视视锥实体与线框几何（FrustumGeometry / FrustumOutlineGeometry 与局部 ENU 姿态矩阵）、全球经纬度解算与等角航线容差（±90° 极点贯通与地心零点安全规避），以及显式多边形纹理坐标与嵌套内孔贴图（textureCoordinates 与 POSITION_NORMAL_ST 格式）。详见 [本批验证](polylines-frustum-polygon-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.31

新增 Geometry Offset Attribute - Dev、Geometry Offset Attribute (Box, Cylinder, Ellipsoid) - Dev、CZML Polygon Intervals and Availability、CZML Custom Properties 四个完整案例，累计 **71 / 309**，剩余 238 个。126 项普通 Rust、72 项实际 GPU（全套 102 项）、8 组 Node、71 个浏览器 WebGPU 案例及 76 个桌面 90 帧配置通过。验证包含几何法向偏移管线（GeometryOffsetAttribute TOP/ALL/NONE 与 attributes.offset）、多边形层级内孔与拉伸高度（holes 与 height）、CZML 时间区间/有效性动态生命周期（intervals 与 availability）、以及非视觉自定义属性对象（custom properties 与采样时间序列）。详见 [本批验证](geometry-offset-czml-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.30

新增 Rectangle、Z-Indexing Geometry、CZML Z-Index、Volumes - Dev 四个完整案例，累计 **67 / 309**，剩余 242 个。126 项普通 Rust、68 项实际 GPU（全套 98 项）、8 组 Node、67 个浏览器 WebGPU 案例及 72 个桌面 90 帧配置通过。验证包含多层几何 Z-Indexing 稳定层叠与 Reverse-Z 深度测试无 Z-Fighting、矩形拉伸/高度/旋转与独立纹理旋转角（stRotation）、CZML zIndex 解析与 Glow 发光线材质扩充，以及体图元（EllipsoidPrimitive）解码与 RimLighting/Stripe 材质绑定。详见 [本批验证](rectangle-zindex-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.29

新增 Polyline Volume、Polyline Volume - Dev、Polyline Volume Outline - Dev、CZML Polyline Volume 四个完整案例，累计 **63 / 309**，剩余 246 个。126 项普通 Rust、64 项实际 GPU、8 组 Node、63 个浏览器 WebGPU 案例及 68 个桌面 90 帧配置通过。验证包含三维空间截面沿轨扫掠几何核、Frenet/Darboux 局部标架、Rounded/Beveled/Mitered 拐角拓扑、首尾截面 earcutr 端面封盖、纵向棱线与首尾环线框提取，以及 Entity/Primitive/CZML 统一支持。详见 [本批验证](polyline-volume-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.28

新增 CZML Rectangle、Partial Ellipsoids 两个完整案例，累计 **59 / 309**，剩余 250 个。122 项普通 Rust、60 项实际 GPU、8 组 Node、59 个浏览器 WebGPU 案例及 64 个桌面 90 帧配置通过。验证包含内径中空壳体（innerRadii）、时钟与天顶切口（clock/cone angles）、四元数姿态、CZML 拉伸与旋转、条纹/图像材质绑定、中空表面精确三角面拾取以及 WebAssembly 跨平台时间抽象。详见 [本批验证](czml-rectangle-partial-ellipsoids-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.26

新增 Polyline、Polyline - Dev、Polyline Color - Dev、Polyline Material - Dev、Polyline Dash、CZML Polyline 六个完整案例，累计 **57 / 309**，剩余 252 个。115 项普通 Rust、85 项实际 GPU、8 组 Node、57 个浏览器案例及 62 个桌面 90 帧配置通过。验证包含像素线宽、全部原材质参数、Material 默认值与 Property 参数的区别、逐实体拾取和透明虚线空隙。详见 [本批验证](polyline-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.25

新增 Polygon、Polygon - Dev、Coplanar Polygon - Dev、CZML Polygon、Points、Offline 六个完整案例，累计 **51 / 309**，剩余 258 个。109 项普通 Rust、78 项实际 GPU、8 组 Node、51 个浏览器案例及 56 个桌面 90 帧配置通过。详见 [本批验证](polygon-points-offline-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.24

本批新增 CZML Corridor、CZML Position Definitions、Show or Hide Entities、HTML Overlays、Resolution Scaling、Hello World 六个完整案例，累计 **45 / 309**，其余 264 项待完成。105 项普通 Rust、72 项实际 GPU、8 组 Node、45 项浏览器 WebGPU 与 45 项桌面 90 帧检查通过。HTML / DPR 控件在浏览器验收，三个场景案例的桌面检查仅覆盖共享地球与投影内核。详见 [本批验证](scene-czml-groups-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.21

本批新增 Box、Cylinder、Sphere、Ellipsoid、Wall、Ellipse - Dev 六个完整实体 Primitive 案例，累计 **24 / 309**，其余 285 项待完成。最新范围与证据见 [实体 Primitive 完整案例](primitive-solid-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.20

本批完整移植六个 Primitive 轮廓开发案例（Box、Cylinder、Sphere、Ellipsoid、Rectangle、Wall Outline - Dev），累计 **18 / 309**，其余 291 个待完成。最新范围与证据见 [Primitive 完整案例](primitive-outline-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.19

本批完整移植六个案例：CZML Box、Wall、CZML Wall、Circles and Ellipses、CZML Circles and Ellipses、CZML Point。累计 **12 / 309**，其余 297 项待完整验收。最新范围与证据见 [墙体、椭圆与 Point 完整案例](wall-ellipse-point-validation.md)。以下保留各历史版本记录，最新共享报告以 0.0.19 为准。

# 当前完整案例进度（0.0.18）

本批新增 Spheres and Ellipsoids、CZML Spheres and Ellipsoids、CZML Cones and Cylinders；累计 **6 / 309**，其余 303 项待完整验收。详见 [本批完整案例](ellipsoid-czml-validation.md)。以下为各历史版本记录。

# 当前完整案例进度（0.0.17）

**Box、Cylinders and Cones、Plane 三个完整案例的脚本行为已全部实现并验收**，见 [完整案例验收](full-recipes-validation.md)。新增完整案例按钮与目录中的完整案例入口，功能通过数为 3 / 309。原始源码目录保持不修改，验收状态由 full-recipes.json 与真实测试证据维护；不声明官方逐像素一致或完整 Viewer 控件兼容。

以下各批记录中的“完整配方通过 0”是对应历史版本的状态。

# Sandcastle 范围与验证（0.0.15）

本批新增五项屏幕效果：Label 背景、Label 描边、Billboard 旋转、距离像素偏移和距离深度策略。五个独立配方、组合 CZML 及证据见 `screen-effects-validation.md`。

本版本继续扩展 Rust/wgpu 实现，**尚未完成 Sandcastle 全功能兼容**。全案例完成数为 0；这指逐个完整上游配方的视觉和交互验收，不否定已有基础模块和本版本的功能测试。

## 可复现的案例范围

固定 Cesium **1.146**，commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。`packages/sandcastle/gallery` 中有 **309 个 sandcastle.yaml 案例**（包括开发配方），311 个顶层目录项不能直接当作案例数量。此范围是该版本全部元数据案例，不保证覆盖未来更新或部署站点临时增加的内容。

`tests/sandcastle/catalog.json` 保留标题、标签、源码 SHA-256、显式 Cesium API 引用和准确的 `?id=` 官方参考链接。`sources/` 保存配方 main.js 和 YAML，仅供查看，**不会加载 CesiumJS 代替 Rust 引擎运行**。上游公开 JWT / Google key 已脱敏；摘要记录原始源码内容。API 引用是静态扫描线索，不能当作依赖完整分析或兼容证明。上游许可证见同目录 UPSTREAM-LICENSE.md。

更新脚本：

```bash
# commit 参数必须是已经解析出的 Cesium 仓库 commit。
python3 scripts/update-sandcastle-catalog.py --commit b8d3a36fe98a3e432eb89253c95d5f20e605e0f1
```

## 启动与当前功能

```bash
python3 scripts/start-test-ui.py
# 打开 http://127.0.0.1:8000/bindings/web/sandcastle.html
# 或点击原工作台顶部“Sandcastle 全案例”。
```

全部案例可搜索、按标签筛选、查看源码/API 和官方参考；本机验收笔记可保存和导出。参考站点在独立标签页打开，不算本地实现。WebGPU 初始化失败时目录仍可浏览。当前页面不提供把完整案例标为通过的快捷按钮，人工笔记也不自动升级兼容状态。

| 模块 | 本版本行为 | 边界 |
| --- | --- | --- |
| Entity | 箱体、椭球、圆柱、圆锥、ENU 水平面及地理折线/多边形/矩形/椭圆/墙体；纯色/程序材质及透明、位置、可用区间、名称和属性 | 自定义 JSON API，非 CesiumJS Entity API 的直接替换；支持静态 PNG/JPEG 图片和一像素轮廓；支持静态 Point/Billboard 像素图形；支持静态 Label、距离属性与锚点；无贴地、宽几何轮廓和完整文本样式 |
| 集合 | 事务式整组替换、完整描述 upsert、显隐、按 ID 删除、清空、定位、拾取 | 一组最多 256 个实体、32 MiB 几何、100,000 个总样本；输入最多 8 MiB |
| 时间 | 播放、暂停、倍速、倒放、跳转；UNBOUNDED / CLAMPED / LOOP_STOP | UTC Unix 秒；非 JulianDate/TAI；闰秒明确拒绝。LOOP_STOP 向前循环，向后在起点停止 |
| 位置动画 | f64 ECEF 线性采样，每帧更新 GPU 模型变换而非重新上传几何 | 样本范围外隐藏，不外推；朝向保持当地 ENU，无速度朝向或姿态采样 |
| CZML | document 1.0、时钟、availability、FIXED cartesian / cartographicDegrees、epoch 或 ISO 时间采样、box / ellipsoid / cylinder / polygon / rectangle / ellipse / wall / point / billboard / label、solidColor/stripe/checkerboard/grid/image RGBA 和静态 fill/outline、properties | 严格子集，未知属性直接报错；不支持过程式 process/增量包、delete、引用、INERTIAL、路径、完整文本样式、模型、完整 Billboard 参数或其他材质；不自动从 availability 推导时钟 |
| 相机 | setView HPR、flyTo/取消、移动实体跟随、固定目标 lookAt；focusEntity、地球缩放/旋转与地形定位 | 自定义有界 API，详见 camera-validation.md；无完整飞行选项、2D/Columbus view |
| UI | 五种几何配方、动态箱体、JSON/CZML 编辑器、时间轴、属性拾取 | 这些本地配方是功能子集，不代表 Geometry and Appearances 等完整上游案例通过 |

图片资源 API：`loadEntities(valueOrUrl, {baseUrl, upsert})`、`loadCzmlData(valueOrUrl, {baseUrl})`、`cancelEntityLoads()`。加载失败保留旧场景，清空和销毁会取消未完成资源。

SDK 接口：`setEntities(array)`、`loadCzmlBytes(Uint8Array)`、`loadCzml(url)`、`clock`、`setClock(options)`、`removeEntity(id)`、`clearEntities()`、`focusEntity(id, range)`。网络 CZML 最多 4 个并发读取，8 MiB 限制与场景 generation guard 防止旧请求提交到新场景。`setEntities` / CZML 均替换整个实体集合，保留地球、影像、地形、GeoJSON 和 3D Tiles。

本地示例：`tests/entities/shapes.json`、`tests/entities/motion.czml`。桌面同样运行共享 Scene 与时钟：

```bash
CESIUM_OFFLINE=1 CESIUM_ENTITIES=tests/entities/shapes.json CESIUM_VIEW_LOCATION=110,25,180000 cargo run --locked -p cesium-viewer --bin desktop-viewer
CESIUM_OFFLINE=1 CESIUM_CZML=tests/entities/motion.czml CESIUM_ENTITY_FOCUS=moving-box cargo run --locked -p cesium-viewer --bin desktop-viewer
# 空格暂停/播放；R 切换时间方向；+/- 缩放。
```

## 验证证据

- 全工作区 66 个普通 Rust 测试通过；22 个实际 GPU 测试通过（Mesa Vulkan），包括原有影像、地形、材质和拾取回归。
- 新 GPU 测试渲染五种实体并逐个中心拾取；确认采样移动改变帧缓冲像素；验证删除、失效拾取与事务式替换。实际输出：`docs/entities-preview.png`。
- 原生/WASM 编译及绑定生成通过；Rust fmt、clippy 检查通过。桌面五个独立配方与组合 CZML 各运行 90 帧，Entities 1 / 1 / 1 / 1 / 2 / 6，错误数均为 0；详见 `screen-effects-validation.md`。
- `npm test`：六组测试通过，包括 Sandcastle DOM/目录与图片资源测试。SDK 验证毫秒转秒、后台间隔上限、CZML 并发限制、源切换丢弃过期结果；UI 验证全部 309 个案例、筛选、参考源码文本显示、笔记、实体操作、暂停/跳转/倒放及无 GPU 时目录浏览。
- DOM 测试替换了 SDK/GPU 边界，**不是浏览器 WebGPU 渲染验收**。浏览器端到端脚本已加入 Sandcastle 画面/拾取/时间轴断言，但本轮未能启动：缺少 Chromium，补装时下载内容不是有效 ZIP，安装失败。先前版本另有 DeviceLost(Destroyed) 未解决记录；不能用 WASM 编译或原生 GPU 测试替代。

## 仍需实施的主线

完整 Entity/DataSource（标签、点、billboard、路径、完整墙体/挤出/贴地和事件）、完整 CZML/KML/GPX、glTF 动画/蒙皮/压缩资源、完整 3D Tiles（点云/实例/隐式切片等）、时间影像/WMS/WMTS、地形高级水面/法线、材质/透明/阴影/大气/后处理、2D 与同步视图、相机飞行/追踪、测量和编辑工具等。每类还包含独立案例特有的数据、交互和视觉效果；需要继续逐批实现与验收。

0.0.16 新增五项相机功能与测试 UI；完整范围和证据见 [相机验证](camera-validation.md)。目录与完整配方验收数未变。
