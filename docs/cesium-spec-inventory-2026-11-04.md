# CesiumJS Spec 测试对照清单（2026-11-04）

该清单按本地 Cesium 源码的 `packages/*/Specs/**/*Spec.js` 枚举，追踪每个上游 Spec 文件；文件不等于单个测试用例。“待逐项核对”不表示 Rust 没有相应测试；“部分”也不表示该文件已全部移植。与 Sandcastle 309 案例验收分开统计。

本地上游 HEAD：`77d30241863d30f299e7271a22f95b7d589c1c75`。Rust 工程对照旧案例基线 `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`；逐项移植时先冻结对应版本及源码哈希。

共 **746 个 Spec 文件**：core 141、engine 539、gltf 26、widgets 40。本清单不包括 packages 外的 Specs、端到端测试、构建脚本测试和非 `*Spec.js` 测试；它们也需要单独盘点。

本次补充 `cesium-math/tests/cesium_reference.rs`，参考 CartographicSpec 的 WGS84 参考点、地表上下高度，以及 IntersectionTestsSpec 的射线场景；仅覆盖局部语义。原有 Rust `ray_ellipsoid` 返回首个非负表面交点；新增 `ray_ellipsoid_interval` 支持自定义椭球及 Cesium 区间语义。新接口不隐式归一化方向，使用单位方向才能按距离解释参数；其余 IntersectionTests 仍待移植。

另补充 BoundingSphere 的平移、缩放、点距离和投影区间检查，并实现 `distance_squared_to`、`compute_plane_distances`；`cesium-viewer/tests/clock_cesium_reference.rs` 覆盖时钟边界、循环、暂停和继续。后者以显式 elapsed 秒驱动，尚不具备 Cesium 的 JulianDate、ClockStep 或 onEnd 事件；对应 Spec 仍为部分覆盖。

进一步增加 `cesium-math/src/intersections.rs` 和 `tests/intersection_spec.rs`，实现并对照射线与平面、射线/线段与三角形、射线/线段与球体的交点。射线参数使用调用方给出的方向，线段/球体区间使用从线段起点起算的距离。其他相交测试、退化输入的 JS 异常语义及剩余功能仍需逐项核对。

后续补充线段与平面、三角形与平面切分，以及射线所在直线与轴对齐包围盒的区间。三角形六种前后分布核对索引顺序、平面两侧及面积；包围盒的平行轴显式处理以避免除零产生 NaN，返回区间可位于起点之后或之前。零方向和无效包围盒由 Rust 返回 `None`，不声称与 JS 异常/NaN 行为完全一致。

新增二次和三次多项式判别式与实根接口，按上游 Spec 验证重根、降阶、复根与消减误差。三次 `a*x³+d` 保留 Cesium 返回三个相同值的约定；`x*(x²+1)` 则修正上游 `roots.Length` 拼写导致的无效值，返回唯一实根零。Rust 非有限输入返回空集合，JS 缺失参数错误语义由类型系统约束；未宣称这些 Spec 全部通过。

四次多项式已实现判别式与 original/Neumark 求根路径；对照上游重根、不同实根、复根、退化及三组稳定性用例，并验证整数根组合。JS 缺失参数异常与非有限输入仍按 Rust 接口语义分别处理，不能将文件直接标为全部等价。

已新增椭球掠射最低高度接口与球体/WGS84 参考测试。曾发现 WGS84 第二个上游参考点的 X 分量相差约 0.125 mm；通过本地 Cesium 中间数据对照，定位为向量归一化的逐分量除法与倒数相乘产生不同舍入，并在四次求根中放大。现已调整计算顺序并恢复 `wgs84_reference_cases` 的上游逐分量容差测试；其余相交 Spec 仍需逐项核对。

新增 `cesium-math::Ellipsoid`，支持自定义半径、缩放空间、测地法线、双向坐标转换、测地/地心表面投影、旋转椭球法线与 Z 轴交点、局部曲率及矩形范围表面积；上游参考点、近地心、三轴椭球与独立面积积分回归见 `ellipsoid_spec.rs`。零半径保留可构造语义，但需要逆半径的投影返回 `None`；面积采用上游的十点 Gauss-Legendre 近似，Rust 接口验证角度范围。

新增 `GeoRectangle` 地理矩形与 `rectangle_spec.rs`，覆盖角点、宽高、中心、点集构造、跨日期变更线的交并/包含、子区域以及椭球采样。构造时验证坐标范围；空点集返回 `None`，与 JS 对空数组生成极值矩形的行为不同。矩形其他构造方法、pack 及错误语义仍需核对。

已补从包围球计算矩形和通用椭球 ENU 变换，并核对上游中心、赤道与极点用例；对非旋转三轴椭球保留上游以位置计算东向的约定，其 ENU 轴不一定正交。用于包围球采样时按上游单独归一化东向和北向。

包围球增加点集（Ritter/包围盒候选）、带步长顶点、高低位编码、角点、椭球、多球、二维/三维矩形、定向盒和变换构造，以及 union、expand、平面分类和 `project_to_2d`；测试包含地球尺度点集的包含性及上游投影参考。极点额外提供东向降级以避免上游零向量归一化失败。定向盒和变换构造均检查对角线以容纳剪切轴，剪切时可能比上游列长度公式得到更大的保守球。Rust 对无效步长、截断和非有限点返回错误，编码数组长度不匹配也返回错误，与上游空球回退行为的差异待 API 兼容层明确处理。其余断言仍待核对。

新增 `GeographicProjection`，使用椭球最大半径进行正反投影；覆盖上游投影、反投影和单位球用例。Rust 对零半径椭球与非有限坐标显式拒绝，JS 参数异常由类型与结果处理表达。

新增 `WebMercatorProjection`、`GeographicTilingScheme` 和 `WebMercatorTilingScheme`，对应测试见 `cesium-math/tests/tiling_spec.rs`：覆盖极区裁剪、投影往返、默认根网格、四角和边缘瓦片、跨日期变更线、自定义墨卡托范围与原生坐标转换。跨日期变更线瓦片经度归一化和层级溢出返回 `None` 的行为需与上游逐项对照；JS 结果对象复用、异常行为以及其他测试仍待核对。这些公开数学接口尚未接入 viewer 现有影像瓦片路径。

两类瓦片方案现在实现公共 `TilingScheme` trait，投影实现 `MapProjection` trait；泛型测试验证接口、投影往返、相邻边界和自定义范围外的坐标。Rust 以返回值替代 JS 可选 result 参数，接口一致性不表示每条上游断言都已移植。

新增 `AxisAlignedBoundingBox`，包括角点和点集构造、显式中心、相等比较、平面分类与盒间相交；上游正常输入及接触边界的测试见 `axis_aligned_bounding_box_spec.rs`。Rust 的值复制和必填参数替代 JS 的 result/undefined 语义，非有限点返回错误；因此仍列为部分覆盖。

新增 `Plane`：点法式、四维系数、点距和投影、逆转置矩阵变换，并提供射线、包围球和包围盒的适配方法；`plane_spec.rs` 覆盖上游参考与非均匀缩放、平移及奇异变换。参数缺失和 result 对象复用由 Rust 类型/值语义替代，非有限输入与不可逆矩阵返回错误；尚需逐条核对其他异常行为。

新增数学核心标量与基础代数模块（`cesium-math`）：
- `core_math` 及 `math_spec.rs`：对齐 CesiumJS `Math.js` 的常数（EPSILON1..20）、`sign`、`signNotZero`（含 -0.0 语义）、`toSNorm`/`fromSNorm`、`asinClamped`/`acosClamped`、`chordLength`、`negativePiToPi`/`zeroToTwoPi`、`equalsEpsilon`（相对/绝对双容差）。
- `cartesian` 及 `cartesian_spec.rs`：对齐 `Cartesian2`、`Cartesian3`、`Cartesian4` 的行/数组构造、pack/unpack、基本四则、点乘叉乘、夹角、正交轴选择、投影、WGS84 椭球地表投影、Color 互转与插值。
- `color` 及 `color_spec.rs`：对齐 `Color` 预定义色表、RGBA 数值与字节互转、CSS 十六进制与 rgb/rgba 解析、`brighten`/`darken` 与打包。
- `matrices` 及 `matrix_spec.rs`：对齐 `Matrix2`、`Matrix3`、`Matrix4` 行主序构造与列主序索引、矩阵与点/向量乘法、方向变换（`multiply_by_direction`）、行列式、逆变换及仿射逆矩阵（`inverse_transformation`）。
- `quaternion`、`scalar` 及 `quaternion_hpr_spec.rs`、`near_far_scalar_spec.rs`：对齐 `Quaternion` 轴角/旋转矩阵互转与 slerp 插值；对齐 `HeadingPitchRoll` 欧拉角四元数转换；对齐 `HeadingPitchRange` 与 `NearFarScalar` 的构造、比较与序列化打包。
- `time_constants`、`julian_date`、`time_interval` 及 `julian_date_spec.rs`、`time_interval_spec.rs`：对齐 `TimeConstants` 常量表、官方 28 条历次闰秒表（`LEAP_SECONDS`）、UTC/TAI 真实双向闰秒补偿转换（`JulianDate`）、天文算法年月日与儒略日高精互转（`GregorianDate`、ISO 8601 解析与输出）、`TimeInterval` 区间包含与相交测试（`intersect`、端点开闭与判空）。
- `encoded_cartesian` 及 `encoded_cartesian3_spec.rs`：对齐 `EncodedCartesian3` 高精度拆分编码（`encode` 65536 阶跃）、从 `Cartesian3` 生成高低位向量及向顶点属性交错写入浮点数组（`write_elements`）。
- `geometry`、`vertex_format`、`box_geometry` 及 `vertex_format_spec.rs`、`box_geometry_spec.rs`、`box_outline_geometry_spec.rs`：对齐 `VertexFormat` 顶点属性位掩码与常量/打包解包；对齐几何属性缓冲（`GeometryAttribute`、`Geometry`、`PrimitiveType`）；对齐 `BoxGeometry`（仅位置 8 顶点与全属性 24 独立顶点网格生成）和 `BoxOutlineGeometry`（12 边线框图元）。
- `plane_geometry`、`ellipsoid_geometry`、`sphere_geometry` 及 `plane_geometry_spec.rs`、`sphere_geometry_spec.rs`、`ellipsoid_geometry_spec.rs`：对齐 `PlaneGeometry`（4 顶点 2 三角形网格、法线/切线/纹理属性）与 `PlaneOutlineGeometry`（4 边线框）；对齐 `EllipsoidGeometry`（参数化三轴椭球分片堆栈划分、三角形网格与法线/纹理属性生成）与 `EllipsoidOutlineGeometry`（经纬度分段线框）；对齐 `SphereGeometry` 与 `SphereOutlineGeometry`（等半径球体及其线框图元）。
- `cylinder_geometry` 及 `cylinder_geometry_spec.rs`：对齐 `CylinderGeometry`（参数化圆柱/圆台圆周分片、侧面与上下底面三角形索引、退化参数校验）与 `CylinderOutlineGeometry`（顶底圆周与垂直线段线框图元生成）。
全部通过 `cargo test -p cesium-math` 真实测试，Clippy 无警告，支持 `wasm32-unknown-unknown` 编译。

新增 `EllipsoidTangentPlane`，支持由原点或点集构造、径向和法向投影、批量投影及径向投回椭球；`ellipsoid_tangent_plane_spec.rs` 覆盖上游单位球和 WGS84 往返用例。非有限输入使用 `Option`/`Result`，空点集或无法投影到地表时构造失败；上游零半径椭球用例及对象复用行为尚未等价验证。

新增 `BoundingRectangle`，支持点集与地理投影构造、union、expand、边界相交及带偏移 pack/unpack，测试见 `bounding_rectangle_spec.rs`。跨日期变更线沿用上游仅投影西南与东北角的规则，可能得到负宽度；Rust 对非有限点与数组越界返回错误，JS 可选参数和对象复用语义仍需单独映射。

新增 `Ray` 和通用 `Interval` 接口，并将射线与现有平面、球体、椭球、包围盒、三角形相交连接；`ray_spec.rs` 对照构造、方向归一化及正负参数。Rust 拒绝非有限输入，使用值复制替代 JS result/undefined 语义；这些差异仍须纳入兼容层。

新增 `EllipsoidalOccluder`，覆盖点和缩放空间可见性、椭球下方收缩、点集/带步长顶点/矩形的地平线剔除点；`ellipsoidal_occluder_spec.rs` 对照可见性、地下和退化方向等用例。要求正半径和显式相机，顶点截断返回 `None`；余下上游数值与错误语义仍待逐项移植。viewer 地形与影像仍使用各自的近似地平线判断，尚未接入本实现，亦未进行渲染或性能验证。

补充上游多点和矩形地平线相切测试，以球面射线最近距离独立验证剔除点，并核对带偏移的七分量顶点与点集结果一致；顶点处理改用迭代器，避免构造临时点数组。该改动尚无性能基准数据。

新增球形 `Occluder`，支持相机更新、点/球可见性及 none/partial/full 分类；`occluder_spec.rs` 对照上游遮挡、相交和较大被遮挡球的用例。相机在遮挡体内且被遮挡球更大时，保留上游两种查询方法的不同结果。已补充点集和矩形的剔除点计算及主要参考用例；退化输入、JS 对象/错误语义及其余断言仍需核对，viewer 尚未接入。

新增 `CullingVolume` 与 `BoundingVolume` trait，支持包围球、轴对齐包围盒、球构造的六平面及父节点掩码，测试见 `culling_volume_spec.rs`。掩码沿用上游只记录前 31 个平面的约定；超过 31 个平面时无法表达后续平面的相交状态，层级裁剪需避免将该结果当作完整的内部判定。

新增 `PerspectiveFrustum` 和 `PerspectiveOffCenterFrustum`：六裁剪平面、OpenGL 深度范围的有限/无限远投影、偏移、屏幕误差分母及像素尺寸；裁剪测试现通过视锥生成平面。矩阵不直接适用于 wgpu 的深度约定；现有 viewer 仍使用独立反向 Z 相机，尚未接入。已补透视视锥 pack/unpack 和四类视锥的近似比较；JS 可选属性/错误语义及其余上游用例待核对，Rust 对退化视锥与非正交相机方向采用更严格验证。

新增 `OrthographicFrustum` 和 `OrthographicOffCenterFrustum`，覆盖正交投影、非对称边界、六裁剪面、像素尺寸及正交视锥 pack/unpack；`orthographic_frustum_spec.rs` 核对投影角点、各平面、近似比较与输入边界。JS 对象/异常语义仍待核对；OpenGL 投影尚未接入 viewer 的 wgpu 相机。

新增 `OrientedBoundingBox`，支持矩阵转换、八角点、平面分类、正交轴（含退化轴）点距、投影区间和打包，并实现 `BoundingVolume` 以供裁剪体使用；`oriented_bounding_box_spec.rs` 验证旋转、退化与矩阵往返。点集 PCA 构造采用上游协方差和 Jacobi 主轴，已测试上游轴顺序、旋转平移与地球尺度线状点集。地理矩形构造已覆盖高度、跨日期变更线和宽度超过半球的密集采样；Rust 要求正半径和有序高度。遮挡接口使用可包围剪切盒的球；点距对剪切轴返回 `None`，大量上游断言仍待移植。

`cesium-formats` 新增 b3dm 20/24 字节历史头兼容和旧 `BATCHID` 语义规范化；使用上游 `B3dmParserSpec` 的两份真实样例验证容器、10 个 feature ID 和元数据的完整 CPU 解码，并测试无效版本、空内容及截断。已补充二进制 Batch Table 的标量、向量和矩阵分量解码，使用上游真实样例验证 feature 属性，并检查长度、对齐及解码预算。元数据保留大小估算现与几何及图像合计，图像解码使用扣除元数据后的预算；该估算不包含分配器和解析过程的临时峰值。样例来源记录在 `tests/fixtures/cesium-legacy-b3dm-source.md`。上游弃用告警、WebGL 渲染、层级元数据与属性修改等仍未覆盖。

新增独立 `parse_pnts`，解析无压缩 PNTS 的分段、坐标（含量化）、RTC、颜色/法线数据范围和 Batch Table；`pnts_spec.rs` 使用上游 RGB、量化法线和批次样例验证坐标及元数据，并核对必需字段、截断和批次索引。已增加 RGB/RGBA/RGB565/常量颜色、浮点/八面体法线及批次索引逐点读取。支持位置、颜色、法线及批次 ID 的 JSON 内联数组，解析时转换为二进制；Rust 对内联整数范围采用严格验证。`decode_point_cloud` 输出 f32 局部点与双精度原点、元数据，并合计估算内联数据、元数据与点数组预算；估算不包含解析临时峰值。原生及 Web viewer 的瓦片加载现已按 PNTS 魔数分派到独立解码路径，wgpu 以实例化四边形绘制点并支持批次 ID 拾取；默认单像素，有法线时使用 Cesium 的 Lambert 强度与 0.4 环境下限，使用瓦片矩阵乘双精度 RTC 平移，不经过 glTF 的 Y-up 转换；新增上游批次样例顶点映射和真实 GPU 点拾取测试。当前并非完整点云渲染：Draco、EDL、逐点样式表达式和分类等仍待实现；`decode_content` 仍不接收 PNTS；原生和 Web 的独立文件入口已分派至 `decode_point_cloud`，并以原始批次样例进行原生 GPU 场景测试。量化局部坐标在加入大偏移前解码，避免加减偏移导致局部细节丢失。点云上传暂会构造一份通用顶点及索引缓冲，峰值 CPU 内存和加载成本需要后续优化。

