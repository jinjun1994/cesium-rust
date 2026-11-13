# 真实资产浏览器端到端测试

本套测试在真实 Chromium WebGPU 中通过公开 SDK 加载两组未修改的 CesiumJS
3D Tiles 测试数据，验证 HTTP 加载、瓦片驻留、实际画布像素和截图回归；
建筑案例还点击真实瓦片并检查二进制 Batch Table 的 Height 和 Longitude。
资产来源与固定上游提交见 [fixtures/README.md](fixtures/README.md)。

```sh
npm ci
python3 scripts/build-web.py
npx playwright install chromium
npm run test:e2e
```

`build-web.py` 要求安装与 Cargo.lock 一致的 wasm-bindgen-cli；`pkg/` 是被
Git 忽略的生成文件，测试前必须从当前源码构建，避免误测旧 WASM。也可用
`CHROMIUM_PATH` 指定兼容 WebGPU 的浏览器可执行文件。WebGPU 初始化失败
或基准图缺失均视为失败；端口 18765 必须可用。运行结果及失败 trace 位于
`target/e2e-results/`。

截图保存在 `cesium-assets.spec.cjs-snapshots/`，目前提供的是 macOS arm64、
Google Chrome 157.0.8103.51 环境下由本项目生成的基准。测试使用固定画布、
本地中性影像、无地形和离线资产；不同浏览器、系统、GPU 或驱动仍可能产生
差异。其他环境需要使用受控的相同环境，或审查后建立该环境的基准：

```sh
npm run test:e2e:update
npm run test:e2e
```

更新截图会覆盖基准，需先检查实际画面及差异，不能仅为消除失败而更新。
现有图是 Rust 输出的回归基准，不是 CesiumJS 官方截图，也不能证明与原版
视觉或 API 完全一致。当前仅覆盖两组资产，尚未覆盖完整 Sandcastle、在线
服务、动画或跨浏览器。功能断言与截图比较应共同使用。

## 原版 CesiumJS Sandcastle 同场景截图对照

为了验证 Rust 项目与原版 CesiumJS Sandcastle 在同一场景下的视觉一致性，
新增了基于 960x540 画布的 Sandcastle 对照测试套件：

- [sandcastle-harness.html](sandcastle-harness.html)：960x540 固定视口、本地 Natural Earth II 影像的 Rust WebGPU 承载页，支持全部 309 个 `full-recipes.json` 案例（`entities`、`czml`、`primitives`、`groups`、`scene`、`variants`）。
- [scripts/capture-sandcastle-references.mjs](../../scripts/capture-sandcastle-references.mjs)：连接本地原版 CesiumJS 服务（`http://localhost:8080/Apps/Sandcastle2/standalone.html?id=<id>`），固定时钟 `2023-12-25T14:00:00Z` 抓取 960x540 基准截图至 `tests/e2e/sandcastle-reference/<id>.png`。
- [sandcastle-compare.spec.cjs](sandcastle-compare.spec.cjs) 与 [visual-compare.cjs](visual-compare.cjs)：在 Rust WebGPU 中渲染 309 个案例，检查零报错与非空像素，并与 `tests/e2e/sandcastle-reference/<id>.png` 计算差异像素比例（threshold=0.25）及 RMSE，生成三联差异图（`target/e2e-results/sandcastle-diff/<id>.png`）及汇总报告（`docs/sandcastle-visual-compare.md` / `.json`）。

常用命令：

```sh
# 1. 在 ../cesium 启动原版 CesiumJS 服务后，抓取原版 Sandcastle 基准图
npm run capture:sandcastle:refs -- --skip-existing --concurrency=4

# 2. 运行全部 309 个案例（或用 SANDCASTLE_IDS=box,wall 过滤）
npm run test:e2e:sandcastle

# 3. 更新 Rust 侧回归快照
npm run test:e2e:sandcastle:update
```
