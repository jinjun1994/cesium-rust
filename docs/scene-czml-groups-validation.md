# 六个完整案例（0.0.24）

固定 Cesium 1.146、commit `b8d3a36fe98a3e432eb89253c95d5f20e605e0f1`。本批新增六个完整案例验收，累计 **45 / 309**，剩余 264 个。Hello World 将已有地球基础链路纳入完整源码与交互验收，未把它描述为新建的渲染器。

| 案例 | 完整案例行为 |
| --- | --- |
| CZML Corridor | 原始三个 200 km 宽走廊及全部 CZML packet；圆角、斜接角、倒角，高度与挤出、黑/白轮廓；红 alpha=127/255、绿色字节 255、蓝色不透明；异步加载后全集合取景 |
| CZML Position Definitions | 原始三个 Point，分别为角度、ECEF 米坐标与弧度位置，原颜色、像素尺寸、名称及原默认视角（原脚本没有 zoomTo）；原弧度纬度为 3π/4，经过 WGS84 ECEF 转换后保留同一真实位置 |
| Show or Hide Entities | 原三个无几何父实体与 15 个图形子实体；固定种子 1234 的原始随机颜色、五层高度、箱体/球体/椭球尺寸；三按钮分别切换父级 show，保留子级自身 show；关闭 infoBox、全集合取景 |
| HTML Overlays | 原 Cesium PNG、原地理锚点及 HTML 绝对定位；preRender 每帧投影到 CSS 坐标、复用结果对象；随相机、窗口尺寸及渲染分辨率更新，切换案例移除监听 |
| Resolution Scaling | 原始 0.25 倍与非推荐模式；滑块、文本、推荐开关；原 Number 转换、NaN 回退 1、0.1–2 裁剪；实际 WebGPU framebuffer 与 DPR 模式切换 |
| Hello World | 原单条 Viewer 创建语句对应的地球初始化、默认观察视角与已有影像链路；真实 WASM 地球像素、滚轮缩放及前案例清理；不加入额外实体 |

## 实现与输入

新增 `entity_groups.rs`、`scene_projection.rs` 和 `scene_recipes.rs`，复用现有几何、相机、地球和拾取通路。CZML 原始字节直接进入 Rust 解析器；分组原始配方编译为 `.groups.json`，父级与图形独立存储；浏览器场景使用 `.scene.json`。对应 `.json` 为独立预期快照，三个场景案例的实体数量为零。

构建记录器执行六份原始脚本的全部语句，执行原分组按钮及分辨率订阅回调，并保留 Viewer 选项和默认取景语义。运行时由 Rust / WGSL 绘制，未引入 CesiumJS 渲染回退。HTML / PNG 来自同一固定 commit，清单记录脚本、HTML 与图片 SHA-256。图片来源为 `packages/sandcastle/images/Cesium_Logo_overlay.png`；保留上游 Apache 2.0 许可 `tests/sandcastle/UPSTREAM-LICENSE.md`，原商标标识仅用于重现该案例。

新接口包括 Web `loadGroupedEntities` / `entityGroups` / `showEntityGroup`、`scene.preRender`、`scene.cartesianToCanvasCoordinates`、`resolutionScale` 与 `useBrowserRecommendedResolution`。分组导入有大小、并发、源切换和 destroy 保护；非法成员关系或几何导入保持已有集合。推荐模式使用 CSS 尺寸；非推荐模式乘浏览器 DPR，两种模式均再乘 resolutionScale。拾取坐标继续转换到实际 framebuffer，resize 会清除过期拾取。

## 启动和手动测试

```bash
npm start
```

打开 `http://127.0.0.1:8000/bindings/web/sandcastle.html`，点击新增六个“完整”按钮，或在目录选择案例后点击“运行当前完整案例”。

- 分组案例：点击 Toggle Boxes / Ellipsoids / Spheres，检查对应五个图形隐藏，再点击恢复。
- 覆盖层案例：拖拽和缩放，图片保持绑定原地理位置；切换其他案例后图片与监听消失。
- 分辨率案例：拖动滑块、输入文本并切换推荐开关；画布 CSS 大小保持布局尺寸，内部渲染分辨率改变。高 DPI 屏幕可观察两个模式差异。
- 两个 CZML 案例：检查全部三项图形、颜色、轮廓、选择与缩放；走廊自动取景，坐标案例保留默认视角并支持手动取景。
- Hello World：回到无实体的地球视图，滚轮与缩放按钮均可操作。

交付包已包含本批编译后的 WASM。需要重编译时运行 `python3 scripts/build-web.py`，要求 wasm32 目标和锁定版本 wasm-bindgen-cli 0.2.129。

## 验证证据

105 项普通 Rust、72 项实际 GPU、8 组 Node 测试通过；45 个完整案例在同一 Chromium WebGPU 主会话回归，另建 DPR=2 上下文检查真实 framebuffer；45 个桌面配置分别运行 90 帧，`errors 0`。fmt、clippy、原生构建及 WASM/绑定生成通过。

GPU 对新增几何逐实例检查可见像素、隐藏帧差异和整数 ID 拾取；分组检查三个父级的真实帧变化、恢复、子级自身隐藏保留和错误导入原子性。浏览器核对原始输入、全部图形、三个分组按钮、infoBox 隐藏、投影结果复用、相机/resize/分辨率变化、案例清理及销毁。

DPR=2 的专项实测：CSS 615×468、scale=0.25 时，非推荐模式 framebuffer 308×234，推荐模式 154×117；后者仍有 822 个可见地球像素。像素读取在实际 requestAnimationFrame 内进行。测试 HTTP 服务现在正确提供 CSS/PNG MIME 类型，45 案例按实际页面布局重新运行。

证据：`full-recipes-validation.json`、`full-recipes-*-validation.log`、`full-recipes-browser.json`、`full-<id>-preview.png` 和 `full-<id>-smoke.log`。另外提供三个 `full-<scene-id>-browser.png`，其中覆盖层图片为实际浏览器 DOM 截图；GPU 场景预览仅显示共享地球画面。

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

## 验收边界

完整指这六份固定源码的全部案例行为，不表示全 Cesium API 或逐像素一致。分组为平级、单父级的无几何文件夹，未扩展到任意嵌套 Entity 属性继承；走廊仍为有界静态输入，未实现动态宽度、地形贴合或完整 CZML 引用系统。

HTML DOM、原网页控件和 DPR 的完整行为在 Web 端验收。三个场景案例的原生 90 帧测试仅覆盖共享地球/投影内核，桌面端没有声称提供浏览器 DOM 或这些 HTML 控件。地球影像的浏览器自动化使用本地确定性栅格；实时 ion/Bing 仍需用户 token。测试使用显式 Vulkan / SwiftShader；默认浏览器后端 device loss 与完整恢复仍待完成。Viewer 全套控件、2D/Columbus、高级后处理等其余 264 个案例继续推进。