点云尺寸支持 `PointCloudOptions`：默认 `point_size=1`、`attenuation=false`、`geometric_error_scale=1`；启用衰减时按透视投影和点到相机的深度计算像素大小，几何误差依次取正的瓦片误差、`base_resolution` 或点集包围盒体积估计，限制在 1 像素至 `maximum_attenuation`。场景的 `set_point_cloud_options` 更新已驻留和以后加载的点，Web 暴露 `setPointCloudOptions`（camelCase JSON）和 `pointCloudOptions` 读取接口。支持上游默认值 `normalShading=true`、`backFaceCulling=false`，有法线时按眼坐标法线 Z 分量剔除背面点，无法线数据时不剔除。默认最大值对瓦片取场景 `max_sse`、独立点云取 1。原生 GPU 测试验证固定大小、距离衰减、像素颜色、法线着色切换、双视角背面剔除、缺少法线、中心裁剪及拾取；浏览器仅编译，尚无真实 WebGPU 图像验证。与上游仍有差异：像素比、2D/正交相机、ADD refinement 默认最大值与尺寸小于 1 的语义尚未实现；误差缺省估计对瓦片变换使用保守比例，和上游变换公式不完全一致。当前实例化仍保留通用索引缓冲和 CPU 展开开销，尚无性能基准。`PointCloudShadingSpec.js` 的 EDL 及设备能力判断尚未移植，不能标记为全部通过。

新增 `3DTILES_draco_point_compression` 扩展支持，接入纯 Rust Draco 解码器（原生及 WebAssembly 均无需本地 C++ 工具链依赖）。支持解压 Draco 编码的位置（POSITION）、法线（NORMAL）、颜色（RGB/RGBA）及批次标识（BATCH_ID），同时支持 Batch Table 中 Draco 压缩的标量与向量元数据属性解压。解码过程严格执行点数上限与重构内存预算限制，防御解压膨胀与截断畸形数据。`pnts_spec.rs` 使用上游真实 Draco 样例（完整 Draco、部分 Draco、带批次元数据 Draco 以及损坏流）验证了坐标精度、法线、颜色及元数据映射；原生加载入口与渲染场景已验证 Draco 批次点云的端到端加载、实例化绘制与基于批次属性的拾取。目前点云逐点样式表达式、EDL 与分类仍待后续实现。

新增 2D/3D 三角形重心坐标、点在三角形内判定与 2D 求交模块（`cesium-math`）：
- `barycentric_coordinates.rs` 及 `barycentric_coordinates_spec.rs`：对齐 CesiumJS `barycentricCoordinates`，支持 2D（`Cartesian2`）与 3D（`Cartesian3`）重心坐标计算，支持 `BarycentricCoordinates` 泛型 trait，覆盖顶点重合快速判定、各边中点插值、三角形内部测试、共线退化返回 `None` 以及极大尺度等长边数值稳定性测试。
- `point_inside_triangle.rs` 及 `point_inside_triangle_spec.rs`：对齐 CesiumJS `pointInsideTriangle`，严格判断点是否位于三角形内部（排除顶点与边线接触），支持 2D 与 3D 泛型调用。
- `intersections_2d.rs` 及 `intersections_2d_spec.rs`：对齐 CesiumJS `Intersections2D`，支持轴对齐阈值 2D 三角形几何切割（`clip_triangle_at_axis_aligned_threshold`，精确生成多边形逆时针顶点序列与边插值距离比例），支持快速 2D 三角形重心坐标计算（`compute_barycentric_coordinates`），以及两线段求交点计算（`compute_line_segment_line_segment_intersection`，覆盖相交、顶点接触、平行与重合线段检测）。
- 相应 40 项单元测试全部通过，workspace Clippy 静态检查零警告，通过 `wasm32-unknown-unknown` 构建。

新增通用插值与多项式逼近算法模块（`cesium-math`）：
- `linear_approximation.rs` 及 `linear_approximation_spec.rs`：对齐 CesiumJS `LinearApproximation`，支持多步长一阶线性逼近与 0 阶插值（`interpolate_order_zero`），覆盖步长合法性检查、除零错误防御以及 STK Components 对应数值校验。
- `lagrange_polynomial_approximation.rs` 及 `lagrange_polynomial_approximation_spec.rs`：对齐 CesiumJS `LagrangePolynomialApproximation`，支持任意次数 Lagrange 多项式插值、所需数据点计算（`get_required_data_points`），在航天轨道高精基准数据点下达到 $10^{-15}$ 级数值一致性。
- `hermite_polynomial_approximation.rs` 及 `hermite_polynomial_approximation_spec.rs`：对齐 CesiumJS `HermitePolynomialApproximation`，支持包含导数信息的 Hermite-Newton 均差商三角插值表构造（`fill_coefficient_list`）、递归多项式导数展开项（`calculate_coefficient_term`），支持 0 阶插值与任意高阶导数输出插值（`interpolate`），并在 `core_math.rs` 中补齐 `factorial` 阶乘与 `increment_wrap`。
- 相应 11 项单元测试全部通过，workspace Clippy 静态检查零警告，通过 `wasm32-unknown-unknown` 构建。

新增仿射变换组合、地形高程垂直夸张、序列容差去重与二分查找核心算法（`cesium-math`）：
- `translation_rotation_scale.rs` 及 `translation_rotation_scale_spec.rs`：对齐 CesiumJS `TranslationRotationScale`，包含平移（`Cartesian3`）、旋转（`Quaternion`）、缩放（`Cartesian3`）分量，支持默认单位值、相等比较及生成仿射 `Matrix4` 变换矩阵。
- `vertical_exaggeration.rs` 及 `vertical_exaggeration_spec.rs`：对齐 CesiumJS `VerticalExaggeration`，支持基准高程缩放（`get_height`）与 WGS84 椭球笛卡尔空间坐标的高程垂直夸张（`get_position`），覆盖地表上下基准高程偏移与近地心奇异回退。
- `array_remove_duplicates.rs` 及 `array_remove_duplicates_spec.rs`：对齐 CesiumJS `arrayRemoveDuplicates`，支持相邻元素的 $\varepsilon$-容差比较去重、首尾闭合线段环绕去重（`wrap_around`）及被移除索引序列记录（`removed_indices`）。
- `binary_search.rs` 及 `binary_search_spec.rs`：对齐 CesiumJS `binarySearch`，支持自定义比较器，提供返回命中索引或按位取反（`~index`）插入位置的 Cesium 规范语义及 Rust 标准 `Result<usize, usize>` 接口。
- 相应 22 项单元测试全部通过，workspace Clippy 静态检查零警告，通过 `wasm32-unknown-unknown` 构建。

新增空间索引与 GPU 顶点缓存优化模块（`cesium-math`）：
- `hilbert_order.rs` 及 `hilbert_order_spec.rs`：对齐 CesiumJS `HilbertOrder`，支持 2D 希尔伯特空间填充曲线正向编码（`encode_2d`）与反向解码（`decode_2d`），支持任意层级（1..31）与超大整数索引计算，严格处理层级与坐标越界错误。
- `tipsify.rs` 及 `tipsify_spec.rs`：对齐 CesiumJS `Tipsify`，支持根据 2007 SIGGRAPH 经典论文算法进行后顶点着色器（post-vertex-shader）缓存优化（`tipsify`），自动维护一环邻域、死胡同栈回退与游标扫描；支持计算平均缓存未命中率（`calculate_acmr`），在大规模网格三角形拓扑重排序中显著降低 GPU 顶点着色器 ACMR。
- 相应 12 项单元测试全部通过，workspace Clippy 静态检查零警告，通过 `wasm32-unknown-unknown` 构建。

后续逐个登记：上游测试名称和源码哈希、Rust 测试路径与名称、适用平台、通过证据、未实现能力及差异。图形测试须在原生 wgpu 与浏览器 WebGPU 上保留截图、参数和设备信息；上游 WebGL 实现细节需按可观察行为验证。

