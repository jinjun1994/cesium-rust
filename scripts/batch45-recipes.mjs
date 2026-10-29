import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');

export const batch45Ids = [
  'clustering',
  'geojson-simplestyle',
  'geojson-and-topojson',
  'kml',
];

export function extractBatch45(id, source, item, hash) {
  const fullJsonPath = path.join(root, 'tests/sandcastle/full', `${id}.json`);
  const variantsJsonPath = path.join(root, 'tests/sandcastle/full', `${id}.variants.json`);

  const entities = JSON.parse(fs.readFileSync(fullJsonPath, 'utf8'));
  const variants = JSON.parse(fs.readFileSync(variantsJsonPath, 'utf8'));

  const manifests = {
    'clustering': {
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
        'Global facilities placemark clustering with dynamic count badge and distance threshold',
        'EntityCluster support with billboard and label clustering modes',
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
    'geojson-simplestyle': {
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
        'GeoJSON simplestyle standard specification feature loading',
        'Marker symbols, route stroke/strokeWidth, and district fill/fillOpacity',
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
    'geojson-and-topojson': {
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
        'TopoJSON / GeoJSON US states polygon vector boundaries',
        'Default styling, basic styling (hotpink stroke/pink fill), population extrusion, and random coloring',
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
    'kml': {
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
        'KML / KMZ data source features across multiple geographic datasets',
        'Global facilities, GDP per capita extruded polygons, and gx bike tour track',
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
