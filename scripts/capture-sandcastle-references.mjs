#!/usr/bin/env node
/**
 * Capture original CesiumJS Sandcastle standalone screenshots (960x540)
 * into tests/e2e/sandcastle-reference/<id>.png for same-scene visual comparison.
 *
 * Requires the local CesiumJS dev server running on CESIUM_JS_URL (default http://localhost:8080).
 *
 * Options:
 *   --ids=a,b,c       Comma-separated case IDs (default: all 309 from tests/sandcastle/catalog.json)
 *   --offset=N        Start index in case list (default: 0)
 *   --limit=N         Max number of cases to capture (default: all)
 *   --skip-existing   Skip cases that already have a PNG in tests/e2e/sandcastle-reference/
 *   --keep-ui         Do not hide Cesium viewer widgets / console bar
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const ROOT = path.resolve(__dirname, '..');
const OUT_DIR = path.join(ROOT, 'tests/e2e/sandcastle-reference');
const MANIFEST_PATH = path.join(OUT_DIR, 'manifest.json');
const CATALOG_PATH = path.join(ROOT, 'tests/sandcastle/catalog.json');
const BASE_URL = process.env.CESIUM_JS_URL || 'http://localhost:8080';

function parseArgs(argv) {
  const opts = { ids: null, offset: 0, limit: Infinity, skipExisting: false, keepUi: false, concurrency: 4 };
  for (const arg of argv) {
    if (arg.startsWith('--ids=')) opts.ids = arg.slice(6).split(',').map(s => s.trim()).filter(Boolean);
    else if (arg.startsWith('--offset=')) opts.offset = Number(arg.slice(9)) || 0;
    else if (arg.startsWith('--limit=')) opts.limit = Number(arg.slice(8)) || Infinity;
    else if (arg.startsWith('--concurrency=')) opts.concurrency = Math.max(1, Number(arg.slice(14)) || 1);
    else if (arg === '--skip-existing') opts.skipExisting = true;
    else if (arg === '--keep-ui') opts.keepUi = true;
  }
  return opts;
}

async function hideUiChrome(page) {
  await page.evaluate(() => {
    const s = document.createElement('style');
    s.textContent = `
      [class*="ConsoleMirror"], button[aria-label*="Console"],
      .console-mirror, footer { display: none !important; }
      iframe {
        position: fixed !important;
        inset: 0 !important;
        width: 960px !important;
        height: 540px !important;
        border: 0 !important;
        z-index: 99999 !important;
      }
    `;
    document.head?.appendChild(s);
  }).catch(() => {});

  for (const frame of page.frames()) {
    const u = frame.url();
    if (!u || u === 'about:blank') continue;
    await frame.evaluate(() => {
      const s = document.createElement('style');
      s.textContent = `
        .cesium-viewer-toolbar,
        .cesium-viewer-animationContainer,
        .cesium-viewer-timelineContainer,
        .cesium-viewer-bottom,
        .cesium-viewer-fullscreenContainer,
        .cesium-navigationHelpButton-wrapper { display: none !important; }
      `;
      document.head?.appendChild(s);
    }).catch(() => {});
  }
}

async function main() {
  const opts = parseArgs(process.argv.slice(2));
  fs.mkdirSync(OUT_DIR, { recursive: true });

  const catalog = JSON.parse(fs.readFileSync(CATALOG_PATH, 'utf8'));
  let ids = opts.ids ?? catalog.cases.map(c => c.id);
  ids = ids.slice(opts.offset, Number.isFinite(opts.limit) ? opts.offset + opts.limit : undefined);

  let manifest = { cesiumUrl: BASE_URL, fixedTime: '2023-12-25T14:00:00Z', viewport: { width: 960, height: 540 }, cases: {} };
  if (fs.existsSync(MANIFEST_PATH)) {
    try { manifest = JSON.parse(fs.readFileSync(MANIFEST_PATH, 'utf8')); } catch {}
  }

  const executablePath = process.env.CHROMIUM_PATH || (fs.existsSync('/Applications/Google Chrome.app/Contents/MacOS/Google Chrome') ? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' : undefined);
  const browser = await chromium.launch({ executablePath, args: ['--use-angle=gl'] });

  let nextIdx = 0;
  let completed = 0;

  async function worker() {
    while (true) {
      const idx = nextIdx++;
      if (idx >= ids.length) return;
      const id = ids[idx];
      const outPath = path.join(OUT_DIR, `${id}.png`);
      if (opts.skipExisting && fs.existsSync(outPath)) {
        completed++;
        console.log(`[${completed}/${ids.length}] skip ${id}`);
        continue;
      }

      const context = await browser.newContext({ viewport: { width: 960, height: 540 }, deviceScaleFactor: 1 });
      const page = await context.newPage();
      const consoleErrors = [];
      const pageErrors = [];
      page.on('console', m => {
        if (m.type() === 'error') {
          const text = m.text();
          if (!text.includes('about:blank')) consoleErrors.push(text);
        }
      });
      page.on('pageerror', e => pageErrors.push(e.message));

      let status = 'ok';
      try {
        await page.clock.setFixedTime(new Date('2023-12-25T14:00:00Z'));
        await page.goto(`${BASE_URL}/Apps/Sandcastle2/standalone.html?id=${encodeURIComponent(id)}`, { timeout: 30000 });
        for (let t = 0; t < 4; t++) await page.clock.runFor(1000);
        await page.waitForFunction(() => {
          const iframe = document.querySelector('iframe');
          const text = iframe?.contentDocument?.body?.innerText || '';
          return iframe?.contentDocument?.readyState === 'complete' && !text.includes('Loading...');
        }, { timeout: 15000 }).catch(() => {});
        await page.waitForLoadState('networkidle', { timeout: 20000 }).catch(() => {});
        for (let t = 0; t < 8; t++) await page.clock.runFor(500);
        if (!opts.keepUi) await hideUiChrome(page);
        await page.waitForTimeout(300);
        await page.screenshot({ path: outPath });
        completed++;
        console.log(`[${completed}/${ids.length}] captured ${id} (errors: ${consoleErrors.length + pageErrors.length})`);
      } catch (err) {
        status = 'error';
        consoleErrors.push(err.message);
        completed++;
        console.error(`[${completed}/${ids.length}] FAILED ${id}: ${err.message}`);
      } finally {
        manifest.cases[id] = {
          status,
          consoleErrors,
          pageErrors,
          capturedAt: new Date().toISOString(),
          file: `${id}.png`,
        };
        await context.close();
        if (completed % 20 === 0) {
          fs.writeFileSync(MANIFEST_PATH, JSON.stringify(manifest, null, 2) + '\n');
        }
      }
    }
  }

  try {
    const workerCount = Math.min(opts.concurrency, ids.length || 1);
    await Promise.all(Array.from({ length: workerCount }, () => worker()));
  } finally {
    fs.writeFileSync(MANIFEST_PATH, JSON.stringify(manifest, null, 2) + '\n');
    await browser.close();
  }
}

main().catch(err => {
  console.error(err);
  process.exitCode = 1;
});
