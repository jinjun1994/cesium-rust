# 全球地形 · 0.0.8

日期：2026-10-13。实现与真实验证范围如下。

| 项目 | 结果 / 范围 |
| --- | --- |
| 公开全球高程 | Mapzen / Tilezen Terrarium，Web Mercator，默认启用、无需 token；超出 ±85.0511288° 为椭球回退 |
| Cesium World Terrain | ion asset 1 endpoint、Bearer 授权、geographic TMS quantized-mesh-1.0；需您自己的有权限 token，未实测实时 ion 服务 |
| 自定义来源 | quantized-mesh layer.json；Terrarium XYZ 模板，最高 15 级 |
| 解码 | bounded gzip、zig-zag 坐标、16/32 位 HWM 索引、边界索引/裙边、metadata 可用范围；法线由网格计算 |
| 地形显示 | per-tile RTC；父级回退；可见子瓦片整组就绪才替换；带 pin 的缓存；混合 LOD 裙边 |
| 影像贴地 | 取当前最细 resident geographic / Mercator 影像并重投影为 256×256 地形纹理；NASA 回退；每帧最多刷新一个贴图 |
| 相机 | 全球/近景缩放、已加载地形的 10 m 避地、跟随缓存高程的倾斜观察 |
| 高程查询 | 最高已缓存级别的三角形重心插值；未知位置返回 null；返回原始数据高度，未乘显示倍率 |
| Rust 验证 | 37 个普通测试通过；5 个 GPU 测试另行执行并通过；fmt、clippy -D warnings 通过 |
| 官方资产 | CesiumGS quantized-mesh、32 位索引、metadata 样本均通过解码；实际 metadata 可用范围更新通过 |
| 真实地形 | 公开服务珠峰周边 92 瓦片、LOD 9、0 错误；真实 HTTP 下载/解码和 native Vulkan 渲染/readback 完成 |
| 图像回读 | 地形与椭球对比 389,053 个不同像素；terrain-preview.png 与 terrain-ellipsoid-preview.png 为真实 GPU 输出 |
| 桌面 | Xvfb / Vulkan，120 帧本地 quantized-mesh smoke；2 个 resident 地形、0 错误 |
| Web | WASM 与绑定生成通过；npm test 的控制器、真实 DOM UI 和 SDK 生命周期测试通过 |
| 浏览器 GPU | 本轮未完成完整浏览器 GPU 验收；先前环境存在 Chromium 启动 / DeviceLost 限制，不能将 DOM 或原生 GPU 测试算作浏览器渲染通过 |

在线验证先使用真实 HTTP 获取了 92 瓦片，再用这批缓存重新渲染最终版本；缓存不随源码包分发。空间范围只覆盖珠峰周边及加载路径的低级父瓦片，不等于全世界所有等级已经视觉验收。独立 z=12 珠峰真实 PNG 夹具确认存在 >8,000 m 的高程；实时缓存查询可能停在较粗级别，数值不是测量级峰顶高度。

## 启动与操作

```bash
python3 scripts/start-test-ui.py
# 浏览 http://127.0.0.1:8000/
```

源码包自带 WASM。顶部“地形”链接定位右侧控制区域。启动默认启用公开地形；点击“定位（珠峰）”，等待逐级数据到达，再用“倾斜观察”查看起伏。输入自己的 ion token 可启用 Cesium World Terrain；也可加载 layer.json 或 {z}/{x}/{y} 的 Terrarium 模板。高度倍率 1×/2×/5×、细化误差 4/8/16/32 px 可调。离线合成地形明确标记为测试数据。

```js
const viewer = await attachCanvas(canvas); // 默认公开地形，可传 {terrain:false}
await viewer.loadWorldTerrain(ionToken);
await viewer.loadTerrain('https://server.example/terrain/layer.json');
viewer.setTerrariumTerrain('https://server.example/{z}/{x}/{y}.png', 15);
viewer.setTerrainOptions(2, 8);
viewer.focusLocation(86.92528, 27.9881, 20000);
viewer.lookAtTerrain(86.92528, 27.9881, 20000);
const height = viewer.sampleHeight(86.92528, 27.9881); // 已缓存数据或 null
viewer.clearTerrain();
```

自定义来源必须具有全球根瓦片；区域无数据处回退到椭球/父级，不会虚构未加载高程。GLB 独立模型场景没有地球，启用地形前先“显示地球”。3D Tiles 地球场景可以同时显示地形。

桌面默认公开地形；CESIUM_ION_TOKEN 同时用于航拍和地形。CESIUM_TERRAIN_URL 指向本地/HTTP layer.json 或 Terrarium 模板；CESIUM_TERRAIN=0 禁用。CESIUM_OFFLINE=1 禁用默认网络来源，但仍允许显式配置本地地形。CESIUM_VIEW_LOCATION=lon,lat,height 设置初始视角。

## 预算与异常处理

每类地形最多 4 个下载/解码任务；输入及 gzip 输出各 8 MiB；源顶点最多 262,144、索引最多 1,500,000；高度查询加速结构最多 1,000,000 三角形引用；metadata 1 MiB；地形 resident 几何/查询数据和 GPU 网格约 128 MiB 预算。地形显示倍率范围 0..10，SSE 2..64。缓存满时保留父级，最多 3 次请求尝试；这些预算不代表整个进程/浏览器/驱动内存上限。

切换来源、关闭、场景替换和 destroy 都隔离旧结果。元数据到达时再次切换 generation，取消在等待元数据期间产生的旧源请求。Bearer 只允许发送到 provider 的 origin / 目录，不跟随重定向；错误和导出不保存 token。

## 尚未实现的范围

parentUrl 层链、heightmap-1.0/LERC/GeoTIFF、流式最精细高程查询、水面材质、动态高程编辑/裁剪、地形 Worker 解码、设备丢失恢复不在本版。oct-normal / watermask 扩展检查长度；法线重算，watermask 不绘制水面。GeoJSON/3D Tiles 保留原始高度，不自动贴地。Terrarium 高程保留其原始高度基准，不做 geoid / ellipsoid datum 转换；这会影响精确高程对齐。

## 复测与来源

```bash
cargo test --locked --workspace
cargo test --locked -p cesium-viewer terrain_replacement -- --ignored
npm ci && npm test
cargo run --locked -p cesium-viewer --bin terrain-render-check -- --stages=24
# 已缓存后可加 --cached-only
```

格式：https://github.com/CesiumGS/quantized-mesh
公开数据：https://registry.opendata.aws/terrain-tiles/
Terrarium：https://github.com/tilezen/joerd/blob/master/docs/formats.md
归属和许可：terrain-attribution.md、cesium-terrain-fixtures-LICENSE.md；原始 URL 与 SHA-256：tests/fixtures/terrain/provenance.json。
