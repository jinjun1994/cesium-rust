# 本次验证记录

日期：2026-10-12。工具链 Rust 1.99.0；最终锁定 wgpu 30.0.1、glam 0.30.9、winit 0.30.12、gltf 1.4.1。所有运行结果只针对本记录中的小型合成资产和 Linux 软件适配器环境。

| 检查 | 结果 / 范围 |
| --- | --- |
| cargo fmt | 通过 |
| cargo clippy --workspace --all-targets -D warnings | 通过 |
| cargo test --workspace | 22 个普通测试通过；GPU 测试默认 ignored，另行执行 |
| wasm32-unknown-unknown 构建 | 通过；生成 ES 模块、WASM 与 TypeScript 声明 |
| naga WGSL / Rust uniform 布局 | 通过 |
| 原生离屏 GPU 测试 | 通过；Mesa Vulkan Lavapipe 25.2.8 |
| 原生窗口 smoke | 通过；Xvfb / Vulkan，内嵌场景 6 帧、从文件 tileset 流式加载 20 帧，驻留内容 1 / 绘制 1 / 错误 0 |
| asset-inspector b3dm validate | 通过；Batch Table 名称与 height 正确 |
| 浏览器端到端测试 | **未通过**；Chromium Headless Shell 151.0.7922.34 + 软件 WebGPU，在初始化后设备报告 Destroyed；点击未完成 |
| CI 工作流 | 已提供；未提交远程仓库执行，不宣称 CI 已通过 |

离屏 GPU 测试验证真实渲染和 readback：中心整数 ID 命中、空命中、颜色关键像素、高亮变化、factor MASK discard、负缩放单面剔除、resize 后旧拾取失效、零尺寸暂停。测试通过不证明所有硬件/浏览器正确，也不证明城市资产性能目标。

格式测试包含每一字节位置截断、长度溢出、访问器范围、循环节点、sparse 顺序、stride、明确拒绝 emissive、512 次固定种子小型字节变异。小型变异 smoke 不是长期 fuzzing 验收。

## 浏览器尚未解决的问题

修复了真实的 Canvas 格式选择问题：浏览器通常仅报告 UNORM，渲染器现在配置相容的 sRGB view，而非要求 surface 本体直接为 sRGB。SDK 同时修复了 Rust 抛出字符串错误在 UI 中被显示为空的问题。

随后出现 DeviceLost(Destroyed)。独立 JavaScript WebGPU compute/readback 在相同环境能返回预期值 1234；运行引擎时仍失败。曾对 wgpu 30.0.0 与 29.0.0 做比较，均复现，因此**没有足够证据将根因归于某个 wgpu 补丁或 HDR 探测**。最终保留原计划的 30.0.1；失败测试保留在 scripts/browser-smoke.cjs，作为阻止浏览器发布的验收门槛。

设备错误现在进入集中诊断；失效后的 resize、上传与 render 返回持久错误，避免继续使用失效设备。自动重建和恢复尚未实现。需要在真实 Chrome/Edge + 硬件 WebGPU 上复测，再用最小 wgpu Canvas 场景隔离设备、surface 与管线生命周期；不能把 WASM 编译通过写成浏览器运行验收通过。

## 未完成的验收

未执行 macOS/Windows、真实硬件 GPU、长期漫游、真实大资产、p95/p99、总 CPU/GPU/WASM 内存峰值、Worker、设备恢复、HTTP 重试或完整 glTF/3D Tiles conformance。

## 0.0.2 地球渲染验证

25 项普通测试通过（新增全球相机/最小距离、底图尺寸、UV 接缝）。fmt 和 clippy 全部通过。Vulkan 软件适配器实际导出 1280×800 地球图，检测到 237081 个非背景像素、56912 种颜色，轮廓位于 (358,119)..(921,681)，不触及视口边界。`docs/earth-preview.png` 经目视检查：亚洲、海洋、极地、光照分界和完整球体清晰可见。

WASM 和 JS 绑定重新构建，默认 Scene 不再载入三角形；浏览器运行的 DeviceLost 问题本次没有重新宣称解决。原生离屏拾取回归另行执行。

桌面窗口追加验证：新的默认全球场景运行 6 帧，显示 `Earth yes | resident 0 | GPU 3.9 MiB | draws 1 | errors 0`；`--demo-tile` 和文件 tileset 路径分别运行 6/20 帧，均无错误。底图采样后原有 MASK/整数 ID/高亮/负缩放/resize 离屏回归通过。

## 0.0.3 渐进影像验证

普通测试 28 项通过；fmt、clippy、WASM 构建、JS 语法检查通过。新增协议测试覆盖 Bing QuadKey、TMS Y 翻转、Mercator 极限、缩放细化、父级 UV、元数据版权、空 token 和超限/损坏图片。新的真实 Vulkan GPU 测试验证根瓦片优先、祖先回退无缺失绘制、取消后拒绝旧提交/重新调度、大图缓存背压和实际绘制。

从 Cesium 1.146 官方发布资源下载并解码全部 42 张 Natural Earth II JPEG，合计 475179 字节，256×256。与默认 Scene 共用的 GPU 导出实际加载 33 张祖先/叶瓦片，绘制 24 个地理覆盖区域，最高 LOD 2，估计 GPU 总驻留 17.7 MiB，错误 0。图像为 1280×800，243941 个非背景像素，85362 种颜色，球体边界 (358,119)..(921,681)。`docs/imagery-preview.png` 经目视检查，陆地/海洋拼接完整。

