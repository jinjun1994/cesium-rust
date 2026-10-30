import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');

export const batch79Ids = [
  'material-with-custom-glsl',
  'montreal-point-cloud',
  'moon',
  'msaa',
];

export function extractBatch79(id, source, item, hash) {
  const fullJsonPath = path.join(root, 'tests/sandcastle/full', `${id}.json`);
  const variantsJsonPath = path.join(root, 'tests/sandcastle/full', `${id}.variants.json`);

  const entities = JSON.parse(fs.readFileSync(fullJsonPath, 'utf8'));
  const variants = JSON.parse(fs.readFileSync(variantsJsonPath, 'utf8'));

  const manifests = {
    'material-with-custom-glsl': {
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
        'Cesium.Material with custom Fabric schema CustomBoxShader',
        'Custom MaterialAppearance with vertex and fragment shader sources and uniform time animation',
        'Dynamic Phong shading and sinusoidal RGB color modulation',
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
    'montreal-point-cloud': {
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
        'Montreal Canada 2015 20cm resolution point cloud tiled on Cesium ion',
        'Multiple stored city viewpoints (Overview, Mount Royal, Old Port)',
        'Camera flyTo with orientation direction and up vectors over Cesium World Terrain',
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
    'moon': {
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
        'Ellipsoid.MOON default ellipsoid frame with Moon Terrain 3D Tiles (Ion 2684829)',
        'LROC Mare boundary clamped GeoJSON overlays and Artemis 3 candidate landing sites',
        'Apollo 11 Tranquility Base landmark and crater exploration views',
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
    'msaa': {
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
        'Scene.msaaSamples multisample anti-aliasing configuration (1x, 2x, 4x, 8x)',
        'Scene.msaaSupported WebGPU / WebGL hardware capability guard',
        'Anti-aliased model silhouetted edges against Cesium World Terrain',
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
