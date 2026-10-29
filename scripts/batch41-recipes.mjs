import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');

export const batch41Ids = [
  'cartographic-limit-rectangle',
  'clamp-entities-to-ground',
  'billboards-instancing-dev',
  'geometry-and-appearances',
];

export function extractBatch41(id, source, item, hash) {
  const fullJsonPath = path.join(root, 'tests/sandcastle/full', `${id}.json`);
  const variantsJsonPath = path.join(root, 'tests/sandcastle/full', `${id}.variants.json`);

  const entities = JSON.parse(fs.readFileSync(fullJsonPath, 'utf8'));
  const variants = JSON.parse(fs.readFileSync(variantsJsonPath, 'utf8'));

  const manifests = {
    'cartographic-limit-rectangle': {
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
        'Coffee Belt cartographic limit rectangle [-180, -23.43687, 180, 23.43687]',
        '10 outline bounding rectangle entities from 0m to 45000m altitude',
        'Toggle controls for limitEnabled and showBounds',
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
    'clamp-entities-to-ground': {
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
        'Clamping entities to ground with HeightReference.CLAMP_TO_GROUND',
        '8 toolbar entity types: Point with Label, Billboard, Corridor, Polygon, Textured Polygon, Rectangle, Model, Polyline',
        'depthTestAgainstTerrain enabled and camera tracking',
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
    'billboards-instancing-dev': {
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
        'BillboardCollection grid across Seneca area with instancing',
        'Configurable counts from 4 up to 100489 billboards',
        'Static vs animated billboards and scale variations',
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
    'geometry-and-appearances': {
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
        '58 diverse geometric entities including Stripe and Checkerboard materials',
        'Extruded polygons, rectangles, ellipses, cylinders, and polyline volumes',
        'Stacked transparent and opaque boxes, ellipsoids, and spheres',
        'Deep sky blue polyline glow and varying height walls',
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
