/**
 * Compare two PNG buffers inside a Playwright Page using 2D Canvas ImageData.
 * Returns pixel diff metrics and an optional 3-panel (ref | actual | diff) PNG buffer.
 */
async function comparePngBuffers(page, refPngBuffer, actualPngBuffer, options = {}) {
  const channelThreshold = Math.round((options.threshold ?? 0.25) * 255);
  const generateDiffImage = options.generateDiffImage !== false;

  const refB64 = refPngBuffer.toString('base64');
  const actualB64 = actualPngBuffer.toString('base64');

  const result = await page.evaluate(
    async ({ refB64, actualB64, channelThreshold, generateDiffImage }) => {
      async function loadBitmap(b64) {
        const res = await fetch(`data:image/png;base64,${b64}`);
        const blob = await res.blob();
        return createImageBitmap(blob);
      }

      const [refImg, actImg] = await Promise.all([loadBitmap(refB64), loadBitmap(actualB64)]);
      const width = Math.min(refImg.width, actImg.width);
      const height = Math.min(refImg.height, actImg.height);
      const sizeMismatch = refImg.width !== actImg.width || refImg.height !== actImg.height;

      const cRef = new OffscreenCanvas(width, height);
      const ctxRef = cRef.getContext('2d');
      ctxRef.drawImage(refImg, 0, 0, width, height);
      const refData = ctxRef.getImageData(0, 0, width, height).data;

      const cAct = new OffscreenCanvas(width, height);
      const ctxAct = cAct.getContext('2d');
      ctxAct.drawImage(actImg, 0, 0, width, height);
      const actData = ctxAct.getImageData(0, 0, width, height).data;

      let diffPixels = 0;
      let sumAbs = 0;
      let sumSq = 0;
      let rustNonBlackPixels = 0;
      const totalPixels = width * height;

      let diffData = null;
      let cDiff = null;
      let ctxDiff = null;
      if (generateDiffImage) {
        cDiff = new OffscreenCanvas(width, height);
        ctxDiff = cDiff.getContext('2d');
        diffData = ctxDiff.createImageData(width, height);
      }

      for (let i = 0; i < refData.length; i += 4) {
        const dr = Math.abs(refData[i] - actData[i]);
        const dg = Math.abs(refData[i + 1] - actData[i + 1]);
        const db = Math.abs(refData[i + 2] - actData[i + 2]);
        const maxD = Math.max(dr, dg, db);

        if (actData[i] > 25 || actData[i + 1] > 25 || actData[i + 2] > 25) {
          rustNonBlackPixels++;
        }

        if (maxD > channelThreshold) {
          diffPixels++;
        }
        sumAbs += (dr + dg + db) / (3 * 255);
        sumSq += (dr * dr + dg * dg + db * db) / (3 * 255 * 255);

        if (diffData) {
          if (maxD > channelThreshold) {
            diffData.data[i] = 255;
            diffData.data[i + 1] = Math.max(0, 255 - maxD);
            diffData.data[i + 2] = 0;
            diffData.data[i + 3] = 255;
          } else {
            const gray = Math.round((actData[i] + actData[i + 1] + actData[i + 2]) / 6);
            diffData.data[i] = gray;
            diffData.data[i + 1] = gray;
            diffData.data[i + 2] = gray;
            diffData.data[i + 3] = 255;
          }
        }
      }

      let compositeB64 = null;
      if (generateDiffImage) {
        ctxDiff.putImageData(diffData, 0, 0);
        const composite = new OffscreenCanvas(width * 3, height);
        const ctxComp = composite.getContext('2d');
        ctxComp.drawImage(cRef, 0, 0);
        ctxComp.drawImage(cAct, width, 0);
        ctxComp.drawImage(cDiff, width * 2, 0);
        const blob = await composite.convertToBlob({ type: 'image/png' });
        const buf = await blob.arrayBuffer();
        const bytes = new Uint8Array(buf);
        let binary = '';
        for (let i = 0; i < bytes.length; i += 8192) {
          binary += String.fromCharCode(...bytes.subarray(i, i + 8192));
        }
        compositeB64 = btoa(binary);
      }

      return {
        width,
        height,
        sizeMismatch,
        totalPixels,
        diffPixels,
        diffPixelRatio: diffPixels / totalPixels,
        mae: sumAbs / totalPixels,
        rmse: Math.sqrt(sumSq / totalPixels),
        rustNonBlackPixels,
        compositeB64,
      };
    },
    { refB64, actualB64, channelThreshold, generateDiffImage }
  );

  return {
    ...result,
    compositeBuffer: result.compositeB64 ? Buffer.from(result.compositeB64, 'base64') : null,
  };
}

module.exports = { comparePngBuffers };
