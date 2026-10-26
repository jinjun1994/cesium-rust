# 十一个完整案例（0.0.22）

累计 **35 / 309** 个完整案例通过，剩余 274 个。100 项普通 Rust、62 项实际 GPU、8 组 Node、35 项同会话浏览器 WebGPU，以及 35 项桌面 90 帧验证通过。

本轮移植固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1` 的 11 个案例。案例原始脚本全部在构建期执行；运行时读取原始 Primitive / CZML 输入，由 Rust 与 WGSL 绘制。功能验收按案例中的全部实例、参数、材质和时间行为进行，不代表 CesiumJS API、所有 Viewer 控件或像素级画面完全一致。

| 案例 | 本轮实现与完整案例检查 |
| --- | --- |
| Ellipse Outline - Dev | 两个原始中心及长短轴；第二个 45°、0–150 km 挤出；保留指定的 10 根垂直线 |
| Circle Outline - Dev | 200 km 地表圆；100 km 半径、0–200 km 挤出的圆及 16 根垂直线 |
| Rectangle - Dev | 原红色半透明地表矩形；绿色 45° 旋转、100–300 km 挤出矩形；closed=true、实例颜色字节量化 |
| Per Instance Color - Dev | 四个相邻矩形；每个独立随机 RGB、alpha=0.7 的字节量化；原默认外观状态 |
| Polygon Outline - Dev | 普通多边形；带孔的 500 km 挤出轮廓及全部角点竖线；第三个逐顶点高度轮廓 |
| Coplanar Polygon Outline - Dev | 保留原 ECEF 直线边，不投影成地表曲线；外环、孔和孔内岛三层层级全部画出 |
| Simple Polyline - Dev | 黄色 WGS84 曲线；高低交替的 ECEF 直线逐段随机颜色；红到蓝的逐顶点渐变曲线 |
| Material - Dev | 原两面 300 km 高墙；Checkerboard / Stripe 的全部默认颜色、透明度、重复数和方向 |
| Ellipsoid Surface - Dev | 原四个矩形；Stripe、Dot、Checkerboard、Grid 的默认参数，Dot 圆点边缘过渡及透明度 |
| CZML Colors | rgba 字节 alpha=100/255 与 rgbaf alpha=0.39 两个红矩形；黑色轮廓、全部集合取景 |
| CZML Point - Time Dynamic | 四个带 epoch 的经纬度高度样本；转换后 ECEF 线性插值；原 300 秒 availability、15 px 点及 3 px 红边；shouldAnimate=true |

## 使用

```bash
python3 scripts/start-test-ui.py
# http://127.0.0.1:8000/bindings/web/sandcastle.html
```

“完整案例移植”有 11 个新增按钮；案例目录中的相同案例也可直接运行。编辑器展示真实输入，可修改后重新应用。支持全部/单项取景、逐实例显隐与删除、实际像素拾取、滚轮缩放、调整窗口大小、播放、暂停和时间轴。

除 CZML Colors 外，本批原始脚本没有 zoomTo。工作台采用统一北美宽视角；“自动取景全部实体”是明确的用户操作，不登记为原脚本调用。动态点启动时保留原 shouldAnimate，其他本批案例暂停时钟。动态点按 availability 自动设置 300 秒时间轴和 duration/120 = 2.5 倍速，可暂停后拖至四个样本及中间时刻。

原始 ECEF 线性插值在部分中间时刻进入椭球内，点会被地球遮挡；没有把路径人为抬高。自动取景在这些时刻增大相机距离，保持相机在地表外。GPU 验证同时检查样本时刻可拾取、中间时刻按实际高度遮挡及循环回绕。

随机颜色案例原本未设种子。提取与验收环境使用 MT19937 种子 5489，记录一组可复现的合法原始颜色，并非把随机 RGB 替换成手写配色；实例颜色仍按 Cesium floatToByte 规则量化。种子在 manifest 中公开。

## 验证与证据

最终机器结果见 `full-recipes-validation.json`；每个案例需同时具备源码/真实输入 SHA-256、GPU、浏览器及桌面记录，才由 `record-full-recipe-results.py` 标记为通过。

- 原始脚本全部执行，严格记录构造选项；独立预期快照与 Rust 编译结果对照。
- 实际 GPU 对每个实例对比显示/隐藏帧，从真实可见像素检查拾取 ID；线条保留空白内部；错误输入保持原集合不变。
- GPU 额外验证 Dot 透明背景的拾取穿透，动态点样本/中间时刻、椭球遮挡和时钟回绕。
- Chromium 的真实 Rust WASM / WebGPU 测试核对全部参数、逐实例像素与拾取、显隐、滚轮相机高度变化、调整窗口和取景；动态点额外检查时钟、拖动时间轴、播放和暂停。
- 桌面程序按真实输入逐例运行 90 帧，要求 `errors 0`。

```bash
npm ci
npm test
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace -- --ignored --test-threads=1
python3 scripts/build-web.py
npm run test:recipes:web
cargo build -p cesium-viewer --bin desktop-viewer
python3 scripts/full-recipes-native.py
python3 scripts/record-full-recipe-results.py
```

日志位于 `full-recipes-*-validation.log`；逐例桌面记录为 `full-<id>-smoke.log`，GPU 图像为 `full-<id>-preview.png`。`full-recipes-browser.json` 记录浏览器实际输入和预期数据的哈希、每个实例的像素数和拾取结果。

## 实现边界

本批增加有界 LineSet（单像素、WGS84 曲线/直线、逐段/逐顶点颜色）、几何层级轮廓、旋转矩形的地理顶点、可配置椭圆竖线及 Dot 程序材质。共享实体集合最多 256 项；Primitive 加载仍原子替换该集合，并非完整独立 PrimitiveCollection。填充沿用 10 m、地理线沿用 10.5 m 深度偏移。材质使用现有光照与 UV 投影，不承诺 Cesium Phong/OIT 或像素级一致性。

浏览器验证使用 Chromium 155、显式 Vulkan / SwiftShader 参数及本地 256×256 中性色影像，以隔离外部影像网络。默认浏览器后端的 device loss 与通用恢复问题仍待解决。尚未通过的目录案例不能计入完整功能进度。
