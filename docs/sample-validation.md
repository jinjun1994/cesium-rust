# bertt 样本验证与测试 UI

日期：2026-10-13；引擎版本：0.0.7。
来源：https://github.com/bertt/cesium_3dtiles_samples
固定 commit：`e703196140dfbcf828b17136d6da691cd3cc9ce9`。清单使用 GitHub 完整递归树，拒绝 truncated 响应；样本 URL 固定到此 commit。

## 验证结果

| 检查范围 | 结果 |
| --- | --- |
| 所有 tileset JSON | 3,721 项均下载并调用生产 Tileset::parse；3,657 通过，64 失败 |
| 默认主要入口 | 94 项；30 个 JSON 解析通过、64 个失败 |
| 点云子树 JSON | 3,627 项；均解析通过；这不代表 pnts 可渲染 |
| 主要入口直接内容 | 每入口最多前 2 个 URI，共 40 个去重 URI；13 通过，25 失败，2 个 HTTP 404 |
| 内容通过项 | 10 个模型解码通过、3 个外部 JSON 解析通过 |
| 原生 GPU 单个模型 | 10 / 10 个解码通过模型均上传、离屏渲染和 readback 成功；观察到非背景像素，PNG 已附 |
| 核心 Rust 测试 | 34 个普通测试通过；4 个 GPU 测试另行执行并通过 |
| UI / 控制器 / SDK 生命周期 | npm test 通过；UI 用真实 DOM、替换 GPU 边界，验证目录、筛选、按钮、轮测、导出、视觉确认和失败状态 |
| 启动服务 | 重定向、页面模块、WASM MIME、目录下载通过 |
| 格式 / lint / WASM | cargo fmt、clippy -D warnings、WASM 构建与绑定生成通过 |
| 浏览器完整渲染 | 当前环境 Chromium 因 socket 权限限制启动失败，未完成；不得将 DOM 测试算作浏览器 GPU 验收 |

JSON 解析通过不证明内容格式受支持；内容解码通过不证明扩展语义完整。原生预览将单个内容独立放入场景，使用自动 framing，不验证整个树的 transform、LOD、网络流式加载、地球遮挡或性能。报告的 `renderStatus: not-tested` 指整个 tileset 尚未视觉验收。

主要失败原因是隐式切片 / viewerRequestVolume、缺少顶层或节点 geometricError、非法 region、要求的扩展，以及 BLEND、动画、i3dm / pnts、无三角形内容或预算超限。两条 HTTP 404 按真实下载失败保留，不改写 URL 隐藏问题。EXT_mesh_features、EXT_structural_metadata 等可选扩展可能只显示几何回退，仍未实现其完整属性语义。

检查器与查看器使用相同预算：64 MiB 输入 / 解码内容、10,000 节点；下载阶段 JSON 限 8 MiB。直接内容的外部 JSON 仅解析，不递归下载所有后代。未逐一解码点云子树引用的 pnts，不进行大城市长时漫游验收。

## 启动及操作

```bash
python3 scripts/start-test-ui.py
# http://127.0.0.1:8000/
# 无自动浏览器：python3 scripts/start-test-ui.py --no-browser
```

源码包自带 WASM；启动不依赖 npm 或 Rust 编译。浏览器需要 WebGPU 与 localhost / HTTPS。目录、批量报告、离屏 PNG 预览在 WebGPU 不可用时仍能打开；实时场景加载与操作按钮会禁用并显示原因。目录默认隐藏点云子树，勾选后可搜索全部条目；列表最多展示 300 条，轮测使用全部筛选条目。

选择条目查看原始 URL、解析错误、内容检查范围和扩展列表，点击“加载选中样本”观察。轮测每项最长观察 10 秒，要求一秒无在途内容且结果稳定才提前结束；记录请求、解码计数、错误和 stats，仍不自动认定视觉通过。“停止”停止排队；当前场景保留。视觉确认只作用于正在加载的选中样本，导出包含人工确认、批量结果与单内容 GPU 结果，不保存 ion token。

功能入口包括全球渐进 Natural Earth II、需 ion token 的默认全球航拍、本地拾取 fixture、GLB / b3dm 文件、自定义 URL、GeoJSON 添加 / 定位 / 清除、旋转、缩放与属性拾取。所有已实现子集以 capabilities.json 为准。

## 复测

```bash
cargo build --locked -p asset-inspector
python3 scripts/validate-samples.py --workers 24 --content-count 2
cargo build --locked -p cesium-viewer --bin sample-render-check
python3 scripts/render-samples.py --max-samples 100
npm ci
npm test
# 在支持 WebGPU 的浏览器环境执行真实端到端：
npx playwright install chromium
npm run test:web
```

更新仓库目录：`python3 scripts/update-sample-catalog.py --ref master`，随后重新验证。UI 拒绝版本不一致的旧报告。下载缓存位于 .sample-cache，不随源码包分发；模型不内嵌，在线加载需要访问 GitHub。清单、JSON 报告和预览全部位于 tests/samples。

本次将 region 的全球保守球改为 WGS84 区间包围球，小区域定位更准确；跨日期变更线、曲面内部与极区的包含性测试通过。pnts/i3dm/cmpt 现在明确返回“不支持”而非误报 GLB 魔数错误。

仓库许可证见 samples-repository-LICENSE.txt。模型来源、URL 与 commit 保留在报告，GPU 预览仅用于此项目测试。
