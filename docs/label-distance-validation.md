后续 0.0.15 增加文字背景/描边、旋转、距离像素偏移和深度策略，见 [屏幕效果验证](screen-effects-validation.md)。下文保留 0.0.14 记录。

# Label 与距离属性：五项功能验证（0.0.14）

本批实际实现五项功能，并为每项提供独立 UI 配方和独立 GPU 测试。全部运行在 Rust/wgpu Scene；没有用 CesiumJS 替代引擎。

| 功能 | 当前可运行行为 | GPU 证据 |
| --- | --- | --- |
| Label 文字实体 | 中文/英文静态文字、多行、字号、颜色、scale/pixelOffset；继承显隐、采样位置、拾取、替换与删除 | 中文字形的实际像素可拾取，空白透明区域穿透，非法文本替换不破坏旧实体 |
| distanceDisplayCondition | Point/Billboard/Label 按相机到锚点距离显示；包含近/远边界 | 40/50 km 显示，15/160 km 隐藏；颜色与拾取一致 |
| scaleByDistance | 静态 NearFarScalar 控制屏幕几何缩放，范围外钳制；独立保留 pixelOffset | 点填充从 64 px 变为 16 px；Billboard 放大后仍保持 40/20 px 偏移 |
| translucencyByDistance | 静态 NearFarScalar 控制 alpha，乘上原颜色/图片 alpha | 实际帧缓冲颜色介于背景与不透明结果之间；零 alpha 不拾取 |
| 锚点对齐 | Billboard/Label 的 LEFT/CENTER/RIGHT、TOP/CENTER/BOTTOM；Label 的 BASELINE | 图片落在正确象限，gyp 的升部/降部分布在基线上下 |

## 启动测试 UI

```bash
python3 scripts/start-test-ui.py
# http://127.0.0.1:8000/bindings/web/sandcastle.html
```

选择五个新增按钮之一，再选实体，点击“近 40km / 中 100km / 远 220km”直接比较。距离显示配方的有效范围是 30..80 km，所以近预设显示、中/远预设隐藏；配方初始视图也约为 40 km。距离缩放、透明度配方在 40 km 和 160 km 两端钳制。按钮操作的是相机到实体的观察距离，不是地表高度。

“CZML 五项功能”配方将五种行为放在一个场景。新文件为 `tests/entities/labels.json`、`distance-display.json`、`distance-scale.json`、`distance-alpha.json`、`screen-origins.json`、`label-distance.czml`。

```js
await viewer.loadEntities('/tests/entities/labels.json');
await viewer.loadCzmlData('/tests/entities/label-distance.czml');
viewer.focusEntity('distance-scale', 20000);
viewer.focusEntity('distance-scale', 220000);
```

```json
[
  {
    "id":"label", "positionDegrees":[110,25,16000], "color":[1,1,1,1],
    "shape":{"type":"label","text":"全球地球 Globe\n中文标签","fontSize":28,"scale":1,"pixelOffset":[0,10]},
    "screen":{
      "horizontalOrigin":"LEFT","verticalOrigin":"BASELINE",
      "distanceDisplayCondition":[0,300000],
      "scaleByDistance":[40000,2,160000,0.5],
      "translucencyByDistance":[40000,1,160000,0]
    }
  }
]
```

JSON 的公共屏幕属性位于 Entity 的 `screen` 中；CZML 位于对应 point/billboard/label 图形内。CZML Label 接受 `text` 字符串或 `{string:...}`、有限 CSS `font`、`fillColor.rgba`、`scale`、`pixelOffset.cartesian2`、`style:"FILL"`。距离属性支持常量数组或标准 `distanceDisplayCondition` / `nearFarScalar` 包装；锚点接受字符串或对应枚举键包装。所有未知或动态属性会明确报错。

## 渲染与范围

