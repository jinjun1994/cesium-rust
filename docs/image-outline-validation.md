# 图片材质与几何轮廓验证（0.0.12）

本批实现 Rust/wgpu 原生功能，并在 Sandcastle 测试 UI 增加“图片与轮廓”和对应 CZML 配方。完整上游案例通过数仍为 0；309 个案例目录、11 个案例的本地子集关联不代表全功能兼容。

## 用法

```bash
python3 scripts/start-test-ui.py
# http://127.0.0.1:8000/bindings/web/sandcastle.html
CESIUM_OFFLINE=1 CESIUM_ENTITIES=tests/entities/image-outline-external.json CESIUM_VIEW_LOCATION=109.8,25.1,230000 cargo run --locked -p cesium-viewer --bin desktop-viewer
CESIUM_OFFLINE=1 CESIUM_CZML=tests/entities/image-outline.czml CESIUM_VIEW_LOCATION=109.8,25.1,230000 cargo run --locked -p cesium-viewer --bin desktop-viewer
```

```js
await viewer.loadEntities('/tests/entities/image-outline-external.json');
await viewer.loadCzmlData('/tests/entities/image-outline.czml');
// 编辑器对象可指定资源基准；upsert 要提供完整实体描述。
await viewer.loadEntities(entities, {baseUrl: location.href, upsert: true});
viewer.cancelEntityLoads();
```

同步 `setEntities`、`upsertEntity`、`loadCzmlBytes` 仅接受内联图片 data URI。异步 API 先解析图片，再事务式提交：失败保留旧集合；新同步提交、清空或销毁会取消旧导入。UI 切换配方会主动取消旧导入。桌面以 JSON/CZML 文件为基准解析相对图片路径。

Entity 图片材质使用 `{type:"image", image:"image-test.png", color:[1,1,1,1], repeat:[1,1], transparent:true}`；CZML 支持 `material.image.image` 字符串或 `{uri:...}`、`repeat.cartesian2`、`color.rgba` 和静态 `transparent`。`transparent=false` 默认忽略图片自身 alpha，颜色 alpha 仍有效；开启时透明像素穿透拾取。

## 范围和限制

| 功能 | 当前支持 |
| --- | --- |
| 轮廓 | Box、Ellipsoid、Cylinder/Cone、Plane、Polygon（含洞/挤出）、Rectangle、Ellipse、Wall；`outline`、`outlineColor`、`outlineWidth:1` |
| 填充 | 静态 `fill=false` 隐藏实体面，轮廓可拾取并返回父实体 ID/属性；显隐、变换和删除作用于面及轮廓 |
| 图片 | 静态 PNG/JPEG、颜色调制、UV 重复、透明；共享颜色/拾取采样 |
| Web 资源 | 相对或绝对 HTTP/HTTPS；每次导入去重，所有导入共享最多四个图片传输槽；支持 AbortController 和过期结果保护 |
| 桌面资源 | 相对文件或 HTTP/HTTPS；同一导入去重，使用已有有界读取路径 |

图片最长边 2048，编码文件最多 4 MiB，单个 data URI 最多 6 MiB；每次导入最多 16 个独立外部图片、256 个引用，解析后的输入最多 8 MiB。实体仍限制为 256 个和 32 MiB 几何/GPU 预算。外部资源只扫描声明的图片材质，不读取任意 properties 中的 URL。拒绝带账号密码的 URL、非允许协议和非 PNG/JPEG 内容。

轮廓使用 LineList 和一像素光栅化，宽度不等于 1 会明确报错；没有 Polyline 轮廓。椭球和圆柱使用固定采样线框，尚不等同 Cesium 的全部 partition 参数。Reverse-Z 顶点深度使用小幅相对偏移，使共面轮廓可见；地理边界沿用细分曲线并轻微抬高，极近共面场景仍有局限。实体 `fill=false` 当前仍验证并分配面网格和图片资源，不能用于绕开资源限制。

图片使用已有投影 ST，并非 Cesium 每种几何的精确纹理坐标。未支持视频/Canvas、动态材质/轮廓/填充、Fabric、贴地、完整墙体/挤出参数、纹理跨 mesh GPU 去重、透明物体逐三角形排序或 OIT。

## 验证

- 54 个普通 Rust 测试、11 个显式实际 GPU 测试通过；fmt 与 clippy 检查通过。
- GPU 检查一像素轮廓拾取与实体属性、前方不透明面遮挡、图片透明孔洞穿透、重复纹理、`fill=false` 内部不拾取、显隐/替换/删除后的结果失效。实际导出见 `image-outline-preview.png`；导出时第一个矩形已切换为 `transparent=false`，中心呈白色，第二个矩形保留透明孔洞。
- PNG/JPEG 解码、非法 MIME/内容、过大尺寸、相对路径和 URL 去重通过；六组 Node 测试检查资源上限、共享并发槽、取消、清空和失败时保留旧集合。
- 原生和 WebAssembly 编译、绑定生成通过；桌面相对图片 JSON 和 CZML 启动各运行 90 帧，Entities 分别为 5 / 2，errors 均为 0；日志为 `image-outline-cesium_entities-smoke.log` 和 `image-outline-cesium_czml-smoke.log`。
- 浏览器端到端脚本增加两个配方断言，但浏览器 WebGPU 验收仍未通过：缺少 Chromium，此前安装包无效，并有尚未解决的 DeviceLost(Destroyed) 记录。Node DOM、WASM 构建和原生 GPU 测试均不能替代这项验收。
