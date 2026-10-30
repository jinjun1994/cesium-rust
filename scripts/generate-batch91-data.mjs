import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');

const batch91Data = {
  'web-map-tile-service-picking': {
    title: 'Web Map Tile Service Picking',
    entities: [
      {
        id: 'wmts-ocean-model-box',
        name: 'North-West Shelf Oceanographic Analysis Box',
        show: true,
        positionDegrees: [2.0, 56.0, 10000.0],
        shape: {
          type: 'box',
          dimensions: [300000.0, 300000.0, 20000.0],
        },
        fill: true,
        outline: true,
        color: [0.15, 0.75, 0.85, 0.85],
        outlineColor: [0.75, 0.95, 1.0, 1.0],
      },
      {
        id: 'wmts-current-sensor-cylinder',
        name: 'Acoustic Doppler Current Profiler (ADCP) Mooring Cylinder',
        show: true,
        positionDegrees: [2.5, 56.5, 15000.0],
        shape: {
          type: 'cylinder',
          length: 30000.0,
          topRadius: 40000.0,
          bottomRadius: 40000.0,
        },
        fill: true,
        outline: true,
        color: [0.95, 0.65, 0.1, 0.9],
        outlineColor: [1.0, 0.85, 0.4, 1.0],
      },
      {
        id: 'wmts-feature-pick-point-sphere',
        name: 'Hydrodynamic Sample Station Sphere',
        show: true,
        positionDegrees: [1.5, 55.5, 12000.0],
        shape: {
          type: 'ellipsoid',
          radii: [25000.0, 25000.0, 25000.0],
        },
        fill: true,
        outline: true,
        color: [0.3, 0.9, 0.35, 0.9],
        outlineColor: [0.7, 1.0, 0.75, 1.0],
      },
      {
        id: 'wmts-shelf-coverage-rect',
        name: 'North Sea WMTS Tile Coverage Rectangle',
        show: true,
        positionDegrees: null,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [0.0, 54.0, 4.0, 58.0],
          height: 500.0,
          extrudedHeight: 5000.0,
        },
        fill: true,
        outline: true,
        color: [0.2, 0.4, 0.7, 0.5],
        outlineColor: [0.5, 0.7, 1.0, 1.0],
      },
    ],
  },
  'web-map-tile-service-with-time': {
    title: 'Web Map Tile Service with Time',
    entities: [
      {
        id: 'wmts-time-earth-observation-box',
        name: 'NASA GIBS Earth Observation Bounding Box',
        show: true,
        positionDegrees: [-95.0, 38.0, 50000.0],
        shape: {
          type: 'box',
          dimensions: [400000.0, 400000.0, 100000.0],
        },
        fill: true,
        outline: true,
        color: [0.95, 0.75, 0.15, 0.85],
        outlineColor: [1.0, 0.9, 0.5, 1.0],
      },
      {
        id: 'wmts-time-orbit-groundtrack-cylinder',
        name: 'Terra Satellite Ground Track Column Cylinder',
        show: true,
        positionDegrees: [-95.0, 41.0, 80000.0],
        shape: {
          type: 'cylinder',
          length: 160000.0,
          topRadius: 50000.0,
          bottomRadius: 50000.0,
        },
        fill: true,
        outline: true,
        color: [0.2, 0.6, 0.95, 0.9],
        outlineColor: [0.65, 0.85, 1.0, 1.0],
      },
      {
        id: 'wmts-time-daily-mosaic-sphere',
        name: 'Daily MODIS Composite Centroid Sphere',
        show: true,
        positionDegrees: [-91.0, 38.0, 75000.0],
        shape: {
          type: 'ellipsoid',
          radii: [60000.0, 60000.0, 60000.0],
        },
        fill: true,
        outline: true,
        color: [0.25, 0.85, 0.5, 0.9],
        outlineColor: [0.7, 1.0, 0.8, 1.0],
      },
      {
        id: 'wmts-time-continental-mosaic-rect',
        name: 'MODIS Reflectance Tile Grid Extent Rectangle',
        show: true,
        positionDegrees: null,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [-98.0, 35.0, -92.0, 41.0],
          height: 1000.0,
          extrudedHeight: 20000.0,
        },
        fill: true,
        outline: true,
        color: [0.85, 0.9, 0.95, 0.55],
        outlineColor: [0.95, 0.98, 1.0, 1.0],
      },
    ],
  },
};

for (const [id, data] of Object.entries(batch91Data)) {
  const fullJsonPath = path.join(root, 'tests/sandcastle/full', `${id}.json`);
  const variantsJsonPath = path.join(root, 'tests/sandcastle/full', `${id}.variants.json`);

  fs.writeFileSync(fullJsonPath, JSON.stringify(data.entities, null, 2) + '\n');

  const variants = {
    initial: 0,
    variants: [
      {
        label: `${data.title} - Variant 1 (Default Focus)`,
        entities: data.entities,
      },
      {
        label: `${data.title} - Variant 2 (Alternative Parameters)`,
        entities: data.entities.slice(0, 3),
      },
      {
        label: `${data.title} - Variant 3 (Minimal Subset)`,
        entities: [data.entities[0], data.entities[1]],
      },
      {
        label: `${data.title} - Variant 4 (Full Inspection Overview)`,
        entities: data.entities,
      },
    ],
  };

  fs.writeFileSync(variantsJsonPath, JSON.stringify(variants, null, 2) + '\n');
  console.log(`Generated ${id}.json and ${id}.variants.json`);
}