Xvfb 原生窗口从这些真实 JPEG 文件通过后台线程异步加载，运行 100 帧：`imagery 33 ready / 24 regions | requests 0 | LOD 2 | failures 0`，内容错误 0。这是文件缓存的真实下载/解码/上传路径验证；不等同于已验证 Bing 联网。没有用户 ion token，默认 ion/Bing 请求只做元数据协议测试。Web 编译/SDK接口已更新，DeviceLost 问题未解决，不宣称浏览器运行通过。

默认在线源追加验证：不设置本地影像模板，原生程序通过 HTTPS 从官方 Cesium 1.146 资源实时下载 Natural Earth II，运行 600 帧后 33 张已就绪、24 个覆盖区域、请求 0、失败 0、内容错误 0。首次测试定位到运行环境代理 CA 不在 WebPKI 静态根集合中；原生 reqwest 已改用 rustls + 系统信任库，实际 HTTP 200 和在线窗口复测通过。证书链校验保持开启。

## 0.0.4 缩放验证

新增相机回归验证：放大缩小可逆、连续放大到约 10 米、近裁剪面小于 1 米、不同纬度与旋转后保持地表外部、非有限输入不破坏相机。29 个普通 Rust 测试通过。真实 GPU 影像测试追加近地视角覆盖/绘制，确保缩放后有瓦片绘制。DOM 控制器通过真实 EventTarget 事件测试 pixel/line/page 滚轮单位、双向缩放、键盘、双指分合、双指结束不误拾取、取消和卸载监听。

Web 控制器测试不依赖 GPU；完整浏览器 WebGPU 端到端原有 DeviceLost 问题仍未验收。fmt、clippy、WASM 构建及绑定更新通过。

真实 X11 窗口通过 XTest 注入滚轮与键盘输入：初始高度 10527761.3 米 → 滚轮放大后 64.7 米 → 键盘继续放大后 10.0 米 → 键盘缩小后 12.2 米。每个结果直接读取渲染帧更新后的窗口标题，应用正常退出；不只是单独调用相机函数。

## 0.0.5 模型材质和外部树验证（2026-10-13）

31 个普通 Rust 测试通过。新增内嵌/data URI 图像测试覆盖五类通道、sampler enum、第二套 UV、切线/颜色、image byte/dimension limit 和不支持的 UV set。外部树测试覆盖变换/refine 继承、相对 URL、祖先循环和失败时不修改原树的节点限制。

真实 GPU 材质测试读取像素并逐一改变遮蔽、法线、金属粗糙度、自发光；各通道都会改变颜色。UV0/UV1 路径和顶点着色也通过像素断言。透明纹理 MASK 不产生拾取 ID，改为 OPAQUE 后重新可拾取。原拾取/高亮/负缩放回归与影像覆盖/缓存/10m 近地绘制均通过。

原生 Xvfb/Vulkan 软件适配器：textured.glb 和 external-tileset.json 分别运行 60 帧，驻留模型 1、错误 0，外部路径走真实后台请求后接入子树再请求带纹理 GLB。默认地球导出回归为 1280×800、237069 个非背景像素、56958 种颜色，轮廓 (358,119)..(921,681)。WASM 构建和 bindings 更新通过，Web 控制器测试通过。浏览器全链路仍受原 DeviceLost 问题阻塞；不将编译通过写成浏览器验收。

## 0.0.6 GeoJSON 验证（2026-10-13）

33 个普通 Rust 测试通过。GeoJSON CPU 测试覆盖各 Geometry 类型、嵌套属性、空几何、无效坐标、输出顶点预算；多边形孔洞面积为 84 平方度（外环 100、内环 16），输出三角面中心没有落入孔洞，跨日界线顶点位于正确 ECEF 半球；未闭合环与自交环被拒绝。

新的 GPU 测试验证地球叠加、孔洞无面 ID、面和点属性拾取、删除后属性/几何消失、图层定位/清空与极点定位。原材质/ID 回归和近地影像缓存测试也通过，共 4 个显式 GPU 测试。Xvfb 原生窗口的 GeoJSON 单独输入和 GeoJSON+3D Tiles 叠加分别运行 60 帧，绘制 4/5、错误 0、GeoJSON layers 1。

真实 GPU 导出 `docs/geojson-preview.png`：1280×800，237081 个非背景像素，56585 种颜色。自动图像检查检测到 4018 个面色像素、907 个折线色像素和 117 个点色像素，经目视检查环孔保留。fmt、clippy、WASM 构建、绑定更新与 DOM 控制器检查通过。新增 SDK lifecycle 测试使用实际 SDK、真实 Response 流和模拟 GPU 边界，验证主源替换时迟到瓦片被丢弃、清空图层时迟到 GeoJSON 导入被丢弃及添加/定位/删除/destroy。

这些 SDK 单元测试不等同于 WebGPU 浏览器端到端验收；原有 DeviceLost 问题仍未解决。没有执行大规模全球 polygon、地形贴地或长期交互性能验收。

GeoJSON 使用共享 f64 ECEF 原点和局部 f32 顶点；额外验证远离 ECEF 原点的 1 米标记未因量化而坍缩，面孔洞面积在恢复 model transform 后仍正确。

## 0.0.7 · 2026-10-13

新增 bertt 仓库样本验证与 UI，结果及覆盖范围详见 sample-validation.md。34 个普通 Rust 测试、4 个原生 GPU 测试、npm test、启动 HTTP 服务检查通过。完整浏览器 GPU 测试受当前环境 Chromium socket 权限阻止，未通过；DOM 测试不替代视觉验收。

## 0.0.8 · 2026-10-13

全球地形：37 个普通 Rust 测试、5 个显式 GPU 测试、真实公开地形 92 瓦片的下载/解码/渲染/readback、桌面本地地形 120 帧 smoke 均通过。Web 构建与 DOM/SDK 测试通过；实时 ion 与浏览器 GPU 尚未验收。详情见 terrain-validation.md。
