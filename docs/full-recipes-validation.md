# 最新完整案例进度：0.0.87

新增 Terrain Entity Batching - Dev、Terrain Exaggeration、Terrain Performance - Dev、Terrain Tweaks - Dev 四个完整案例，累计 **295 / 309**，剩余 14 个。147 项普通 Rust、296 项实际 GPU、8 组 Node、295 个浏览器 WebGPU 案例通过。验证包含多类型实体地形表面批处理渲染与 10x10 网格 100 要素分类覆层（基准 10x10 表面网格与贴地混合图元模式、高密度 100 要素表面网格压力测试模式、高对比橙黄双色棋盘格覆层模式、45 度纹理旋转贴地椭圆形变模式 4 种变体）、场景垂向高程比例动态夸张放大与基准高程基准面绑定（基准 2.0x 垂向夸张与 2400m 相对基准面高程带模式、4.0x 极限高陡峻峭阿尔卑斯地形夸张模式、1.0x 真实物理未缩放基准对照模式、0m 海平面基准 2.5x 夸张模式 4 种变体）、珠穆朗玛峰全空域飞行轨迹性能基准套件与帧率及瓦片加载队列调度监控（基准珠峰水平巡航飞行性能模式、正射垂直俯视瓦片流式加载模式、45度大倾角斜向多层 LOD 视锥负荷模式、高通量快速缩放极限压力测试 4 种变体）、以及全球地形瓦片流式加载参数微调与调试覆层（基准标准 20 深度子级限制与平衡预加载模式、40 深度激进预加载与青色填充高亮模式、5 深度低延迟保守预加载与洋红高亮模式、生产环境纯净地表模式 4 种变体）。详见 [本批验证](terrain-batching-exaggeration-perf-tweaks-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.86

新增 Styled glTF Lines - Dev、Styled glTF Points - Dev、Terrain、Terrain Clipping Planes 四个完整案例，累计 **291 / 309**，剩余 18 个。146 项普通 Rust、292 项实际 GPU、8 组 Node、291 个浏览器 WebGPU 案例通过。验证包含 glTF 材质折线样式拓展与屏幕像素级虚线图案渲染（基准 Bentley 材质线样式模式、16 位虚线掩码点画模式、高对比荧光线框模式、斜视角边缘细部巡检模式 4 种变体）、glTF 材质变径点云图元拓展与多色彩多尺度点集渲染（基准 Bentley 材质点样式模式、高密微点微观模式、大直径信标点图元模式、多变量色彩强度映射模式 4 种变体）、全球地形多重服务商对比与水面法线/太阳动态光照（基准全球地形工作台、平坦 WGS84 椭球体几何基准模式、自定义正弦波周期高度图模式、破晓低日照角高对比阴影模式 4 种变体）、以及地球表面多边形与长方体多剖切面开挖裁剪（基准开挖基坑裁剪、圣海伦斯火山口多边形开挖模式、科罗拉多大峡谷孤岛平顶山模式、地下负空间洞室高亮模式 4 种变体）。详见 [本批验证](lines-points-terrain-terrainclip-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.85

新增 Shadows - Dev、Shared Context、SplitDirection、Expand Clustered Billboards (Star Burst) 四个完整案例，累计 **287 / 309**，剩余 22 个。145 项普通 Rust、288 项实际 GPU、8 组 Node、287 个浏览器 WebGPU 案例通过。验证包含高级阴影贴图深度偏差校准与多光源级联投射（基准宾州 Pinnacle 4 级联阴影模式、Seneca Rocks 陡峭地形自阴影模式、优胜美地半圆顶深谷级联阴影模式、高空飞行器长距离投影阴影模式 4 种变体）、跨双视口 WebGL/WebGPU 显存上下文与几何资源共享（基准共享上下文双视口模式、双视图同步青色盒图元模式、共享多视锥球体几何模式、独立高速缓冲区失效压力测试 4 种变体）、屏幕空间左右分屏滑动对比与图元裁剪（基准左右分屏点图元与广告牌模式、模型几何分屏对比模式、半透明贴地折线与多边形分区模式、中心滑动分割线诊断模式 4 种变体）、以及聚集点广告牌星芒放射状展开与交互折叠（基准聚集广告牌星芒爆发模式、高密度 60 要素银河圆环模式、双层同心同构层级爆发模式、折叠单点原位静止基准模式 4 种变体）。详见 [本批验证](shadows-sharedctx-splitdir-starburst-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.84

新增 Procedural Terrain、Projection、Sample Height from 3D Tiles、Scene Rendering Performance 四个完整案例，累计 **283 / 309**，剩余 26 个。144 项普通 Rust、284 项实际 GPU、8 组 Node、283 个浏览器 WebGPU 案例通过。验证包含自定义高度图程序化地形生成与噪声分形驱动（基准值噪声分形布朗运动高山模式、周期正弦波山脊模式、高频涟漪程序化地形模式、平滑低频沙丘地貌模式 4 种变体）、相机透视与正交平行投影切换及运奶车跟踪（基准正交平行投影模式、透视投影模式、正交长焦特写巡检模式、广角鸟瞰透视视口模式 4 种变体）、3D Tiles 倾斜摄影与 BIM 瓦片表面高程精细插值采样（基准贴合高程采样、高密度 60 点精细地形剖面、稀疏 10 点结构大跨度巡检、正交横截面贴合高程断面 4 种变体）、以及显式按需渲染优化与时空交互能效管理（基准鼠标悬停交互按需重绘模式、动画飞机仿真场景模式、3D 瓦片静态城市场景模式、显式后台 API 状态失效重绘模式 4 种变体）。详见 [本批验证](procedural-projection-sampleheight-scenerender-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.83

新增 Pick From Ray - Dev、Picking - Dev、Polylines on 3D Tiles、Post Processing 四个完整案例，累计 **279 / 309**，剩余 30 个。143 项普通 Rust、280 项实际 GPU、8 组 Node、279 个浏览器 WebGPU 案例通过。验证包含三维空间射线拾取、视线遮挡通视求交与多图元穿透探测（基准单次射线求交模式、穿透式多层图元探测模式、视线通视遮挡模式、无障碍直达射线模式 4 种变体）、图元实例层级综合拾取开发套件、广告牌/文字/点云/折线几何图元交互（基准图元与几何实例拾取、广告牌与文字标签集合拾取模式、点图元集合与宽折线图元拾取模式、穿透深度层级拾取检查模式 4 种变体）、3D Tiles 瓦片分类贴模型折线、工业管道布线与倾斜摄影地物包络（基准 BIM 工业管道模式、倾斜摄影山地道路模式、地形与 3D 瓦片双重贴地模式、地下综合管廊布线模式 4 种变体）、以及多阶段全屏后处理滤镜链、轮廓/黑白/亮度/夜视管线组合（基准轮廓边缘检测阶段、黑白单色滤镜阶段、亮度与对比度增强阶段、夜视光学模拟阶段 4 种变体）。详见 [本批验证](raypick-picking-polylines3d-postprocess-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.82

新增 Particle System Weather、PBR Lighting - Dev、Per-Feature Post Processing、Physically-Based Materials 四个完整案例，累计 **275 / 309**，剩余 34 个。142 项普通 Rust、276 项实际 GPU、8 组 Node、275 个浏览器 WebGPU 案例通过。验证包含高山高空雨雪天气模拟与大气光学衰减（基准高山暴风雪模式、倾盆大雨暴雨模式、冰霰暴风雪混合模式、雪后初霁晴空模式 4 种变体）、基于物理环境光照与动态天空反射探测（基准完整 PBR 图像光照、纯漫反射环境光照模式、纯镜面反射高光模式、直射日光影棚模式 4 种变体）、逐要素屏幕空间鼠标交互高亮与黑白量化后处理（基准鼠标悬停高亮石灰绿轮廓线阶段、鼠标悬停黑白灰度量化阶段、双阶段复合渲染模式、直通无过滤基础模型渲染 4 种变体）、以及基于物理渲染 PBR 材质与全天时时照光线漫游（基准正前方高光反射视角、45 度侧角日出掠射视角、正上方正午直射视角、仰角黄昏地平线边缘光视角 4 种变体）。详见 [本批验证](weather-pbr-postprocessing-pbm-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.81

新增 Panorama、Particle System、Particle System Fireworks、Particle System Tails 四个完整案例，累计 **271 / 309**，剩余 38 个。141 项普通 Rust、272 项实际 GPU、8 组 Node、271 个浏览器 WebGPU 案例通过。验证包含 360 度全景沉浸式球体/立方体贴图漫游与视场 FOV 缩放（基准等距长方圆柱 360 全景、六面成立方体天空盒全景模式、望远长焦光学变焦模式、超广角沉浸式全景模式 4 种变体）、运奶车运动轨迹跟踪、Circle/Sphere/Cone 多型发射器与局部引力扰动（基准圆形发射器尾气烟流、球形发射器全向膨胀模式、高速方向性锥体喷流、重力偏转下沉烟羽模式 4 种变体）、夜空多级爆裂烟花、随机三维空间偏移与径向速度衰减截止（基准多级烟花齐射、高空翡翠与青蓝星芒、极速连发压轴华章模式、近地超重型礼炮单团绽放 4 种变体）、以及超音速客机尾迹、彗尾冰晶流与火箭热能喷注双模切换（基准彗尾深冷粒子飘带、火箭助推热力加力模式、双翼尖对称空气动力学涡流凝结尾迹模式、低速大气烟带宽幅漂移弥散模式 4 种变体）。详见 [本批验证](panorama-particles-fireworks-tails-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.80

新增 Multifrustum Snapping - Dev、Multiple Shadows - Dev、Multiple Synced Views、PAMAP Terrain 四个完整案例，累计 **267 / 309**，剩余 42 个。140 项普通 Rust、268 项实际 GPU、8 组 Node、267 个浏览器 WebGPU 案例通过。验证包含多视锥体渲染管线下的客户端几何边缘/表面吸附、对数深度关闭与远近截锥诊断（基准多视锥体几何吸附、表面吸附模式、未命中回退光线投射模式、两点测量模式 4 种变体）、多点光源阴影摄像机、ShadowMap 调试截锥显示与世界地形深度阴影（基准双点光源全向阴影、第二光源调试视角切换模式、动画阴影摆动模式、全球地形自阴影与自遮挡模式 4 种变体）、2D/3D 双视图时空强耦合同步、共享 ClockViewModel 与中心椭球射线解算（基准 2D/3D 双视口时空同步、2D 正射地图跟踪 3D 地球表面中心焦点投影、动态相机距离偏移强耦合、并排双视口地籍边界检视模式 4 种变体）、以及美国宾夕法尼亚 PASDA 高分辨率激光雷达地形、顶峰地貌与阿巴拉契亚山脊漫游（基准 PAMAP 宾夕法尼亚高精地形、尼塔尼山地质山脊微地形飞行动画、阿巴拉契亚山脊等高地貌高亮检视、PASDA 全州激光雷达空间地理覆盖边界 4 种变体）。详见 [本批验证](multifrustum-shadows-synced-pamap-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.79

新增 Material with Custom GLSL、Montreal Point Cloud、Moon、MSAA 四个完整案例，累计 **263 / 309**，剩余 46 个。139 项普通 Rust、264 项实际 GPU、8 组 Node、263 个浏览器 WebGPU 案例通过。验证包含 Fabric 架构自定义 GLSL 着色器与时序颜色调制（基准自定义 Fabric GLSL 材质外观、正弦时间周期 RGB 色度动态调制模式、双扫描明亮透明度光带模式、连续 Z 轴顶点旋转模式 4 种变体）、加拿大蒙特利尔高精 20cm 倾斜激光雷达点云与全景观览（基准蒙特利尔 20cm 分辨率激光雷达点云、皇家山公园植被地形点云分类模式、老港历史滨水街区近景巡检模式、圣劳伦斯河沿岸水域勘测边界模式 4 种变体）、月球全境 3D 瓦片地形、LROC 月海玄武岩边界与阿波罗/阿尔忒弥斯着陆点（基准月球 3D 瓦片地形、阿尔忒弥斯 3 号南极永久阴影坑科学考察模式、LROC 全球月海玄武岩边界贴地 GeoJSON 矢量模式、月球高地斜长岩风化层反射率光谱分析模式 4 种变体）、以及场景级硬件多重采样抗锯齿 MSAA 与亚像素几何平滑（基准 MSAA 8x 多重采样抗锯齿激活、MSAA 4x 性能平衡抗锯齿模式、MSAA 2x 轻量边缘平滑模式、MSAA 1x 走样基准直通模式 4 种变体）。详见 [本批验证](glsl-montreal-moon-msaa-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.78

新增 KHR_mesh_primitive_restart - Dev、LensFlare、Many Clipping Planes - Dev、Mars 四个完整案例，累计 **259 / 309**，剩余 50 个。138 项普通 Rust、260 项实际 GPU、8 组 Node、259 个浏览器 WebGPU 案例通过。验证包含 glTF KHR_mesh_primitive_restart 图元内联重启索引批处理、Hilbert 空间填充曲线与 WebGL 2 原生管线（基准希尔伯特空间填充曲线图元重启、线环多不相交拓扑模式、三角带批处理几何模式、三角扇不相交拓扑检视模式 4 种变体）、相机镜头眩光后处理全屏阶段、彩虹色散光环、光学镜片污迹与深空天体观察（基准太阳镜头眩光后处理阶段、高色散彩虹光环模式、高强度光学表面污渍模拟模式、微光大气边缘眩光模式 4 种变体）、大数量径向剖切面几何裁剪、32/64 面圆柱剖开剖切体与多类型图元裁剪适配（基准 32 径向剖切面、高密度 64 剖切面多面体高精竖井钻孔模式、并集外壳反向剖切模式、地下地形径向扇区剖切包络模式 4 种变体）、以及火星全境 3D 瓦片地形、火星大气散射参数、好奇号与毅力号漫游车实迹交互（基准火星 3D 瓦片实景地形与火星大气层、盖尔撞击坑好奇号漫游车穿越轨迹、毅力号与机智号直升机侦察飞行范围、《火星救援》跨大陆远征路线 4 种变体）。详见 [本批验证](khrrestart-lensflare-clipping-mars-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.77

新增 Ion Snapping - Dev、iTwin Feature Service、iTwin Gaussian Splats、Japan Buildings 四个完整案例，累计 **255 / 309**，剩余 54 个。137 项普通 Rust、256 项实际 GPU、8 组 Node、255 个浏览器 WebGPU 案例通过。验证包含 Cesium ion 服务端几何对齐吸附、最近/关键点/中心多模式吸附与交互容差孔径（基准 Ion BIM/CAD 服务端几何吸附、关键点吸附模式、中心吸附模式、高容差搜索孔径模式 4 种变体）、Bentley iTwin 地理要素服务、点线面多尺度现实数据分类渲染与图层控制台（基准 Bentley iTwin 要素服务、点状基础设施隔离模式、线性工程标线模式、鸟瞰航拍地籍多边形区域 4 种变体）、Bentley iTwin 3D 高斯泼溅真实感模型、模型矩阵高程微调与全球地形结合（基准 3D 高斯泼溅真实数据、大坝坝顶近景巡检模式、地形基底矩阵偏移校准、勘测外包范围边界模式 4 种变体）、以及日本全境 3D 建筑物大数据集、局部地形加载、多地标漫游与动态相机光源（基准日本 3D 建筑大数据集与区域地形、京都二条城文化遗产地标视角、相机动态平行光源、富士山-御殿场地势高程模式 4 种变体）。详见 [本批验证](ionsnap-itwin-splats-japan-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.76

新增 Imagery Assets available from ion、Imagery Layers Split、Imagery Layers Texture Filters、iModel Mesh Export Service 四个完整案例，累计 **251 / 309**，剩余 58 个。136 项普通 Rust、252 项实际 GPU、8 组 Node、251 个浏览器 WebGPU 案例通过。验证包含 Cesium ion 影像资产目录、多源航拍遥感图层与费城多图层动态切换（基准谷歌 2D 卫星正射影像、微软 Azure 航拍高精巡检模式、欧空局 Sentinel-2 全球多光谱地表覆盖、谷歌 2D 地形等高线专题图 4 种变体）、影像图层卷帘分屏对齐、日夜双底图对比与屏幕空间交互滑块（基准日夜影像卷帘对比、夜间灯光辐射主导左侧分割、日间制图街道地图主导、高对比大陆海岸线边界 4 种变体）、影像纹理过滤、双线性插值平滑与最近邻像素化放大走样对比（基准双线性平滑与最近邻像素过滤卷帘对比、纯最近邻像素艺术制图模式、纯双线性滤波平滑渐变模式、五大湖水岸高对比模式 4 种变体）、以及 Bentley iTwin / iModel 基础设施工程网格导出服务、要素悬停黄色高亮与属性标签遮罩（基准 Bentley iModel 基础设施网格、鼠标悬停黄色高亮模式、结构钢构架独立隔离模式、空间包围体量范围模式 4 种变体）。详见 [本批验证](ionassets-split-filters-imodel-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.75

新增 I3S Building Scene Layer、I3S Feature Picking、I3S Integrated Mesh Layer、Image-Based Lighting 四个完整案例，累计 **247 / 309**，剩余 62 个。135 项普通 Rust、248 项实际 GPU、8 组 Node、247 个浏览器 WebGPU 案例通过。验证包含 Esri I3S 建筑场景图层、BIM 专业子图层分类与 Turanga 图书馆全景（基准新西兰基督城 Turanga 图书馆建筑场景图层、建筑专业独立隔离模式、机电 HVAC 暖通管网巡检、高对比空间包络线框诊断 4 种变体）、Esri I3S 纽约高密集建筑属性要素拾取、字段动态加载与 EGM2008 水准面转码（基准纽约曼哈顿属性化 I3S 要素拾取、建筑高度与地籍税块属性检视、中城商业建筑密度分类、高精度大地水准面高程对齐 4 种变体）、Esri I3S 摄影测量实景三维连续网格、法兰克福城市全景与防遮挡渲染（基准德国法兰克福倾斜摄影连续三维网格、美因河滨水连续网格巡检、历史老城区高密纹理模式、高空大范围网格覆盖边界模式 4 种变体）、以及基于图像的真实感物理环境光照 IBL、Filament 9 项二阶球谐漫反射与 KTX2 镜面反射立方体贴图（基准 Kiara 晴午实景环境贴图与球谐光照、程序化日光环境光模式、高反射镜面金属镜像模式、环境地面漫反射辉光着色模式 4 种变体）。详见 [本批验证](bsl-featurepicking-integratedmesh-ibl-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.74

新增 Ground Primitive Materials - Dev、High Dynamic Range、Hybrid Snapping - Dev、I3S 3D Object Layer 四个完整案例，累计 **243 / 309**，剩余 66 个。134 项普通 Rust、244 项实际 GPU、8 组 Node、243 个浏览器 WebGPU 案例通过。验证包含贴地几何图元高级材质着色、法线扰动与多图层贴地混合（基准棋盘格贴地覆层、凹凸贴图法线扰动贴地面、等高线流式条带贴地图元、程序化木纹地表着色 4 种变体）、高动态范围 HDR、色调映射 Tonemapping 与自动曝光控制（基准高动态范围开启、日光高亮度过曝基准、深阴影暗光辐射模式、标准低动态范围基准对照 4 种变体）、混合吸附：客户端网格几何顶点/边缘吸附与服务端高程拾取融合（基准混合吸附、地表射线地形高程吸附、模型包围盒角点与面吸附、交互式多点测量吸附轨迹 4 种变体）、以及 Esri I3S 索引三维场景图层、旧金山城市建筑与属性图层解析（基准旧金山 I3S 3D 对象图层、密集中心地段高层建筑纹理检视、符号化高程颜色分类模式、线框包围几何体诊断 4 种变体）。详见 [本批验证](groundprimmat-hdr-hybridsnap-i3s-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.73

新增 Google Photorealistic 3D Tiles、Google Photorealistic 3D Tiles with Building Insert、Google Street View Panorama、Ground Primitive - Dev 四个完整案例，累计 **239 / 309**，剩余 70 个。133 项普通 Rust、240 项实际 GPU、8 组 Node、239 个浏览器 WebGPU 案例通过。验证包含谷歌全球高保真真实感 3D 瓦片集、Googleplex 视角与天空大气渲染（基准谷歌总部高保真真实感实景三维、日照航拍巡检视角、硅谷都会区山麓微地形、高对比建筑边界模式 4 种变体）、真实感城市三维底座上拟建建筑插入与开发地块分类覆层（基准拟建高层大厦嵌合与分类地块、拟建建筑体量独立显隐模式、目标规划地块高亮模式、丹佛市中心城市天际线视角 4 种变体）、谷歌全景街景球体/立方体贴图流式呈现与视线姿态跟踪（基准 360 度街景全景气泡、六面体立方体全景透视、高空地图与微观街景无缝转换、人行道路网导航指引覆层 4 种变体）、以及底层 GroundPrimitive、地形贴地多边形/走廊/圆形图元与半透明颜色属性（基准重叠半透明贴地图元、纯贴地多边形几何模式、恒向线走廊与圆形贴地图元、高对比贴地图元轮廓模式 4 种变体）。详见 [本批验证](googlephotorealistic-insert-streetview-groundprim-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.72

新增 glTF Planar Fill - Dev、Google 2D Tiles、Google 2D Tiles with Custom Styles、Google Earth Enterprise 四个完整案例，累计 **235 / 309**，剩余 74 个。132 项普通 Rust、236 项实际 GPU、8 组 Node、235 个浏览器 WebGPU 案例通过。验证包含 BENTLEY_materials_planar_fill 扩展 CAD 平面填充与边缘特征显示（基准多配置 CAD 平面填充显示、纯线框填充诊断模式、背景填充遮罩模式、共面后置图层优先级模式 4 种变体）、谷歌全球 2D 实景航拍遥感图层、费城市中心自适应对齐与地形集成（基准谷歌全球 2D 正射遥感影像、高精城市核心区巡检模式、特拉华滨水区设色模式、三州都会区宏观全景模式 4 种变体）、谷歌 2D 瓦片多图层叠加、矢量路网定制色调饱和度与道路几何简化（基准卫星底图叠加青色路网、极简高亮道路几何模式、单色高对比制图填充模式、夜景暗色底图与血脉公路模式 4 种变体）、以及谷歌地球企业版 GEE 协议服务、三维地形流与黑大理石夜景全球覆层（基准 GEE 地形与黑大理石夜景影像服务、企业版高程三角网线框模式、夜景图层发光增强模式、高对比海岸企业服务边界模式 4 种变体）。详见 [本批验证](gltfplanarfill-google2dtiles-customstyles-gee-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.71

新增 Globe Materials 3D Tiles Terrain、Globe Materials Water Mask Elevation Map、Globe Translucency、glTF PBR Extensions 四个完整案例，累计 **231 / 309**，剩余 78 个。131 项普通 Rust、232 项实际 GPU、8 组 Node、231 个浏览器 WebGPU 案例通过。验证包含地球材质与 3D Tiles 地形等高线/坡度坡向着色器组合（基准高程等高线与坡度坡向着色、分层设色高程带模式、坡度倾角分析材质、高对比复合地形诊断模式 4 种变体）、地球水体掩膜与全球高程纹理设色动态光照（基准水体掩膜镜面反射与高程图、纯水体掩膜反射诊断、全球高程分级着色、高对比海岸陆海过渡融合 4 种变体）、地球全域半透明度、地下管网透视与距离淡入淡出调谐（基准地下半透明混合、视距渐变衰减模式、高对比地下深层管廊模式、全域开窗透视天际模式 4 种变体）、以及 glTF 2.0 PBR 扩展材质微表面物理渲染：清漆层、各向异性与镜面光泽（基准多扩展 PBR 影棚模式、纯清漆汽车面漆模式、纯各向异性拉丝金属诊断、镜面反射权重与色彩实验室 4 种变体）。详见 [本批验证](globemat-watermask-translucency-gltfpbr-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.70

新增 Edge Visibility - Dev、FXAA、Geometry and Appearances - Dev、Globe Interior 四个完整案例，累计 **227 / 309**，剩余 82 个。130 项普通 Rust、228 项实际 GPU、8 组 Node、227 个浏览器 WebGPU 案例通过。验证包含 EXT_mesh_primitive_edge_visibility 扩展三维模型表面/边缘独立显隐调度（双模型基准表面与边缘共显模式、纯表面模式、纯边缘线框模式、双模型非对称混合模式 4 种变体）、全屏快速近似抗锯齿后处理阶段、亚像素几何边缘平滑与建筑天际线对比（基准 FXAA 开启模式、FXAA 直通停用模式、高对比建筑天际线剪影填充模式、城市地平线与地形平滑模式 4 种变体）、底层 Primitive 与 GeometryInstance 组合、星形/圆形拉伸与色彩外观（基准每实例色彩外观、半透明外观覆写模式、平面纯色线框诊断模式、高对比复合截面填充模式 4 种变体）、以及地球同心圈层内地核/外核/地幔结构、地表半透明度开窗与海洋色彩透明过滤（基准内地核聚焦模式、外核地磁发电机壳层模式、地幔圈层展示模式、地表半透明矩形窗口模式 4 种变体）。详见 [本批验证](edgevis-fxaa-geomapp-globeinterior-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.69

新增 Custom Shaders Property Textures、CZML 3D Tiles、Depth of Field、Overlay Imagery on 3D Tiles 四个完整案例，累计 **223 / 309**，剩余 86 个。129 项普通 Rust、224 项实际 GPU、8 组 Node、223 个浏览器 WebGPU 案例通过。验证包含 MAXAR OWT Muscatatuk 实景三维属性纹理、CE90/LE90 不确定度着色器（基准水平不确定度 CE90、垂直不确定度 LE90 模式、水平与垂直双不确定度联合模式、无不确定度纯净地貌表面 4 种变体）、CZML 1.0 规约声明式 3D Tiles 数据包、CzmlDataSource 异步流式加载与视口自适应（基准 CZML 瓦片集数据包、隔离单体建筑特征模式、半透明体量检视模式、高对比线框包络模式 4 种变体）、景深后处理阶段、高斯卷积模糊、弥散圆半径调谐与 13 具热气球走廊焦距追踪（基准景深开启模式、近景特写聚焦模式、远景地平线聚焦模式、直通停用景深模式 4 种变体）、以及 Bentley iTwin 实景三维网格表面贴地影像投影、Bing Maps 矢量路网注记叠加（基准费城实景三维与 Bing 地名注记贴地投影、高分卫星正射影像贴模模式、地籍矢量宗地地块贴模模式、无纹理白模网格模式 4 种变体）。详见 [本批验证](customproptex-czml3dtiles-depthoffield-drapeimagery-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.68

新增 Custom Post Process、Custom Primitive - Dev、Custom Shaders 3D Tiles、Custom Shaders Models 四个完整案例，累计 **219 / 309**，剩余 90 个。128 项普通 Rust、220 项实际 GPU、8 组 Node、219 个浏览器 WebGPU 案例通过。验证包含自定义卷积核马赛克后处理阶段、步长解算与全屏后处理管线（16x16 卷积核马赛克基准：青色与紫罗兰模型像素化模糊阶段、高密度步长模式、反色对比度诊断模式、直通观察模式 4 种变体）、自定义底层图元 DrawCommand、VertexArray 与双波干涉动态曲面网格（双波平面驻波干涉基准：鸭翅绿与明金色波动能量分布、高频驻波模式、一维色彩纹理映射模式、四边形线框拓扑检查模式 4 种变体）、3D Tiles 自定义着色器、相机距离渐变色带与程序化动态纹理（相机距离着色基准：蓝绿平滑深度梯度过渡、动态程序化条纹模式、无光照纯净立面模式、高对比度景深分段蒙版 4 种变体）、以及 glTF 模型自定义着色器、程序化棋盘格、半透明梯度与顶点法向膨胀（自定义动画贴图基准：赛车红与动态流动条纹、程序化棋盘格蒙版模式、半透明对角透明度渐变、顶点法向位移膨胀模式 4 种变体）。详见 [本批验证](custompostprocess-customprimitive-customshaders-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.67

新增 Constant LOD - Dev、Custom DataSource、Custom Geocoder、Custom Per-Feature Post Process 四个完整案例，累计 **215 / 309**，剩余 94 个。127 项普通 Rust、216 项实际 GPU、8 组 Node、215 个浏览器 WebGPU 案例通过。验证包含 glTF 2.0 恒定细节层次网格与多贴图材质（基准棋盘格恒定细节层次、草地纹理恒定细节层次模式、自发光增强棋盘格模式、45度斜倾平面与法线贴图变换 4 种变体）、WebGL Globe 自定义数据源与人口密度柱体图谱（1990 年代基准人口密度序列、1995 年代城市人口增长序列、千禧年峰值人口序列、动态高度缩放倍率模式 4 种变体）、OpenStreetMap Nominatim 异步地理编码器与多边形范围解算（基准 Nominatim 查询：费城地名空间矩形包围盒解算、聚焦费城市政厅地标建筑模式、大都会扩展行政区界模式、高对比度地名检索锚点模式 4 种变体）、以及基于 `czm_selected()` 片元着色器逐要素后处理高亮（未选中基准模型与后处理舞台、鼠标悬停要素高亮模式、多图元联合高亮模式、后处理舞台色调诊断模式 4 种变体）。详见 [本批验证](constantlod-customds-customgeocoder-custompostprocess-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.66

新增 Classification Types、Client-side Snapping - Dev、Clipping Performance - Dev、Clipping Regions 四个完整案例，累计 **211 / 309**，剩余 98 个。126 项普通 Rust、212 项实际 GPU、8 组 Node、211 个浏览器 WebGPU 案例通过。验证包含多目标分类类型与材质覆写（基准双重分类：同时对地表地形与 3D Tiles 实景建筑进行多边形与折线空间覆写、仅分类地形模式：绿色与大地色调高亮多边形、仅分类 3D 瓦片网格模式：电光青色建筑分类填充、棋盘金纹理材质模拟模式 4 种变体）、三维瓦片网格边缘与表面客户端实时几何捕捉测距（仅边缘模式捕捉：黄色测量锚点与端点拾取、表面与边缘混合捕捉：青色表面垂足与黄色轮廓边缘、仅表面几何投影捕捉：橙色表面法线垂足对齐、显示瓦片包围盒调试荧光绿空间包围体积 4 种变体）、参数化多边形阵列开挖裁剪与距离纹理性能诊断（地形目标裁剪基准：深红与琥珀色圆形多边形阵列、3D Tiles 目标裁剪模式：电青色高精实景体量遮罩、距离纹理调试模式：高对比紫色有向距离场深度检视、高密度精密模式：黄金高精度参数化圆形边缘 4 种变体）、以及任意多边形地块开挖裁剪与拟建建筑瓦片集嵌合（3D Tiles 地块开挖基准：丹佛市中心地块裁剪与拟建大楼嵌合、地表地形开挖模式：祖母绿核心筒深基坑钻孔轴、反向裁剪模式：反转保留地块内部并剔除外部空腔、拟建建筑独立特写模式：高对比天际线建筑轮廓 4 种变体）。详见 [本批验证](classificationtypes-clientsnapping-clippingperf-clippingregions-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.65

新增 Cesium OSM Buildings、Cesium World Terrain、Clamp to 3D Model、Classification 四个完整案例，累计 **207 / 309**，剩余 102 个。125 项普通 Rust、208 项实际 GPU、8 组 Node、207 个浏览器 WebGPU 案例通过。验证包含 OpenStreetMap 全球建筑白模实景三维瓦片与元数据属性着色（丹佛市中心 OSM 3D 建筑白模基准全景、基于建筑要素高度元数据渐变着色模式、高层超高商业大厦条件过滤模式：高度 > 100米、线框拓扑与光照高对比材质诊断模式 4 种变体）、全球高保真地貌切片与水体法线光照反射解算（雷尼尔雪山与 Cesium World Terrain 全球地貌基准、水体遮罩高光与动态法线反光增强模式、高海拔地形法线阴影与山体坡度着色模式、火山口边缘地貌微地形与等高线高程网格模式 4 种变体）、三维模型表面实时贴地附着与点线面拓扑吸附（模型表面吸附传感器与巡检路径基准、高亮贴模表面法线垂足巡检点模式、沿模型几何轮廓表面贴合飞行走廊轨迹线模式、模型表面投影与结构基底包围盒贴合模式 4 种变体）、以及三维空间几何分类体图元与瓦片/地表分区覆写（城市地块空间几何分类基准、商业规划区琥珀色高亮多边形体分类模式、住宅区与生态绿地青绿双色空间分类模式、反转空间遮罩与重点工程冲突分类检视模式 4 种变体）。详见 [本批验证](osmbuildings-worldterrain-clampto3d-classification-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.64

新增 Aerometrex San Francisco、ArcGIS Tiled Elevation Terrain、ArcticDEM、Cesium Inspector 四个完整案例，累计 **203 / 309**，剩余 106 个。124 项普通 Rust、204 项实际 GPU、8 组 Node、203 个浏览器 WebGPU 案例通过。验证包含 Aerometrex 旧金山倾斜摄影实景三维与微米级街道网格（旧金山滨水区与标志性建筑基准全景、渡轮大厦街景增强 ~6mm/px 细节模式、39 号码头游艇港湾 2cm/px 高精度模式、泛美金字塔天际线与九曲花街折线透视 4 种变体）、Esri ArcGIS 全球瓦片高程切片服务与 LERC 压缩地貌解算（科罗拉多大峡谷真实地势高程切片基准、大峡谷南缘高原测绘基准标高点模式、峡谷底深槽科罗拉多河床水准面验证模式、全高程网格地面控制点高对比校准 4 种变体）、北极地理空间中心 ArcticDEM 2米极高精北极测绘地形与极地巡礼（迪纳利峰与阿拉斯加山脉极高海拔地势基准、北美最高峰麦金利峰 6190 米顶峰定位模式、安克雷奇楚加奇山脉与卡希尔特纳冰川干流模式、极地侧碛冰碛碎屑垄与辫状河床平原 4 种变体）、以及 Cesium Inspector 调试混入与图元深度测试（检视器多几何图元综合诊断基准、半透明红色 BoxGeometry 封闭外观图元检视、高空 100km 蓝色拉伸矩形几何体图元检视、广告牌图层深度缓冲遮挡与地形碰撞检视 4 种变体）。详见 [本批验证](aerometrex-arcgis-arcticdem-inspector-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.63

新增 Design Model – Category Filtering、Design Model – Metadata Styling、AEC Snapping、Design Model – Tileset Metadata Filtering 四个完整案例，累计 **199 / 309**，剩余 110 个。123 项普通 Rust、200 项实际 GPU、8 组 Node、199 个浏览器 WebGPU 案例通过。验证包含 Autodesk Revit 建筑构件 CategoryID 隔离过滤（展示所有构件分类基准、仅隔离展示墙体构件、仅隔离展示楼板构件、仅隔离展示核心筒楼梯构件、仅隔离展示出入口门构件、仅隔离展示照明灯具构件 6 种变体）、嵌入式机电元数据动态着色与双滑块风管长度过滤（高亮全范围 0m~50m HVAC 管线基准、仅高亮 0m~10m 末端送风散流管段、仅高亮 10m~25m 支路送风风管、仅高亮 25m~50m 主干送风管路 4 种变体）、BIM 构件混合式端点与边缘捕捉（边缘捕捉模式：青色悬停边缘/青柠客户端瞬时捕捉/红色服务器精化端点、表面捕捉模式：黄色表面悬停/表面捕捉端点、地形悬停模式：灰色光标反馈防漂移、全捕捉标定端点激活模式 4 种变体）、以及三维瓦片集并行元数据结构解析与构件单选拾取（所有分类子分类可见基准、CTRL+点击单选结构立柱、子分类过滤隐藏幕墙竖框、交互拾取隐藏屋顶与楼板 4 种变体）。详见 [本批验证](aec-isolate-metadata-snapping-filtering-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.62

新增 Vertical Exaggeration with 3D Tiles、Design Model – Layer Control and Metadata Picking、Design Model – Terrain Inset、Design Model – Elements API Query 四个完整案例，累计 **195 / 309**，剩余 114 个。122 项普通 Rust、196 项实际 GPU、8 组 Node、195 个浏览器 WebGPU 案例通过。验证包含谷歌实景三维高程垂直夸张参数调谐（3 倍地形垂直起伏夸张基准、5 倍极致高山峡谷夸张模式、1 倍真实物理尺度真实地形起伏基准与引入 1500 米非零相对高度基准面偏移 4 种变体）、Autodesk Revit 斯诺登大厦多专业协同 BIM 设计图层与属性拾取（建筑与幕墙图层基准激活、结构工程与暖通机电专业聚焦模式、七大专业全开与纯实景三维环境对比 4 种变体）、建筑基地 GeoJSON 多边形地形开挖与设计模型嵌合（设计方案嵌合基准、反向裁剪模式剔除全球瓦片仅保留基坑内部、仅保留地块贴地多边形模式与原始地表全貌 4 种变体）、以及 Cesium ion 资产构件级 Elements API 结构化检索与空间定位（所有墙体构件检索基准、承重主体结构检索模式、建筑门窗洞口构件过滤模式与空间规划过滤模式 4 种变体）。详见 [本批验证](3dtiles-verticalexag-aec-design-clipping-elements-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.61

新增 3D Tiles Point Cloud Styling、Global Rivers Vector Tiles with Property-based LOD、3D Tiles Split - Dev、3D Tiles Terrain Classification 四个完整案例，累计 **191 / 309**，剩余 118 个。121 项普通 Rust、192 项实际 GPU、8 组 Node、191 个浏览器 WebGPU 案例通过。验证包含点云声明式样式语言、逐点温度条件色谱、三角/几何坐标函数与动态点图元尺寸（温度二值红蓝阶跃着色基准、10 级温度条件色带与透明度通道混合模式、空间归一化三维坐标位置色彩渐变模式与动态宏定义点尺寸缩放 4 种变体）、全球 HydroRIVERS 矢量水系流速层级与属性 LOD 地形贴地渲染（1-10 级对数水系流量彩虹色带基准、干流主江加粗高亮模式、末端细小支流强化模式与单色灰度线宽分级 4 种变体）、14 种多格式 3D Tileset 滑动分屏对比与阴影检视（基准 50% 居中无分割全景视图、左侧视口单侧遮罩剖切模式、右侧视口单侧遮罩剖切模式与实例化 I3DM 瓦片集分屏与实时阴影投射 4 种变体）、以及维也纳城市矢量 3D Tiles 全球地形覆层分类与交互高亮（维也纳半透明白色地块贴地分类基准、鼠标悬停动态黄色要素高亮模式、绿蓝规划功能分区覆层多边形模式与关闭分类维也纳纯地形 4 种变体）。详见 [本批验证](3dtiles-pointcloudstyling-river-splitdev-terrainclass-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.60

新增 3D Tiles Picking Dev、3D Tiles Point Cloud、3D Tiles Point Cloud Classification、3D Tiles Point Cloud Shading 四个完整案例，累计 **187 / 309**，剩余 122 个。120 项普通 Rust、188 项实际 GPU、8 组 Node、187 个浏览器 WebGPU 案例通过。验证包含多数据源 3D Tiles 深度拾取与射线交汇测试（谷歌高精实景三维瓦片集射线拾取基准、Cesium OSM 建筑群向量几何特征拾取模式、Aerometrex 倾斜摄影表面精确拾取模式与纯深度缓冲光线投射探测 4 种变体）、哥伦比亚大学机器人实验室激光扫描点云与 PNTS 格式几何流式呈现（哥伦比亚实验室高精激光扫描点云基准、按激光反射强度与回波计数着色模式、高程梯度色彩映射模式与固定点尺寸强化呈现 4 种变体）、三维点云几何向量单体化分类与空间拓扑着色（基准红色半透明分类体量覆盖、绿色植被与树冠体量分类覆层、蓝色建筑物与结构设施分类覆层与关闭单体化分类纯激光点云视图 4 种变体）、以及基于视穹光照 EDL 与几何误差衰减的点云高品质着色（圣海伦斯火山点云基准启用 EDL 边缘着色、启用几何误差屏幕衰减模式、教堂高精激光扫描点云全功能光照衰减组合与经典直接光照基准模式 4 种变体）。详见 [本批验证](3dtiles-pickingdev-pointcloud-class-shading-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.59

新增 3D Tiles NGA GPM Visualization、3D Tiles Performance Testing、3D Tiles Photogrammetry、3D Tiles Photogrammetry Classification 四个完整案例，累计 **183 / 309**，剩余 126 个。119 项普通 Rust、184 项实际 GPU、8 组 Node、183 个浏览器 WebGPU 案例通过。验证包含 NGA GPM v1.2 高精网格规约、位置误差协方差椭球与 Maxar 全球数据源支持（NGA GPM 规约网格基准与 1-Sigma 误差空间体量、高精度位置不确定度误差椭球体模式、Maxar 高精地表高程网格剖面模式与严格 SSE 细化模式 4 种变体）、3D Tiles & Terrain 流式传输性能巡检、多视点相机飞行测试与几何误差热力图（8 秒默认巡航基准性能测试、瓦片几何误差属性热力图模式、高频相机快速跳跃压力模式与缓存高压及显存压力检验 4 种变体）、Aerometrex 城市倾斜摄影实景三维网格、PBR 纹理表面与全球地形无缝贴合（墨尔本实景三维倾斜摄影网格基准、物理光照阴影贴合、三角网几何线框增强模式与宏观城市天际线全景视角 4 种变体）、以及 B3DM 瓦片集对实景倾斜摄影三维分类覆层与半透明体量风格化（红色半透明分类体量覆盖基准、绿色功能区规划分类覆层、蓝色滨水空间分类覆层与关闭分类纯倾斜摄影视图 4 种变体）。详见 [本批验证](3dtiles-ngagpm-perf-photogrammetry-classification-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.58

新增 3D Tiles Gaussian Splatting Comparison、3D Tiles Inspector、3D Tiles Interactivity、3D Tiles Interior 四个完整案例，累计 **179 / 309**，剩余 130 个。118 项普通 Rust、180 项实际 GPU、8 组 Node、179 个浏览器 WebGPU 案例通过。验证包含双高斯溅射模型滑动分屏比对（50% 居中对称分割比对基准、25% 偏左滑块位置、75% 偏右滑块位置与关闭分屏单模型 4 种变体）、NYC 3D Buildings 瓦片集几何与包围盒检视器（层级随机着色与三角网线框基准、定向包围盒 OBB 与外接球调试显示、冻结瓦片动态请求并调试 SSE 几何误差与关闭调试覆层纯物理材质光照白模 4 种变体）、鼠标交互/属性拾取/动态 3D 标注文档（右键添加标注/中键隐藏构件基准交互、激活 3D 文字标注浮层、选择性建筑构件隐藏模式与重置所有构件可见性 4 种变体）、以及 San Miguel 经典室内高精度建筑模型室内漫游（室内天井走廊自然视角基准、室内自然天光增强与暖色调氛围漫游模式、拱门与立柱几何结构轮廓增强线框模式与俯视剖切鸟瞰透视 4 种变体）。详见 [本批验证](3dtiles-gscompare-inspector-interactivity-interior-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.57

新增 3D Tiles Feature Styling、3D Tiles Formats、3D Tiles Gaussian Splats with LOD、3D Tiles Gaussian Splatting 四个完整案例，累计 **175 / 309**，剩余 134 个。117 项普通 Rust、176 项实际 GPU、8 组 Node、175 个浏览器 WebGPU 案例通过。验证包含西雅图 Cesium OSM Buildings 建筑群声明式样式语言、材质条件分类与高度渐变着色（西雅图白模基准、建筑高度 HSL 连续色谱渐变、材质分类离散调色盘与高度阈值条件过滤 4 种变体）、3D Tiles 全格式多模态瓦片集、B3DM/I3DM/PNTS/CMPT 综合展示与检视器（B3DM 不透明基准、B3DM 半透明模型、PNTS 点云 RGB 着色与 I3DM 实例化矩阵 4 种变体）、层次细节空间八叉树 3D 高斯溅射辐射场、点预算约束与地表融合（全层级细节基准、高密度 400 万点、粗糙低细节预览与溅射半径放大 4 种变体）、以及实时 3D Gaussian Splatting 辐射场重构、协方差矩阵投影与前向深度排序（3DGS 真实辐射场基准、球谐函数色彩增益、低透明度调试与高对比轮廓增强 4 种变体）。详见 [本批验证](3dtiles-styling-formats-gaussiansplats-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.56

新增 3D Tiles Clipping Planes、3D Tiles Compare、3D Tiles Edge Visibility、3D Tiles Feature Picking 四个完整案例，累计 **171 / 309**，剩余 138 个。116 项普通 Rust、172 项实际 GPU、8 组 Node、171 个浏览器 WebGPU 案例通过。验证包含 3D Tiles 任意方向动态裁剪平面集合、轮廓描边边缘样式与多模型目标适配（基准单向内凹裁剪剖切、45度斜向剖切平面与霓虹绿色发光高亮描边、正交六面体剖面切割组合与法向量反向遮罩剖切 4 种变体）、墨尔本实景三维与 Cesium OSM Buildings 双视口滑动分割实时对比渲染（50% 居中对称分割比对基准、25% 偏左滑块位置、75% 偏右滑块位置与全屏纯实景摄影基准视图 4 种变体）、建筑构件边缘几何特征线与外轮廓剪影提取与四态显隐模式（基准 EDGES_AND_SILHOUETTES 开启模式、纯结构边缘线模式、纯外轮廓剪影模式与关闭轮廓纯 PBR 模式 4 种变体）、以及曼哈顿纽约建筑群悬停交互高亮、单机要素选中、属性表提取与属性浮窗联动（基准鼠标悬停绿色剪影轮廓与点击黄色高亮锁定、高对比红色选中模式、全量属性展示模式与清除选择状态 4 种变体）。详见 [本批验证](3dtiles-clipping-compare-edge-picking-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.55

新增 3D Native Vector Data、Offset 3D Tileset Height、3D Tiles Styling、BIM Design Model with 3D Tiles 四个完整案例，累计 **167 / 309**，剩余 142 个。115 项普通 Rust、168 项实际 GPU、8 组 Node、167 个浏览器 WebGPU 案例通过。验证包含 3D Tiles 原生三维矢量点/线/面几何、青色发光折线与地形无缝叠加（青色 3 像素发光矢量折线基准、高对比橙色矢量走廊、高亮选定矢量要素与隐藏矢量图层 4 种变体）、3D Tiles 瓦片集动态垂直高程偏移、地形深度测试与平移矩阵实时变换（零高程偏移自然基准、抬升 +100 米悬浮模式、下潜 -50 米基坑模式与地表深度遮挡测试 4 种变体）、3D Tiles 分层批次表继承结构、区域/建筑/门/门把手级联属性与正则样式（纯门构件橙色着色、门及子构件派生着色、所属建筑分色与正则匹配类名/高度样式 4 种变体）、以及 Bentley 工业热电厂 BIM 数字化交付设计模型、全局构件 ID 映射与要素级交互（工业电厂全景基准、逐要素交互选定、全景内部构件透视与 REPLACE 颜色替换模式 4 种变体）。详见 [本批验证](3dtiles-nativevector-adjustheight-hierarchy-bim-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.54

新增 CDB Metadata in 3D Tiles、3D Tiles Photogrammetry with KTX2、3D Tiles Photogrammetry Classification、S2 Globe with 3D Tiles 四个完整案例，累计 **163 / 309**，剩余 146 个。114 项普通 Rust、164 项实际 GPU、8 组 Node、163 个浏览器 WebGPU 案例通过。验证包含 3D Tiles 1.1 CDB 军规语义元数据、建筑高度分级着色与地貌基底分类（纯净 CDB 未着色基准、建筑高度梯度分段着色、地表基底材质分类着色与要素拾取元数据浮层 4 种变体）、3D Tiles 1.1 KTX2 压缩纹理实景三维倾斜摄影、双分屏比对与载入基准测速（AGI 宾州总部双视口对称分屏、墨尔本城市倾斜摄影、瓦片集重载耗时评测与单全屏视图 4 种变体）、旧金山渡轮大厦单体化分类、半透明窗体与 PBR 自定义材质覆写（构件要素分类着色、建筑玻璃半透明化、PBR 物理材质风格化与金光滤镜多要素集 4 种变体）、以及 Maxar One World Terrain 全球 S2 空间分区网格球体、包围盒调试与动态片元着色器（S2 全球基础网格地球基准、逐级调试 S2 包围盒、地表覆盖语义分类着色与程序化波纹着色器 4 种变体）。详见 [本批验证](3dtiles-cdbyemen-photogrammetry-classification-s2globe-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.53

新增 Imagery Color to Alpha、Imagery Cutout、Imagery Layers、Imagery Layers Manipulation 四个完整案例，累计 **159 / 309**，剩余 150 个。113 项普通 Rust、160 项实际 GPU、8 组 Node、159 个浏览器 WebGPU 案例通过。验证包含色度键抠像、指定色彩转透明与容差阈值调节（纯绿色海洋抠像基准、深蓝海洋透明抠像、黑色陆地抠像与零容差极值匹配 4 种变体）、地理矩形遮罩抠图、开窗透视与运动单瓦片视窗（太平洋动态移动开窗基准、赤道狭长观测窗口、北美大陆内嵌透视窗口与关闭剪裁窗口 4 种变体）、多图层混合合成、Black Marble 夜光图层与单瓦片覆盖（三图层复合基准、纯夜光微光遥感增强、单瓦片标识置顶与纯日间自然底图 4 种变体）、以及影像图层堆栈动态重排、上移下移与置顶置底控制（默认堆栈层级基准、红外气象置顶透视、夜间灯光置顶与极简单层底图 4 种变体）。详见 [本批验证](imagery-coloralpha-cutout-layers-manipulation-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.52

新增 Bathymetry、Bing Maps Labels Only、Azure 2D Tiles、Imagery Adjustment 四个完整案例，累计 **155 / 309**，剩余 154 个。112 项普通 Rust、156 项实际 GPU、8 组 Node、155 个浏览器 WebGPU 案例通过。验证包含全球深海海底地形拓扑、CMOCEAN 深度色谱与动态山体晕渲（-10000m 至 2000m 复合高程等深线带谱、纯深度色带渐变、500m 间隔等深线与关闭光照阴影平面基准 4 种变体）、分屏双视口 Bing Maps 影像与独立注记图层（对称分屏基准、隐藏右侧注记图层、左窄右宽注记聚焦与左宽右窄一体化航拍 4 种变体）、微软 Azure Maps 2D 瓦片地图服务与全球地形集成（费城都市核心区宏观概览、高反差山地真实地形起伏、中心城摩天楼微观特写与特拉华河航运走廊 4 种变体）、以及影像图层多通道实时色彩、明度与伽马参数调节（高饱和度鲜艳模式、单色高对比黑白胶片、暖金色夕阳滤镜与月球冷色调辐射衰减 4 种变体）。详见 [本批验证](bathymetry-binglabels-azure-adjustment-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.51

新增 Earth at Night、Natural Earth II、Sentinel-2、ArcGIS MapServer 四个完整案例，累计 **151 / 309**，剩余 158 个。110 项普通 Rust、152 项实际 GPU、8 组 Node、151 个浏览器 WebGPU 案例通过。验证包含 NASA Black Marble 2017 全球夜间灯光、晨昏线与时序动态光照（全球陆地夜光、晨昏蒙影分界线过渡、4000x 自转倍速漫游与背阳抑制模式 4 种变体）、Natural Earth II 全球地貌阴影晕渲、水体与水系汇水网络（地貌晕渲基准、水体汇水加重高亮、高反差山地晕渲与陆海掩膜岸线过渡 4 种变体）、欧空局 Sentinel-2 全球 10 米多光谱无云高分辨率影像（10 米真彩色合成基准、农业近红外假彩色、苏伊士/迪拜沿海高精工程形态与荒漠风成地貌 4 种变体）、以及 ArcGIS MapServer 瓦片服务多底图标准套件（Satellite 卫星影像、Hillshade 地貌晕渲、Oceans 海洋海床水深与 Topographic 地形综合 4 种底图变体）。详见 [本批验证](earthatnight-naturalearth-sentinel-arcgis-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.50

新增 Ambient Occlusion、Blue Marble、Camera Controllers、Camera Tutorial 四个完整案例，累计 **147 / 309**，剩余 162 个。110 项普通 Rust、148 项实际 GPU、8 组 Node、147 个浏览器 WebGPU 案例通过。验证包含屏幕空间环境光遮蔽 SSAO 后处理与裂缝自阴影渲染（高对比度 SSAO、柔和环境阴影、极深裂缝凹陷暗角与关闭后处理 4 种变体）、NASA Blue Marble Next Generation 全球卫星影像与地理生物圈基准（夏季植被、冬季积雪、赤道高分对焦与无云地形水深 4 种变体）、地表露天开采与地下井下工程模块化混合相机控制器（模块化平移、俯仰倾斜扫描、纵深穿透下潜与锁定枢轴环绕 4 种变体）、以及无人机第一人称视角走航与键盘鼠标飞行课程（WASD 前后平移、90度垂直俯视、信标塔战术环绕与低空加速走航 4 种变体）。详见 [本批验证](ambientocclusion-bluemarble-controllers-tutorial-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.49

新增 Lighting、Fog、Cloud Parameters、Clouds 四个完整案例，累计 **143 / 309**，剩余 166 个。108 项普通 Rust、144 项实际 GPU、8 组 Node、143 个浏览器 WebGPU 案例通过。验证包含动态多光源照明系统（太阳光、月光、相机手电筒、定制暖琥珀色平行光在 CesiumBalloon 航空气球、基座圆柱、半透明球与方块 4 种变体）、地表气象多高度层大气雾效与视距衰减（标准山地雾、极高海拔雪峰雾、山谷逆温浓雾与清朗晴空 4 种变体）、程序化积云三维形态参数化调谐系统（标准白积云、金橙日落云、风暴深灰雨云与高空扁平卷云 4 种变体）、以及宏观山地层云与低空天幕天气系统（喀斯喀特山脉低空云、高海拔翻腾云海、晴好疏散积云与阴霾风暴 4 种变体）。详见 [本批验证](lighting-fog-cloudparameters-clouds-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.48

新增 Shadows、Atmosphere、Fog Post Process、Globe Materials 四个完整案例，累计 **139 / 309**，剩余 170 个。108 项普通 Rust、140 项实际 GPU、8 组 Node、139 个浏览器 WebGPU 案例通过。验证包含地表、建筑、3D 模型与几何体动态阴影投射（Cesium Air、地面特种车辆、Cesium Man、了望塔、阴影测试建筑群、阴影立方体与球体在 Exton、半圆顶、珠峰与近地轨道 4 种变体）、地球高动态范围地面与天空大气散射系统（标准晴空、黄昏日落红移、火星稀薄扬尘与高动态 HDR 4 种变体）、基于深度缓冲的距离雾效后期处理管线（暗黑浓雾、晨间薄雾、工业烟雾与深海夜雾 4 种变体）、以及地形高程等高线/坡度/坡向复合材质库（高程等高线复合、坡度陡崖着色、坡向方位色谱与高密等高线 4 种变体）。详见 [本批验证](shadows-atmosphere-fog-globematerials-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.47

新增 Drawing on Terrain、Rotatable 2D Map、Polylines On Terrain - Dev、Elevation Band Material 四个完整案例，累计 **135 / 309**，剩余 174 个。108 项普通 Rust、136 项实际 GPU、8 组 Node、135 个浏览器 WebGPU 案例通过。验证包含真实地形交互式点线面标绘（地表贴地纯白折线、青蓝半透明多边形填充面与 2 具红色顶点图元 4 种变体）、二维平面旋转地图全向航向漫游（SCENE2D 模式下 Heading -45°~180° 旋转、覆盖矩形与 42°N 恒向线 4 种变体）、大峡谷复杂地形多路径贴地折线与地表分类面体（发卡弯主航迹、东部环线发光轨迹与峡谷底地表矩形 4 种变体）、以及喜马拉雅山脉多级海拔等高线带谱与梯度材质（4200m 基底、7000m 中高带、7500m 高海拔带与 8848m 珠峰冲顶面 4 种变体）。详见 [本批验证](drawing-rotatable-polylines-elevation-validation.md)。以下保留历史记录。

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

# 最新完整案例进度：0.0.21

本批新增 Box、Cylinder、Sphere、Ellipsoid、Wall、Ellipse - Dev 六个完整实体 Primitive 案例，累计 **24 / 309**，其余 285 项待完成。最新范围与证据见 [实体 Primitive 完整案例](primitive-solid-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.20

本批完整移植六个 Primitive 轮廓开发案例（Box、Cylinder、Sphere、Ellipsoid、Rectangle、Wall Outline - Dev），累计 **18 / 309**，其余 291 个待完成。最新范围与证据见 [Primitive 完整案例](primitive-outline-validation.md)。以下保留历史记录。

# 最新完整案例进度：0.0.19

本批完整移植六个案例：CZML Box、Wall、CZML Wall、Circles and Ellipses、CZML Circles and Ellipses、CZML Point。累计 **12 / 309**，其余 297 项待完整验收。最新范围与证据见 [墙体、椭圆与 Point 完整案例](wall-ellipse-point-validation.md)。以下保留各历史版本记录，最新共享报告以 0.0.19 为准。

# 最新进度：0.0.18

累计六个完整功能案例通过；本批新增三个案例及证据见 [椭球与 CZML 完整案例](ellipsoid-czml-validation.md)。共享验证报告以最新版本为准。以下为 0.0.17 当批记录。

# 三个完整 Sandcastle 案例（0.0.17）

本批完整移植并验收 **Box、Cylinders and Cones、Plane** 三个案例，固定上游 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。验收对象是三个 `main.js` 中的全部案例行为，不再用通用几何演示替代原案例。

| 原案例 | 原始内容全部保留 | 验收 |
| --- | --- | --- |
| Box | -114° / -107° / -100°，40°N，300 km；400×300×500 km 三个箱体；蓝色实心、红色 0.5 alpha 加黑边、黄色仅轮廓；全集合 zoomTo | 源码参数、GPU、桌面 90 帧、浏览器 WebGPU 通过 |
| Cylinders and Cones | -100° / -105°，40°N，200 km；400 km 长、200 km 半径；半透明 CSS GREEN 圆柱加黑边、红色零顶半径圆锥；全集合 zoomTo | 同上 |
| Plane | -114° / -107° / -100°，40°N，300 km；400×300 km；UNIT_X / UNIT_Y / UNIT_Z 三种平面；蓝色实心、红色 0.5 alpha 加黑边、黄色仅轮廓；全集合 zoomTo | 同上 |

三个脚本没有额外工具栏、动态数据源、外部模型或自定义时间事件。它们要求的实体、材质、轮廓与初始自动取景都已实现。渲染由共享 Rust/wgpu 执行，浏览器使用生成的 WASM，不嵌入 CesiumJS 渲染器。

## 直接测试

```bash
python3 scripts/start-test-ui.py
# http://127.0.0.1:8000/bindings/web/sandcastle.html
```

“完整案例移植”区域有三个独立按钮及“自动取景全部实体”。也可以在左侧目录搜索 Box、Cylinders and Cones 或 Plane，然后点击“运行当前完整案例”。实体选择、定位、显隐、删除、编辑、拾取和滚轮缩放均可继续使用。普通功能子集配方保持单独记录。

```js
await viewer.loadEntities('/tests/sandcastle/full/box.json');
viewer.zoomToEntities();                  // 全部当前可见且可用的实体
viewer.zoomToEntities(['box-1','box-2']);  // 可选 ID 子集
```

新增 Plane JSON 字段 `normal`、`distance`，位于 `shape`，遵循实体 ENU 坐标系中的 `normal·position + distance = 0`。normal 默认为 [0,0,1]，必须为有限单位向量；distance 默认为 0，范围 ±1,000,000 m。填充和轮廓共用与固定上游 `PlaneGeometryUpdater.createPrimitiveMatrix` 一致的切向基，旧水平 Plane 配方继续兼容。圆柱/圆锥面分段增至默认 128，轮廓圆环 128 段、16 条竖线。

集合取景按实际几何顶点的 ECEF 范围计算保守包围球，包含 fill=false 的轮廓实体；依据当前画布宽高比选择水平/垂直较窄视角并加余量，避免硬编码视距裁掉模型。隐藏和时间不可用实体不参与；显式未知 ID、空可见集合或非法视口原子报错，保留旧相机。该 API 是当前自定义 SDK 的同步接口，不声明兼容 CesiumJS zoomTo 的全部输入类型或 Promise 生命周期。它会终止旧飞行/跟随并使旧拾取失效。

桌面：

```bash
CESIUM_OFFLINE=1 CESIUM_ENTITIES=tests/sandcastle/full/plane.json \
  CESIUM_ZOOM_ENTITIES=1 cargo run --locked -p cesium-viewer --bin desktop-viewer
```

另两案例替换 JSON 文件名即可。CESIUM_ZOOM_ENTITIES 在实体加载与其他初始相机设置之后执行。

## 源码和证据

- `scripts/build-full-recipes.mjs` 在 Node VM 中执行三个固定 main.js 的全部语句，记录 Viewer、entities.add 与 zoomTo；没有省略分支或用另一个演示替代。仅移除 ES import，记录器只允许三个案例使用的类和字段。脚本校验目录中的 sourceSha256，生成完整实体 JSON；不是通用 CesiumJS 兼容层。
- `scripts/full-recipes-test.mjs` 独立重新执行原始脚本，将全部转换结果与交付 JSON 逐字段比较。名称、位置、尺寸、RGB/alpha、fill、outline、outlineColor、Plane 法向与距离、集合取景调用均有证据。
- `crates/cesium-viewer/tests/full-recipes-gpu.rs` 为每个案例执行独立真实 wgpu/Mesa 测试。横屏与竖屏投影检查所有顶点处于画幅；填充几何返回实体 ID；黄色轮廓内侧穿透；黑/黄色边线的实际像素与拾取正确；将 alpha=0.5 改为 1 后超过 100 个像素改变。导出 `docs/full-*-preview.png`。
- `docs/full-*-smoke.log`：三个桌面案例各 90 帧，Entities 3 / 2 / 3，errors 0。
- `scripts/full-recipes-browser.cjs` 使用真实 Chromium 155、Rust WASM 和 WebGPU。验证全部原始实体参数、实际 canvas 像素颜色、实体点击拾取、滚轮缩放、resize 后取景、目录中的完整案例入口与 destroy。报告 `docs/full-recipes-browser.json` 关联源文件和 JSON 的 SHA-256。
- 浏览器使用显式 **Vulkan / SwiftShader**，并将外部影像响应固定为本地 NASA 底图纹理，避免网络改变结果。该测试不验证实时影像服务、用户设备驱动或所有浏览器。默认后端仍可复现设备丢失；保留 Instance/Adapter 句柄是生命周期保护，尚不等于完成设备自动恢复。
- `scripts/record-full-recipe-results.py` 只有在源/数据哈希、GPU、浏览器与桌面证据匹配时，才将 `full-recipes.json` 对应条目提升为 functional-passed。笔记不会改变通过数；重新生成配方会回到待验收状态，需重新执行验证。

全工作区 **74 个普通 Rust 测试、30 个实际 GPU 测试、八组 Node 测试通过**；原生/WASM 构建、绑定生成、fmt/clippy 通过。浏览器专项三个案例通过；历史通用 `test:web` 脚本不计为本次全部通过。

## “完整”的边界

完整功能案例通过数从 **0 增至 3**；目录仍为 309 个，其他 306 个尚未完成完整案例验收。19 个历史功能子集关联继续单独保留。

本次完成三个固定脚本的全部案例行为；不声明与官方像素完全一致，不包含完整 Viewer 通用控件（例如地名搜索、所有底图选择）、CesiumJS API 二进制/对象兼容或全部渲染器特性。光照、底图清晰度、抗锯齿和透明排序仍可能与官方有视觉差异；复杂相交透明体、OIT、设备恢复等缺口仍存在。图形案例完成不代表其他使用相同 API 的复杂案例也完成。

原始源码与官方入口：[Box](https://sandcastle.cesium.com/?id=box)、[Cylinders and Cones](https://sandcastle.cesium.com/?id=cylinders-and-cones)、[Plane](https://sandcastle.cesium.com/?id=plane)。
