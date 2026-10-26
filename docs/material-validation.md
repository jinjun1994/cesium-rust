后续 0.0.12 增加图片材质与轮廓，见 [图片与轮廓验证](image-outline-validation.md)。下文保留 0.0.11 的验证记录。

# 透明与常用材质（0.0.11）

本批扩展共享 Rust/wgpu 渲染器，支持 Entity/CZML 的 RGBA 透明色及条纹、棋盘格、网格材质，并加入 GLB 的 `alphaMode: BLEND` 解码与渲染。

## 运行测试

```bash
python3 scripts/start-test-ui.py
# 打开 http://127.0.0.1:8000/bindings/web/sandcastle.html
```

新增“透明 / 常用材质”“CZML 材质”“原案例透明墙体”按钮。JSON 编辑器可修改材质并通过“按 ID 添加 / 更新”重建对应实体；未改变的实体保留 GPU mesh ID。`tests/entities/sandcastle-wall.czml` 使用固定 1.146 上游 CZML Wall 的原始坐标、高度与 RGBA `[255,0,0,150]`，不是替换为不透明颜色或合成低墙。

现在 11 个案例有本地功能子集入口；完整上游案例仍为 `not-verified`，不把目录收录、功能配方或打开官方网站视为完整兼容。

## 透明管线

- OPAQUE/MASK 先绘制，写入 Reverse-Z 深度；BLEND 后绘制，执行线性颜色空间的 straight-alpha 混合并保留不透明深度测试，不写颜色通道的深度。
- 每帧按透明网格包围盒中心的相机深度从远到近排序，排序后绘制、uniform 槽和拾取 ID 范围保持一致。
- 拾取通道单独写深度，返回最近有覆盖的图形。BLEND 的有效 alpha 等于材质、纹理、顶点颜色及程序材质 alpha 的乘积；alpha 为 0 的片元被剔除，可拾取后方图形。任何大于 0 的覆盖都可拾取，不做感知透明度阈值。
- 原有 MASK 截断继续作用于颜色与拾取，OPAQUE 继续忽略纹理 alpha。

这是**网格级排序**，不是 OIT。相交透明网格、透明闭合体内部三角形、包含多个 Batch Feature 的大型透明瓦片可能有顺序差异；不保证每个透明三角形都正确排序。没有折射、透射、透明阴影或多层选择功能。

## 自定义 Entity 材质

`color` 支持有限 0..1 的 sRGB RGBA，进入渲染器前转换为线性 RGB；glTF 的 baseColorFactor 保持其规范的线性值。此前 Entity 颜色直接作为线性值的行为已修正，所以中间亮度颜色可能改变。

```json
{
  "id": "stripe",
  "shape": {
    "type": "rectangle",
    "coordinatesDegrees": [109, 25, 109.4, 25.4],
    "height": 8000
  },
  "material": {
    "type": "stripe",
    "evenColor": [0.1, 0.5, 1, 1],
    "oddColor": [1, 1, 1, 0],
    "repeat": 12,
    "orientation": "VERTICAL",
    "offset": 0
  }
}
```

| 材质 | 字段 | 范围与行为 |
| --- | --- | --- |
| stripe | evenColor、oddColor、repeat、offset、orientation | 常量双色，HORIZONTAL/VERTICAL；重复 0.01..1024，有限 offset；shader 使用 `(coord-offset)*(repeat*0.5)` 的周期 |
| checkerboard | evenColor、oddColor、repeat:[x,y] | 常量双色棋盘，各轴重复 0.01..1024 |
| grid | color、lineCount:[x,y]、lineThickness:[x,y]、cellAlpha | 各轴线数 1..1024，线宽 0..32，单元 alpha 0..1；导数估算线宽/边缘；线宽 0 禁用该轴 |
| 纯色 | color:[r,g,b,a] | 无 material 时使用 color；alpha 小于 1 自动走透明管线 |

指定程序材质时其颜色覆盖 Entity.color。程序材质使用 unlit 输出，不受当前近似 PBR 灯光影响。几何 ST 由局部坐标范围和面法线投影生成；墙体用垂直轴。该映射不等同于 Cesium 全部几何的精确 ST；球体、圆柱和复杂挤出可出现投影接缝。没有 stRotation、纹理图片、法线图、动态采样材质或 Fabric/自定义 GLSL。条纹和棋盘格还没有完整的远景抗锯齿。

GeoJSON API 的 style 仍仅支持不透明颜色；新透明与程序材质适用于 Entity/CZML 和 glTF BLEND，不能把这批范围泛化为全部图层。

## CZML 与测试数据

CZML material 新增 `stripe`、`checkerboard`、`grid`，支持常量 RGBA、数字以及 Cartesian2 包装；grid.lineOffset 和其他未实现字段明确报错。

- `tests/entities/materials.json`：程序材质、空白 alpha 单元及重叠透明矩形。
- `tests/entities/materials.czml`：对应 CZML 配方。
- `tests/entities/sandcastle-wall.czml`：原始墙体数据/颜色。
- `tests/fixtures/transparent.glb`：保留嵌入纹理的 BLEND 解码测试。
- `docs/materials-preview.png`：实际 GPU 画面读回。

```bash
CESIUM_OFFLINE=1 CESIUM_ENTITIES=tests/entities/materials.json CESIUM_VIEW_LOCATION=109.8,25,230000 cargo run --locked -p cesium-viewer --bin desktop-viewer
CESIUM_OFFLINE=1 CESIUM_CZML=tests/entities/sandcastle-wall.czml CESIUM_VIEW_LOCATION=-102.5,50,8000000 cargo run --locked -p cesium-viewer --bin desktop-viewer
```

## 验证与未完成项

48 个普通 Rust 测试、9 个实际 Mesa Vulkan GPU 测试通过；GPU 读回核对两层 alpha 混色的预期 sRGB 值、输入顺序独立性、不透明遮挡、透明零覆盖拾取穿透，以及条纹/棋盘格/网格的透明区域。实体 GPU 测试验证真实 ST 映射及原始 CZML 墙体；原有地形、影像、MASK、纹理、拾取和集合生命周期回归仍通过。

材质 Entity、原始 CZML 透明墙体和 BLEND GLB 桌面示例各运行 90 帧，错误数均为 0。

5 组 Node/DOM/SDK 测试覆盖新按钮、JSON/CZML 配方、原始墙体数据和已有操作。WASM、绑定生成及 fmt/clippy 验证；浏览器 WebGPU 端到端仍未通过，当前缺少 Chromium 可执行文件，原生/DOM 验证不能替代它。

后续继续补几何轮廓、图像材质与精确 ST，继而推进屏幕点/billboard/label、像素折线、相机飞行/追踪、CZML 增量/引用、模型动画、高级 3D Tiles、2D 与后处理等。当前尚未全部完成 Sandcastle 功能。
