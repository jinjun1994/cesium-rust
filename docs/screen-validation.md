后续 0.0.14 增加 Label、距离属性与锚点，见 [Label 与距离验证](label-distance-validation.md)。下文保留 0.0.13 记录。

# Point / Billboard 屏幕实体验证（0.0.13）

新增 Rust/wgpu 屏幕实体，沿用 Entity ID、属性、show/availability、删除/upsert/focus 和时钟。Sandcastle UI 增加“Point / Billboard”“CZML Point / Billboard”“动态 Point / Billboard”配方。上游目录仍为固定版本 1.146 的 309 个案例；16 个案例关联本地功能子集，完整上游配方验收通过数仍为 0。

## 启动

```bash
python3 scripts/start-test-ui.py
# http://127.0.0.1:8000/bindings/web/sandcastle.html
CESIUM_OFFLINE=1 CESIUM_ENTITIES=tests/entities/screen-external.json CESIUM_VIEW_LOCATION=109.9,25.1,100000 cargo run --locked -p cesium-viewer --bin desktop-viewer
CESIUM_OFFLINE=1 CESIUM_CZML=tests/entities/screen-motion.czml CESIUM_VIEW_LOCATION=109.9,25.1,100000 cargo run --locked -p cesium-viewer --bin desktop-viewer
```

```js
await viewer.loadEntities('/tests/entities/screen-external.json');
await viewer.loadCzmlData('/tests/entities/screen-motion.czml');
viewer.setClock({shouldAnimate:true});
```

```json
[
  {"id":"point","positionDegrees":[110,25,1000],"color":[1,0,0,1],
   "shape":{"type":"point","pixelSize":32,"outlineWidth":4,"outlineColor":[0,1,1,1]}},
  {"id":"image","positionDegrees":[110.1,25,1000],"color":[1,1,1,1],
   "shape":{"type":"billboard","image":"sprite.png","width":64,"height":48,"scale":1,"pixelOffset":[40,20]}}
]
```

外部图片需要异步 API；同步 API 使用 PNG/JPEG base64 data URI。沿用每导入最多 16 个独立图片、四个共享 Web 传输槽、相对路径、去重和旧请求取消；未知 URI 对象参数直接拒绝。CZML 图片为字符串或 `{uri:"..."}`。

## 当前语义

| 属性 | 行为与限制 |
| --- | --- |
| point pixelSize | 填充直径，默认 1；大于 0，最多 2048 像素 |
| point outlineWidth / outlineColor | 点自己的像素轮廓，不是 Entity 几何 LineList 轮廓；默认 0 / 黑色，宽度 0..128，外径为 pixelSize + 2 × outlineWidth，最多 2048 |
| billboard width / height | 默认各 64，显式大小覆盖图片大小；当前不从图片原尺寸推导默认宽高 |
| billboard scale | 大于 0，最多 64；缩放后各边最多 2048 像素 |
| billboard pixelOffset | 默认 [0,0]，正 x 向右、正 y 向下，各分量绝对值最多 4096 |
| color | 自定义 JSON Entity 颜色，CZML 支持静态 RGBA；CZML 默认白色；图片 alpha 与颜色 alpha 相乘 |
| 位置 | 静态 WGS84 或 FIXED ECEF 线性位置采样；复用已有 UTC 时钟及 availability |

局部 XY 顶点是 framebuffer 像素。顶点 shader 从世界锚点计算深度，在裁剪坐标按 viewport 调整 XY；相机缩放/旋转不会把像素尺寸转换为米。实体与地球/不透明几何共享深度测试，背面锚点被遮挡。透明排序使用锚点深度；没有 OIT。图形以中心为原点、固定面向相机；图片纹理左上角对应 UV (0,0)。圆形和图像 alpha 在颜色/拾取通道共享，圆外、完全透明图片区域不遮挡拾取。

这里的像素是实际 framebuffer 像素，尚未加入 CSS/devicePixelRatio 尺寸补偿。圆点采用硬边圆形裁切，暂未实现抗锯齿软边。没有 label、水平/垂直原点、旋转、alignedAxis、eyeOffset、sizeInMeters、距离缩放/透明属性、distanceDisplayCondition、深度测试距离、贴地、纹理子区域或图像动态属性。JSON 屏幕图形要求 fill=true，禁止外层 geometry outline 或 surface material；point 轮廓在 shape 内定义。CZML 只接受声明的静态参数，未知参数报错。

## 验证证据

- 56 个普通 Rust 测试通过；12 个实际 GPU 测试通过，包括已有地球、影像、地形、地理实体、材质和轮廓回归。fmt、clippy 与原生/WASM 构建通过，WASM 绑定已重新生成。
- 新 GPU 测试在 40,000 / 400,000 米观察距离逐像素统计：红色点填充直径均为 32。检查点四角穿透、像素轮廓返回父 ID、地球遮挡背面点、billboard 透明孔洞、64 像素边界、pixelOffset 和生命周期。
- 动态 CZML point/billboard 的时间变化改变实际帧缓冲；移动后原位置不再拾取，focus 到当前位置可重新拾取。GPU 导出为 `screen-preview.png`。
- 六组 Node 测试通过，UI 验证三组新配方；资源测试覆盖 billboard JSON/CZML URI、相对 URL、重复资源以及严格 URI 参数检查。DOM 测试替代 GPU 边界，不是浏览器视觉验收。
- 桌面 JSON、静态 CZML、动态 CZML 各运行 90 帧，Entities 3 / 3 / 2，errors 均为 0，记录于 `screen-*-smoke.log`。
- 浏览器端到端脚本加入新配方和 point 拾取断言，当前启动仍失败：缺少 Chromium。此前 DeviceLost(Destroyed) 尚未解决，浏览器 WebGPU 不能标记为通过。

后续优先 label 的字体布局与渲染、距离属性，然后逐步扩大相机、DataSource、3D Tiles 和其他案例覆盖；本批不等同全部 Sandcastle 功能完成。
