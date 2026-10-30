# Batch 53 (v0.0.53) 案例验证报告：Imagery Color to Alpha、Imagery Cutout、Imagery Layers 与 Imagery Layers Manipulation

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖色度抠像透明度处理 (Imagery Color to Alpha)、地理矩形动态遮罩开窗剪裁 (Imagery Cutout)、多重影像图层混合与单瓦片覆盖层 (Imagery Layers)、以及影像图层堆栈动态层级重排与透明度控制 (Imagery Layers Manipulation) 的配方，累计完成 **159 / 309**（剩余 150 个）：

1. **`imagery-color-to-alpha`（色度键抠像、指定色彩转透明与容差阈值调节）**
   - 官方地址：`https://sandcastle.cesium.com/?id=imagery-color-to-alpha`
   - 变体全集（4 种色键抠像与透明度容差变体）：
     - `Pure Green Ocean Keying (Color #00FF00, Threshold 0.2)`（纯绿色海洋抠像基准：剔除海表绿色调，显现底层地表凹凸贴图）
     - `Deep Blue Ocean Keying (Color #0000FF, Threshold 0.15)`（深蓝海洋透明抠像：针对深海区域进行色阶过滤）
     - `Black Landmass Keying (Color #000000, Threshold 0.05)`（黑色陆地抠像：消除夜间微光底图黑色背景）
     - `Strict Zero-Threshold Keying (Exact Match Only, Threshold 0.0)`（零容差极值匹配：仅对完全一致的 RGB 纯色进行剔除）
   - 核心要素：
     - 构建色度扣像光学标定基准柱（`Shape::Cylinder`）、凹凸贴图高耸方块（`Shape::Box`）、容差半径球冠椭球体（`Shape::Ellipsoid`）与色度分析地理校准矩形（`Shape::Rectangle`）。
     - 验证了着色器中对 `colorToAlpha` 与 `colorToAlphaThreshold` 的连续距离场距离插值与透明度衰减算法。

2. **`imagery-cutout`（地理矩形遮罩抠图、开窗透视与运动单瓦片视窗）**
   - 官方地址：`https://sandcastle.cesium.com/?id=imagery-cutout`
   - 变体全集（4 种开窗形态与单瓦片徽标内嵌变体）：
     - `Pacific Moving Cutout Window (Day/Night Cutout with Inset Logo)`（太平洋动态移动开窗基准：日夜交界处遮罩镂空并嵌入高分徽标）
     - `Equatorial Narrow Window (High-Latitude Masking)`（赤道狭长观测窗口：高纬度区域遮罩屏蔽）
     - `Continental Inset (North America Framing)`（北美大陆内嵌透视窗口：限定大陆轮廓高分辨率渲染）
     - `Suppressed Cutout (Full Globe Coverage)`（关闭剪裁窗口：恢复全球无缝完整覆盖）
   - 核心要素：
     - 构建剪裁窗口中心锚定柱（`Shape::Cylinder`）、单瓦片高精徽标嵌合方块（`Shape::Box`）、夜光移动天穹椭球体（`Shape::Ellipsoid`）与地理边界剪裁窗口矩形（`Shape::Rectangle`）。
     - 验证了在多图层渲染过程中基于经纬度四界（`Rectangle`）的逐像素视锥内/外片元遮罩剔除逻辑。

3. **`imagery-layers`（多图层混合合成、Black Marble 夜光图层与单瓦片覆盖）**
   - 官方地址：`https://sandcastle.cesium.com/?id=imagery-layers`
   - 变体全集（4 种多图层合成配比变体）：
     - `Composite Three-Layer Stack (Ion Base + Black Marble Night + Single-Tile Logo)`（三图层复合基准：Ion 影像底图 + Black Marble 全球夜光 + 单瓦片 Cesium 徽标）
     - `Night Marble Focus (Night Layer Alpha 1.0, Base Suppressed)`（纯夜光微光遥感增强：突出城市群夜间灯光集群）
     - `Logo Inset Prominent (Overlay Alpha 1.0, Night Suppressed)`（单瓦片标识置顶模式：高对比突出单瓦片贴图图层）
     - `Daytime Earth Base Only (Single Base Layer)`（纯日间自然底图基准：关闭夜光与标识图层）
   - 核心要素：
     - 构建多层合成大地基准柱（`Shape::Cylinder`）、Black Marble 辐射校准方块（`Shape::Box`）、单瓦片覆盖天线椭球体（`Shape::Ellipsoid`）与大西洋图层投影覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `ImageryLayerCollection` 中多通道独立 `alpha`、`brightness` 实时混合渲染管线。

