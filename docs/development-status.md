# 最新完整案例进度：0.0.46

新增 GPX、KML Tours、Export KML、Ground Polyline Material - Dev 四个完整案例，累计 **131 / 309**，剩余 178 个。108 项普通 Rust、132 项实际 GPU、8 组 Node、131 个浏览器 WebGPU 案例通过。验证包含全球定位系统航迹、路线、地标与多样式符号系统（博尔德登山步道航迹、起点/冲顶/终点地标图钉、下山山脊虚线与 4 种变体）、KML/KMZ 自动化三维相机导览巡检与巡游时序控制（埃菲尔铁塔地标、发光立体巡航折线、特罗卡德罗/塔顶观景台停靠点 4 种变体）、天地一体化多载体场景要素 KML/KMZ 打包与模型导出（GeoEye-1 卫星、500km 轨道折线、地面追踪站图钉与 Cesium Milk Truck 3D 车辆模型 4 种变体）、以及圣海伦斯火山复杂地形地表贴地折线材质库（Glow 发光、Dash 虚线、Outline 双色描边地表折线与线宽拓展 4 种变体）。详见 [本批验证](gpx-kmltours-exportkml-groundpolyline-validation.md)。以下保留历史记录。

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

新增 3D Models Coloring、3D Models - Dev、Labels - Dev、Billboards - Dev 四个完整案例，累计 **95 / 309**，剩余 214 个。97 项普通 Rust、122 项实际 GPU（全套 96 测试项）、8 组 Node、95 个浏览器 WebGPU 案例及 149 个桌面 90 帧配置通过。验证包含 3D 模型材质颜色着色（color、colorBlendMode、colorBlendAmount）与高亮轮廓描边（silhouetteColor、silhouetteSize）、底层 Model.fromGltf 基元与局部 ENU 参考系到 ECEF 四元数变换姿态解算、底层 LabelCollection 基元排版/字体/背景框/多行文本/局部 ENU 坐标系多标签与近远视距曲线、以及底层 BillboardCollection 广告牌基元属性/物理米级尺寸（sizeInMeters）/多点纯色广告牌阵列/大图集精灵切片子区域采样（imageSubRegion: [x, y, w, h]）归一化 UV 映射。详见 [本批验证](models-coloring-dev-labels-billboards-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.36

新增 Labels、3D Models、CZML Model Node Transformations、CZML Model Articulations 四个完整案例，累计 **91 / 309**，剩余 218 个。97 项普通 Rust、118 项实际 GPU（全套 92 测试项）、8 组 Node、91 个浏览器 WebGPU 案例及 122 个桌面 90 帧配置通过。验证包含 Labels 画廊 8 个工具栏变体（字体、填充与描边、背景框、视距偏移、视距渐隐与缩放、希伯来语/英语双向多语言及外描边宽度动画）、3D Models 画廊 10 个模型实体变体（Aircraft、Drone、Vehicle、Balloon、MilkTruck、Man、UnlitBox、Draco、KTX2、InstancedBox）及相机追踪、CZML 模型节点骨骼四元数旋转时间序列（CesiumMan 100km 视距跟踪）、以及 CZML 运载火箭分离关节动画（launchvehicle 10km 高空整流罩二次多项式插值与分离动画）。详见 [本批验证](labels-3d-models-articulations-validation.md)。以下保留历史记录。

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

# 最新完整案例进度：0.0.23

本批新增 Circle - Dev、Corridor、Corridor - Dev、Corridor Outline - Dev 四个完整案例，累计 **39 / 309**，其余 270 项待完成。103 项普通 Rust、66 项实际 GPU、8 组 Node、39 项同会话浏览器 WebGPU 和 39 项桌面 90 帧验证通过。范围与独立几何参考见 [本轮验证](corridor-circle-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.22

本批新增 11 个完整几何、材质及 CZML 案例，累计 **35 / 309**，其余 274 项待完成。100 项普通 Rust、62 项实际 GPU、8 组 Node、35 项同会话浏览器 WebGPU 与 35 项桌面 90 帧验证通过。范围与证据见 [本轮验证](geometry-material-validation.md)。以下保留历史记录。

# 开发进度与下一批任务

当前交付是可构建、可运行的基础开发版，并非整个 44 周方案的完成结果。

| 文档首批任务 | 状态 | 本次落实 | 剩余工作 |
| --- | --- | --- | --- |
| 01 Workspace / 双端 / CI | 已实现基础 | 6 个 library crate、inspector、锁文件、CI、桌面/Web 示例 | CI 需实际提交仓库执行；多操作系统实测 |
| 02 WGS84 / ENU | 已实现基础 | 极区、日界线往返；ENU；射线 | 更深地内域定义、随机独立参考、大规模性质测试 |
| 03 相机相对 / Reverse-Z | 已实现基础 | f64 差分、0..1 深度、法线逆转置 | 正交相机、极大局部资产分块、真实近景图像精度 |
| 04 limits / generation | 部分 | 受限网络/文件读取、任务 token、JS epoch | 统一 Fetcher、每内容 generation、取消与重试 |
| 05 b3dm / inspector | 子集实现 | 严格 header/length、RTC、JSON Batch Table、截断与变异测试 | 二进制 Batch Table、兼容模式、模糊测试 target |
| 06 glTF | 子集实现 | triangles、BIN、sparse/stride、节点、因子材质 | 外链资源、动画/皮肤、BLEND、IBL、其他 primitive |
| 07 tileset | 子集实现 | 显式单内容、refine 继承、空节点、bounds | 请求体积、多内容、隐式树、紧致 region 包围体 |
| 08 覆盖选择 | 已实现基础 | 整组 REPLACE 回退、ADD 保留父 | 滞回、复杂拓扑/网络轨迹、viewerRequestVolume |
| 09 预算 / pin | 部分 | 下载/CPU 预留、输出几何 GPU LRU、保留回退祖先 | 完整内存峰值记账、staging、上传字节/时间预算 |
| 10 拾取 / 高亮 | 子集实现 | 整数 ID、JSON 属性、factor MASK、高亮 | 帧相机/scene epoch 完整快照、明确失效状态、纹理 MASK |
| 11 Worker / 恢复 | 待实现 | SDK destroy、resize 失效 | Web Worker、device lost 回调与重建 |
| 12 压测 / 发布 | 待实现 | 合成 fixture、能力清单、验证命令 | 真实资产、持续漫游、内存与 p95/p99、发布验收 |

下一批按依赖顺序：

1. glTF 资源解析器与纹理：baseColor sRGB、normal/metalRough linear、sampler、缺省纹理、纹理预算和 MASK 取样共享。
2. 外部 tileset 拓扑：资源身份、循环检查、world transform、refine 上下文和空节点请求；完成原生/Web 对照 fixture。
3. 调度生命周期：每内容 generation、abort、429/5xx 退避、任务预算归还、CPU 缓存与 GPU 分段上传。
4. region 紧致包围体、SSE 滞回、点云内容与批次属性。
5. Worker、设备恢复、真实资产的受控网络和漫游压测。

使用方案的 V0.1 完成定义作为发布门槛；不能因为编译通过、单一 fixture 成功或 WebGPU 能画模型，就宣称完整引擎已交付。

## 0.0.2 地球可视化闭环

默认桌面/Web Scene 改为地球；测试三角形通过 `--demo-tile` 显式打开。新增 NASA sRGB 底图、完整全球视角、相机外部缩放限制、Home 复位与真实 GPU PNG 导出。`render-earth` 使用与默认查看器完全相同的 Scene/Renderer，避免只通过窗口启动而未检查画面的问题。通用 glTF 纹理、地形和物理大气仍未实现。浏览器已重建 WASM，但原有 DeviceLost 运行问题仍作为发布阻塞项。

## 0.0.3 全球渐进影像

新增 `cesium-viewer::imagery`：ion/Bing provider 协议、QuadKey、地理 TMS、Mercator、可见覆盖细化、祖先 UV 回退、共享 GPU 材质、缓存预算、裙边和 NASA 极区。原生后台下载/解码，Web 受限 fetch + epoch/abort。无 token 使用 Cesium Natural Earth II 示例影像；有 token 使用默认 asset 2 航拍。真实 Natural Earth 下载/渲染/原生流式加载已验收。用户未提供 ion token，因此 ion/Bing 实网、自动续期和各供应商兼容性未验收。原有 Web DeviceLost 阻塞项保留。

## 0.0.4 缩放修复

移除地球模式 1.02 赤道半径（约 128 公里高度）硬限制。改为按相机所在方向的 WGS84 局部地表半径计算离地高度，缩放按高度指数变化，最近保持约 10 米，旋转保持原离地高度，速度和近裁剪面适配近景。Web 规范化 wheel deltaMode，支持触控双指、键盘 +/−/Home 和显式缩放按钮；SDK 新增 zoom/resetView。更新 WASM 绑定；原有浏览器 DeviceLost 发布阻塞项仍保留。

## 0.0.5 模型材质与外部树

实现 glTF 的五类核心纹理、PNG/JPEG bufferView/base64 data URI、UV0/1、顶点颜色、切线、sampler/mipmap、sRGB/linear 通道、法线 scale、occlusion strength、自发光和纹理 MASK/ID 共享。保留 GGX + ambient 基础光照。外部 tileset 已连接原生和 Web 加载器，提供变换/refine 继承、每文档相对 URI、祖先循环与跨文档预算保护，原子更新拓扑。

此批不代表“完整 Cesium”已经实现。后续关键缺口仍包括外链 glTF 资源、多内容和隐式 3D Tiles、pnts/i3dm/cmpt、quantized-mesh 地形、GeoJSON/CZML/实体和时间系统、Worker、设备恢复、交互 flyTo/pan、完整材质/透明排序和多平台性能验收。原 Web DeviceLost 阻塞项尚未解决。

## 0.0.6 GeoJSON 数据叠加

实现 bounded GeoJSON decode、点圆盘、折线 ribbon、曲面细分面和孔洞，Multi 类型/GeometryCollection、WGS84/高度、原始 id/properties 和独立 feature picking。Scene 提供独立图层添加/删除/清空/定位，原生 CLI 支持 GeoJSON 输入与 --geojson 叠加，Web SDK/UI 支持图层操作。增加极区相机定位与旧请求/拾取失效保护。当前仍是静态 GeoJSON 子集，不包含完整实体系统、terrain clamp、挤出/标签、时钟/CZML、Worker 或地形。

## 0.0.7 · 真实仓库样本工作台

已完成固定 commit 的 3,721 项目录、批量生产解析器检查、直接内容预算验证、10 个模型 GPU 离屏预览、可启动的测试 UI 与交互检查。结果见 sample-validation.md；隐式切片、实例化、点云和现代扩展语义仍是后续工作，不能把点云子树 JSON 解析通过当作这些格式已支持。

## 0.0.8 · 全球地形

新增公开 Terrarium、Cesium ion quantized-mesh 与自定义 layer.json，双端地形流式加载、影像贴地、RTC、整组替换、相机避地、缓存高程查询及 UI 控制。37 个普通测试、5 个显式 GPU 测试、真实公开地形离屏渲染和桌面 smoke 完成；实时 ion、浏览器 GPU、地形贴地要素和 datum 转换尚未验收。记录见 terrain-validation.md。

## 0.0.12 · 图片材质与实体轮廓

实现静态 PNG/JPEG Entity/CZML 图片材质、重复纹理及 alpha 拾取；新增八类几何的一像素轮廓和 fill=false。Web 异步导入共享四个图片传输槽，支持相对 URL、去重、取消、旧结果丢弃及失败原子性；桌面支持相对图片文件。测试 UI 添加 JSON/CZML 配方。54 个普通 Rust 测试、11 个 GPU 测试、六组 Node 测试通过；相对资源桌面配方各运行 90 帧，Entities 5 / 2，errors 0。详细边界见 image-outline-validation.md。

下一批优先推进 point/billboard/label 等屏幕坐标实体与 CZML 属性，再扩展相机飞行/跟随；宽轮廓、动态材质、完整 DataSource 和浏览器设备恢复仍待完成。全上游案例验收通过数仍为 0。


## 0.0.13 · Point / Billboard 屏幕实体

新增 framebuffer 像素尺寸圆点、点轮廓与图片 billboard；支持静态尺寸、scale、pixelOffset、RGBA、透明孔洞拾取和深度遮挡。CZML 支持 point/billboard 与已有 FIXED 线性位置采样；UI 添加静态 JSON/CZML 与动态配方，关联上游功能子集从 11 个增至 16 个。56 个普通 Rust 测试、12 个真实 GPU 测试、六组 Node 测试通过。全案例通过数仍为 0；字体排版/label、near/far 属性、贴地、完整 Billboard 参数和浏览器设备恢复仍待开发。下一批优先 label 与距离显示属性。详见 screen-validation.md。


## 0.0.14 · 五项屏幕实体功能

实现中英文静态 Label、distanceDisplayCondition、scaleByDistance、translucencyByDistance 和 Billboard/Label origins（含 Label BASELINE）。内置 OFL 字体子集由原生/WASM 共用；距离曲线采用固定 Cesium 1.146 的 squared-distance/pow(0.2) 插值，按 f64 相机距离逐帧计算；pixelOffset 保持独立。UI 添加五个独立配方、组合 CZML 与近中远定位。61 个普通 Rust 测试、17 个 GPU 测试、六组 Node 测试通过；六个桌面配方各 90 帧 errors 0。完整上游配方验收通过数仍为 0。

下一批优先文本背景/描边、更多 Billboard 参数、距离像素偏移及相机控制；完整字体排版、DataSource 与其他引擎主线仍待逐批实现。详见 label-distance-validation.md。


## 0.0.15 · 再扩展五项屏幕效果

实现 Label 背景/padding、灰度膨胀描边三种样式、Billboard 旋转、pixelOffsetScaleByDistance 和有限 disableDepthTestDistance。屏幕覆盖率在颜色/ID 通道统一采用 1/255 阈值；无需深度测试的图形独立排在普通图形之后，颜色/ID 均使用 Always 且不写深度。UI 增加五个独立配方和组合 CZML。66 个普通测试、22 个 GPU 测试、六组 Node 测试及六个 90 帧桌面配方通过。完整上游配方验收数仍为 0，浏览器 WebGPU 尚未通过。下一批优先相机飞行/追踪、屏幕图形更多属性及数据源链路。


## 0.0.16 · 五项相机控制

新增 setView 地理眼点/HPR、flyTo 真实时间插值、cancelFlight/替换/手动接管、移动 Entity 跟随及固定目标 lookAt。新增 Web SDK 与 cameraState、六个测试面板按钮、五组原生命令配置。飞行独立于实体时钟；跟随响应显隐、删除、集合替换和可用区间。相机变化使旧异步拾取失效，停止后可重新拾取。

71 个普通 Rust 测试、27 个实际 GPU 测试、七组 Node 测试通过；fmt/clippy、原生/WASM 构建与绑定生成通过；五组桌面配置各 90 帧 errors 0。浏览器缺少 Chromium，WebGPU 未验收，历史 DeviceLost 问题仍待修复。完整上游配方验收数仍为 0。下一批继续数据源链路、相机更多选项和浏览器设备恢复；仍未实现全部 Sandcastle 功能。详见 camera-validation.md。


## 0.0.17 · 三个完整案例

Box、Cylinders and Cones、Plane 保留固定原始源码的所有案例参数与行为；补齐平面法向/距离、128 段圆柱与 16 条轮廓竖线、按实际几何和画布宽高比自动取景。新增全案例入口、源码执行提取/哈希校验和有证据才提升状态的记录器。完整功能案例通过数从 0 增至 3，其余 306 个仍待完成；原有 19 个功能子集不升级为完整案例。

74 个普通 Rust 测试、30 个实际 GPU 测试、八组 Node 测试及三组 90 帧桌面配置通过。浏览器专项三个案例在 Chromium 155 + 显式 Vulkan/SwiftShader + 固定本地影像条件下通过；默认后端仍可能丢失设备，未完成通用设备恢复或历史 test:web 全量验收。不声明官方逐像素一致或完整 Viewer 通用控件兼容。详见 full-recipes-validation.md。


## 0.0.18 · 再完成三个完整案例

新增 Spheres and Ellipsoids、CZML Spheres and Ellipsoids、CZML Cones and Cylinders；完整案例累计 6 / 309。实现椭球分段/轮廓网格、平滑法线/极点处理、CZML rgbaf、真实原 CZML 输入及异步完成后取景。构建记录器验证 CzmlDataSource.load/add/zoomTo Promise 关系，验收记录绑定原始输入哈希。

78 项普通 Rust 测试、33 项实际 GPU 测试、八组 Node 测试、六组 90 帧桌面配置和六个浏览器 WebGPU 案例通过；构建/fmt/clippy 通过。浏览器使用显式 Vulkan/SwiftShader 及固定影像，默认后端设备丢失和完整恢复仍未解决。不声明逐像素或完整 Viewer/API 兼容。详见 ellipsoid-czml-validation.md。


## 0.0.19 · 再完成六个完整案例

新增 CZML Box、Wall、CZML Wall、Circles and Ellipses、CZML Circles and Ellipses、CZML Point；完整案例累计 12 / 309。保留原始所有实体、位置、颜色字节与透明度、填充/轮廓及异步加载后集合取景。补齐 WGS84 Vincenty 墙体曲面路径、最高 1,000 km 半轴的椭圆与旋转、独立图形高度和实体原位置、径向细分顶底盖及 16 条挤出轮廓竖线。新增六个完整案例按钮及目录运行入口。

84 项普通 Rust 测试、39 项实际 GPU 测试、八组 Node 测试通过；12 组桌面配置各 90 帧 errors 0，12 个案例通过真实 Chromium 155 WebGPU 参数/颜色/拾取/滚轮高度变化/窗口重取景验收。fmt/clippy、原生/WASM 构建与绑定通过。浏览器使用显式 Vulkan/SwiftShader 和固定本地影像；默认后端 DeviceLost 及完整恢复仍未解决。不声明官方逐像素一致或全部 Viewer API。详见 wall-ellipse-point-validation.md。下一批继续剩余完整案例、数据源能力和设备恢复。


## 0.0.20 · 六个完整 Primitive 轮廓案例

新增 Box Outline - Dev、Cylinder Outline - Dev、Sphere Outline - Dev、Ellipsoid Outline - Dev、Rectangle Outline - Dev、Wall Outline - Dev，累计 18 / 309。新增严格静态 GeometryInstance 编译入口，保留原 ECEF 模型矩阵、局部 ENU 平移、分段和实例颜色，使用一像素 LineList；Web 公开 maximumAliasedLineWidth=1，与原脚本 Math.min 表达式匹配。Entity 新增受限仿射 modelMatrix，JSON 启用精确浮点往返解析。运行时加载原 Primitive 输入，预转换快照仅供独立核对。

新增六个 UI 按钮、原配置编辑/加载、原生 CESIUM_PRIMITIVES。原脚本不调用 zoomTo，入口提供统一北美宽视角及可选手动取景。90 项普通 Rust 测试、45 项实际 GPU 测试、八组 Node 测试通过；十八个桌面配置各 90 帧 errors 0，十八个真实浏览器 WebGPU 案例通过。fmt/clippy、原生/WASM 构建和绑定通过。浏览器使用显式 Vulkan/SwiftShader 与固定本地影像；默认后端设备丢失及通用恢复仍待解决。完整 PrimitiveCollection、任意 Appearance 与全部 Viewer 通用控件未声明兼容。详见 primitive-outline-validation.md。


## 0.0.21 · 六个完整实体 Primitive 案例

新增 Box - Dev、Cylinder - Dev、Sphere - Dev、Ellipsoid - Dev、Wall - Dev、Ellipse - Dev，累计 24 / 309。严格编译原始填充 Primitive 输入，保留实例顺序、矩阵和颜色字节量化；补齐 flat/closed/translucent 默认值、派生 faceForward、闭合不透明剔除、闭合透明两遍绘制、漫反射路径与圆柱侧面平滑法线。新增六个完整 UI 按钮及独立输入/预期快照。

96 项普通 Rust 测试、51 项实际 GPU 测试、八组 Node 测试通过；二十四组桌面原配置各 90 帧 errors 0，二十四个浏览器 WebGPU 案例通过。默认宽视角与可选手动取景分别验收；不把手动取景当作原脚本相机调用。fmt/clippy、原生/WASM 构建和绑定通过。浏览器使用显式 Vulkan/SwiftShader 与固定中性影像；默认后端设备丢失、通用恢复、OIT、任意 Appearance 和全部 Viewer 通用控件仍待完成。不声明官方逐像素或太阳/Phong 完全一致。详见 primitive-solid-validation.md。


## 0.0.36 · Labels 变体、3D Models、CZML 关节与骨骼变换体系

新增 Labels（8 种变体：包含希伯来语/RTL 文本安全排版降级、视距淡出与缩放、文字动画）、3D Models（10 种变体工具栏：包含飞机、无人机、车辆、气球、牛奶卡车、蒙皮角色、Draco/KTX2 等模型及朝向四元数与相机跟踪）、CZML Model Node Transformations（CesiumMan 骨骼节点旋转动画与视距跟踪）以及 CZML Model Articulations（运载火箭整流罩展开/分离/下落铰接动画与插值）；完整案例累计 **91 / 309**。

底层在 `entities.rs` 中补齐 CZML 与 Shape::Model 的 `runAnimations`、`nodeTransformations`、`articulations`、`viewFrom` 属性通道；在 `label.rs` 中引入希伯来/阿拉伯语 RTL Unicode 字符集安全验证与字形丢失平滑兜底，彻底杜绝复杂排版恐慌。

97 项普通 Rust 测试、118 项实际 GPU 测试（含 92 个全案例测试与变体离屏渲染）、8 组 Node 测试、122 组桌面 90 帧 smoke 测试（全部 errors 0）、91 个浏览器 WebGPU 完整案例测试全部通过。详见 labels-3d-models-articulations-validation.md。