相机到世界锚点的距离以 f64 每帧计算。NearFarScalar 顺序为 `[near, nearValue, far, farValue]`；使用固定 Cesium 1.146 源码的平方距离归一化和 `pow(t,0.2)` 公式，在端点外钳制。相机移动和采样位置都影响该距离，无需重新上传文字或图片。scaleByDistance 作用于相对锚点的几何，pixelOffset 不随其变化。显隐/透明度同时用于颜色与拾取；透明排序、深度遮挡沿用前批范围。

Label 使用 fontdue 和内置 OFL 衍生字体 `GlobeSans-Regular.ttf`，原生/WASM 共用字形排版与灰度 alpha 纹理，透明背景 RGB 保持白色以避免过滤产生黑边。字体源、源/子集 SHA-256 和许可证在 `assets/fonts`，生成脚本为 `scripts/subset-label-font.py`。覆盖 GB2312 常用简体中文及选定拉丁/希腊/西里尔字符；不在子集的字形明确拒绝。没有系统字体回退或任意字体下载。

Label 最多 256 个字符、8 行和 4096 UTF-8 字节；字号为 8..96 px，默认 30。栅格纹理最多 2048×512，静态 scale 最大 8，缩放后本体各边仍限制 2048。每标签独立纹理，现有 256 实体、32 MiB 集合几何/GPU 预算继续有效。全局字体解析内存单独存在，不包含在该实体预算内；尚未提供 GPU 字形 atlas 或跨标签纹理共享。

距离限为 0..1e9 米，近/远间隔至少 1 mm；距离 scale 因子为 0..8，透明度因子为 0..1。Point 不支持锚点参数，其他地理/立体几何尚不支持这些距离属性。BASELINE 只适用于 Label，表示第一行基线。当前尺寸为 framebuffer 像素，尚无 CSS/devicePixelRatio 补偿。

本批尚未支持字体粗体/斜体/完整 CSS、字距或复杂文字 shaping/双向排版、组合附加符、emoji、完整字符集、文字描边/背景、动态文本/样式、pixelOffsetScaleByDistance、eyeOffset、深度测试距离、旋转、贴地或 label 与其他图形组成同一个 CZML 实体。未知属性严格拒绝；自定义 JSON API 不是完整 CesiumJS API 替换。

## 已执行验证

- **61 个普通 Rust 测试、17 个显式实际 GPU 测试通过**。五个新 GPU 测试在 `tests/screen-advanced-gpu.rs`，覆盖上表全部功能；已有 12 个 GPU 测试回归通过。
- 六组 Node 测试通过：五个 UI 按钮、组合 CZML、近中远定位参数、GPU 不可用时的禁用状态，以及既有资源/生命周期测试。
- 原生与 WASM 构建、WASM 绑定生成、fmt/clippy 检查通过。
- 六个桌面配方各运行 90 帧，Entities 3 / 1 / 1 / 1 / 3 / 5，errors 全部为 0；记录在 `docs/labels-smoke.log`、`distance-*-smoke.log`、`screen-origins-smoke.log` 和 `label-distance-smoke.log`。
- 实际 GPU 导出为 `docs/label-distance-preview.png`，包括中文 Label、距离属性的点和锚点图片。
- 浏览器端到端脚本已加入五项配方，但本轮仍因缺少 Chromium 无法启动；此前 DeviceLost(Destroyed) 尚未解决。Node DOM/WASM/原生 GPU 不能代替浏览器 WebGPU 视觉验收。

Sandcastle 目录维持固定版本的 309 个案例，其中 **19 个案例有本地功能子集关联，完整上游配方验收通过数仍为 0**。五项功能完成于以上明确范围，不等同五个完整上游案例已通过。下一批继续文本样式、更多屏幕图形参数与相机功能。

官方参考：
- [LabelGraphics](https://cesium.com/learn/cesiumjs/ref-doc/LabelGraphics.html)
- [NearFarScalar](https://cesium.com/learn/cesiumjs/ref-doc/NearFarScalar.html)
- [固定版本的 nearFarScalar shader](https://github.com/CesiumGS/cesium/blob/b8d3a36fe98a3e432eb89253c95d5f20e605e0f1/packages/engine/Source/Shaders/Builtin/Functions/nearFarScalar.glsl)