| 上游 Spec 文件 | 状态 |
| --- | --- |
| [packages/core/Specs/AssociativeArraySpec.js](../../cesium/packages/core/Specs/AssociativeArraySpec.js) | 已移植：关联数组键值映射、数组高效迭代、增删查改、清空与键存在性校验（associative_array_spec.rs） |
| [packages/core/Specs/AttributeCompressionSpec.js](../../cesium/packages/core/Specs/AttributeCompressionSpec.js) | 已移植：八面体法线压缩与解压（2分量/4分量/float/双float打包）、纹理坐标压缩、ZigZag Delta 解码、RGB8/RGB565 编解码、泛型解量化 dequantize（attribute_compression_spec.rs） |
| [packages/core/Specs/AttributeTypeSpec.js](../../cesium/packages/core/Specs/AttributeTypeSpec.js) | 已移植：顶点与3D Tiles属性类型（标量/向量/矩阵）、GLSL类型名映射、分量计数及着色器位置数量（attribute_type_spec.rs） |
| [packages/core/Specs/AxisAlignedBoundingBoxSpec.js](../../cesium/packages/core/Specs/AxisAlignedBoundingBoxSpec.js) | 部分：构造、点集、相等、平面及盒相交；JS 对象复用及异常语义不同 |
| [packages/core/Specs/BoundingRectangleSpec.js](../../cesium/packages/core/Specs/BoundingRectangleSpec.js) | 部分：构造、投影、合并、扩展、相交、打包；JS 对象复用和异常语义不同 |
| [packages/core/Specs/BoundingSphereSpec.js](../../cesium/packages/core/Specs/BoundingSphereSpec.js) | 部分：点/顶点/编码/多球/矩形/定向盒/变换构造、合并、平面、距离、projectTo2D；其余待核对 |
| [packages/core/Specs/BoxGeometrySpec.js](../../cesium/packages/core/Specs/BoxGeometrySpec.js) | 已移植：最小最大角点构造、POSITION_ONLY 8 顶点优化网格、ALL 全属性 24 独立顶点与法线/纹理坐标（box_geometry_spec.rs） |
| [packages/core/Specs/BoxOutlineGeometrySpec.js](../../cesium/packages/core/Specs/BoxOutlineGeometrySpec.js) | 已移植：线框顶点与 12 边索引缓冲生成、尺寸与 AABB 构造（box_outline_geometry_spec.rs） |
| [packages/core/Specs/Cartesian2Spec.js](../../cesium/packages/core/Specs/Cartesian2Spec.js) | 已移植：构造、打包、四则运算、点叉乘、模长、距离、归一化、夹角与插值（cartesian_spec.rs） |
| [packages/core/Specs/Cartesian3Spec.js](../../cesium/packages/core/Specs/Cartesian3Spec.js) | 已移植：构造、打包、四则运算、点叉乘、模长、距离、正交轴、投影、WGS84 椭球转换（cartesian_spec.rs） |
| [packages/core/Specs/Cartesian4Spec.js](../../cesium/packages/core/Specs/Cartesian4Spec.js) | 已移植：构造、打包、四则运算、模长、Color 互转与插值（cartesian_spec.rs） |
| [packages/core/Specs/CartographicSpec.js](../../cesium/packages/core/Specs/CartographicSpec.js) | 部分：WGS84 参考点及正负高度；其余待核对 |
| [packages/core/Specs/CatmullRomSplineSpec.js](../../cesium/packages/core/Specs/CatmullRomSplineSpec.js) | 已移植：Catmull-Rom 三次样条曲线、端点切线自动推导与外部注入、Hermite 混合边界求值（catmull_rom_spline_spec.rs） |
| [packages/core/Specs/CheckSpec.js](../../cesium/packages/core/Specs/CheckSpec.js) | 已移植：参数前置条件类型检查（Check::defined, Check::type_of_number, Check::type_of_bool）（check_spec.rs） |
| [packages/core/Specs/CircleGeometrySpec.js](../../cesium/packages/core/Specs/CircleGeometrySpec.js) | 已移植：圆几何多边形剖分、顶点属性生成、退化参数校验与包围球计算（circle_geometry_spec.rs） |
| [packages/core/Specs/CircleOutlineGeometrySpec.js](../../cesium/packages/core/Specs/CircleOutlineGeometrySpec.js) | 已移植：圆周线框、拉伸圆柱线框、垂直连接线定制与退化参数校验（circle_geometry_spec.rs） |
| [packages/core/Specs/ClockSpec.js](../../cesium/packages/core/Specs/ClockSpec.js) | 部分：时间范围和暂停；JulianDate、ClockStep、事件等待实现 |
| [packages/core/Specs/ColorGeometryInstanceAttributeSpec.js](../../cesium/packages/core/Specs/ColorGeometryInstanceAttributeSpec.js) | 已移植：几何实例单体颜色属性、RGBA 字节映射、归一化与比较（color_geometry_instance_attribute_spec.rs） |
| [packages/core/Specs/ColorSpec.js](../../cesium/packages/core/Specs/ColorSpec.js) | 已移植：常量色表、RGBA/Bytes 转换、CSS 解析、亮暗调整与打包（color_spec.rs） |
| [packages/core/Specs/ComponentDatatypeSpec.js](../../cesium/packages/core/Specs/ComponentDatatypeSpec.js) | 已移植：WebGL 组件数据类型字节大小、名称解析、值校验与映射（component_datatype_spec.rs） |
| [packages/core/Specs/ConstantSplineSpec.js](../../cesium/packages/core/Specs/ConstantSplineSpec.js) | 已移植：泛型恒定值样条曲线、插值求值、周期环绕与截断（constant_spline_spec.rs） |
| [packages/core/Specs/CoplanarPolygonGeometrySpec.js](../../cesium/packages/core/Specs/CoplanarPolygonGeometrySpec.js) | 已移植：共面任意多边形三角剖分、顶点属性（法线/切线/副切线/ST）、退化与重复点滤除及包围球计算（coplanar_polygon_geometry_spec.rs） |
| [packages/core/Specs/CoplanarPolygonOutlineGeometrySpec.js](../../cesium/packages/core/Specs/CoplanarPolygonOutlineGeometrySpec.js) | 已移植：共面多边形轮廓线段索引提取、闭合环线框生成与退化校验（coplanar_polygon_geometry_spec.rs） |
| [packages/core/Specs/CorridorGeometryLibrarySpec.js](../../cesium/packages/core/Specs/CorridorGeometryLibrarySpec.js) | 已移植：走廊中心线切平面法向偏移、共线与非共线拐角判定（corridor_geometry_library_spec.rs） |
| [packages/core/Specs/CorridorGeometrySpec.js](../../cesium/packages/core/Specs/CorridorGeometrySpec.js) | 已移植：走廊网格细分、顶点与法线/切线/ST计算、拉伸挤出网格、偏置属性及退化点校验（corridor_geometry_spec.rs） |
| [packages/core/Specs/CorridorOutlineGeometrySpec.js](../../cesium/packages/core/Specs/CorridorOutlineGeometrySpec.js) | 已移植：走廊平面与拉伸线框轮廓线提取、顶点偏置与退化校验（corridor_geometry_spec.rs） |
| [packages/core/Specs/CubicRealPolynomialSpec.js](../../cesium/packages/core/Specs/CubicRealPolynomialSpec.js) | 已移植：一元三次实多项式判别式计算、单实根、三重根与三不等实根求解及有限输入数值验证（cubic_real_polynomial_spec.rs） |
| [packages/core/Specs/CullingVolumeSpec.js](../../cesium/packages/core/Specs/CullingVolumeSpec.js) | 已移植：视锥裁剪体积对包围球与轴对齐包围盒分类（Inside/Outside/Intersecting）及父级平面掩码继承优化（culling_volume_spec.rs） |
| [packages/core/Specs/CylinderGeometrySpec.js](../../cesium/packages/core/Specs/CylinderGeometrySpec.js) | 已移植：参数化圆柱/圆台圆周分片、网格顶点、侧面与上下底三角形索引、退化参数校验（cylinder_geometry_spec.rs） |
| [packages/core/Specs/CylinderOutlineGeometrySpec.js](../../cesium/packages/core/Specs/CylinderOutlineGeometrySpec.js) | 已移植：圆柱顶底圆周与垂直线段线框图元顶点与索引生成（cylinder_geometry_spec.rs） |
| [packages/core/Specs/DeveloperErrorSpec.js](../../cesium/packages/core/Specs/DeveloperErrorSpec.js) | 已移植：开发期错误类型、类名、异常消息与字符串格式化输出（developer_error_spec.rs） |
| [packages/core/Specs/DistanceDisplayConditionGeometryInstanceAttributeSpec.js](../../cesium/packages/core/Specs/DistanceDisplayConditionGeometryInstanceAttributeSpec.js) | 已移植：距离显示条件实例属性构造、近远距有效性校验与 Float32Array 映射（distance_display_condition_geometry_instance_attribute_spec.rs） |
| [packages/core/Specs/DistanceDisplayConditionSpec.js](../../cesium/packages/core/Specs/DistanceDisplayConditionSpec.js) | 已移植：视距显示条件默认构造、近远距范围判等与定长打包/解包（distance_display_condition_spec.rs） |
| [packages/core/Specs/EllipseGeometrySpec.js](../../cesium/packages/core/Specs/EllipseGeometrySpec.js) | 已移植：椭球表面椭圆扇形网格剖分、顶点与法线生成、长短半轴校验（ellipse_geometry_spec.rs） |
| [packages/core/Specs/EllipseOutlineGeometrySpec.js](../../cesium/packages/core/Specs/EllipseOutlineGeometrySpec.js) | 已移植：椭圆轮廓采样、拉伸轮廓、线框图元顶点与索引生成（ellipse_geometry_spec.rs） |
| [packages/core/Specs/EllipsoidGeodesicSpec.js](../../cesium/packages/core/Specs/EllipsoidGeodesicSpec.js) | 已移植：Vincenty 测地大圆正反算、起止方位角与表面弧长解算、分段与按距离沿程插值（ellipsoid_geodesic_spec.rs） |
| [packages/core/Specs/EllipsoidGeometrySpec.js](../../cesium/packages/core/Specs/EllipsoidGeometrySpec.js) | 已移植：三轴椭球分片网格生成、顶点与三角形索引、负/零半径校验（ellipsoid_geometry_spec.rs） |
| [packages/core/Specs/EllipsoidOutlineGeometrySpec.js](../../cesium/packages/core/Specs/EllipsoidOutlineGeometrySpec.js) | 已移植：三轴椭球经纬线框顶点与索引生成（ellipsoid_geometry_spec.rs） |
| [packages/core/Specs/EllipsoidRhumbLineSpec.js](../../cesium/packages/core/Specs/EllipsoidRhumbLineSpec.js) | 已移植：等角航线/恒向线表面距离计算、墨卡托等量纬度差计算、起始航向与给定航向推算插值（ellipsoid_rhumb_line_spec.rs） |
| [packages/core/Specs/EllipsoidSpec.js](../../cesium/packages/core/Specs/EllipsoidSpec.js) | 部分：构造、法线、转换、投影、轴交点、曲率及面积；数组/pack 与其他情况待核对 |
| [packages/core/Specs/EllipsoidTangentPlaneSpec.js](../../cesium/packages/core/Specs/EllipsoidTangentPlaneSpec.js) | 部分：构造、单点/批量投影及 WGS84 往返；零半径及 JS 对象语义待核对 |
| [packages/core/Specs/EllipsoidalOccluderSpec.js](../../cesium/packages/core/Specs/EllipsoidalOccluderSpec.js) | 部分：可见性、地下椭球、剔除点和顶点；其余数值与错误语义待核对 |
| [packages/core/Specs/EncodedCartesian3Spec.js](../../cesium/packages/core/Specs/EncodedCartesian3Spec.js) | 已移植：高低位拆分编码、从 Cartesian3 构造、顶点属性数组交错写入（encoded_cartesian3_spec.rs） |
| [packages/core/Specs/EventSpec.js](../../cesium/packages/core/Specs/EventSpec.js) | 已移植：通用事件发布订阅模式、回调重入动态增删（支持在回调内部安全注册与移除监听器）、监听器计数（event_spec.rs） |
| [packages/core/Specs/FixedFrameTransformsSpec.js](../../cesium/packages/core/Specs/FixedFrameTransformsSpec.js) | 已移植：局部坐标系到椭球固定参考系变换（ENU/NED/NUE/NWU与通用正交轴生成器）、极点与原点退化处理、HPR变换互转与四元数提取、视口窗口坐标映射、位置速度旋转矩阵、2D地图投影模型矩阵转换（fixed_frame_transforms_spec.rs） |
| [packages/core/Specs/GeographicProjectionSpec.js](../../cesium/packages/core/Specs/GeographicProjectionSpec.js) | 部分：WGS84/单位球正反投影；JS 参数/结果对象语义待核对 |
| [packages/core/Specs/GeometryAttributeSpec.js](../../cesium/packages/core/Specs/GeometryAttributeSpec.js) | 已移植：几何顶点属性构造、ComponentDatatype、分量数与归一化标识校验、非法分量数抛错（geometry_attribute_spec.rs） |
| [packages/core/Specs/GeometryInstanceAttributeSpec.js](../../cesium/packages/core/Specs/GeometryInstanceAttributeSpec.js) | 已移植：几何实例属性构造、数据类型/分量数/归一化属性、TypedArray 映射与非法分量校验（geometry_instance_attribute_spec.rs） |
| [packages/core/Specs/GeometryInstanceSpec.js](../../cesium/packages/core/Specs/GeometryInstanceSpec.js) | 已移植：几何实例构造、模型变换矩阵、标识符 id 与动态单体实例属性映射传递（geometry_instance_spec.rs） |
| [packages/core/Specs/GeometryPipelineSpec.js](../../cesium/packages/core/Specs/GeometryPipelineSpec.js) | 已补齐（`crates/cesium-math/tests/geometry_pipeline_spec.rs`，覆盖 toWireframe, createLineSegmentsForVectors, createAttributeLocations, reorderForPre/PostVertexCache, fitToUnsignedShortIndices, encodeAttribute, projectTo2D, transformToWorldCoordinates, combineInstances, computeNormal, computeTangentAndBitangent, compressVertices, splitLongitude） |
| [packages/core/Specs/GeometrySpec.js](../../cesium/packages/core/Specs/GeometrySpec.js) | 已移植：通用几何体包装容器构造、属性集/索引缓冲/图元拓扑类型/包围球封装（geometry_spec.rs） |
| [packages/core/Specs/GregorianDateSpec.js](../../cesium/packages/core/Specs/GregorianDateSpec.js) | 已移植：年月日时分秒、毫秒与闰秒标识、默认与有效性构造（julian_date_spec.rs） |
| [packages/core/Specs/HeadingPitchRangeSpec.js](../../cesium/packages/core/Specs/HeadingPitchRangeSpec.js) | 已移植：构造、字段读取、clone、相等比较（quaternion_hpr_spec.rs） |
| [packages/core/Specs/HeadingPitchRollSpec.js](../../cesium/packages/core/Specs/HeadingPitchRollSpec.js) | 已移植：弧度/角度构造、四元数互转与等价验证（quaternion_hpr_spec.rs） |
| [packages/core/Specs/HermitePolynomialApproximationSpec.js](../../cesium/packages/core/Specs/HermitePolynomialApproximationSpec.js) | 已移植：Hermite 多项式逼近（0阶/高阶导数混合差商插值、阶乘、STK Components航天基准数值）（hermite_polynomial_approximation_spec.rs） |
| [packages/core/Specs/HermiteSplineSpec.js](../../cesium/packages/core/Specs/HermiteSplineSpec.js) | 已移植：Hermite 三次样条曲线、自然样条与夹紧样条构造、三对角方程求解与基函数插值求值（hermite_spline_spec.rs） |
| [packages/core/Specs/HilbertOrderSpec.js](../../cesium/packages/core/Specs/HilbertOrderSpec.js) | 已移植：2D 希尔伯特空间填充曲线正反编解码（encode2D/decode2D、支持大整型索引与层级越界校验）（hilbert_order_spec.rs） |
| [packages/core/Specs/IndexDatatypeSpec.js](../../cesium/packages/core/Specs/IndexDatatypeSpec.js) | 已移植：索引数据类型（8/16/32位无符号整型）、字节大小、顶点数自动选型（64KB阈值）及ArrayBuffer重构（index_datatype_spec.rs） |
| [packages/core/Specs/IntersectionTestsSpec.js](../../cesium/packages/core/Specs/IntersectionTestsSpec.js) | 部分：射线/线段、盒体、平面切分和掠射；其他场景待核对 |
| [packages/core/Specs/Intersections2DSpec.js](../../cesium/packages/core/Specs/Intersections2DSpec.js) | 已移植：轴对齐阈值2D三角形切割（多边形顶点与生成比例）、2D重心坐标与线段求交（intersections_2d_spec.rs） |
| [packages/core/Specs/IntervalSpec.js](../../cesium/packages/core/Specs/IntervalSpec.js) | 已移植：默认闭区间构造 [0, 0] 与显式闭区间构造 [start, stop]（interval_spec.rs） |
| [packages/core/Specs/JulianDateSpec.js](../../cesium/packages/core/Specs/JulianDateSpec.js) | 已移植：默认构造、分数天、TAI 基础构造、闰秒偏移、时间算术、秒/天差值、ISO 8601 与格里高利历互转（julian_date_spec.rs） |
| [packages/core/Specs/LagrangePolynomialApproximationSpec.js](../../cesium/packages/core/Specs/LagrangePolynomialApproximationSpec.js) | 已移植：Lagrange 多项式逼近（0阶多维数据插值、所需数据点计算、STK基准高精航天数值）（lagrange_polynomial_approximation_spec.rs） |
| [packages/core/Specs/LeapSecondSpec.js](../../cesium/packages/core/Specs/LeapSecondSpec.js) | 已移植：构造、历次闰秒表检验（julian_date_spec.rs） |
| [packages/core/Specs/LinearApproximationSpec.js](../../cesium/packages/core/Specs/LinearApproximationSpec.js) | 已移植：线性逼近算法（多步长0阶多维数据插值、参数合法性与除零校验、所需数据点）（linear_approximation_spec.rs） |
| [packages/core/Specs/LinearSplineSpec.js](../../cesium/packages/core/Specs/LinearSplineSpec.js) | 已移植：分段线性样条曲线、泛型数值与Cartesian3支持、周期时间环绕与夹紧（linear_spline_spec.rs） |
| [packages/core/Specs/MathSpec.js](../../cesium/packages/core/Specs/MathSpec.js) | 已移植：常数、符号、SNorm 转换、截断三角、角度归一化与相对/绝对容差比较（math_spec.rs） |
| [packages/core/Specs/Matrix2Spec.js](../../cesium/packages/core/Specs/Matrix2Spec.js) | 已移植：行主序构造、列主序存储、矩阵与向量乘法、转置与打包（matrix_spec.rs） |
| [packages/core/Specs/Matrix3Spec.js](../../cesium/packages/core/Specs/Matrix3Spec.js) | 已移植：行主序构造、缩放/旋转矩阵、乘法、逆矩阵与打包（matrix_spec.rs） |
| [packages/core/Specs/Matrix4Spec.js](../../cesium/packages/core/Specs/Matrix4Spec.js) | 已移植：平移/旋转、点与方向变换、仿射逆矩阵、打包（matrix_spec.rs） |
| [packages/core/Specs/NearFarScalarSpec.js](../../cesium/packages/core/Specs/NearFarScalarSpec.js) | 已移植：构造、打包解包、相等与容差比较（near_far_scalar_spec.rs） |
| [packages/core/Specs/OccluderSpec.js](../../cesium/packages/core/Specs/OccluderSpec.js) | 部分：点和球可见性、点集与矩形剔除点；JS 错误语义等待核对 |
| [packages/core/Specs/OrientedBoundingBoxSpec.js](../../cesium/packages/core/Specs/OrientedBoundingBoxSpec.js) | 部分：点集、矩形、变换、角点、平面、距离、遮挡、打包；更多边界待移植 |
| [packages/core/Specs/OrthographicFrustumSpec.js](../../cesium/packages/core/Specs/OrthographicFrustumSpec.js) | 部分：投影、像素尺寸、裁剪、打包；比较及 JS 语义待核对 |
| [packages/core/Specs/OrthographicOffCenterFrustumSpec.js](../../cesium/packages/core/Specs/OrthographicOffCenterFrustumSpec.js) | 部分：投影、像素尺寸和裁剪；比较及 JS 语义待核对 |
| [packages/core/Specs/PerspectiveFrustumSpec.js](../../cesium/packages/core/Specs/PerspectiveFrustumSpec.js) | 部分：视角、偏移、投影、像素尺寸与裁剪；pack/比较及 JS 语义待补齐 |
| [packages/core/Specs/PerspectiveOffCenterFrustumSpec.js](../../cesium/packages/core/Specs/PerspectiveOffCenterFrustumSpec.js) | 部分：投影、像素尺寸与裁剪；比较及 JS 语义待补齐 |
| [packages/core/Specs/PixelFormatSpec.js](../../cesium/packages/core/Specs/PixelFormatSpec.js) | 已移植：像素格式与数据类型枚举、WebGL1/2内部格式转换、多通道垂直反转（flipY）（pixel_format_spec.rs） |
| [packages/core/Specs/PlaneGeometrySpec.js](../../cesium/packages/core/Specs/PlaneGeometrySpec.js) | 已移植：单面 4 顶点 2 三角形网格、全属性（法线/切线/副切线/纹理）（plane_geometry_spec.rs） |
| [packages/core/Specs/PlaneOutlineGeometrySpec.js](../../cesium/packages/core/Specs/PlaneOutlineGeometrySpec.js) | 已移植：矩形 4 边线框图元顶点与索引生成（plane_geometry_spec.rs） |
| [packages/core/Specs/PlaneSpec.js](../../cesium/packages/core/Specs/PlaneSpec.js) | 部分：构造、距离、投影、变换及比较；JS 对象复用和异常语义不同 |
| [packages/core/Specs/PolygonGeometryLibrarySpec.js](../../cesium/packages/core/Specs/PolygonGeometryLibrarySpec.js) | 已全量移植：多边形几何库（恒向线/大地大圆细分采样与顶点打包、赤道多边形相交检测与连线分割重构、凹多边形、相交与折返边界全用例覆盖）（polygon_geometry_library_spec.rs） |
| [packages/core/Specs/PolygonGeometrySpec.js](../../cesium/packages/core/Specs/PolygonGeometrySpec.js) | 已全量移植：多边形立体几何体（Earcut三角剖分、测地/恒向线细分、逐点高度、拉伸与侧壁挤出闭合、嵌套多层孔洞、经纬度矩形包围盒极射赤平算法[南北半球/赤道跨越/日界线跨越/包围极点]、全顶点属性法线/切线/副切线/ST、顶点偏移属性、Pack/Unpack 序列化）（polygon_geometry_spec.rs） |
| [packages/core/Specs/PolygonOutlineGeometrySpec.js](../../cesium/packages/core/Specs/PolygonOutlineGeometrySpec.js) | 已全量移植：多边形轮廓几何体（测地/恒向线细分、逐点高度、拉伸挤出立体轮廓、嵌套孔洞多边形层级、逆时针绕向保序、高低空包围球、顶点偏移属性 Top/All、Pack/Unpack 序列化）（polygon_outline_geometry_spec.rs） |
| [packages/core/Specs/PolygonPipelineSpec.js](../../cesium/packages/core/Specs/PolygonPipelineSpec.js) | 已移植：多边形流水线（2D面积与绕向判定、Earcut孔洞剖分、球面/恒向线三角形细分与纹理插值、大地高程缩放）（polygon_pipeline_spec.rs） |
| [packages/core/Specs/PolylineGeometrySpec.js](../../cesium/packages/core/Specs/PolylineGeometrySpec.js) | 已移植：参数化宽折线网格展开（M*4-4 顶点 / M*6-6 索引）、前驱/后继点坐标、展开方向与线宽属性、逐顶点/逐段颜色与重复点去重（polyline_geometry_spec.rs） |
| [packages/core/Specs/PolylinePipelineSpec.js](../../cesium/packages/core/Specs/PolylinePipelineSpec.js) | 已全量移植：折线流水线（大地线/恒向线细分采样与高程插值、180度国际日界线跨越分割与模型矩阵局部空间变换、单点/空数组边界处理、高程提取）（polyline_pipeline_spec.rs） |
| [packages/core/Specs/PolylineVolumeGeometryLibrarySpec.js](../../cesium/packages/core/Specs/PolylineVolumeGeometryLibrarySpec.js) | 已全量移植：多段线体几何库（共线点边界平滑挤出、拐角方向与切平面转角判定、2D截面环绕去重与切平面投影）（polyline_volume_geometry_library_spec.rs） |
| [packages/core/Specs/PolylineVolumeGeometrySpec.js](../../cesium/packages/core/Specs/PolylineVolumeGeometrySpec.js) | 已全量移植：折线体积几何体（圆角/斜角/尖角侧壁四边形与端面剖分盖子、全顶点属性[法线/切线/副切线/ST]、顺逆时针截面自适应、Pack/Unpack 序列化）（polyline_volume_geometry_spec.rs） |
| [packages/core/Specs/PolylineVolumeOutlineGeometrySpec.js](../../cesium/packages/core/Specs/PolylineVolumeOutlineGeometrySpec.js) | 已全量移植：折线体轮廓线框几何体（端面闭合环框与纵向骨干棱线索引、Pack/Unpack 序列化）（polyline_volume_outline_geometry_spec.rs） |
| [packages/core/Specs/PrimitiveTypeSpec.js](../../cesium/packages/core/Specs/PrimitiveTypeSpec.js) | 已全量移植：图元拓扑类型枚举与校验（点/线/三角形拓扑分类、validate合法性检查、isLines与isTriangles拓扑判定）（primitive_type_spec.rs） |
| [packages/core/Specs/QuadraticRealPolynomialSpec.js](../../cesium/packages/core/Specs/QuadraticRealPolynomialSpec.js) | 部分：判别式及有限输入实根场景；JS 参数异常不适用 |
| [packages/core/Specs/QuarticRealPolynomialSpec.js](../../cesium/packages/core/Specs/QuarticRealPolynomialSpec.js) | 部分：判别式与有限输入求根场景；JS 参数异常不适用 |
| [packages/core/Specs/QuaternionSpec.js](../../cesium/packages/core/Specs/QuaternionSpec.js) | 已移植：单位四元数、轴角构造、旋转矩阵互转与 slerp 球面插值（quaternion_hpr_spec.rs） |
| [packages/core/Specs/QuaternionSplineSpec.js](../../cesium/packages/core/Specs/QuaternionSplineSpec.js) | 已全量移植：四元数球面线性插值样条曲线（双控制点退化、多点插值、时间区间查找、越界抛错与 slerp 球面平滑求值）（quaternion_spline_spec.rs，4 项测试全部通过） |
| [packages/core/Specs/QueueSpec.js](../../cesium/packages/core/Specs/QueueSpec.js) | 已移植：队列入队出队、首元素查看 peek、包含性检查、就地排序与清空（queue_spec.rs） |
| [packages/core/Specs/RaySpec.js](../../cesium/packages/core/Specs/RaySpec.js) | 部分：构造、方向归一化、取点；JS 对象复用语义不同 |
| [packages/core/Specs/RectangleGeometrySpec.js](../../cesium/packages/core/Specs/RectangleGeometrySpec.js) | 已全量移植：矩形立体几何体（极点退化三角扇、IDL跨越、旋转与ST纹理坐标旋转重映射、拉伸侧壁网格与法线/切线/副切线、顶部/全部applyOffset偏移属性、退化检测与Pack/Unpack序列化）（rectangle_geometry_spec.rs，28 项测试全部通过） |
| [packages/core/Specs/RectangleOutlineGeometrySpec.js](../../cesium/packages/core/Specs/RectangleOutlineGeometrySpec.js) | 已全量移植：矩形轮廓线框几何体（经纬线圈环绕索引、极点边退化、高度拉伸四角与极点柱边线框、顶部/全部applyOffset偏移属性、退化检测与Pack/Unpack序列化）（rectangle_outline_geometry_spec.rs，18 项测试全部通过） |
| [packages/core/Specs/RectangleSpec.js](../../cesium/packages/core/Specs/RectangleSpec.js) | 部分：角点、宽高、点集、包围球、交并、包含、子区域与采样；其余待核对 |
| [packages/core/Specs/RuntimeErrorSpec.js](../../cesium/packages/core/Specs/RuntimeErrorSpec.js) | 已移植：运行时错误类型、类名、异常消息与字符串格式化输出（runtime_error_spec.rs） |
| [packages/core/Specs/ShowGeometryInstanceAttributeSpec.js](../../cesium/packages/core/Specs/ShowGeometryInstanceAttributeSpec.js) | 已移植：显隐几何体实例属性构造、布尔值字节映射与数组填充（show_geometry_instance_attribute_spec.rs） |
| [packages/core/Specs/SimplePolylineGeometrySpec.js](../../cesium/packages/core/Specs/SimplePolylineGeometrySpec.js) | 已移植：单像素折线图元生成、测地线/恒向线细分、逐段颜色（顶点拆分）与逐顶点颜色映射、线段索引构建（simple_polyline_geometry_spec.rs） |
| [packages/core/Specs/SphereGeometrySpec.js](../../cesium/packages/core/Specs/SphereGeometrySpec.js) | 已移植：等半径球体网格、分片划分（3x3 -> 36 顶点 18 三角形）、零半径退化（sphere_geometry_spec.rs） |
| [packages/core/Specs/SphereOutlineGeometrySpec.js](../../cesium/packages/core/Specs/SphereOutlineGeometrySpec.js) | 已移植：等半径球体经纬线框（2x2x2 -> 12 顶点 6 线段）、零半径退化（sphere_geometry_spec.rs） |
| [packages/core/Specs/SphericalSpec.js](../../cesium/packages/core/Specs/SphericalSpec.js) | 已全量移植：球坐标系统（方位角clock、天顶角cone、距离magnitude、Cartesian3互转、向量归一化、容差比较与字符串序列化）（spherical_spec.rs） |
| [packages/core/Specs/SplineSpec.js](../../cesium/packages/core/Specs/SplineSpec.js) | 已全量移植：样条基类与时间区间接口（时间环绕 wrap_time、时间截断 clamp_time、带 startIndex 优化的高效二分/顺序时间区间查找 find_time_interval、点类型提取与非法类型抛错）（spline_spec.rs，7 项测试全部通过） |
| [packages/core/Specs/SteppedSplineSpec.js](../../cesium/packages/core/Specs/SteppedSplineSpec.js) | 已全量移植：阶梯常数样条曲线（泛型浮点数、Cartesian3、Quaternion 支持、阶跃分段常量求值与边界区间校验）（stepped_spline_spec.rs，5 项测试全部通过） |
| [packages/core/Specs/StereographicSpec.js](../../cesium/packages/core/Specs/StereographicSpec.js) | 已全量移植：极射赤平投影（半球/两极投影切平面、地心表面投影、共形纬度、经纬度往返转换、批量投影）（stereographic_spec.rs） |
| [packages/core/Specs/TimeIntervalCollectionSpec.js](../../cesium/packages/core/Specs/TimeIntervalCollectionSpec.js) | 已移植：非重叠时间区间集合、同数据区间自动合并、异数据区间重叠裁切与拆分、区间挖洞移除与时间二分查找（time_interval_collection_spec.rs） |
| [packages/core/Specs/TimeIntervalSpec.js](../../cesium/packages/core/Specs/TimeIntervalSpec.js) | 已移植：时间区间构造、点包含判定（含开闭区间边界）、判空、相等比较与重叠求交（time_interval_spec.rs） |
| [packages/core/Specs/TipsifySpec.js](../../cesium/packages/core/Specs/TipsifySpec.js) | 已移植：后顶点着色器缓存三角形优化与ACMR未命中率计算（tipsify/calculateACMR、deadEnd回退与邻域跳跃）（tipsify_spec.rs） |
| [packages/core/Specs/TranslationRotationScaleSpec.js](../../cesium/packages/core/Specs/TranslationRotationScaleSpec.js) | 已移植：仿射变换平移/四元数旋转/缩放结构体、默认值、判等与仿射Matrix4生成（translation_rotation_scale_spec.rs） |
| [packages/core/Specs/TridiagonalSystemSolverSpec.js](../../cesium/packages/core/Specs/TridiagonalSystemSolverSpec.js) | 已移植：Thomas 三对角线性方程组求解器、Cartesian3 向量右端项回代计算（tridiagonal_system_solver_spec.rs） |
| [packages/core/Specs/VertexFormatSpec.js](../../cesium/packages/core/Specs/VertexFormatSpec.js) | 已移植：格式位掩码、常量定义、clone、pack/unpack（vertex_format_spec.rs） |
| [packages/core/Specs/VerticalExaggerationSpec.js](../../cesium/packages/core/Specs/VerticalExaggerationSpec.js) | 已移植：垂直地形高程夸张计算（getHeight/getPosition、基准高程缩放、WGS84坐标转换）（vertical_exaggeration_spec.rs） |
| [packages/core/Specs/WallGeometrySpec.js](../../cesium/packages/core/Specs/WallGeometrySpec.js) | 已移植：墙体几何顶点交错排布、高程拉伸、折线插值采样、退化参数校验与三角形剖分（wall_geometry_spec.rs） |
| [packages/core/Specs/WallOutlineGeometrySpec.js](../../cesium/packages/core/Specs/WallOutlineGeometrySpec.js) | 已移植：墙体轮廓线框顶点生成、顶底水平与侧面垂直线条索引及退化参数校验（wall_geometry_spec.rs） |
| [packages/core/Specs/WebMercatorProjectionSpec.js](../../cesium/packages/core/Specs/WebMercatorProjectionSpec.js) | 部分：正反投影、极区裁剪；结果对象及其他语义待核对 |
| [packages/core/Specs/arrayRemoveDuplicatesSpec.js](../../cesium/packages/core/Specs/arrayRemoveDuplicatesSpec.js) | 已移植：数组相邻重复项去除（ε容差去重、首尾闭合环绕去重与删除索引数组追踪）（array_remove_duplicates_spec.rs） |
| [packages/core/Specs/barycentricCoordinatesSpec.js](../../cesium/packages/core/Specs/barycentricCoordinatesSpec.js) | 已移植：2D/3D 三角形顶点、边缘与内部重心坐标计算、退化共线判定（barycentric_coordinates_spec.rs） |
| [packages/core/Specs/binarySearchSpec.js](../../cesium/packages/core/Specs/binarySearchSpec.js) | 已移植：有序序列二分查找（命中索引、未命中按按位取反~返回插入点、支持自定义比较器）（binary_search_spec.rs） |
| [packages/core/Specs/cloneSpec.js](../../cesium/packages/core/Specs/cloneSpec.js) | 已移植：JSON 结构深克隆与浅克隆（clone_spec.rs） |
| [packages/core/Specs/combineSpec.js](../../cesium/packages/core/Specs/combineSpec.js) | 已移植：JSON 对象浅层/深层递归合并，冲突键优先级与空值处理（combine_spec.rs） |
| [packages/core/Specs/createGuidSpec.js](../../cesium/packages/core/Specs/createGuidSpec.js) | 已移植：RFC 4122 v4 UUID 生成器与格式校验（create_guid_spec.rs） |
| [packages/core/Specs/definedSpec.js](../../cesium/packages/core/Specs/definedSpec.js) | 已移植：变量是否已定义判定（defined_spec.rs） |
| [packages/core/Specs/deprecationWarningSpec.js](../../cesium/packages/core/Specs/deprecationWarningSpec.js) | 已移植：废弃接口警告单次输出机制与防重复计数（deprecation_warning_spec.rs） |
| [packages/core/Specs/getAbsoluteUriSpec.js](../../cesium/packages/core/Specs/getAbsoluteUriSpec.js) | 已移植：相对 URI 基于 Base URI 转换为绝对 URI（get_absolute_uri_spec.rs） |
| [packages/core/Specs/getBaseUriSpec.js](../../cesium/packages/core/Specs/getBaseUriSpec.js) | 已移植：基础 URI 路径提取、查询参数与锚点分离保留（get_base_uri_spec.rs） |
| [packages/core/Specs/getExtensionFromUriSpec.js](../../cesium/packages/core/Specs/getExtensionFromUriSpec.js) | 已移植：URI 扩展名解析与小写化、忽略查询参数与片段（get_extension_from_uri_spec.rs） |
| [packages/core/Specs/getFilenameFromUriSpec.js](../../cesium/packages/core/Specs/getFilenameFromUriSpec.js) | 已移植：URI 文件名提取、斜杠结尾与参数清洗（get_filename_from_uri_spec.rs） |
| [packages/core/Specs/getJsonFromTypedArraySpec.js](../../cesium/packages/core/Specs/getJsonFromTypedArraySpec.js) | 已移植：TypedArray 二进制缓冲区/切片 JSON 解析（get_json_from_typed_array_spec.rs） |
| [packages/core/Specs/getStringFromTypedArraySpec.js](../../cesium/packages/core/Specs/getStringFromTypedArraySpec.js) | 已移植：TypedArray 字节序列/子区域 UTF-8 字符串解码与越界检查（get_string_from_typed_array_spec.rs） |
| [packages/core/Specs/isLeapYearSpec.js](../../cesium/packages/core/Specs/isLeapYearSpec.js) | 已移植：公历闰年判定（4年一闰、百年不闰、四百年又闰）（is_leap_year_spec.rs） |
| [packages/core/Specs/mergeSortSpec.js](../../cesium/packages/core/Specs/mergeSortSpec.js) | 已移植：稳定归并排序（数值排序、同键稳定性、自定义对象包围球距离排序）（merge_sort_spec.rs） |
| [packages/core/Specs/objectToQuerySpec.js](../../cesium/packages/core/Specs/objectToQuerySpec.js) | 已移植：URL 查询参数序列化、空值跳过与数组多值编码（object_to_query_spec.rs） |
| [packages/core/Specs/oneTimeWarningSpec.js](../../cesium/packages/core/Specs/oneTimeWarningSpec.js) | 已移植：一次性警告去重触发器与标识符校验（one_time_warning_spec.rs） |
| [packages/core/Specs/pointInsideTriangleSpec.js](../../cesium/packages/core/Specs/pointInsideTriangleSpec.js) | 已移植：2D/3D 点在三角形内部判定（严格内部/顶点/边线边界测试）（point_inside_triangle_spec.rs） |
| [packages/core/Specs/queryToObjectSpec.js](../../cesium/packages/core/Specs/queryToObjectSpec.js) | 已移植：URL 查询字符串解析反序列化为键值/列表映射、支持分号分隔与百分比解码（object_to_query_spec.rs） |
| [packages/core/Specs/srgbToLinearSpec.js](../../cesium/packages/core/Specs/srgbToLinearSpec.js) | 已移植：sRGB 颜色空间到线性色彩空间非线性转换（srgb_to_linear_spec.rs） |
| [packages/core/Specs/subdivideArraySpec.js](../../cesium/packages/core/Specs/subdivideArraySpec.js) | 已移植：数组均分/非均分分块切分、边界校验（subdivide_array_spec.rs） |
| [packages/engine/Specs/Core/ApproximateTerrainHeightsSpec.js](../../cesium/packages/engine/Specs/Core/ApproximateTerrainHeightsSpec.js) | 已移植：粗略地形高程查找表、地理瓦片矩形层级检索、经纬度矩形最小/最大高度估算与对角线弦长修正、包围球计算（approximate_terrain_heights_spec.rs） |
| [packages/engine/Specs/Core/ArcGISTiledElevationTerrainProviderSpec.js](../../cesium/packages/engine/Specs/Core/ArcGISTiledElevationTerrainProviderSpec.js) | 已补齐（`crates/cesium-math/tests/arcgis_tiled_elevation_terrain_provider_spec.rs`，覆盖元数据解析、EPSG:3857/4326 切片方案推导与范围截断、几何误差计算、瓦片可用性检测、LERC 编码与 HeightmapTerrainData 生成） |
| [packages/engine/Specs/Core/BingMapsGeocoderServiceSpec.js](../../cesium/packages/engine/Specs/Core/BingMapsGeocoderServiceSpec.js) | 已补齐（`crates/cesium-math/tests/bing_maps_geocoder_service_spec.rs`，覆盖 Bing 地理编码 URL 构建、key 与 culture 查询参数拼接、resourceSets 结果反序列化与经纬度矩形映射、Credit 图标与显示属性校验） |
| [packages/engine/Specs/Core/CartographicGeocoderServiceSpec.js](../../cesium/packages/engine/Specs/Core/CartographicGeocoderServiceSpec.js) | 已移植：地理经纬度坐标地理编码器服务、NSEW 与度数格式解析、默认高度与笛卡尔空间坐标映射（cartographic_geocoder_service_spec.rs） |
| [packages/engine/Specs/Core/CelestialFrameTransformsSpec.js](../../cesium/packages/engine/Specs/Core/CelestialFrameTransformsSpec.js) | 已移植：天球参考系变换（TEME到伪地固系旋转矩阵、GMST自转角、ICRF到月球固连系Moon-Fixed旋转矩阵与NASA/STK高精度匹配、恒星日与恒星月周期验证）（celestial_frame_transforms_spec.rs） |
| [packages/engine/Specs/Core/Cesium3DTilesTerrainDataSpec.js](../../cesium/packages/engine/Specs/Core/Cesium3DTilesTerrainDataSpec.js) | 已补齐（`crates/cesium-math/tests/cesium_3d_tiles_terrain_data_spec.rs`，覆盖必填参数校验、四叉树子瓦片掩码解析与位运算查询、高程双线性插值与边界 clamp、地形网格 TerrainMesh/TerrainEncoding 生成及夸张系数支持、单级 upsample 上采样地形衍生与状态标记） |
| [packages/engine/Specs/Core/Cesium3DTilesTerrainProviderSpec.js](../../cesium/packages/engine/Specs/Core/Cesium3DTilesTerrainProviderSpec.js) | 已补齐（`crates/cesium-math/tests/cesium_3d_tiles_terrain_provider_spec.rs`，覆盖 URL 必填与合法性检测、切片方案与信用信息绑定、水面掩码标志与 request_tile_geometry 瓦片高程请求） |
| [packages/engine/Specs/Core/CesiumTerrainProviderSpec.js](../../cesium/packages/engine/Specs/Core/CesiumTerrainProviderSpec.js) | 已补齐（`crates/cesium-math/tests/cesium_terrain_provider_spec.rs`，覆盖 layer.json 解析与版本校验、EPSG:4326/3857 切片方案、TMS y 坐标转换与 availability 解算、Heightmap/QuantizedMesh 瓦片请求、水面与法线掩码标志等） |
| [packages/engine/Specs/Core/CustomHeightmapTerrainProviderSpec.js](../../cesium/packages/engine/Specs/Core/CustomHeightmapTerrainProviderSpec.js) | 已移植：自定义高程图地形提供器构建、宽度/高度/回调参数校验、信用信息/投影切分方案绑定、0 级最大几何误差与逐级误差减半解算、多类型高程图缓冲加载与回调返回空判定（custom_heightmap_terrain_provider_spec.rs） |
| [packages/engine/Specs/Core/DoubleEndedPriorityQueueSpec.js](../../cesium/packages/engine/Specs/Core/DoubleEndedPriorityQueueSpec.js) | 已移植：基于 Min-Max 堆的双端优先队列、最值检索与移除、最大容量满额淘汰与就地重新堆化（double_ended_priority_queue_spec.rs） |
| [packages/engine/Specs/Core/DoublyLinkedListSpec.js](../../cesium/packages/engine/Specs/Core/DoublyLinkedListSpec.js) | 已移植：双向链表增删、指定节点拼接 splice 迁移、头尾指针及长度维护（doubly_linked_list_spec.rs） |
| [packages/engine/Specs/Core/EarthOrientationParametersSpec.js](../../cesium/packages/engine/Specs/Core/EarthOrientationParametersSpec.js) | 已移植：地球定向参数 (EOP) 反序列化、动态闰秒检测与注册、常规时间区间线性插值、跨闰秒 UT1-UTC 跳变补偿与不连续性插值、本地真实 JSON 文件加载解析（earth_orientation_parameters_spec.rs） |
| [packages/engine/Specs/Core/EllipsoidTerrainProviderSpec.js](../../cesium/packages/engine/Specs/Core/EllipsoidTerrainProviderSpec.js) | 已移植：纯椭球平坦地形提供器构建、16x16 零高程图数据生成、0 级最大几何误差估计、水面掩码与法线默认关闭状态、数据可用性判定（ellipsoid_terrain_provider_spec.rs） |
| [packages/engine/Specs/Core/FeatureDetectionSpec.js](../../cesium/packages/engine/Specs/Core/FeatureDetectionSpec.js) | 已移植：浏览器与硬件特性检测（全屏、WebWorker、类型化数组 TypedArrays、BigInt64/Uint64、WebAssembly、WebP支持异步初始化与状态捕获、Chrome/Safari/Webkit/Edge/Firefox/iOS 浏览器与内核版本号精准解析提取）（feature_detection_spec.rs） |
| [packages/engine/Specs/Core/FrustumGeometrySpec.js](../../cesium/packages/engine/Specs/Core/FrustumGeometrySpec.js) | 已移植：视锥体六表面 24 独立顶点生成、法线切线副切线纹理坐标、12 三角形索引与包围球计算（frustum_geometry_spec.rs） |
| [packages/engine/Specs/Core/FrustumOutlineGeometrySpec.js](../../cesium/packages/engine/Specs/Core/FrustumOutlineGeometrySpec.js) | 已移植：视锥体 8 角点框线、近/远平面边框与连接线索引、包围球计算（frustum_geometry_spec.rs） |
| [packages/engine/Specs/Core/FullscreenSpec.js](../../cesium/packages/engine/Specs/Core/FullscreenSpec.js) | 已移植：全屏 API 支持检测、启用状态判断、全屏元素获取、进入/退出全屏状态请求与事件名（fullscreenchange/fullscreenerror）映射（fullscreen_spec.rs） |
| [packages/engine/Specs/Core/GeographicTilingSchemeSpec.js](../../cesium/packages/engine/Specs/Core/GeographicTilingSchemeSpec.js) | 已移植：地理瓦片方案接口、瓦片范围索引计算、NW原点方向、相邻边界贴合、弧度/度数转换及四角与中心定位（geographic_tiling_scheme_spec.rs） |
| [packages/engine/Specs/Core/GoogleEarthEnterpriseMetadataSpec.js](../../cesium/packages/engine/Specs/Core/GoogleEarthEnterpriseMetadataSpec.js) | 已补齐（`crates/cesium-math/tests/google_earth_enterprise_metadata_spec.rs`，覆盖 tileXY 与 QuadKey 双向互转、decode 数据对称加解密与 key 长度校验、TileInformation 四叉树位掩码与属性继承、子树元数据查询与 populateSubtree） |
| [packages/engine/Specs/Core/GoogleEarthEnterpriseTerrainDataSpec.js](../../cesium/packages/engine/Specs/Core/GoogleEarthEnterpriseTerrainDataSpec.js) | 已补齐（`crates/cesium-math/tests/google_earth_enterprise_terrain_data_spec.rs`，覆盖二进制 quad 缓冲区解析与负高程转换、四叉树子瓦片掩码西南东南东北西北方向映射、高度插值、地形网格 TerrainMesh/TerrainEncoding 生成及 upsample 单级上采样） |
| [packages/engine/Specs/Core/GoogleEarthEnterpriseTerrainProviderSpec.js](../../cesium/packages/engine/Specs/Core/GoogleEarthEnterpriseTerrainProviderSpec.js) | 已补齐（`crates/cesium-math/tests/google_earth_enterprise_terrain_provider_spec.rs`，覆盖 metadata 构造与 terrainPresent 校验、切片方案与几何误差、瓦片可用性检索与 request_tile_geometry 请求） |
| [packages/engine/Specs/Core/GoogleGeocoderServicesSpec.js](../../cesium/packages/engine/Specs/Core/GoogleGeocoderServicesSpec.js) | 已补齐（`crates/cesium-math/tests/google_geocoder_services_spec.rs`，覆盖 key 参数校验、Google 地理编码 API URL 生成、OK 状态视口经纬度矩形映射与 Credit attribution 属性、ZERO_RESULTS 返回空列表、异常响应抛出 RuntimeError） |
| [packages/engine/Specs/Core/GroundPolylineGeometrySpec.js](../../cesium/packages/engine/Specs/Core/GroundPolylineGeometrySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Core/HeapSpec.js](../../cesium/packages/engine/Specs/Core/HeapSpec.js) | 已移植：基于数组的堆结构、自定义排序比较器、下滤/上浮调整、最大容量上限与溢出剔除、resort 重排（heap_spec.rs） |
| [packages/engine/Specs/Core/HeightmapTerrainDataSpec.js](../../cesium/packages/engine/Specs/Core/HeightmapTerrainDataSpec.js) | 已移植：地形高程图数据构建、Lerc 与原生缓冲类型推断、createMesh 网格生成与异步任务节流上限、子瓦片双三角形对角线插值 upsample（含 stride/大小端打包与最高/最低高程截断）、四叉树子瓦片可用性掩码解算（heightmap_terrain_data_spec.rs） |
| [packages/engine/Specs/Core/ITwinPlatformSpec.js](../../cesium/packages/engine/Specs/Core/ITwinPlatformSpec.js) | 已补齐（`crates/cesium-math/tests/itwin_platform_spec.rs`，覆盖 Authorization 请求头生成逻辑与 Basic/Bearer 优先级、realityData 容器 URL 与 rootDocument 相对路径拼接及 query 保留、401/403/404/422/429 HTTP 异常状态码解析与 RuntimeError 映射、ExportStatus 与 RealityDataType 枚举） |
| [packages/engine/Specs/Core/Iau2000OrientationSpec.js](../../cesium/packages/engine/Specs/Core/Iau2000OrientationSpec.js) | 已移植：IAU 2000 月球自转与极轴定向参数高精度计算（赤经、赤纬、自转角及瞬时自转角速度）（iau2000_orientation_spec.rs） |
| [packages/engine/Specs/Core/Iau2006XysDataSpec.js](../../cesium/packages/engine/Specs/Core/Iau2006XysDataSpec.js) | 已移植：IAU 2006 XYS 天球中介极坐标数据分块按需加载与预加载、9 阶 Lagrange 多项式内插、历元边界越界防护、STK Components 高精度比对（iau_2006_xys_data_spec.rs） |
| [packages/engine/Specs/Core/IauOrientationAxesSpec.js](../../cesium/packages/engine/Specs/Core/IauOrientationAxesSpec.js) | 已移植：国际天球参考系 ICRF 到月球固定系 Moon Fixed 旋转矩阵计算（赤道岁差/章动与自转四元数合成）（iau_orientation_axes_spec.rs） |
| [packages/engine/Specs/Core/IonGeocoderServiceSpec.js](../../cesium/packages/engine/Specs/Core/IonGeocoderServiceSpec.js) | 已补齐（`crates/cesium-math/tests/ion_geocoder_service_spec.rs`，覆盖默认 access token 与 server 端点、IonGeocodeProviderType 校验及枚举解析、动态切换 Google/Bing/Default geocoder 查询参数、委托底层 Pelias 地理编码请求与结果反序列化） |
| [packages/engine/Specs/Core/IonResourceSpec.js](../../cesium/packages/engine/Specs/Core/IonResourceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Core/IonSnapServiceSpec.js](../../cesium/packages/engine/Specs/Core/IonSnapServiceSpec.js) | 已补齐（`crates/cesium-math/tests/ion_snap_service_spec.rs`，覆盖 DEFAULT_SNAP_APERTURE 常量、IonSnapMode/IonSnapHeat/IonSnapGeometryType 枚举值、NDC 到视口像素空间投影矩阵计算、ECEF 行优先变换矩阵反序列化与吸附请求接口） |
| [packages/engine/Specs/Core/ManagedArraySpec.js](../../cesium/packages/engine/Specs/Core/ManagedArraySpec.js) | 已移植：受管动态数组、显式长度与预留容量管理、动态扩缩容、末尾元素出入栈与引用清理（managed_array_spec.rs） |
| [packages/engine/Specs/Core/MortonOrderSpec.js](../../cesium/packages/engine/Specs/Core/MortonOrderSpec.js) | 已移植：2D/3D 莫顿空间填充曲线编码与解码、位交错与反交错、边界值与越界校验（morton_order_spec.rs） |
| [packages/engine/Specs/Core/OpenCageGeocoderServiceSpec.js](../../cesium/packages/engine/Specs/Core/OpenCageGeocoderServiceSpec.js) | 已补齐（`crates/cesium-math/tests/open_cage_geocoder_service_spec.rs`，覆盖 URL 与 API Key 构造校验、query 与 params URL 拼接、带 bounds 的经纬度矩形映射、无 bounds 的笛卡尔空间点 Cartesian3 映射、空结果处理及 OpenStreetMap Credit 版权信息） |
| [packages/engine/Specs/Core/PeliasGeocoderServiceSpec.js](../../cesium/packages/engine/Specs/Core/PeliasGeocoderServiceSpec.js) | 已补齐（`crates/cesium-math/tests/pelias_geocoder_service_spec.rs`，覆盖 URL 必填校验、Search 与 Autocomplete 路径及参数构建、Point 空间点 Cartesian3 与 bbox 经纬度矩形映射、attributions 归属信息绑定与空结果处理） |
| [packages/engine/Specs/Core/PinBuilderSpec.js](../../cesium/packages/engine/Specs/Core/PinBuilderSpec.js) | 已补齐（`crates/cesium-math/tests/pin_builder_spec.rs`，覆盖纯色、文字、URL 图标、Maki 图标生成、尺寸/颜色校验及缓存复用） |
| [packages/engine/Specs/Core/PixelDatatypeSpec.js](../../cesium/packages/engine/Specs/Core/PixelDatatypeSpec.js) | 已移植：像素数据类型枚举、分量与浮点格式判断、字节大小校验与数值转换（pixel_datatype_spec.rs） |
| [packages/engine/Specs/Core/QuantizedMeshTerrainDataSpec.js](../../cesium/packages/engine/Specs/Core/QuantizedMeshTerrainDataSpec.js) | 已移植：量子化网格地形数据构建、createMesh 网格生成与裙边顶点/索引扩展、高程夸大编码传递、重心坐标高度插值与越界 Clamping、四边形与带额外顶点的子瓦片几何裁剪分割升采样（upsampleQuantizedTerrainMesh）、八面体法线保留、子瓦片掩码可用性解算（quantized_mesh_terrain_data_spec.rs） |
| [packages/engine/Specs/Core/RectangleCollisionCheckerSpec.js](../../cesium/packages/engine/Specs/Core/RectangleCollisionCheckerSpec.js) | 已移植：2D 矩形碰撞判定树、重叠检测、动态插入与删除（rectangle_collision_checker_spec.rs） |
| [packages/engine/Specs/Core/RequestErrorEventSpec.js](../../cesium/packages/engine/Specs/Core/RequestErrorEventSpec.js) | 已移植：请求错误事件封装、响应头与状态码解析、克隆与字符串格式化（request_error_event_spec.rs） |
| [packages/engine/Specs/Core/RequestSchedulerSpec.js](../../cesium/packages/engine/Specs/Core/RequestSchedulerSpec.js) | 已移植：异步请求优先级调度器、服务器并发限制与分流、动态优先级更新与重排、请求淘汰与取消统计、完成事件通知（request_scheduler_spec.rs） |
| [packages/engine/Specs/Core/RequestSpec.js](../../cesium/packages/engine/Specs/Core/RequestSpec.js) | 已移植：资源请求任务封装、优先级与取消处理、请求状态流转与克隆（request_spec.rs） |
| [packages/engine/Specs/Core/ResourceSpec.js](../../cesium/packages/engine/Specs/Core/ResourceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Core/S2CellSpec.js](../../cesium/packages/engine/Specs/Core/S2CellSpec.js) | 已移植：S2 空间球面四叉分片与希尔伯特曲线索引、64位CellID与十六进制Token互转、父子层级解算、中心与CCW顶点椭球地理坐标投影（s2_cell_spec.rs） |
| [packages/engine/Specs/Core/ScreenSpaceEventHandlerSpec.js](../../cesium/packages/engine/Specs/Core/ScreenSpaceEventHandlerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Core/Simon1994PlanetaryPositionsSpec.js](../../cesium/packages/engine/Specs/Core/Simon1994PlanetaryPositionsSpec.js) | 已移植：Simon 1994 太阳与月球地心惯性系（J2000/GCRF）行星轨道摄动级数解算、Delaunay变量与Kepler偏近点角迭代、地固系东升西落方位角验证（simon1994_planetary_positions_spec.rs） |
| [packages/engine/Specs/Core/SnapServiceSpec.js](../../cesium/packages/engine/Specs/Core/SnapServiceSpec.js) | 已补齐（`crates/cesium-math/tests/snap_service_spec.rs`，覆盖 SnapService 基础 trait 定义、DEFAULT_SNAP_APERTURE 默认声明及基类未实现 snap 报错逻辑） |
| [packages/engine/Specs/Core/TaskProcessorSpec.js](../../cesium/packages/engine/Specs/Core/TaskProcessorSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Core/TerrainEncodingSpec.js](../../cesium/packages/engine/Specs/Core/TerrainEncodingSpec.js) | 已移植：地形顶点量化（NONE/BITS12）、顶点编码/解码、纹理坐标压缩、高程夸大、八面体法线打包与大地水准面法线增删、着色器顶点属性布局（terrain_encoding_spec.rs） |
| [packages/engine/Specs/Core/TerrainMeshSpec.js](../../cesium/packages/engine/Specs/Core/TerrainMeshSpec.js) | 已移植：地形网格结构（TerrainMesh）元数据封装、3D/2D/ColumbusView 模型到世界坐标变换（OrientedBoundingBox 投影与 Swizzle 3D 到 2D 变换）、高程夸大动态重算、场景模式切换感知与拾取变换维护（terrain_mesh_spec.rs） |
| [packages/engine/Specs/Core/TerrainPickerSpec.js](../../cesium/packages/engine/Specs/Core/TerrainPickerSpec.js) | 已移植：地形四叉树空间拾取器（TerrainPicker）、包围盒与光线求交过滤、多重三角形沿光线最近点解算、光线反向三角形剔除与背面剔除判定（cullBackFaces）、多次拾取幂等稳定性（terrain_picker_spec.rs） |
| [packages/engine/Specs/Core/TileAvailabilitySpec.js](../../cesium/packages/engine/Specs/Core/TileAvailabilitySpec.js) | 已移植：瓦片四叉树可用性索引（TileAvailability）、点处最大可用等级对数查询、矩形区域多层级覆盖差集解算（含180度日界线穿越）、子瓦片掩码计算与矩形排序（tile_availability_spec.rs） |
| [packages/engine/Specs/Core/TrustedServersSpec.js](../../cesium/packages/engine/Specs/Core/TrustedServersSpec.js) | 已补齐（`crates/cesium-math/tests/trusted_servers_spec.rs`，覆盖 host 与 port 校验、HTTP/HTTPS 默认端口推导、自定义端口匹配、用户名密码认证 URL 剥离、相对路径过滤、协议无关 URL 及全局单例 TrustedServers API） |
| [packages/engine/Specs/Core/VRTheWorldTerrainProviderSpec.js](../../cesium/packages/engine/Specs/Core/VRTheWorldTerrainProviderSpec.js) | 已补齐（`crates/cesium-math/tests/vr_the_world_terrain_provider_spec.rs`，覆盖 TileMap XML 元数据解析与 EPSG:4326 SRS 校验、各级几何误差推导与折半计算、DataExtents 范围子瓦片可用性掩码解算、TMS y 坐标翻转瓦片 URL 构造与 HeightmapTerrainData 瓦片几何请求） |
| [packages/engine/Specs/Core/VectorProviderSpec.js](../../cesium/packages/engine/Specs/Core/VectorProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Core/VideoSynchronizerSpec.js](../../cesium/packages/engine/Specs/Core/VideoSynchronizerSpec.js) | 已补齐（`crates/cesium-math/tests/video_synchronizer_spec.rs`，覆盖构造参数校验、Clock 仿真时钟与视频播放时间同步计算、时钟启停状态联动、时钟速率倍率应用及对象生命周期销毁） |
| [packages/engine/Specs/Core/WebMercatorTilingSchemeSpec.js](../../cesium/packages/engine/Specs/Core/WebMercatorTilingSchemeSpec.js) | 已移植：Web 墨卡托瓦片方案、单根瓦片全域覆盖、瓦片索引与原生米坐标映射、极区裁剪与多层级瓦片定位（web_mercator_tiling_scheme_spec.rs） |
| [packages/engine/Specs/Core/WireframeIndexGeneratorSpec.js](../../cesium/packages/engine/Specs/Core/WireframeIndexGeneratorSpec.js) | 已移植：网格线框索引生成器，支持三角形/三角带/三角扇（Triangles/TriangleStrip/TriangleFan）索引重排与线段缓冲生成、非三角形图元过滤、线框索引容量计算（wireframe_index_generator_spec.rs） |
| [packages/engine/Specs/Core/addAllSpec.js](../../cesium/packages/engine/Specs/Core/addAllSpec.js) | 已移植：数组批量合并、None/空源切片安全过滤、大容量数组扩展（add_all_spec.rs） |
| [packages/engine/Specs/Core/appendForwardSlashSpec.js](../../cesium/packages/engine/Specs/Core/appendForwardSlashSpec.js) | 已移植：URL 末尾斜杠自动规范化、空字符串与已包含斜杠安全处理（append_forward_slash_spec.rs） |
| [packages/engine/Specs/Core/buildModuleUrlSpec.js](../../cesium/packages/engine/Specs/Core/buildModuleUrlSpec.js) | 已移植：Cesium 模块相对/绝对 URL 拼接与正则校验、CESIUM_BASE_URL 动态配置与跨平台路径合成（build_module_url_spec.rs） |
| [packages/engine/Specs/Core/createWorldBathymetryAsyncSpec.js](../../cesium/packages/engine/Specs/Core/createWorldBathymetryAsyncSpec.js) | 已补齐（`crates/cesium-math/tests/create_world_bathymetry_async_spec.rs`，覆盖 Cesium World Bathymetry Ion 资产 ID 2426648 实例化、requestVertexNormals 标志透传及 CesiumTerrainProvider 解析） |
| [packages/engine/Specs/Core/createWorldTerrainAsyncSpec.js](../../cesium/packages/engine/Specs/Core/createWorldTerrainAsyncSpec.js) | 已补齐（`crates/cesium-math/tests/create_world_terrain_async_spec.rs`，覆盖 Cesium World Terrain Ion 资产 ID 1 实例化、requestVertexNormals 与 requestWaterMask 标志透传、默认 WGS84 椭球体与自定义椭球体配置及 CesiumTerrainProvider 解析） |
| [packages/engine/Specs/Core/getImageFromTypedArraySpec.js](../../cesium/packages/engine/Specs/Core/getImageFromTypedArraySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Core/isBlobUriSpec.js](../../cesium/packages/engine/Specs/Core/isBlobUriSpec.js) | 已移植：Blob URI 前缀判定、大小写不敏感解析与普通 URL 过滤（is_blob_uri_spec.rs） |
| [packages/engine/Specs/Core/isCrossOriginUrlSpec.js](../../cesium/packages/engine/Specs/Core/isCrossOriginUrlSpec.js) | 已移植：跨域 URL 判定、相对路径同源直通、协议/主机/端口差异比对（is_cross_origin_url_spec.rs） |
| [packages/engine/Specs/Core/isDataUriSpec.js](../../cesium/packages/engine/Specs/Core/isDataUriSpec.js) | 已移植：Data URI 协议前缀判定、Base64 数据标识检测与普通 URL 过滤（is_data_uri_spec.rs） |
| [packages/engine/Specs/Core/loadImageFromTypedArraySpec.js](../../cesium/packages/engine/Specs/Core/loadImageFromTypedArraySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Core/loadKTX2Spec.js](../../cesium/packages/engine/Specs/Core/loadKTX2Spec.js) | 待逐项核对 |
| [packages/engine/Specs/Core/parseResponseHeadersSpec.js](../../cesium/packages/engine/Specs/Core/parseResponseHeadersSpec.js) | 已补齐（`crates/cesium-math/tests/parse_response_headers_spec.rs`，覆盖 null/None/空字符串假值安全处理、CRLF 响应头分割、键值冒号解析及特殊值含冒号保持） |
| [packages/engine/Specs/Core/resizeImageToNextPowerOfTwoSpec.js](../../cesium/packages/engine/Specs/Core/resizeImageToNextPowerOfTwoSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Core/sampleTerrainMostDetailedSpec.js](../../cesium/packages/engine/Specs/Core/sampleTerrainMostDetailedSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Core/sampleTerrainSpec.js](../../cesium/packages/engine/Specs/Core/sampleTerrainSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Core/writeTextToCanvasSpec.js](../../cesium/packages/engine/Specs/Core/writeTextToCanvasSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/BillboardGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/BillboardGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/BillboardVisualizerSpec.js](../../cesium/packages/engine/Specs/DataSources/BillboardVisualizerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/BoxGeometryUpdaterSpec.js](../../cesium/packages/engine/Specs/DataSources/BoxGeometryUpdaterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/BoxGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/BoxGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/CallbackPositionPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/CallbackPositionPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/CallbackPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/CallbackPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/Cesium3DTilesetGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/Cesium3DTilesetGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/Cesium3DTilesetVisualizerSpec.js](../../cesium/packages/engine/Specs/DataSources/Cesium3DTilesetVisualizerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/CheckerboardMaterialPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/CheckerboardMaterialPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/ColorMaterialPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/ColorMaterialPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/CompositeEntityCollectionSpec.js](../../cesium/packages/engine/Specs/DataSources/CompositeEntityCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/CompositeMaterialPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/CompositeMaterialPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/CompositePositionPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/CompositePositionPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/CompositePropertySpec.js](../../cesium/packages/engine/Specs/DataSources/CompositePropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/ConstantPositionPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/ConstantPositionPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/ConstantPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/ConstantPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/CorridorGeometryUpdaterSpec.js](../../cesium/packages/engine/Specs/DataSources/CorridorGeometryUpdaterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/CorridorGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/CorridorGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/CustomDataSourceSpec.js](../../cesium/packages/engine/Specs/DataSources/CustomDataSourceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/CylinderGeometryUpdaterSpec.js](../../cesium/packages/engine/Specs/DataSources/CylinderGeometryUpdaterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/CylinderGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/CylinderGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/CzmlDataSourceSpec.js](../../cesium/packages/engine/Specs/DataSources/CzmlDataSourceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/DataSourceClockSpec.js](../../cesium/packages/engine/Specs/DataSources/DataSourceClockSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/DataSourceCollectionSpec.js](../../cesium/packages/engine/Specs/DataSources/DataSourceCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/DataSourceDisplaySpec.js](../../cesium/packages/engine/Specs/DataSources/DataSourceDisplaySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/DynamicGeometryUpdaterSpec.js](../../cesium/packages/engine/Specs/DataSources/DynamicGeometryUpdaterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/EllipseGeometryUpdaterSpec.js](../../cesium/packages/engine/Specs/DataSources/EllipseGeometryUpdaterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/EllipseGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/EllipseGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/EllipsoidGeometryUpdaterSpec.js](../../cesium/packages/engine/Specs/DataSources/EllipsoidGeometryUpdaterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/EllipsoidGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/EllipsoidGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/EntityClusterSpec.js](../../cesium/packages/engine/Specs/DataSources/EntityClusterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/EntityCollectionSpec.js](../../cesium/packages/engine/Specs/DataSources/EntityCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/EntitySpec.js](../../cesium/packages/engine/Specs/DataSources/EntitySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/EntityViewSpec.js](../../cesium/packages/engine/Specs/DataSources/EntityViewSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/GeoJsonDataSourceSpec.js](../../cesium/packages/engine/Specs/DataSources/GeoJsonDataSourceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/GeometryUpdaterSetSpec.js](../../cesium/packages/engine/Specs/DataSources/GeometryUpdaterSetSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/GeometryUpdaterSpec.js](../../cesium/packages/engine/Specs/DataSources/GeometryUpdaterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/GeometryVisualizerSpec.js](../../cesium/packages/engine/Specs/DataSources/GeometryVisualizerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/GpxDataSourceSpec.js](../../cesium/packages/engine/Specs/DataSources/GpxDataSourceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/GridMaterialPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/GridMaterialPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/GroundGeometryUpdaterSpec.js](../../cesium/packages/engine/Specs/DataSources/GroundGeometryUpdaterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/ImageMaterialPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/ImageMaterialPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/KmlDataSourceSpec.js](../../cesium/packages/engine/Specs/DataSources/KmlDataSourceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/KmlTourFlyToSpec.js](../../cesium/packages/engine/Specs/DataSources/KmlTourFlyToSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/KmlTourSpec.js](../../cesium/packages/engine/Specs/DataSources/KmlTourSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/LabelGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/LabelGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/LabelVisualizerSpec.js](../../cesium/packages/engine/Specs/DataSources/LabelVisualizerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/ModelGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/ModelGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/ModelVisualizerSpec.js](../../cesium/packages/engine/Specs/DataSources/ModelVisualizerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/NodeTransformationPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/NodeTransformationPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PathGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/PathGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PathVisualizerSpec.js](../../cesium/packages/engine/Specs/DataSources/PathVisualizerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PlaneGeometryUpdaterSpec.js](../../cesium/packages/engine/Specs/DataSources/PlaneGeometryUpdaterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PlaneGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/PlaneGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PointGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/PointGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PointVisualizerSpec.js](../../cesium/packages/engine/Specs/DataSources/PointVisualizerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PolygonGeometryUpdaterSpec.js](../../cesium/packages/engine/Specs/DataSources/PolygonGeometryUpdaterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PolygonGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/PolygonGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PolylineArrowMaterialPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/PolylineArrowMaterialPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PolylineDashMaterialPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/PolylineDashMaterialPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PolylineGeometryUpdaterSpec.js](../../cesium/packages/engine/Specs/DataSources/PolylineGeometryUpdaterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PolylineGlowMaterialPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/PolylineGlowMaterialPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PolylineGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/PolylineGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PolylineOutlineMaterialPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/PolylineOutlineMaterialPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PolylineVisualizerSpec.js](../../cesium/packages/engine/Specs/DataSources/PolylineVisualizerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PolylineVolumeGeometryUpdaterSpec.js](../../cesium/packages/engine/Specs/DataSources/PolylineVolumeGeometryUpdaterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PolylineVolumeGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/PolylineVolumeGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PositionPropertyArraySpec.js](../../cesium/packages/engine/Specs/DataSources/PositionPropertyArraySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PropertyArraySpec.js](../../cesium/packages/engine/Specs/DataSources/PropertyArraySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/PropertyBagSpec.js](../../cesium/packages/engine/Specs/DataSources/PropertyBagSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/RectangleGeometryUpdaterSpec.js](../../cesium/packages/engine/Specs/DataSources/RectangleGeometryUpdaterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/RectangleGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/RectangleGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/ReferencePropertySpec.js](../../cesium/packages/engine/Specs/DataSources/ReferencePropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/RotationSpec.js](../../cesium/packages/engine/Specs/DataSources/RotationSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/SampledPositionPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/SampledPositionPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/SampledPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/SampledPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/StaticGeometryColorBatchSpec.js](../../cesium/packages/engine/Specs/DataSources/StaticGeometryColorBatchSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/StaticGeometryPerMaterialBatchSpec.js](../../cesium/packages/engine/Specs/DataSources/StaticGeometryPerMaterialBatchSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/StaticGroundGeometryColorBatchSpec.js](../../cesium/packages/engine/Specs/DataSources/StaticGroundGeometryColorBatchSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/StaticGroundGeometryPerMaterialBatchSpec.js](../../cesium/packages/engine/Specs/DataSources/StaticGroundGeometryPerMaterialBatchSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/StaticGroundPolylinePerMaterialBatchSpec.js](../../cesium/packages/engine/Specs/DataSources/StaticGroundPolylinePerMaterialBatchSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/StaticOutlineGeometryBatchSpec.js](../../cesium/packages/engine/Specs/DataSources/StaticOutlineGeometryBatchSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/StripeMaterialPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/StripeMaterialPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/TerrainOffsetPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/TerrainOffsetPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/TimeIntervalCollectionPositionPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/TimeIntervalCollectionPositionPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/TimeIntervalCollectionPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/TimeIntervalCollectionPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/VelocityOrientationPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/VelocityOrientationPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/VelocityVectorPropertySpec.js](../../cesium/packages/engine/Specs/DataSources/VelocityVectorPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/WallGeometryUpdaterSpec.js](../../cesium/packages/engine/Specs/DataSources/WallGeometryUpdaterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/WallGraphicsSpec.js](../../cesium/packages/engine/Specs/DataSources/WallGraphicsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/createMaterialPropertyDescriptorSpec.js](../../cesium/packages/engine/Specs/DataSources/createMaterialPropertyDescriptorSpec.js) | 待逐项核对 |
| [packages/engine/Specs/DataSources/exportKmlSpec.js](../../cesium/packages/engine/Specs/DataSources/exportKmlSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/AutomaticUniformSpec.js](../../cesium/packages/engine/Specs/Renderer/AutomaticUniformSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/BufferSpec.js](../../cesium/packages/engine/Specs/Renderer/BufferSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/BuiltinFunctionsSpec.js](../../cesium/packages/engine/Specs/Renderer/BuiltinFunctionsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/ClearCommandSpec.js](../../cesium/packages/engine/Specs/Renderer/ClearCommandSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/ClearSpec.js](../../cesium/packages/engine/Specs/Renderer/ClearSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/ComputeCommandSpec.js](../../cesium/packages/engine/Specs/Renderer/ComputeCommandSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/ContextSpec.js](../../cesium/packages/engine/Specs/Renderer/ContextSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/CubeMapSpec.js](../../cesium/packages/engine/Specs/Renderer/CubeMapSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/DrawCommandSpec.js](../../cesium/packages/engine/Specs/Renderer/DrawCommandSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/DrawSpec.js](../../cesium/packages/engine/Specs/Renderer/DrawSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/FramebufferManagerSpec.js](../../cesium/packages/engine/Specs/Renderer/FramebufferManagerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/FramebufferSpec.js](../../cesium/packages/engine/Specs/Renderer/FramebufferSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/MultisampleFramebufferSpec.js](../../cesium/packages/engine/Specs/Renderer/MultisampleFramebufferSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/PassStateSpec.js](../../cesium/packages/engine/Specs/Renderer/PassStateSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/PickIdSpec.js](../../cesium/packages/engine/Specs/Renderer/PickIdSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/RenderStateSpec.js](../../cesium/packages/engine/Specs/Renderer/RenderStateSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/RenderbufferSpec.js](../../cesium/packages/engine/Specs/Renderer/RenderbufferSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/SamplerSpec.js](../../cesium/packages/engine/Specs/Renderer/SamplerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/ShaderBuilderSpec.js](../../cesium/packages/engine/Specs/Renderer/ShaderBuilderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/ShaderCacheSpec.js](../../cesium/packages/engine/Specs/Renderer/ShaderCacheSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/ShaderDestinationSpec.js](../../cesium/packages/engine/Specs/Renderer/ShaderDestinationSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/ShaderFunctionSpec.js](../../cesium/packages/engine/Specs/Renderer/ShaderFunctionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/ShaderProgramSpec.js](../../cesium/packages/engine/Specs/Renderer/ShaderProgramSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/ShaderSourceSpec.js](../../cesium/packages/engine/Specs/Renderer/ShaderSourceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/ShaderStructSpec.js](../../cesium/packages/engine/Specs/Renderer/ShaderStructSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/SharedContextSpec.js](../../cesium/packages/engine/Specs/Renderer/SharedContextSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/SyncSpec.js](../../cesium/packages/engine/Specs/Renderer/SyncSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/Texture3DSpec.js](../../cesium/packages/engine/Specs/Renderer/Texture3DSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/TextureAtlasSpec.js](../../cesium/packages/engine/Specs/Renderer/TextureAtlasSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/TextureCacheSpec.js](../../cesium/packages/engine/Specs/Renderer/TextureCacheSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/TextureSpec.js](../../cesium/packages/engine/Specs/Renderer/TextureSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/UniformSpec.js](../../cesium/packages/engine/Specs/Renderer/UniformSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/VertexArrayFacadeSpec.js](../../cesium/packages/engine/Specs/Renderer/VertexArrayFacadeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/VertexArrayFactorySpec.js](../../cesium/packages/engine/Specs/Renderer/VertexArrayFactorySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/VertexArraySpec.js](../../cesium/packages/engine/Specs/Renderer/VertexArraySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/demodernizeShaderSpec.js](../../cesium/packages/engine/Specs/Renderer/demodernizeShaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/freezeRenderStateSpec.js](../../cesium/packages/engine/Specs/Renderer/freezeRenderStateSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Renderer/loadCubeMapSpec.js](../../cesium/packages/engine/Specs/Renderer/loadCubeMapSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/AppearanceSpec.js](../../cesium/packages/engine/Specs/Scene/AppearanceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ArcGisMapServerImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/ArcGisMapServerImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/AxisSpec.js](../../cesium/packages/engine/Specs/Scene/AxisSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Azure2DImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/Azure2DImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/B3dmParserSpec.js](../../cesium/packages/engine/Specs/Scene/B3dmParserSpec.js) | 部分：无效版本/空内容、两种旧头和旧 BATCHID 解码；告警与渲染待验证 |
| [packages/engine/Specs/Scene/BatchTableHierarchySpec.js](../../cesium/packages/engine/Specs/Scene/BatchTableHierarchySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/BatchTableSpec.js](../../cesium/packages/engine/Specs/Scene/BatchTableSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/BatchTextureSpec.js](../../cesium/packages/engine/Specs/Scene/BatchTextureSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/BillboardCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/BillboardCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/BingMapsImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/BingMapsImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/BoundingVolumeSemanticsSpec.js](../../cesium/packages/engine/Specs/Scene/BoundingVolumeSemanticsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/BoxEmitterSpec.js](../../cesium/packages/engine/Specs/Scene/BoxEmitterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/BufferLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/BufferLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/BufferPointCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/BufferPointCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/BufferPointMaterialSpec.js](../../cesium/packages/engine/Specs/Scene/BufferPointMaterialSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/BufferPolygonCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/BufferPolygonCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/BufferPolygonMaterialSpec.js](../../cesium/packages/engine/Specs/Scene/BufferPolygonMaterialSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/BufferPolylineCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/BufferPolylineCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/BufferPolylineMaterialSpec.js](../../cesium/packages/engine/Specs/Scene/BufferPolylineMaterialSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/CameraEventAggregatorSpec.js](../../cesium/packages/engine/Specs/Scene/CameraEventAggregatorSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/CameraFlightPathSpec.js](../../cesium/packages/engine/Specs/Scene/CameraFlightPathSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/CameraSpec.js](../../cesium/packages/engine/Specs/Scene/CameraSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Cesium3DContentGroupSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DContentGroupSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Cesium3DTileBatchTableSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DTileBatchTableSpec.js) | 部分：二进制标量/向量/矩阵读取及真实样例；属性修改、层级、渲染等待移植 |
| [packages/engine/Specs/Scene/Cesium3DTileContentTypeSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DTileContentTypeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Cesium3DTileFeatureSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DTileFeatureSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Cesium3DTileFeatureTableSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DTileFeatureTableSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Cesium3DTilePassStateSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DTilePassStateSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Cesium3DTileSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DTileSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Cesium3DTileStyleSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DTileStyleSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Cesium3DTileVectorFeatureSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DTileVectorFeatureSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Cesium3DTilesVoxelProviderSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DTilesVoxelProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Cesium3DTilesetBaseTraversalSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DTilesetBaseTraversalSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Cesium3DTilesetHeatmapSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DTilesetHeatmapSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Cesium3DTilesetMetadataSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DTilesetMetadataSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Cesium3DTilesetMostDetailedTraversalSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DTilesetMostDetailedTraversalSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Cesium3DTilesetSkipTraversalSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DTilesetSkipTraversalSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Cesium3DTilesetSpec.js](../../cesium/packages/engine/Specs/Scene/Cesium3DTilesetSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/CircleEmitterSpec.js](../../cesium/packages/engine/Specs/Scene/CircleEmitterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ClassificationPrimitiveSpec.js](../../cesium/packages/engine/Specs/Scene/ClassificationPrimitiveSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ClippingPlaneCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/ClippingPlaneCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ClippingPlaneSpec.js](../../cesium/packages/engine/Specs/Scene/ClippingPlaneSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ClippingPolygonCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/ClippingPolygonCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ClippingPolygonSpec.js](../../cesium/packages/engine/Specs/Scene/ClippingPolygonSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/CloudCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/CloudCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Composite3DTileContentSpec.js](../../cesium/packages/engine/Specs/Scene/Composite3DTileContentSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ConditionsExpressionSpec.js](../../cesium/packages/engine/Specs/Scene/ConditionsExpressionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ConeEmitterSpec.js](../../cesium/packages/engine/Specs/Scene/ConeEmitterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ContentMetadataSpec.js](../../cesium/packages/engine/Specs/Scene/ContentMetadataSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/CreditDisplaySpec.js](../../cesium/packages/engine/Specs/Scene/CreditDisplaySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/CubeMapPanoramaSpec.js](../../cesium/packages/engine/Specs/Scene/CubeMapPanoramaSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/DebugAppearanceSpec.js](../../cesium/packages/engine/Specs/Scene/DebugAppearanceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/DebugCameraPrimitiveSpec.js](../../cesium/packages/engine/Specs/Scene/DebugCameraPrimitiveSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/DebugModelMatrixPrimitiveSpec.js](../../cesium/packages/engine/Specs/Scene/DebugModelMatrixPrimitiveSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/DepthPlaneSpec.js](../../cesium/packages/engine/Specs/Scene/DepthPlaneSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/DeviceOrientationCameraControllerSpec.js](../../cesium/packages/engine/Specs/Scene/DeviceOrientationCameraControllerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/DirectionalLightSpec.js](../../cesium/packages/engine/Specs/Scene/DirectionalLightSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/DiscardEmptyTileImagePolicySpec.js](../../cesium/packages/engine/Specs/Scene/DiscardEmptyTileImagePolicySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/DiscardMissingTileImagePolicySpec.js](../../cesium/packages/engine/Specs/Scene/DiscardMissingTileImagePolicySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/DynamicAtmosphereLightingTypeSpec.js](../../cesium/packages/engine/Specs/Scene/DynamicAtmosphereLightingTypeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/DynamicEnvironmentMapManagerSpec.js](../../cesium/packages/engine/Specs/Scene/DynamicEnvironmentMapManagerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/EdgeFramebufferSpec.js](../../cesium/packages/engine/Specs/Scene/EdgeFramebufferSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/EllipsoidPrimitiveSpec.js](../../cesium/packages/engine/Specs/Scene/EllipsoidPrimitiveSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/EllipsoidSurfaceAppearanceSpec.js](../../cesium/packages/engine/Specs/Scene/EllipsoidSurfaceAppearanceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Empty3DTileContentSpec.js](../../cesium/packages/engine/Specs/Scene/Empty3DTileContentSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/EquirectangularPanoramaSpec.js](../../cesium/packages/engine/Specs/Scene/EquirectangularPanoramaSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ExpressionSpec.js](../../cesium/packages/engine/Specs/Scene/ExpressionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/FogSpec.js](../../cesium/packages/engine/Specs/Scene/FogSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/FrameRateMonitorSpec.js](../../cesium/packages/engine/Specs/Scene/FrameRateMonitorSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/FrustumCommandsSpec.js](../../cesium/packages/engine/Specs/Scene/FrustumCommandsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GaussianSplat3DTileContentSpec.js](../../cesium/packages/engine/Specs/Scene/GaussianSplat3DTileContentSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GaussianSplatPrimitiveSpec.js](../../cesium/packages/engine/Specs/Scene/GaussianSplatPrimitiveSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GeoJsonPrimitiveSpec.js](../../cesium/packages/engine/Specs/Scene/GeoJsonPrimitiveSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Geometry3DTileContentSpec.js](../../cesium/packages/engine/Specs/Scene/Geometry3DTileContentSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GeometryRenderingSpec.js](../../cesium/packages/engine/Specs/Scene/GeometryRenderingSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GlobeSpec.js](../../cesium/packages/engine/Specs/Scene/GlobeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GlobeSurfaceTileProviderSpec.js](../../cesium/packages/engine/Specs/Scene/GlobeSurfaceTileProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GlobeSurfaceTileSpec.js](../../cesium/packages/engine/Specs/Scene/GlobeSurfaceTileSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GlobeTranslucencyFramebufferSpec.js](../../cesium/packages/engine/Specs/Scene/GlobeTranslucencyFramebufferSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GlobeTranslucencyStateSpec.js](../../cesium/packages/engine/Specs/Scene/GlobeTranslucencyStateSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GltfBufferViewLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/GltfBufferViewLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GltfDracoLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/GltfDracoLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GltfImageLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/GltfImageLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GltfIndexBufferLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/GltfIndexBufferLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GltfJsonLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/GltfJsonLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GltfLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/GltfLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GltfLoaderUtilSpec.js](../../cesium/packages/engine/Specs/Scene/GltfLoaderUtilSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GltfSpzLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/GltfSpzLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GltfStructuralMetadataLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/GltfStructuralMetadataLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GltfTextureLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/GltfTextureLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GltfVertexBufferLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/GltfVertexBufferLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Google2DImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/Google2DImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GoogleEarthEnterpriseImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/GoogleEarthEnterpriseImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GoogleEarthEnterpriseMapsProviderSpec.js](../../cesium/packages/engine/Specs/Scene/GoogleEarthEnterpriseMapsProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GoogleStreetViewCubeMapPanoramaProviderSpec.js](../../cesium/packages/engine/Specs/Scene/GoogleStreetViewCubeMapPanoramaProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GridImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/GridImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GroundPolylinePrimitiveSpec.js](../../cesium/packages/engine/Specs/Scene/GroundPolylinePrimitiveSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GroundPrimitiveSpec.js](../../cesium/packages/engine/Specs/Scene/GroundPrimitiveSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/GroupMetadataSpec.js](../../cesium/packages/engine/Specs/Scene/GroupMetadataSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/HeightmapTessellatorSpec.js](../../cesium/packages/engine/Specs/Scene/HeightmapTessellatorSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/I3SDataProviderSpec.js](../../cesium/packages/engine/Specs/Scene/I3SDataProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/I3SDecoderSpec.js](../../cesium/packages/engine/Specs/Scene/I3SDecoderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/I3SFieldSpec.js](../../cesium/packages/engine/Specs/Scene/I3SFieldSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/I3SLayerSpec.js](../../cesium/packages/engine/Specs/Scene/I3SLayerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/I3SNodeSpec.js](../../cesium/packages/engine/Specs/Scene/I3SNodeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/I3SSymbologySpec.js](../../cesium/packages/engine/Specs/Scene/I3SSymbologySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/I3dmParserSpec.js](../../cesium/packages/engine/Specs/Scene/I3dmParserSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ITwinDataSpec.js](../../cesium/packages/engine/Specs/Scene/ITwinDataSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ImageBasedLightingSpec.js](../../cesium/packages/engine/Specs/Scene/ImageBasedLightingSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ImageryLayerCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/ImageryLayerCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ImageryLayerSpec.js](../../cesium/packages/engine/Specs/Scene/ImageryLayerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Implicit3DTileContentSpec.js](../../cesium/packages/engine/Specs/Scene/Implicit3DTileContentSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ImplicitAvailabilityBitstreamSpec.js](../../cesium/packages/engine/Specs/Scene/ImplicitAvailabilityBitstreamSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ImplicitMetadataViewSpec.js](../../cesium/packages/engine/Specs/Scene/ImplicitMetadataViewSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ImplicitSubdivisionSchemeSpec.js](../../cesium/packages/engine/Specs/Scene/ImplicitSubdivisionSchemeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ImplicitSubtreeCacheSpec.js](../../cesium/packages/engine/Specs/Scene/ImplicitSubtreeCacheSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ImplicitSubtreeMetadataSpec.js](../../cesium/packages/engine/Specs/Scene/ImplicitSubtreeMetadataSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ImplicitSubtreeSpec.js](../../cesium/packages/engine/Specs/Scene/ImplicitSubtreeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ImplicitTileCoordinatesSpec.js](../../cesium/packages/engine/Specs/Scene/ImplicitTileCoordinatesSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ImplicitTilesetSpec.js](../../cesium/packages/engine/Specs/Scene/ImplicitTilesetSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/InstanceAttributeSemanticSpec.js](../../cesium/packages/engine/Specs/Scene/InstanceAttributeSemanticSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/IonImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/IonImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/JobSchedulerSpec.js](../../cesium/packages/engine/Specs/Scene/JobSchedulerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/JsonMetadataTableSpec.js](../../cesium/packages/engine/Specs/Scene/JsonMetadataTableSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/KeyframeNodeSpec.js](../../cesium/packages/engine/Specs/Scene/KeyframeNodeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/LabelCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/LabelCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/LabelSpec.js](../../cesium/packages/engine/Specs/Scene/LabelSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/LightSpec.js](../../cesium/packages/engine/Specs/Scene/LightSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MapboxImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/MapboxImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MapboxStyleImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/MapboxStyleImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MaterialAppearanceSpec.js](../../cesium/packages/engine/Specs/Scene/MaterialAppearanceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MaterialSpec.js](../../cesium/packages/engine/Specs/Scene/MaterialSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MegatextureSpec.js](../../cesium/packages/engine/Specs/Scene/MegatextureSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MetadataClassPropertySpec.js](../../cesium/packages/engine/Specs/Scene/MetadataClassPropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MetadataClassSpec.js](../../cesium/packages/engine/Specs/Scene/MetadataClassSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MetadataComponentTypeSpec.js](../../cesium/packages/engine/Specs/Scene/MetadataComponentTypeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MetadataEntitySpec.js](../../cesium/packages/engine/Specs/Scene/MetadataEntitySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MetadataEnumSpec.js](../../cesium/packages/engine/Specs/Scene/MetadataEnumSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MetadataEnumValueSpec.js](../../cesium/packages/engine/Specs/Scene/MetadataEnumValueSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MetadataPickingSpec.js](../../cesium/packages/engine/Specs/Scene/MetadataPickingSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MetadataSchemaLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/MetadataSchemaLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MetadataSchemaSpec.js](../../cesium/packages/engine/Specs/Scene/MetadataSchemaSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MetadataTablePropertySpec.js](../../cesium/packages/engine/Specs/Scene/MetadataTablePropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MetadataTableSpec.js](../../cesium/packages/engine/Specs/Scene/MetadataTableSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MetadataTypeSpec.js](../../cesium/packages/engine/Specs/Scene/MetadataTypeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/AlphaPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/AlphaPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/AtmospherePipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/AtmospherePipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/B3dmLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/Model/B3dmLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/BatchTexturePipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/BatchTexturePipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/CPUStylingPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/CPUStylingPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ClassificationModelDrawCommandSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ClassificationModelDrawCommandSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ClassificationPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ClassificationPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/CustomShaderPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/CustomShaderPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/CustomShaderSpec.js](../../cesium/packages/engine/Specs/Scene/Model/CustomShaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/DequantizationPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/DequantizationPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/EdgeVisibilityPipelineStageDecodingSpec.js](../../cesium/packages/engine/Specs/Scene/Model/EdgeVisibilityPipelineStageDecodingSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/EdgeVisibilityRenderingSpec.js](../../cesium/packages/engine/Specs/Scene/Model/EdgeVisibilityRenderingSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/Extensions/Gpm/GltfGpmLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/Model/Extensions/Gpm/GltfGpmLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/Extensions/Gpm/GltfMeshPrimitiveGpmLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/Model/Extensions/Gpm/GltfMeshPrimitiveGpmLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/FeatureIdPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/FeatureIdPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/GeoJsonLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/Model/GeoJsonLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/GeometryPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/GeometryPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/I3dmLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/Model/I3dmLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ImageBasedLightingPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ImageBasedLightingPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ImageryPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ImageryPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/InstancingPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/InstancingPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/LightingPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/LightingPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/MappedPositionsSpec.js](../../cesium/packages/engine/Specs/Scene/Model/MappedPositionsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/MaterialPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/MaterialPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/MetadataPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/MetadataPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/Model3DTileContentSpec.js](../../cesium/packages/engine/Specs/Scene/Model/Model3DTileContentSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelAnimationChannelSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelAnimationChannelSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelAnimationCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelAnimationCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelAnimationSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelAnimationSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelArticulationSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelArticulationSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelArticulationStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelArticulationStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelClippingPlanesPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelClippingPlanesPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelClippingPolygonsPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelClippingPolygonsPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelColorPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelColorPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelDrawCommandSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelDrawCommandSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelFeatureSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelFeatureSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelFeatureTableSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelFeatureTableSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelImageryMappingSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelImageryMappingSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelImagerySpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelImagerySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelMatrixUpdateStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelMatrixUpdateStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelNodeSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelNodeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelPrimitiveImagerySpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelPrimitiveImagerySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelReaderSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelReaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelRenderResourcesSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelRenderResourcesSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelRuntimeNodeSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelRuntimeNodeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelRuntimePrimitiveSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelRuntimePrimitiveSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelSceneGraphSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelSceneGraphSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelSilhouettePipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelSilhouettePipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelSkinSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelSkinSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelSplitterPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelSplitterPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelStatisticsSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelStatisticsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelTypeSpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelTypeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/ModelUtilitySpec.js](../../cesium/packages/engine/Specs/Scene/Model/ModelUtilitySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/MorphTargetsPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/MorphTargetsPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/NodeRenderResourcesSpec.js](../../cesium/packages/engine/Specs/Scene/Model/NodeRenderResourcesSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/NodeStatisticsPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/NodeStatisticsPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/PickingPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/PickingPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/PntsLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/Model/PntsLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/PointCloudStylingPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/PointCloudStylingPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/PrimitiveLoadPlanSpec.js](../../cesium/packages/engine/Specs/Scene/Model/PrimitiveLoadPlanSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/PrimitiveOutlineGeneratorSpec.js](../../cesium/packages/engine/Specs/Scene/Model/PrimitiveOutlineGeneratorSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/PrimitiveOutlinePipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/PrimitiveOutlinePipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/PrimitiveRenderResourcesSpec.js](../../cesium/packages/engine/Specs/Scene/Model/PrimitiveRenderResourcesSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/PrimitiveStatisticsPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/PrimitiveStatisticsPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/SceneMode2DPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/SceneMode2DPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/SelectedFeatureIdPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/SelectedFeatureIdPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/SkinningPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/SkinningPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/TextureManagerSpec.js](../../cesium/packages/engine/Specs/Scene/Model/TextureManagerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/TextureUniformSpec.js](../../cesium/packages/engine/Specs/Scene/Model/TextureUniformSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/TilesetPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/TilesetPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/VerticalExaggerationPipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/VerticalExaggerationPipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/WireframePipelineStageSpec.js](../../cesium/packages/engine/Specs/Scene/Model/WireframePipelineStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/createVectorTileBuffersFromModelComponentsSpec.js](../../cesium/packages/engine/Specs/Scene/Model/createVectorTileBuffersFromModelComponentsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Model/pickModelSpec.js](../../cesium/packages/engine/Specs/Scene/Model/pickModelSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MoonSpec.js](../../cesium/packages/engine/Specs/Scene/MoonSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/MultifrustumSpec.js](../../cesium/packages/engine/Specs/Scene/MultifrustumSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Multiple3DTileContentSpec.js](../../cesium/packages/engine/Specs/Scene/Multiple3DTileContentSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/OpenStreetMapImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/OpenStreetMapImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/OrderedGroundPrimitiveCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/OrderedGroundPrimitiveCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PanoramaSpec.js](../../cesium/packages/engine/Specs/Scene/PanoramaSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ParticleSpec.js](../../cesium/packages/engine/Specs/Scene/ParticleSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ParticleSystemSpec.js](../../cesium/packages/engine/Specs/Scene/ParticleSystemSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PerInstanceColorAppearanceSpec.js](../../cesium/packages/engine/Specs/Scene/PerInstanceColorAppearanceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PickingSpec.js](../../cesium/packages/engine/Specs/Scene/PickingSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PlanarFillIdFramebufferSpec.js](../../cesium/packages/engine/Specs/Scene/PlanarFillIdFramebufferSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PntsParserSpec.js](../../cesium/packages/engine/Specs/Scene/PntsParserSpec.js) | 部分：必需字段、版本、量化位置、批次、内联属性及 Draco 完整/部分/批次点云；其余行为待补 |
| [packages/engine/Specs/Scene/PointCloudEyeDomeLightingSpec.js](../../cesium/packages/engine/Specs/Scene/PointCloudEyeDomeLightingSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PointCloudShadingSpec.js](../../cesium/packages/engine/Specs/Scene/PointCloudShadingSpec.js) | 部分：尺寸、几何误差缩放、衰减、法线着色及背面剔除；EDL 和支持性检测待实现 |
| [packages/engine/Specs/Scene/PointPrimitiveCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/PointPrimitiveCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PolylineCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/PolylineCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PolylineColorAppearanceSpec.js](../../cesium/packages/engine/Specs/Scene/PolylineColorAppearanceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PolylineMaterialAppearanceSpec.js](../../cesium/packages/engine/Specs/Scene/PolylineMaterialAppearanceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PostProcessStageCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/PostProcessStageCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PostProcessStageCompositeSpec.js](../../cesium/packages/engine/Specs/Scene/PostProcessStageCompositeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PostProcessStageLibrarySpec.js](../../cesium/packages/engine/Specs/Scene/PostProcessStageLibrarySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PostProcessStageSpec.js](../../cesium/packages/engine/Specs/Scene/PostProcessStageSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PrimitiveCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/PrimitiveCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PrimitiveCullingSpec.js](../../cesium/packages/engine/Specs/Scene/PrimitiveCullingSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PrimitivePipelineSpec.js](../../cesium/packages/engine/Specs/Scene/PrimitivePipelineSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PrimitiveSpec.js](../../cesium/packages/engine/Specs/Scene/PrimitiveSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PropertyAttributePropertySpec.js](../../cesium/packages/engine/Specs/Scene/PropertyAttributePropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PropertyAttributeSpec.js](../../cesium/packages/engine/Specs/Scene/PropertyAttributeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PropertyTableSpec.js](../../cesium/packages/engine/Specs/Scene/PropertyTableSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PropertyTexturePropertySpec.js](../../cesium/packages/engine/Specs/Scene/PropertyTexturePropertySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/PropertyTextureSpec.js](../../cesium/packages/engine/Specs/Scene/PropertyTextureSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/QuadtreePrimitiveSpec.js](../../cesium/packages/engine/Specs/Scene/QuadtreePrimitiveSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/QuadtreeTileSpec.js](../../cesium/packages/engine/Specs/Scene/QuadtreeTileSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ResourceCacheKeySpec.js](../../cesium/packages/engine/Specs/Scene/ResourceCacheKeySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ResourceCacheSpec.js](../../cesium/packages/engine/Specs/Scene/ResourceCacheSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ResourceCacheStatisticsSpec.js](../../cesium/packages/engine/Specs/Scene/ResourceCacheStatisticsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ResourceLoaderSpec.js](../../cesium/packages/engine/Specs/Scene/ResourceLoaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/SceneSpec.js](../../cesium/packages/engine/Specs/Scene/SceneSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/SceneTransformsSpec.js](../../cesium/packages/engine/Specs/Scene/SceneTransformsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ScreenSpaceCameraControllerSpec.js](../../cesium/packages/engine/Specs/Scene/ScreenSpaceCameraControllerSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ShadowMapShaderSpec.js](../../cesium/packages/engine/Specs/Scene/ShadowMapShaderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ShadowMapSpec.js](../../cesium/packages/engine/Specs/Scene/ShadowMapSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ShadowVolumeAppearanceSpec.js](../../cesium/packages/engine/Specs/Scene/ShadowVolumeAppearanceSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/SingleTileImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/SingleTileImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/SkyAtmosphereSpec.js](../../cesium/packages/engine/Specs/Scene/SkyAtmosphereSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/SkyBoxSpec.js](../../cesium/packages/engine/Specs/Scene/SkyBoxSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/SnappingSpec.js](../../cesium/packages/engine/Specs/Scene/SnappingSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/SpatialNodeSpec.js](../../cesium/packages/engine/Specs/Scene/SpatialNodeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/SpecularEnvironmentCubeMapSpec.js](../../cesium/packages/engine/Specs/Scene/SpecularEnvironmentCubeMapSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/SphereEmitterSpec.js](../../cesium/packages/engine/Specs/Scene/SphereEmitterSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/StructuralMetadataSpec.js](../../cesium/packages/engine/Specs/Scene/StructuralMetadataSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/StyleExpressionSpec.js](../../cesium/packages/engine/Specs/Scene/StyleExpressionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/SunLightSpec.js](../../cesium/packages/engine/Specs/Scene/SunLightSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/SunSpec.js](../../cesium/packages/engine/Specs/Scene/SunSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/SupportedImageFormatsSpec.js](../../cesium/packages/engine/Specs/Scene/SupportedImageFormatsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TerrainFillMeshSpec.js](../../cesium/packages/engine/Specs/Scene/TerrainFillMeshSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TerrainSpec.js](../../cesium/packages/engine/Specs/Scene/TerrainSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TileBoundingRegionSpec.js](../../cesium/packages/engine/Specs/Scene/TileBoundingRegionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TileBoundingS2CellSpec.js](../../cesium/packages/engine/Specs/Scene/TileBoundingS2CellSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TileBoundingSphereSpec.js](../../cesium/packages/engine/Specs/Scene/TileBoundingSphereSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TileBoundingVolumeSpec.js](../../cesium/packages/engine/Specs/Scene/TileBoundingVolumeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TileCoordinatesImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/TileCoordinatesImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TileImagerySpec.js](../../cesium/packages/engine/Specs/Scene/TileImagerySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TileMapServiceImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/TileMapServiceImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TileMetadataSpec.js](../../cesium/packages/engine/Specs/Scene/TileMetadataSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TileOrientedBoundingBoxSpec.js](../../cesium/packages/engine/Specs/Scene/TileOrientedBoundingBoxSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TileReplacementQueueSpec.js](../../cesium/packages/engine/Specs/Scene/TileReplacementQueueSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Tileset3DTileContentSpec.js](../../cesium/packages/engine/Specs/Scene/Tileset3DTileContentSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TilesetMetadataSpec.js](../../cesium/packages/engine/Specs/Scene/TilesetMetadataSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TimeDynamicImagerySpec.js](../../cesium/packages/engine/Specs/Scene/TimeDynamicImagerySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TimeDynamicPointCloudSpec.js](../../cesium/packages/engine/Specs/Scene/TimeDynamicPointCloudSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TranslucentTileClassificationSpec.js](../../cesium/packages/engine/Specs/Scene/TranslucentTileClassificationSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/TweenCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/TweenCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/UrlTemplateImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/UrlTemplateImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Vector3DTileClampedPolylinesSpec.js](../../cesium/packages/engine/Specs/Scene/Vector3DTileClampedPolylinesSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Vector3DTileContentSpec.js](../../cesium/packages/engine/Specs/Scene/Vector3DTileContentSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Vector3DTileGeometrySpec.js](../../cesium/packages/engine/Specs/Scene/Vector3DTileGeometrySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Vector3DTilePointsSpec.js](../../cesium/packages/engine/Specs/Scene/Vector3DTilePointsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Vector3DTilePolygonsSpec.js](../../cesium/packages/engine/Specs/Scene/Vector3DTilePolygonsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/Vector3DTilePolylinesSpec.js](../../cesium/packages/engine/Specs/Scene/Vector3DTilePolylinesSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/VectorGltf3DTileContentSpec.js](../../cesium/packages/engine/Specs/Scene/VectorGltf3DTileContentSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/VertexAttributeSemanticSpec.js](../../cesium/packages/engine/Specs/Scene/VertexAttributeSemanticSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/ViewportQuadSpec.js](../../cesium/packages/engine/Specs/Scene/ViewportQuadSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/VoxelBoxShapeSpec.js](../../cesium/packages/engine/Specs/Scene/VoxelBoxShapeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/VoxelCellSpec.js](../../cesium/packages/engine/Specs/Scene/VoxelCellSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/VoxelCylinderShapeSpec.js](../../cesium/packages/engine/Specs/Scene/VoxelCylinderShapeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/VoxelEllipsoidShapeSpec.js](../../cesium/packages/engine/Specs/Scene/VoxelEllipsoidShapeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/VoxelPrimitiveSpec.js](../../cesium/packages/engine/Specs/Scene/VoxelPrimitiveSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/VoxelRenderResourcesSpec.js](../../cesium/packages/engine/Specs/Scene/VoxelRenderResourcesSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/VoxelShapeTypeSpec.js](../../cesium/packages/engine/Specs/Scene/VoxelShapeTypeSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/VoxelTraversalSpec.js](../../cesium/packages/engine/Specs/Scene/VoxelTraversalSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/WebMapServiceImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/WebMapServiceImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/WebMapTileServiceImageryProviderSpec.js](../../cesium/packages/engine/Specs/Scene/WebMapTileServiceImageryProviderSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/buildVectorGltfFromMVTSpec.js](../../cesium/packages/engine/Specs/Scene/buildVectorGltfFromMVTSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/buildVoxelDrawCommandsSpec.js](../../cesium/packages/engine/Specs/Scene/buildVoxelDrawCommandsSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/computeFlyToLocationForRectangleSpec.js](../../cesium/packages/engine/Specs/Scene/computeFlyToLocationForRectangleSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/createElevationBandMaterialSpec.js](../../cesium/packages/engine/Specs/Scene/createElevationBandMaterialSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/createTangentSpaceDebugPrimitiveSpec.js](../../cesium/packages/engine/Specs/Scene/createTangentSpaceDebugPrimitiveSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/createWorldImageryAsyncSpec.js](../../cesium/packages/engine/Specs/Scene/createWorldImageryAsyncSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/decodeMVTSpec.js](../../cesium/packages/engine/Specs/Scene/decodeMVTSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/findContentMetadataSpec.js](../../cesium/packages/engine/Specs/Scene/findContentMetadataSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/findGroupMetadataSpec.js](../../cesium/packages/engine/Specs/Scene/findGroupMetadataSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/findMeshoptExtensionSpec.js](../../cesium/packages/engine/Specs/Scene/findMeshoptExtensionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/findTileMetadataSpec.js](../../cesium/packages/engine/Specs/Scene/findTileMetadataSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/hasExtensionSpec.js](../../cesium/packages/engine/Specs/Scene/hasExtensionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/parseBatchTableSpec.js](../../cesium/packages/engine/Specs/Scene/parseBatchTableSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/parseFeatureMetadataLegacySpec.js](../../cesium/packages/engine/Specs/Scene/parseFeatureMetadataLegacySpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/parseStructuralMetadataSpec.js](../../cesium/packages/engine/Specs/Scene/parseStructuralMetadataSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/preprocess3DTileContentSpec.js](../../cesium/packages/engine/Specs/Scene/preprocess3DTileContentSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/processVoxelPropertiesSpec.js](../../cesium/packages/engine/Specs/Scene/processVoxelPropertiesSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/renderBufferPointCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/renderBufferPointCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/renderBufferPolygonCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/renderBufferPolygonCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Scene/renderBufferPolylineCollectionSpec.js](../../cesium/packages/engine/Specs/Scene/renderBufferPolylineCollectionSpec.js) | 待逐项核对 |
| [packages/engine/Specs/Widget/CesiumWidgetSpec.js](../../cesium/packages/engine/Specs/Widget/CesiumWidgetSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/ForEachSpec.js](../../cesium/packages/gltf/Specs/ForEachSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/addBufferSpec.js](../../cesium/packages/gltf/Specs/addBufferSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/addDefaultsSpec.js](../../cesium/packages/gltf/Specs/addDefaultsSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/addExtensionsRequiredSpec.js](../../cesium/packages/gltf/Specs/addExtensionsRequiredSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/addExtensionsUsedSpec.js](../../cesium/packages/gltf/Specs/addExtensionsUsedSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/addPipelineExtrasSpec.js](../../cesium/packages/gltf/Specs/addPipelineExtrasSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/addToArraySpec.js](../../cesium/packages/gltf/Specs/addToArraySpec.js) | 待逐项核对 |
| [packages/gltf/Specs/constructBoundingVolumeJsonSpec.js](../../cesium/packages/gltf/Specs/constructBoundingVolumeJsonSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/findAccessorMinMaxSpec.js](../../cesium/packages/gltf/Specs/findAccessorMinMaxSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/forEachTextureInMaterialSpec.js](../../cesium/packages/gltf/Specs/forEachTextureInMaterialSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/getAccessorByteStrideSpec.js](../../cesium/packages/gltf/Specs/getAccessorByteStrideSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/getComponentReaderSpec.js](../../cesium/packages/gltf/Specs/getComponentReaderSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/getNodeTransformSpec.js](../../cesium/packages/gltf/Specs/getNodeTransformSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/moveTechniqueRenderStatesSpec.js](../../cesium/packages/gltf/Specs/moveTechniqueRenderStatesSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/moveTechniquesToExtensionSpec.js](../../cesium/packages/gltf/Specs/moveTechniquesToExtensionSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/numberOfComponentsForTypeSpec.js](../../cesium/packages/gltf/Specs/numberOfComponentsForTypeSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/parseGlbSpec.js](../../cesium/packages/gltf/Specs/parseGlbSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/readAccessorPackedSpec.js](../../cesium/packages/gltf/Specs/readAccessorPackedSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/removeExtensionSpec.js](../../cesium/packages/gltf/Specs/removeExtensionSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/removeExtensionsRequiredSpec.js](../../cesium/packages/gltf/Specs/removeExtensionsRequiredSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/removeExtensionsUsedSpec.js](../../cesium/packages/gltf/Specs/removeExtensionsUsedSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/removePipelineExtrasSpec.js](../../cesium/packages/gltf/Specs/removePipelineExtrasSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/removeUnusedElementsSpec.js](../../cesium/packages/gltf/Specs/removeUnusedElementsSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/updateAccessorComponentTypeSpec.js](../../cesium/packages/gltf/Specs/updateAccessorComponentTypeSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/updateVersionSpec.js](../../cesium/packages/gltf/Specs/updateVersionSpec.js) | 待逐项核对 |
| [packages/gltf/Specs/usesExtensionSpec.js](../../cesium/packages/gltf/Specs/usesExtensionSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/Animation/AnimationSpec.js](../../cesium/packages/widgets/Specs/Animation/AnimationSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/Animation/AnimationViewModelSpec.js](../../cesium/packages/widgets/Specs/Animation/AnimationViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/BaseLayerPicker/BaseLayerPickerSpec.js](../../cesium/packages/widgets/Specs/BaseLayerPicker/BaseLayerPickerSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/BaseLayerPicker/BaseLayerPickerViewModelSpec.js](../../cesium/packages/widgets/Specs/BaseLayerPicker/BaseLayerPickerViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/BaseLayerPicker/ProviderViewModelSpec.js](../../cesium/packages/widgets/Specs/BaseLayerPicker/ProviderViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/Cesium3DTilesInspector/Cesium3DTilesInspectorSpec.js](../../cesium/packages/widgets/Specs/Cesium3DTilesInspector/Cesium3DTilesInspectorSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/Cesium3DTilesInspector/Cesium3DTilesInspectorViewModelSpec.js](../../cesium/packages/widgets/Specs/Cesium3DTilesInspector/Cesium3DTilesInspectorViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/CesiumInspector/CesiumInspectorSpec.js](../../cesium/packages/widgets/Specs/CesiumInspector/CesiumInspectorSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/CesiumInspector/CesiumInspectorViewModelSpec.js](../../cesium/packages/widgets/Specs/CesiumInspector/CesiumInspectorViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/ClockViewModelSpec.js](../../cesium/packages/widgets/Specs/ClockViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/FullscreenButton/FullscreenButtonSpec.js](../../cesium/packages/widgets/Specs/FullscreenButton/FullscreenButtonSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/FullscreenButton/FullscreenButtonViewModelSpec.js](../../cesium/packages/widgets/Specs/FullscreenButton/FullscreenButtonViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/Geocoder/GeocoderSpec.js](../../cesium/packages/widgets/Specs/Geocoder/GeocoderSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/Geocoder/GeocoderViewModelSpec.js](../../cesium/packages/widgets/Specs/Geocoder/GeocoderViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/HomeButton/HomeButtonSpec.js](../../cesium/packages/widgets/Specs/HomeButton/HomeButtonSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/HomeButton/HomeButtonViewModelSpec.js](../../cesium/packages/widgets/Specs/HomeButton/HomeButtonViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/I3SBSLExplorer/I3SBSLExplorerSpec.js](../../cesium/packages/widgets/Specs/I3SBSLExplorer/I3SBSLExplorerSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/I3SBSLExplorer/I3SBSLExplorerViewModelSpec.js](../../cesium/packages/widgets/Specs/I3SBSLExplorer/I3SBSLExplorerViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/InfoBox/InfoBoxSpec.js](../../cesium/packages/widgets/Specs/InfoBox/InfoBoxSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/InfoBox/InfoBoxViewModelSpec.js](../../cesium/packages/widgets/Specs/InfoBox/InfoBoxViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/NavigationHelpButton/NavigationHelpButtonSpec.js](../../cesium/packages/widgets/Specs/NavigationHelpButton/NavigationHelpButtonSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/NavigationHelpButton/NavigationHelpButtonViewModelSpec.js](../../cesium/packages/widgets/Specs/NavigationHelpButton/NavigationHelpButtonViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/PerformanceWatchdog/PerformanceWatchdogSpec.js](../../cesium/packages/widgets/Specs/PerformanceWatchdog/PerformanceWatchdogSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/PerformanceWatchdog/PerformanceWatchdogViewModelSpec.js](../../cesium/packages/widgets/Specs/PerformanceWatchdog/PerformanceWatchdogViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/ProjectionPicker/ProjectionPickerSpec.js](../../cesium/packages/widgets/Specs/ProjectionPicker/ProjectionPickerSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/ProjectionPicker/ProjectionPickerViewModelSpec.js](../../cesium/packages/widgets/Specs/ProjectionPicker/ProjectionPickerViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/SceneModePicker/SceneModePickerSpec.js](../../cesium/packages/widgets/Specs/SceneModePicker/SceneModePickerSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/SceneModePicker/SceneModePickerViewModelSpec.js](../../cesium/packages/widgets/Specs/SceneModePicker/SceneModePickerViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/SelectionIndicator/SelectionIndicatorSpec.js](../../cesium/packages/widgets/Specs/SelectionIndicator/SelectionIndicatorSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/SelectionIndicator/SelectionIndicatorViewModelSpec.js](../../cesium/packages/widgets/Specs/SelectionIndicator/SelectionIndicatorViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/SvgPathBindingHandlerSpec.js](../../cesium/packages/widgets/Specs/SvgPathBindingHandlerSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/ThirdParty/knockoutSpec.js](../../cesium/packages/widgets/Specs/ThirdParty/knockoutSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/Timeline/TimelineSpec.js](../../cesium/packages/widgets/Specs/Timeline/TimelineSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/VRButton/VRButtonSpec.js](../../cesium/packages/widgets/Specs/VRButton/VRButtonSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/VRButton/VRButtonViewModelSpec.js](../../cesium/packages/widgets/Specs/VRButton/VRButtonViewModelSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/Viewer/ViewerSpec.js](../../cesium/packages/widgets/Specs/Viewer/ViewerSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/Viewer/viewerDragDropMixinSpec.js](../../cesium/packages/widgets/Specs/Viewer/viewerDragDropMixinSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/Viewer/viewerPerformanceWatchdogMixinSpec.js](../../cesium/packages/widgets/Specs/Viewer/viewerPerformanceWatchdogMixinSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/VoxelInspector/VoxelInspectorSpec.js](../../cesium/packages/widgets/Specs/VoxelInspector/VoxelInspectorSpec.js) | 待逐项核对 |
| [packages/widgets/Specs/createCommandSpec.js](../../cesium/packages/widgets/Specs/createCommandSpec.js) | 待逐项核对 |
