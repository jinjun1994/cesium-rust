# 四个完整案例（0.0.23）

累计 **39 / 309** 个完整案例通过，剩余 270 个。本轮验证通过 103 项普通 Rust、66 项实际 GPU、8 组 Node、39 项同会话浏览器 WebGPU 及 39 项桌面 90 帧运行。

本轮移植固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1` 的 Circle - Dev、Corridor、Corridor - Dev、Corridor Outline - Dev。原始脚本全部执行并保留输入，运行时由 Rust / WGSL 绘制。

| 案例 | 保留的全部案例行为 |
| --- | --- |
| Circle - Dev | -95°/43° 的红色 250 km 地表圆；-90°/50° 的绿色 250 km 半径、0–300 km 挤出圆；-85°/40° 的蓝色 400 km 半径、150 km 高圆。红/绿 alpha=0.5 经实例字节量化；全部 closed=true，蓝色 translucent=false |
| Corridor | 原三个 200 km 宽折线走廊：红色地表圆角及圆端帽、绿色 100 km 高斜接角及黑轮廓、蓝色 100–200 km 高倒角挤出及白轮廓。保留原中心路径、名称、CSS GREEN、透明度和整个集合 zoomTo |
| Corridor - Dev | 同一组中心路径及三种角型，保留每个 Primitive 独立外观：closed=true，绿色 translucent=false，红/蓝半透明实例颜色字节量化；原脚本没有相机调用 |
| Corridor Outline - Dev | 白色地表圆角走廊；白色 0–100 km 挤出的斜接角走廊，包含两层边界、端部及角点竖线；flat=true，lineWidth=min(2, maximumAliasedLineWidth=1) |

## 启动与测试 UI

```bash
python3 scripts/start-test-ui.py
# http://127.0.0.1:8000/bindings/web/sandcastle.html
```

“完整案例移植”增加四个按钮；目录中选择同名案例也可运行。编辑器显示真实 Entity / Primitive 输入，可修改路径、宽度、角型、高度及挤出后重新应用。支持逐项显隐、删除、定位、拾取、滚轮缩放、调整窗口和全部取景。

只有 Entity Corridor 原脚本调用 zoomTo。三个 Dev 案例沿用工作台北美宽视角，用户可手动“自动取景全部实体”；不把该操作记作原脚本行为。

## 几何与独立参考

新增 `Shape::Corridor`、`CornerType` 和 Rust 走廊构建器。使用固定 Cesium core 的切平面偏移、内侧交点、圆角/斜接角/倒角及端帽方程；中心线由已有 WGS84 Vincenty 求解器细分。填充复用有界地理多边形网格，挤出上下帽按高低排列，轮廓复用同一条边界并连接必要竖线。`CircleGeometry` 编译到已有等长轴椭圆网格，保留独立 height 和 extrudedHeight。

独立数据 `tests/sandcastle/full/corridor-reference.json` 由真实 `@cesium/core@0.1.0` 的 CorridorOutlineGeometry 生成；该包 CorridorGeometryLibrary.js 的 SHA-256 与固定 commit 的文件一致。参考生成器只用于离线测试，不参与运行时绘制或作为 CesiumJS 回退。

三种角型的边界进行双向点到线段检查，限定本组固定 200 km 宽路径的偏差 <300 m；实测最大 236.362 m。参考默认 1° 细分，本实现约 0.1° 细分，曲线上的新顶点与参考折线之间存在弦高差。这是指定案例的网格检查阈值，不是全球测绘精度保证或像素级比对。还检查闭环、斜接角挤出竖线、全部原始字段及错误参数拒绝。

可选重建独立参考：

```bash
npm install --prefix .reference-tools @cesium/core@0.1.0 mersenne-twister earcut
node scripts/build-corridor-reference.mjs
```

生成器先核对固定几何库 SHA-256。参考许可见 `tests/sandcastle/reference-license.md`。

## 验证

最终统计见 `full-recipes-validation.json`。只有原始脚本/输入哈希、实际 GPU、浏览器及桌面记录一致，案例才标记通过。

- 原始脚本全部语句执行；原始输入与独立 Entity 预期快照核对。
- 实际 GPU 逐实例对比显示/隐藏帧，并从可见像素验证 ID 拾取；轮廓保留空白内部，错误导入保持原集合。
- Rust WASM / Chromium WebGPU 核对全部参数、逐实例像素和拾取、滚轮及调整窗口；与此前完整案例共同回归。
- 每个案例的桌面程序使用真实输入运行 90 帧，要求 `errors 0`。
- 普通 Rust、Node、fmt、clippy 以及全套实际 GPU 回归。

```bash
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

证据包括 `full-recipes-*-validation.log`、`corridor-reference-validation.log`、`full-recipes-browser.json`，以及每个 `full-<id>-preview.png` / `full-<id>-smoke.log`。

## 边界

走廊目前为有界静态 3D Entity / Primitive 输入：2–32 个中心位置、宽 1–1,000,000 m、高度 ±1,000,000 m、生成边界最多 1024 点；已有输入/GPU 预算继续有效。连续重复点、折返和产生无效多边形的配置会报错。保留既有填充 10 m、地理轮廓 10.5 m 深度偏移。未实现 Corridor CZML、动态宽度/位置属性、地形贴合或完整 Cesium PrimitiveCollection；这些不能计入本轮案例。

浏览器采用 Chromium 155、显式 Vulkan / SwiftShader 和本地确定性影像；完整 Viewer、光照/OIT、默认后端 device loss 恢复和像素级 Cesium 一致性仍不在已通过范围。
