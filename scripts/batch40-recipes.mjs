import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');

export const batch40Ids = [
  'interpolation',
  'manually-controlled-animation',
  'localtofixedframe',
  'pointprimitives-dev',
];

export function extractBatch40(id, source, item, hash) {
  const fullJsonPath = path.join(root, 'tests/sandcastle/full', `${id}.json`);
  const variantsJsonPath = path.join(root, 'tests/sandcastle/full', `${id}.variants.json`);

  const entities = JSON.parse(fs.readFileSync(fullJsonPath, 'utf8'));
  const variants = JSON.parse(fs.readFileSync(variantsJsonPath, 'utf8'));

  const manifests = {
    interpolation: {
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
      shouldAnimate: true,
      clock: {
        startTime: 1427299200,
        stopTime: 1427299560,
        currentTime: 1427299200,
        multiplier: 10,
        range: 'LOOP_STOP',
      },
      requirements: [
        'SampledPositionProperty with circular waypoints at 45 degree increments',
        'Yellow point entities marking each flight waypoint with transparent fill and outline',
        'Cesium_Air 3D model with VelocityOrientationProperty and minimumPixelSize: 64',
        'PolylineGlowMaterialProperty flight path with glowPower: 0.1 and yellow glow',
        'Simulation clock bounds (2015-03-25 16:00 UTC, 360s duration, 10x multiplier)',
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
    'manually-controlled-animation': {
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
      shouldAnimate: true,
      clock: {
        startTime: 1544626800,
        stopTime: 1544626830,
        currentTime: 1544626800,
        multiplier: 1,
        range: 'LOOP_STOP',
      },
      requirements: [
        'CesiumMan glTF model moving along non-linear accelerated sampled path',
        'Dynamic speed label updated in real-time in km/hr with background box',
        'Camera trackedEntity orientation following position and velocity vector',
        'viewFrom offset: [-30.0, -10.0, 10.0] for close behind tracking',
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
    localtofixedframe: {
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
        '10 Cesium_Air 3D models with 10 local frame converters (NED, NEU, NWU, NWD, ENU, END, SWU, SUW, UEN, DES)',
        'headingPitchRollToFixedFrame matrix computation per model instance',
        '10 orientation labels with Helvetica 18px text and outline',
        'HeadingPitchRoll arrow key controls for pitch, roll, and heading adjustment',
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
    'pointprimitives-dev': {
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
        'PointPrimitiveCollection supporting single, multiple, and clustered point primitives',
        'Custom properties (pixelSize, color, outlineWidth, outlineColor)',
        'scaleByDistance curve using NearFarScalar(150, 2.0, 15000000, 0.5)',
        'translucencyByDistance curve using NearFarScalar(150, 1.0, 15000000, 0.2)',
        'eastNorthUpToFixedFrame reference frame with center, east, north, and up points',
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
