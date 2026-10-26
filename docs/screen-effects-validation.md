# 五项屏幕效果验证（0.0.15）

本批实际实现五项功能，均通过 Rust/wgpu 执行，提供独立 JSON 配方、组合 CZML、测试 UI 和独立 GPU 测试。

| 功能 | 可运行范围 | 验证证据 |
| --- | --- | --- |
| Label 背景 | showBackground、独立 backgroundColor、backgroundPadding | 蓝色背景不被白色文字染色；背景角落可拾取；关闭后该位置穿透 |
| Label 描边 | FILL / OUTLINE / FILL_AND_OUTLINE、独立 outlineColor、outlineWidth | 黄色描边像素返回父实体；OUTLINE 删除原实心笔画内部的填充和拾取 |
| Billboard 旋转 | 静态 rotation，正数逆时针，围绕屏幕锚点旋转 | 96×48 图像旋转 90° 后变为竖向；30 px pixelOffset 保持原位置 |
| 距离像素偏移 | Billboard/Label 的 pixelOffsetScaleByDistance | 图像中心随距离移动，图像仍保持 48 px 尺寸；超出边界不拾取 |
| 距离深度策略 | Point/Billboard/Label 的有限 disableDepthTestDistance | 近处被实体遮挡，远处覆盖实体；颜色和拾取顺序一致，图片透明孔洞保留前景拾取 |

## 启动 UI

```bash
python3 scripts/start-test-ui.py
# http://127.0.0.1:8000/bindings/web/sandcastle.html
```

五个新按钮分别为“Label 背景”“Label 描边”“Billboard 旋转”“距离像素偏移”“深度测试距离”。“CZML 五项效果”组合展示六个实体，包含用于遮挡测试的椭球。选中实体后可以使用近 40 km、中 100 km、远 220 km 按钮；测试深度策略时应选择 `depth-distance`，而不是遮挡椭球。像素偏移配方提高了锚点高度，避免远距离向下偏移后被地球自然遮挡。

```js
await viewer.loadEntities('/tests/entities/label-background.json');
await viewer.loadEntities('/tests/entities/pixel-offset-distance.json');
await viewer.loadCzmlData('/tests/entities/screen-effects.czml');
viewer.focusEntity('depth-distance', 40000);  // 正常深度测试，点被椭球挡住
viewer.focusEntity('depth-distance', 100000); // 超过 80 km 阈值，点可覆盖椭球
```

## JSON 与 CZML

```json
[
  {
    "id":"label", "positionDegrees":[110,25,16000], "color":[1,1,1,1],
    "shape":{
      "type":"label", "text":"地球 Globe", "fontSize":32,
      "appearance":{
        "style":"FILL_AND_OUTLINE", "outlineColor":[1,0.8,0,1], "outlineWidth":3,
        "showBackground":true, "backgroundColor":[0.1,0.2,0.8,0.8], "backgroundPadding":[12,8]
      }
    },
    "screen":{"pixelOffsetScaleByDistance":[40000,1,160000,3],"disableDepthTestDistance":80000}
  }
]
```

JSON 的 Label 样式在 `shape.appearance`，旋转/距离属性在公共 `screen`。Billboard 的 `screen.rotation` 单位为弧度。CZML 对应字段在 label/billboard/point 图形对象内；`style` 支持字符串或 `{labelStyle:...}`，padding 使用 `{cartesian2:[x,y]}`，颜色使用 `{rgba:[r,g,b,a]}`，像素偏移距离使用 `{nearFarScalar:[near,nearValue,far,farValue]}`，深度距离使用常量 number 或 `{number:...}`。支持常量 showBackground boolean 包装；未知或动态参数明确报错。

文字填充使用 Entity color / CZML fillColor；背景和描边有自己的 RGBA，不受文字颜色整体相乘。层之间在线性光下合成，再编码为 straight-alpha sRGB 纹理，最后使用统一的屏幕图形渲染路径。

## 渲染与限制

- 背景 padding 为 0..64 px，栅格化时向上取整；背景矩形按原文本画布扩展。描边宽度为 0..8 px，支持小数，使用 CPU 灰度掩膜膨胀和边缘覆盖近似；不是 Cesium 的 SDF 字体实现。OUTLINE 仅绘制扩展环，FILL_AND_OUTLINE 加回填充。最大画布仍为 2048×512，受字体/文本和 32 MiB 实体集合预算限制。
- 背景与描边所需边距取并集，避免裁切笔画；Label 锚点按最终画布计算，BASELINE 同步加入画布上边距。字形内置子集、复杂文字排版和字体范围沿用 0.0.14 的限制。
- rotation 仅支持静态 Billboard，要求有限值且绝对值不超过 10000 弧度。先按 scaleByDistance 缩放图像相对锚点的位置，再旋转，最后加入独立 pixelOffset。Label 旋转、alignedAxis、eyeOffset、sizeInMeters 仍未实现。
- pixelOffsetScaleByDistance 使用固定 Cesium 1.146 的平方距离/pow(0.2) 曲线；因子范围 0..8，只改变像素偏移，不改变图像尺寸。Point 没有 pixelOffset，因此拒绝这个属性。所有像素仍为 framebuffer 像素，未做 CSS/devicePixelRatio 补偿。
- disableDepthTestDistance 默认为 0，表示始终正常测试深度。有限正值在相机到世界锚点距离严格大于阈值时关闭深度测试，允许 0..1e9 米；目前拒绝 Infinity。只适用于屏幕图形，不修改其他几何。
- 关闭深度测试的屏幕图形排在普通图形之后，颜色/ID 都使用独立 Always pipeline，且不写深度；组内沿用相机深度排序。仍保留视锥/近裁剪与距离显隐，不包含 OIT、物理遮挡或深度拾取距离返回。
- 屏幕图形颜色与 ID 共用 `1/255` alpha 覆盖阈值，消除不可见 mip 残值造成的描边孔洞拾取。该阈值也作用于 Point/Billboard/Label 的极低透明度；非屏幕 glTF/程序材质保留原规则。

## 已执行验证

- 全工作区 **66 个普通 Rust 测试、22 个实际 GPU 测试通过**。新增五个 GPU 测试在 `tests/screen-effects-gpu.rs`，其余 17 个回归测试通过。
- 六组 Node 测试通过；DOM 用例验证五个按钮、组合 CZML 与 GPU 不可用时禁用状态，资源取消/生命周期用例继续保留。
- fmt/clippy 检查通过；原生/WASM 编译与绑定生成通过。
- 六个桌面配方各运行 90 帧，Entities 1 / 1 / 1 / 1 / 2 / 6，errors 均为 0；对应 `docs/*-smoke.log`。
- GPU 实际导出为 `docs/screen-effects-preview.png`。组合图中展示文字背景、描边、旋转图像、距离像素偏移，以及覆盖遮挡椭球的红点。
- 浏览器端到端脚本新增五项配方断言，但本轮仍因缺少 Chromium 无法启动。此前 DeviceLost(Destroyed) 尚未解决；WASM/Node/原生 GPU 不替代浏览器 WebGPU 验收。

目录仍固定为 309 个 Sandcastle 案例，19 个案例关联本地功能子集，完整上游配方验收通过数仍为 0。本批完成上述五项有明确边界的功能，不等同五个完整上游配方已通过。

官方参考：[LabelGraphics](https://cesium.com/learn/cesiumjs/ref-doc/LabelGraphics.html)、[BillboardGraphics](https://cesium.com/learn/cesiumjs/ref-doc/BillboardGraphics.html)。
