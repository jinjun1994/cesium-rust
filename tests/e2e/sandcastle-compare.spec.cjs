const fs = require('node:fs');
const path = require('node:path');
const { test, expect } = require('playwright/test');
const { comparePngBuffers } = require('./visual-compare.cjs');

const ROOT = path.resolve(__dirname, '../..');
const FULL_RECIPES = JSON.parse(fs.readFileSync(path.join(ROOT, 'tests/sandcastle/full-recipes.json'), 'utf8'));
const REF_DIR = path.join(ROOT, 'tests/e2e/sandcastle-reference');
const DIFF_DIR = path.join(ROOT, 'target/e2e-results/sandcastle-diff');
const REPORT_JSON = path.join(ROOT, 'docs/sandcastle-visual-compare.json');
const REPORT_MD = path.join(ROOT, 'docs/sandcastle-visual-compare.md');

function selectCases() {
  let cases = FULL_RECIPES.cases;
  if (process.env.SANDCASTLE_IDS) {
    const wanted = new Set(process.env.SANDCASTLE_IDS.split(',').map(s => s.trim()).filter(Boolean));
    cases = cases.filter(c => wanted.has(c.id));
  } else {
    const offset = Number(process.env.SANDCASTLE_OFFSET || 0);
    const limit = process.env.SANDCASTLE_LIMIT ? Number(process.env.SANDCASTLE_LIMIT) : cases.length;
    cases = cases.slice(offset, offset + limit);
  }
  return cases;
}

const SELECTED_CASES = selectCases();

