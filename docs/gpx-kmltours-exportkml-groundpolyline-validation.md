# Batch 46 (v0.0.46) 案例验证报告：GPX、KML Tours、Export KML 与 Ground Polyline Material - Dev

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖全球定位系统通用数据流 (GPX)、三维空间漫游导览时序巡检 (KML Tours)、全场景资产打包导出 (Export KML/KMZ)、以及地表复杂地形贴地折线多种高级程序化材质 (Ground Polyline Material) 的配方，累计完成 **131 / 309**（剩余 178 个）：

1. **`gpx`（全球定位系统航迹、路线、地标与多样式符号系统）**
   - 官方地址：`https://sandcastle.cesium.com/?id=gpx`
   - 变体全集（4 个导航规划变体）：
     - `Track with Waypoints`（全航迹与关键途经点：博尔德登山主步道绿色宽折线航迹、起点 Trailhead、中途 Summit 冲顶峰与终点 Overlook 图钉广告牌）
     - `Route Navigation`（道路导航规划路线：主干攀登线与下山山脊虚线折线协同呈现，保留起止地标）
     - `Waypoints Only`（地标兴趣点集群：全量显示步道各级补给点、露营地与观景台图钉标记）
     - `Multiple Tracks & Symbology`（多轨迹协同与定制符号系统：金色加粗主轨迹与下行山脊线，搭配自行车与户外图例）
   - 核心要素：
     - 支持 GPX 标准中 `<trk>`、`<rte>` 与 `<wpt>` 要素在 Rust 图元管线中的语义拆解与几何映射。
     - 验证了宽折线地表吸附（`clampToGround: true`）与广告牌图钉的复合时空空间定位。

2. **`kml-tours`（KML/KMZ 自动化三维相机导览巡检与巡游时序控制）**
   - 官方地址：`https://sandcastle.cesium.com/?id=kml-tours`
   - 变体全集（4 个导览视角变体）：
     - `Initial Tour Overview`（导览总览全景：巴黎埃菲尔铁塔地标、发光导览空中巡航三维折线、特罗卡德罗与塔顶观景台停靠点）
     - `Tour Stop 1 (Trocadero Viewpoint)`（停靠点 1 聚焦：塞纳河畔特罗卡德罗观景露台低空仰视铁塔）
     - `Tour Stop 2 (Summit Orbit)`（停靠点 2 环绕：铁塔顶端 350m 高空环形轨道俯瞰巴黎全景）
     - `Complete Tour Experience`（全流程三维动态巡游：空中发光巡航轨迹加粗扩展，多视点高动态串接）
   - 核心要素：
     - 严格遵循 KML 扩展规范中 `<gx:Tour>`、`<gx:FlyTo>` 与 `<gx:Wait>` 空间动画序列契约。
     - 采用 `PolylineGlow` 发光材质呈现空中巡航立体三维轨迹，有效规避视距拉远时的几何穿模。

3. **`export-kml`（天地一体化多载体场景要素 KML/KMZ 打包与模型导出）**
   - 官方地址：`https://sandcastle.cesium.com/?id=export-kml`
   - 变体全集（4 个导出要素组合变体）：
     - `Satellites and Ground Assets`（空地一体全量导出：GeoEye-1 低轨卫星、500km 高空轨道测地折线、AGI 地面遥测站图钉与 Cesium Milk Truck 3D 车辆模型）
     - `Model Asset (Cesium Milk Truck)`（高精度地面车辆模型资产导出：聚焦卡车模型与地面站）
     - `Space Assets (Satellites & Orbits)`（天基轨道资产导出：卫星实体与空间发光轨道）
     - `Full Combined KMZ Package`（完整打包要素集：轨道加粗、天地关联线与图钉多要素一体化呈现）
   - 核心要素：
     - 支持 CZML 与 glTF 3D 模型向 KML/KMZ 结构化元数据与外部二进制资源包的无损序列化解算。
     - 保证模型图元（`CesiumMilkTruck.glb`）在离屏 GPU 与端到端浏览器中的正确光栅化与稳定像素拾取。

