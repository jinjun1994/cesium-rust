const {defineConfig} = require('playwright/test');

module.exports = defineConfig({
  testDir: __dirname,
  testMatch: 'cesium-assets.spec.cjs',
  outputDir: '../../target/e2e-results',
  workers: 1,
  timeout: 60000,
  updateSnapshots: 'none',
  use: {
    baseURL: 'http://127.0.0.1:18765',
    viewport: {width: 1280, height: 800},
    deviceScaleFactor: 1,
    trace: 'retain-on-failure',
    launchOptions: {
      executablePath: process.env.CHROMIUM_PATH || undefined,
      args: ['--enable-unsafe-webgpu', '--enable-unsafe-swiftshader'],
    },
  },
  expect: {toHaveScreenshot: {threshold: 0.2, maxDiffPixelRatio: 0.001}},
  webServer: {
    command: 'python3 scripts/start-test-ui.py --port 18765 --no-browser',
    cwd: require('node:path').resolve(__dirname, '../..'),
    url: 'http://127.0.0.1:18765/',
    reuseExistingServer: false,
    timeout: 30000,
  },
});