test.describe('Sandcastle same-scene screenshot comparison', () => {
  let page;
  let pageErrors = [];
  const results = [];

  async function ensureHarnessReady() {
    if (page && !page.isClosed()) {
      const hasViewer = await page.evaluate(() => Boolean(window.viewer && window.runSandcastleCase)).catch(() => false);
      if (hasViewer) return;
    }
    if (page && !page.isClosed()) await page.close().catch(() => {});
    page = await test.info()._projectInternal ? page : page;
  }

  test.beforeAll(async ({ browser }) => {
    fs.mkdirSync(DIFF_DIR, { recursive: true });
    const context = await browser.newContext({ viewport: { width: 960, height: 540 }, deviceScaleFactor: 1 });
    page = await context.newPage();
    page.on('pageerror', err => pageErrors.push(err.message));
    await page.route('**/*', route => {
      const url = new URL(route.request().url());
      if ((url.hostname === '127.0.0.1' || url.hostname === 'localhost') && url.pathname.startsWith('/tests/SampleData/')) {
        const localPath = path.join('/Users/jinjun/study/cesium-rust/cesium/Apps/SampleData', url.pathname.slice('/tests/SampleData/'.length));
        if (fs.existsSync(localPath) && fs.statSync(localPath).isFile()) {
          return route.fulfill({ path: localPath });
        }
      }
      if (url.hostname === '127.0.0.1' || url.protocol === 'data:') return route.continue();
      if (process.env.OFFLINE_ONLY === '1') return route.abort();
      return route.continue();
    });
    const harnessUrl = process.env.CESIUM_ION_TOKEN !== undefined
      ? `/tests/e2e/sandcastle-harness.html?ionToken=${encodeURIComponent(process.env.CESIUM_ION_TOKEN)}`
      : '/tests/e2e/sandcastle-harness.html';
    await page.goto(harnessUrl);
    await page.waitForFunction(() => Boolean(window.viewer && window.runSandcastleCase), null, { timeout: 30000 });
  });

  test.afterAll(async () => {
    if (page) await page.context().close().catch(() => {});

    let merged = results;
    if (fs.existsSync(REPORT_JSON)) {
      try {
        const prev = JSON.parse(fs.readFileSync(REPORT_JSON, 'utf8'));
        if (Array.isArray(prev.cases)) {
          const byId = new Map(prev.cases.map(c => [c.id, c]));
          for (const r of results) byId.set(r.id, r);
          const order = new Map(ALL_CASES.map((c, i) => [c.id, i]));
          merged = Array.from(byId.values()).sort((a, b) => (order.get(a.id) ?? 9999) - (order.get(b.id) ?? 9999));
        }
      } catch (_) {}
    }

    const summary = {
      generatedAt: new Date().toISOString(),
      totalSelected: merged.length,
      rustRendered: merged.filter(r => r.rustStatus === 'passed').length,
      rustFailed: merged.filter(r => r.rustStatus === 'failed').length,
      withReference: merged.filter(r => r.hasReference).length,
      refMatchedWithin2Pct: merged.filter(r => r.hasReference && r.diffPixelRatio <= 0.02).length,
      avgDiffPixelRatio: (() => {
        const compared = merged.filter(r => r.hasReference && typeof r.diffPixelRatio === 'number');
        return compared.length ? compared.reduce((s, r) => s + r.diffPixelRatio, 0) / compared.length : null;
      })(),
      avgRmse: (() => {
        const compared = merged.filter(r => r.hasReference && typeof r.rmse === 'number');
        return compared.length ? compared.reduce((s, r) => s + r.rmse, 0) / compared.length : null;
      })(),
      cases: merged,
    };

    fs.writeFileSync(REPORT_JSON, JSON.stringify(summary, null, 2) + '\n');

    const lines = [
      '# Sandcastle 同场景视觉对照报告',
      '',
      `- 生成时间: ${summary.generatedAt}`,
      `- 测试案例数: ${summary.totalSelected}`,
      `- Rust WebGPU 渲染成功: ${summary.rustRendered} / ${summary.totalSelected}`,
      `- 已有原版 CesiumJS 基准图: ${summary.withReference}`,
      `- 原版视觉误差 <= 2%: ${summary.refMatchedWithin2Pct} / ${summary.withReference}`,
      ...(summary.avgDiffPixelRatio !== null ? [`- 平均像素差异率 (threshold=0.25): ${(summary.avgDiffPixelRatio * 100).toFixed(2)}%`] : []),
      ...(summary.avgRmse !== null ? [`- 平均 RMSE: ${summary.avgRmse.toFixed(4)}`] : []),
      '',
      '| 案例 ID | 格式 | Rust 状态 | 非空像素 | 原版基准 | 差异像素率 | RMSE | 差异图 |',
      '| --- | --- | --- | ---: | --- | ---: | ---: | --- |',
    ];
    for (const r of merged) {
      const diffPct = r.hasReference && typeof r.diffPixelRatio === 'number' ? `${(r.diffPixelRatio * 100).toFixed(2)}%` : '-';
      const rmse = r.hasReference && typeof r.rmse === 'number' ? r.rmse.toFixed(4) : '-';
      const diffImg = r.diffImage ? `[diff](${path.relative(path.join(ROOT, 'docs'), r.diffImage)})` : '-';
      lines.push(`| ${r.id} | ${r.format} | ${r.rustStatus} | ${r.visiblePixels ?? 0} | ${r.hasReference ? 'yes' : 'no'} | ${diffPct} | ${rmse} | ${diffImg} |`);
    }
    fs.writeFileSync(REPORT_MD, lines.join('\n') + '\n');
  });

  for (const entry of SELECTED_CASES) {
    test(`Sandcastle ${entry.id}`, async () => {
      pageErrors = [];
      await page.evaluate(() => { window.testErrors = []; });

      const record = {
        id: entry.id,
        format: entry.format,
        framing: entry.framing,
        rustStatus: 'passed',
        hasReference: false,
        visiblePixels: 0,
        diffPixelRatio: null,
        mae: null,
        rmse: null,
        diffImage: null,
        error: null,
      };

      try {
        await page.evaluate(id => window.runSandcastleCase(id), entry.id);
        const testErrors = await page.evaluate(() => window.testErrors);
        expect(testErrors).toEqual([]);
        expect(pageErrors).toEqual([]);

        const visiblePixels = await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => {
          const canvas = document.getElementById('canvas');
          const copy = document.createElement('canvas');
          copy.width = canvas.width;
          copy.height = canvas.height;
          const ctx = copy.getContext('2d');
          ctx.drawImage(canvas, 0, 0);
          const pixels = ctx.getImageData(0, 0, copy.width, copy.height).data;
          let count = 0;
          for (let i = 0; i < pixels.length; i += 4) {
            if (pixels[i] > 25 || pixels[i + 1] > 25 || pixels[i + 2] > 25) count++;
          }
          resolve(count);
        })));
        record.visiblePixels = visiblePixels;
        expect(visiblePixels).toBeGreaterThan(100);

        const viewport = page.locator('#viewport');
        const rustPng = await viewport.screenshot({ animations: 'disabled' });

        if (process.env.REGRESSION_SNAPSHOTS === '1') {
          await expect(viewport).toHaveScreenshot(`${entry.id}.png`, { timeout: 20000, animations: 'disabled' });
        }

        const refPath = path.join(REF_DIR, `${entry.id}.png`);
        if (fs.existsSync(refPath)) {
          record.hasReference = true;
          const refPng = fs.readFileSync(refPath);
          const cmp = await comparePngBuffers(page, refPng, rustPng, { threshold: 0.25, generateDiffImage: true });
          record.diffPixelRatio = cmp.diffPixelRatio;
          record.mae = cmp.mae;
          record.rmse = cmp.rmse;
          if (cmp.compositeBuffer) {
            const diffPath = path.join(DIFF_DIR, `${entry.id}.png`);
            fs.writeFileSync(diffPath, cmp.compositeBuffer);
            record.diffImage = diffPath;
          }
          if (process.env.STRICT_REF_MATCH === '1') {
            expect(cmp.diffPixelRatio).toBeLessThanOrEqual(0.02);
          }
        }
      } catch (err) {
        record.rustStatus = 'failed';
        record.error = err.message;
        // Reload harness on failure so subsequent tests start clean
        await page.goto('/tests/e2e/sandcastle-harness.html').catch(() => {});
        await page.waitForFunction(() => Boolean(window.viewer && window.runSandcastleCase), null, { timeout: 30000 }).catch(() => {});
        throw err;
      } finally {
        results.push(record);
      }
    });
  }
});