4. **`ground-polyline-material-dev`（圣海伦斯火山复杂地形地表贴地折线材质库）**
   - 官方地址：`https://sandcastle.cesium.com/?id=ground-polyline-material-dev`
   - 变体全集（4 个材质特性变体）：
     - `All Three Ground Polyline Materials`（三材质同屏对比：青天蓝发光材质 Glow、亮黄虚线材质 Dash、与赤橙色双色描边材质 Outline）
     - `Polyline Glow Focus`（发光材质专项：高能量发光脉冲折线，线宽扩展至 14px）
     - `Polyline Dash Focus`（虚线材质专项：高对比度黑金间隔虚线，线宽扩展至 14px）
     - `Polyline Outline Focus`（描边材质专项：高对比度纯白外轮廓与赤红中心芯线）
   - 核心要素：
     - 充分释放底层 `wide_polyline` 中 `LineMaterial` 枚举的多材质能力（`Solid`、`Glow`、`Dash`、`Outline`）。
     - 地表折线贴地投影（`clampToGround: true`）在 10px ~ 14px 像素宽度下呈现出数百至数千像素的平滑抗锯齿表现。

---

## 2. 核心研发突破

1. **`Polyline` 与 `LineMaterial` 全功能材质体系落地验证**：
   - 验证了 `LineMaterial::Glow`（`glowPower`、`taperPower`）、`LineMaterial::Dash`（`dashLength`、`dashPattern`、`gapColor`）与 `LineMaterial::Outline`（`outlineColor`、`outlineWidth`）在复杂地表贴地模式下的像素着色，光栅化后在 GPU 离屏与浏览器 WebGPU 中分别稳定输出 2,000 ~ 4,000 像素的高质量几何渲染。
2. **`EntityVariants` 反序列化契约健壮性强化**：
   - 统一确立并严格规范了多变体 JSON 配置必须包含 `initial: 0` 字段的硬性约束，避免变体切换索引溢出或解析阶段反序列化失败。
3. **复合动静态多载体（卫星 + 轨道 + 广告牌 + 3D 模型）综合场景验证**：
   - `export-kml` 成功聚合了 500km 轨道空间资产与地面 Milk Truck 3D 车辆模型，验证了在极端跨尺度坐标系下，离屏取景与像素拾取的完全一致性。

---

## 3. 验收结果矩阵

| 案例 ID | 实体数 | 变体数 | Native GPU 渲染 | 浏览器 WebGPU | 像素拾取 | 交互控制 | 状态 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `gpx` | 5 | 4 | **Passed** (90 帧无错误) | **Passed** (5 实体/航迹) | **Passed** (4346 px) | 轨迹/导航/地标切换 | **Passed** |
| `kml-tours` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体) | **Passed** (4115 px) | 导览停靠点与全流程切换 | **Passed** |
| `export-kml` | 4 | 4 | **Passed** (90 帧无错误) | **Passed** (4 实体/模型) | **Passed** (2662 px) | 卫星/卡车/全要素导出切换 | **Passed** |
| `ground-polyline-material-dev` | 3 | 4 | **Passed** (90 帧无错误) | **Passed** (3 地表折线) | **Passed** (10240 px) | 发光/虚线/描边材质切换 | **Passed** |

---

## 4. 验证日志索引

- Native GPU 单体验收日志：
  - `docs/full-gpx-smoke.log`
  - `docs/full-gpx-variant-1-smoke.log` ~ `docs/full-gpx-variant-3-smoke.log`
  - `docs/full-kml-tours-smoke.log`
  - `docs/full-kml-tours-variant-1-smoke.log` ~ `docs/full-kml-tours-variant-3-smoke.log`
  - `docs/full-export-kml-smoke.log`
  - `docs/full-export-kml-variant-1-smoke.log` ~ `docs/full-export-kml-variant-3-smoke.log`
  - `docs/full-ground-polyline-material-dev-smoke.log`
  - `docs/full-ground-polyline-material-dev-variant-1-smoke.log` ~ `docs/full-ground-polyline-material-dev-variant-3-smoke.log`
- Native GPU 整体回归日志：`docs/full-recipes-gpu-validation.log` (132 个测试全部 Passed)
- 浏览器 WebGPU 综合验收日志：`docs/full-recipes-browser.json` (131 个 Sandcastle 案例全部 Passed)
