# 两个新增完整案例（0.0.28）

固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。本批完整实现并验收 **CZML Rectangle** 与 **Partial Ellipsoids** 两个完整源码案例，覆盖 16 个原始实例。累计完整功能案例通过数 **59 / 309**，剩余 250 个待完整验收。

| 案例 | 保留的完整案例行为 |
| --- | --- |
| CZML Rectangle | 四个原始 CZML 矩形实例：红色拉伸矩形（extrudedHeight=300000）、垂直条纹矩形（Stripe material, orientation=VERTICAL, evenColor=GREEN, oddColor=WHITE）、黄色带边框矩形（fill=false, outline=true, outlineColor=YELLOW）、贴图矩形（Image material 引用 `Cesium_Logo_Color.jpg`）；全集合 `viewer.zoomTo` 自动取景 |
| Partial Ellipsoids | 全部十二个完整及局部椭球实例：实心完整球体、内径空心圆环（innerRadii，类似土星环）、带内径的局部球壳、时钟角切口（minimumClock / maximumClock）、天顶角切口（minimumCone / maximumCone）、时钟与天顶复合切口椭球、带方位角四元数（orientation 经 headingPitchRollQuaternion 转换）、外表面、内表面、侧边端盖与轮廓线；全集合 `viewer.zoomTo` 自动取景 |

## 实现与测试 UI

1. **局部与空心椭球几何生成**：
   - 扩展椭球参数模型，支持 `innerRadii`、`minimumClock`、`maximumClock`、`minimumCone`、`maximumCone`。
   - 顶点计算时分流生成外表面网格、内表面网格（反转法线）、时钟角端面与天顶角锥盖。
   - 支持空心几何与环状几何（如 Saturn ring）的表面三角面片几何质心拾取，避免传统原点拾取采样落入空腔的误判。

2. **CZML 矩形与丰富材质**：
   - 支持 CZML 矩形包属性解析：`coordinates`（WGS84 经纬度边界）、`height`、`extrudedHeight`、`rotation`。
   - 支持 CZML 材质体系：`solidColor`、`stripe`（水平/垂直条纹、偶奇颜色配置）、`image`（自动将相对路径资源转为 base64 data URI 并完成 GPU 纹理水合绑定）。
   - 规范 CZML `outlineWidth`（容忍上游 1.0..=10.0 并统一约束到 WebGPU 1 像素物理线宽，拦截非法 0 或越界配置）。

3. **WebAssembly / 跨平台时间抽象**：
   - 在 `crates/cesium-renderer` 中引入 `web-time = "1.1.0"` 依赖，消除 `wasm32-unknown-unknown` 下 `std::time::Instant::now()` 的 platform unreachable panic，保证浏览器端 WebGPU 运行的完全稳定性。

4. **测试 UI**：
   - 运行 `python3 scripts/start-test-ui.py`，访问 `http://127.0.0.1:8000/bindings/web/sandcastle.html`。
   - 新增“完整 CZML Rectangle”与“完整 Partial Ellipsoids”按钮，并可直接在 309 案例目录中搜索并一键执行。

## 验收证据

1. **源码契约与哈希校验**：
   - 固定源码经执行提取真实结构，`czml-rectangle` 与 `partial-ellipsoids` SHA-256 严密绑定。
2. **普通 Rust 单元测试**：
   - 全工作区 122 项单元测试通过（包含 `ellipsoid_tests::batch28_partial_ellipsoids_and_czml_rectangle`）。
3. **真实 GPU 离屏渲染测试**：
   - `crates/cesium-viewer/tests/full-recipes-gpu.rs` 全部 60 项测试通过（包含 `complete_czml_rectangle` 和 `complete_partial_ellipsoids`）。
4. **桌面 90 帧冒烟测试**：
   - 原生运行 90 帧，无任何 GPU 或渲染错误，日志记录于 `docs/full-czml-rectangle-smoke.log` 与 `docs/full-partial-ellipsoids-smoke.log`。
5. **浏览器 WebGPU 端到端测试**：
   - 在真实 Chromium 155 环境下运行 `scripts/full-recipes-browser.cjs`，所有 59 个完整案例全部通过画布颜色反读、实体 ID 拾取、显隐切换、滚轮缩放与窗口自适应检查，销毁状态正常（`destroy: 'passed'`）。
6. **Node 测试套件**：
   - 8 组 Node 测试套件全部通过（`npm test`）。
