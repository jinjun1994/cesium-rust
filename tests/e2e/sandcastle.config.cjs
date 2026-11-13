const fs = require('node:fs');
const path = require('node:path');
const { defineConfig } = require('playwright/test');

const macChrome = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const executablePath = process.env.CHROMIUM_PATH || (fs.existsSync(macChrome) ? macChrome : undefined);

module.exports = defineConfig({
  testDir: __dirname,
  testMatch: 'sandcastle-compare.spec.cjs',
  outputDir: '../../target/e2e-results/sandcastle',
  workers: 1,
  timeout: 120000,
  updateSnapshots: 'none',
  use: {
    baseURL: 'http://127.0.0.1:18766',
    viewport: { width: 960, height: 540 },
    deviceScaleFactor: 1,
    trace: 'retain-on-failure',
    launchOptions: {
      executablePath,
      args: ['--enable-unsafe-webgpu', '--enable-unsafe-swiftshader'],
    },
  },
  expect: {
    toHaveScreenshot: {
      threshold: 0.25,
      maxDiffPixelRatio: 0.02,
    },
  },
  webServer: {
    command: 'python3 scripts/start-test-ui.py --port 18766 --no-browser',
    cwd: path.resolve(__dirname, '../..'),
    url: 'http://127.0.0.1:18766/',
    reuseExistingServer: false,
    timeout: 30000,
  },
});