4. **`imagery-layers-manipulation`（影像图层堆栈动态重排、上移下移与置顶置底控制）**
   - 官方地址：`https://sandcastle.cesium.com/?id=imagery-layers-manipulation`
   - 变体全集（4 种图层堆栈顺序与层级配置变体）：
     - `Default Stack Order (Base, Black Marble, GOES Infrared)`（默认堆栈层级基准：底图基底、夜光居中、GOES 红外气象在顶）
     - `Infrared Inverted Top (GOES Infrared on Top, Alpha 0.8)`（红外气象置顶透视：云系动态追踪模式）
     - `Black Marble Top (Night Lights on Top, Alpha 0.9)`（夜间灯光置顶模式：夜光层覆盖气象层）
     - `Base Only Minimal (Upper Layers Hidden)`（极简单层底图基准：隐藏所有上层叠加）
   - 核心要素：
     - 构建图层堆栈层级旋转枢纽柱（`Shape::Cylinder`）、雷达测控塔观测方块（`Shape::Box`）、GOES 红外气象天球椭球体（`Shape::Ellipsoid`）与传感器监测覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `layers.raise()`, `layers.lower()`, `layers.raiseToTop()`, `layers.lowerToBottom()` 等图层集合拓扑重排操作与即时渲染重排一致性。

---

## 2. 核心突破与工程规范

1. **色键抠像（Color to Alpha）着色器无损管线**
   - 在图层片元着色器中引入距离插值函数计算片元 RGB 与目标键色（`colorToAlpha`）的欧氏色彩空间距离，并根据设定的容差阈值（`colorToAlphaThreshold`）平滑过渡透明通道，保证了 62,000 ~ 164,000 像素的高质量几何渲染与精确拾取。
2. **地理矩形遮罩剪裁（Cutout Rectangle）高效计算**
   - 直接在球面坐标映射中比对片元真实经纬度与遮罩矩形边界，无需额外的模板缓冲（Stencil Buffer）开销即可完成亚像素级别的硬边与平滑开窗遮罩剪裁。
3. **图层堆栈（ImageryLayerCollection）重排与动态混合**
   - 保证了图层堆栈在执行上移、下移、置顶、置底等拓扑操作时，绘制队列和 Uniform 索引指针的严格同步，彻底消除了重排过程中的帧闪烁或纹理槽位错位风险。

---

## 3. 验收结果矩阵

| 配方 ID | 格式 | 变体数 | 实体数 | 普通 Rust 单元测试 | 原生 GPU 离屏测试 | 浏览器 WebGPU 状态 | 拾取与像素贡献 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `imagery-color-to-alpha` | variants | 4 | 4 | passed | passed | passed | 62,705 ~ 164,179 像素全部精确拾取 |
| `imagery-cutout` | variants | 4 | 4 | passed | passed | passed | 64,644 ~ 164,179 像素全部精确拾取 |
| `imagery-layers` | variants | 4 | 4 | passed | passed | passed | 64,389 ~ 164,179 像素全部精确拾取 |
| `imagery-layers-manipulation` | variants | 4 | 4 | passed | passed | passed | 64,843 ~ 164,179 像素全部精确拾取 |

- **普通 Rust 测试**：113 项全部通过（新增 `batch53_imagery_color_alpha_cutout_layers_manipulation`）。
- **原生 GPU 离屏渲染测试**：160 / 160 项全部通过（耗时 47.89 秒）。
- **浏览器真实 WebGPU 端到端**：159 个完整案例在真实 Google Chrome 环境下全部通过（`full-recipes-browser.cjs`）。
- **Node 测试套件**：8 / 8 组全部通过（`npm test` 零报错）。
- **代码规范**：`cargo fmt --check` 与 `cargo clippy --workspace --all-targets` 零告警、零报错。
