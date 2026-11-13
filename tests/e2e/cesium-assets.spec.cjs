const {test, expect} = require('playwright/test');

// Upstream tilesets and payloads are copied verbatim; see fixtures/README.md.
for (const fixture of ['batched', 'point-cloud']) {
  test(`Cesium ${fixture} tileset loads and renders`, async ({page}) => {
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    // Keep tests offline and expose unexpected external dependencies.
    await page.route('**/*', route => {
      const url = new URL(route.request().url());
      return url.hostname === '127.0.0.1' ? route.continue() : route.abort();
    });
    await page.goto('/tests/e2e/harness.html');
    await page.waitForFunction(() => window.viewer, null, {timeout: 30000});
    await page.evaluate(path => window.viewer.load(path), `/tests/e2e/fixtures/${fixture}/tileset.json`);
    await page.waitForFunction(() => window.viewer.stats.includes('resident 1'), null, {timeout: 30000});
    expect(await page.evaluate(() => window.viewer.diagnostics.tileCompleted)).toBe(1);
    expect(await page.evaluate(() => window.testErrors)).toEqual([]);
    expect(errors).toEqual([]);
    await page.evaluate(() => window.viewer.zoom(15));
    const canvas = page.locator('canvas').first();
    // Check the rendered content independently: a blank snapshot must not pass.
    const visiblePixels = await canvas.evaluate(canvas => new Promise(resolve => requestAnimationFrame(() => {
      const copy = document.createElement('canvas');
      copy.width = canvas.width; copy.height = canvas.height;
      const context = copy.getContext('2d'); context.drawImage(canvas, 0, 0);
      const pixels = context.getImageData(0, 0, copy.width, copy.height).data;
      let count = 0;
      for (let i = 0; i < pixels.length; i += 4) {
        if (pixels[i] > 100 && pixels[i + 1] > 100 && pixels[i + 2] > 100) count++;
      }
      resolve(count);
    })));
    expect(visiblePixels).toBeGreaterThan(100);
    await expect(canvas).toHaveScreenshot(`${fixture}.png`, {timeout: 20000, animations: 'disabled'});
    if (fixture === 'batched') {
      await canvas.click({position: {x: 390, y: 205}});
      await page.waitForFunction(() => window.lastPick?.properties?.Height !== undefined);
      const properties = await page.evaluate(() => window.lastPick.properties);
      expect(properties.Height).toBeGreaterThan(0);
      expect(Number.isFinite(properties.Longitude)).toBe(true);
    }
  });
}
