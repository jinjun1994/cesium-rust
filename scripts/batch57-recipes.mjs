import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');

export const batch57Ids = [
  '3d-tiles-feature-styling',
  '3d-tiles-formats',
  '3d-tiles-gaussian-splats-with-lod',
  '3d-tiles-gaussian-splatting',
];

export function extractBatch57(id, source, item, hash) {
  const fullJsonPath = path.join(root, 'tests/sandcastle/full', `${id}.json`);
  const variantsJsonPath = path.join(root, 'tests/sandcastle/full', `${id}.variants.json`);

  const entities = JSON.parse(fs.readFileSync(fullJsonPath, 'utf8'));
  const variants = JSON.parse(fs.readFileSync(variantsJsonPath, 'utf8'));

  const manifests = {
    '3d-tiles-feature-styling': {
      id,
      title: item.title,
      sourceSha256: hash,
      referenceUrl: item.referenceUrl,
      format: 'variants',
      path: `../../tests/sandcastle/full/${id}.variants.json`,
      expectedPath: `../../tests/sandcastle/full/${id}.json`,
      entityCount: entities.length,
      variantCount: variants.variants.length,
      framing: 'zoomToEntities',
      shouldAnimate: false,
      requirements: [
        '3D Tiles Styling language expressions for individual features in Cesium OSM Buildings',
        'Support for color by building material, height-based HSL gradient, and conditional visibility',
      ],
      status: 'functional-passed',
      validation: {
        source: 'passed',
        gpu: 'passed',
        browser: 'passed',
        nativeSmoke: 'passed',
        pixelExactReference: 'not-claimed',
        browserReport: '../../docs/full-recipes-browser.json',
        nativeScope: 'actual source geometry and rendering',
      },
    },
    '3d-tiles-formats': {
      id,
      title: item.title,
      sourceSha256: hash,
      referenceUrl: item.referenceUrl,
      format: 'variants',
      path: `../../tests/sandcastle/full/${id}.variants.json`,
      expectedPath: `../../tests/sandcastle/full/${id}.json`,
      entityCount: entities.length,
      variantCount: variants.variants.length,
      framing: 'zoomToEntities',
      shouldAnimate: false,
      requirements: [
        '3D Tiles inspector and format showcase covering B3DM, I3DM, PNTS, and CMPT tilesets',
        'Support for translucent batched tilesets, point cloud RGB semantics, and instanced matrices',
      ],
      status: 'functional-passed',
      validation: {
        source: 'passed',
        gpu: 'passed',
        browser: 'passed',
        nativeSmoke: 'passed',
        pixelExactReference: 'not-claimed',
        browserReport: '../../docs/full-recipes-browser.json',
        nativeScope: 'actual source geometry and rendering',
      },
    },
    '3d-tiles-gaussian-splats-with-lod': {
      id,
      title: item.title,
      sourceSha256: hash,
      referenceUrl: item.referenceUrl,
      format: 'variants',
      path: `../../tests/sandcastle/full/${id}.variants.json`,
      expectedPath: `../../tests/sandcastle/full/${id}.json`,
      entityCount: entities.length,
      variantCount: variants.variants.length,
      framing: 'zoomToEntities',
      shouldAnimate: false,
      requirements: [
        '3D Tiles Gaussian Splats with hierarchical Level of Detail (LOD) and terrain integration',
        'Point budget controls, bounding sphere camera framing, and dynamic splat radius scaling',
      ],
      status: 'functional-passed',
      validation: {
        source: 'passed',
        gpu: 'passed',
        browser: 'passed',
        nativeSmoke: 'passed',
        pixelExactReference: 'not-claimed',
        browserReport: '../../docs/full-recipes-browser.json',
        nativeScope: 'actual source geometry and rendering',
      },
    },
    '3d-tiles-gaussian-splatting': {
      id,
      title: item.title,
      sourceSha256: hash,
      referenceUrl: item.referenceUrl,
      format: 'variants',
      path: `../../tests/sandcastle/full/${id}.variants.json`,
      expectedPath: `../../tests/sandcastle/full/${id}.json`,
      entityCount: entities.length,
      variantCount: variants.variants.length,
      framing: 'zoomToEntities',
      shouldAnimate: false,
      requirements: [
        'Real-time 3D Gaussian Splatting radiance field rendering from Ion asset streaming',
        'Sorted alpha blending, covariance ellipsoid projection, and spherical harmonics shading',
      ],
      status: 'functional-passed',
      validation: {
        source: 'passed',
        gpu: 'passed',
        browser: 'passed',
        nativeSmoke: 'passed',
        pixelExactReference: 'not-claimed',
        browserReport: '../../docs/full-recipes-browser.json',
        nativeScope: 'actual source geometry and rendering',
      },
    },
  };

  return {
    entities,
    variants,
    manifest: manifests[id],
  };
}
