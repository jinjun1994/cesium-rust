import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');

const batch90Data = {
  'voxels': {
    title: 'Voxel Rendering',
    entities: [
      {
        id: 'voxels-procedural-volume-box',
        name: 'Procedural Voxel Box Volume',
        show: true,
        positionDegrees: [-100.0, 40.0, 100000.0],
        shape: {
          type: 'box',
          dimensions: [400000.0, 400000.0, 200000.0],
        },
        fill: true,
        outline: true,
        color: [0.2, 0.75, 0.85, 0.85],
        outlineColor: [0.95, 0.95, 0.98, 1.0],
      },
      {
        id: 'voxels-cylinder-shape-primitive',
        name: 'Voxel Cylinder Shape Primitive',
        show: true,
        positionDegrees: [-100.0, 43.0, 150000.0],
        shape: {
          type: 'cylinder',
          length: 300000.0,
          topRadius: 80000.0,
          bottomRadius: 80000.0,
        },
        fill: true,
        outline: true,
        color: [0.95, 0.45, 0.15, 0.85],
        outlineColor: [1.0, 0.8, 0.6, 1.0],
      },
      {
        id: 'voxels-ellipsoid-kernel-sphere',
        name: 'Voxel Ellipsoid Density Kernel',
        show: true,
        positionDegrees: [-96.0, 40.0, 150000.0],
        shape: {
          type: 'ellipsoid',
          radii: [120000.0, 120000.0, 120000.0],
        },
        fill: true,
        outline: true,
        color: [0.3, 0.85, 0.35, 0.9],
        outlineColor: [0.7, 1.0, 0.7, 1.0],
      },
      {
        id: 'voxels-bounding-footprint-rect',
        name: 'Voxel Bounding Footprint Rectangle',
        show: true,
        positionDegrees: null,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [-104.0, 37.0, -94.0, 43.0],
          height: 10000.0,
          extrudedHeight: 50000.0,
        },
        fill: true,
        outline: true,
        color: [0.25, 0.35, 0.55, 0.55],
        outlineColor: [0.6, 0.8, 1.0, 1.0],
      },
    ],
  },
  'voxels-in-3d-tiles': {
    title: 'Voxels in 3D Tiles',
    entities: [
      {
        id: 'voxels3d-tileset-bounding-box',
        name: 'Voxel 3D Tileset Bounding Box',
        show: true,
        positionDegrees: [-105.0, 39.5, 50000.0],
        shape: {
          type: 'box',
          dimensions: [250000.0, 250000.0, 100000.0],
        },
        fill: true,
        outline: true,
        color: [0.15, 0.7, 0.9, 0.9],
        outlineColor: [0.8, 0.95, 1.0, 1.0],
      },
      {
        id: 'voxels3d-octree-node-cylinder',
        name: 'Octree Node Subdivision Cylinder',
        show: true,
        positionDegrees: [-105.0, 41.5, 80000.0],
        shape: {
          type: 'cylinder',
          length: 150000.0,
          topRadius: 60000.0,
          bottomRadius: 60000.0,
        },
        fill: true,
        outline: true,
        color: [0.95, 0.65, 0.1, 0.85],
        outlineColor: [1.0, 0.85, 0.5, 1.0],
      },
      {
        id: 'voxels3d-sample-point-sphere',
        name: 'Voxel Sample Evaluation Sphere',
        show: true,
        positionDegrees: [-102.5, 39.5, 80000.0],
        shape: {
          type: 'ellipsoid',
          radii: [75000.0, 75000.0, 75000.0],
        },
        fill: true,
        outline: true,
        color: [0.85, 0.2, 0.7, 0.9],
        outlineColor: [1.0, 0.6, 0.9, 1.0],
      },
      {
        id: 'voxels3d-dataset-coverage-rect',
        name: 'Voxel Dataset Coverage Boundary',
        show: true,
        positionDegrees: null,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [-107.0, 38.0, -101.0, 42.0],
          height: 5000.0,
          extrudedHeight: 30000.0,
        },
        fill: true,
        outline: true,
        color: [0.2, 0.3, 0.6, 0.5],
        outlineColor: [0.5, 0.7, 1.0, 1.0],
      },
    ],
  },
  'washington-dc-2017': {
    title: 'Washington DC 2017',
    entities: [
      {
        id: 'dc2017-capitol-dome-cylinder',
        name: 'US Capitol Dome Rotunda Cylinder',
        show: true,
        positionDegrees: [-77.009, 38.890, 80.0],
        shape: {
          type: 'cylinder',
          length: 80.0,
          topRadius: 35.0,
          bottomRadius: 40.0,
        },
        fill: true,
        outline: true,
        color: [0.95, 0.95, 0.92, 0.95],
        outlineColor: [0.8, 0.8, 0.75, 1.0],
      },
      {
        id: 'dc2017-washington-monument-box',
        name: 'Washington Monument Obelisk Box',
        show: true,
        positionDegrees: [-77.035, 38.889, 85.0],
        shape: {
          type: 'box',
          dimensions: [20.0, 20.0, 170.0],
        },
        fill: true,
        outline: true,
        color: [0.92, 0.9, 0.85, 0.95],
        outlineColor: [0.75, 0.7, 0.65, 1.0],
      },
      {
        id: 'dc2017-lincoln-memorial-sphere',
        name: 'Lincoln Memorial Landmark Sphere',
        show: true,
        positionDegrees: [-77.050, 38.889, 40.0],
        shape: {
          type: 'ellipsoid',
          radii: [25.0, 25.0, 25.0],
        },
        fill: true,
        outline: true,
        color: [0.95, 0.75, 0.2, 0.9],
        outlineColor: [1.0, 0.9, 0.5, 1.0],
      },
      {
        id: 'dc2017-national-mall-rect',
        name: 'National Mall High-Res Imagery Extent',
        show: true,
        positionDegrees: null,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [-77.060, 38.882, -77.000, 38.898],
          height: 10.0,
          extrudedHeight: 50.0,
        },
        fill: true,
        outline: true,
        color: [0.25, 0.6, 0.3, 0.6],
        outlineColor: [0.4, 0.85, 0.5, 1.0],
      },
    ],
  },
  'web-map-service-wms': {
    title: 'Web Map Service (WMS)',
    entities: [
      {
        id: 'wms-offshore-lease-block-box',
        name: 'Offshore Petroleum Lease Block Box',
        show: true,
        positionDegrees: [125.0, -20.0, 50000.0],
        shape: {
          type: 'box',
          dimensions: [400000.0, 300000.0, 100000.0],
        },
        fill: true,
        outline: true,
        color: [0.15, 0.65, 0.7, 0.85],
        outlineColor: [0.7, 0.95, 1.0, 1.0],
      },
      {
        id: 'wms-drilling-platform-cylinder',
        name: 'Offshore Production Platform Cylinder',
        show: true,
        positionDegrees: [125.0, -17.0, 80000.0],
        shape: {
          type: 'cylinder',
          length: 160000.0,
          topRadius: 50000.0,
          bottomRadius: 50000.0,
        },
        fill: true,
        outline: true,
        color: [0.95, 0.5, 0.1, 0.9],
        outlineColor: [1.0, 0.75, 0.4, 1.0],
      },
      {
        id: 'wms-survey-buoy-sphere',
        name: 'Oceanographic Survey Buoy Sphere',
        show: true,
        positionDegrees: [130.0, -20.0, 60000.0],
        shape: {
          type: 'ellipsoid',
          radii: [70000.0, 70000.0, 70000.0],
        },
        fill: true,
        outline: true,
        color: [0.2, 0.8, 0.95, 0.9],
        outlineColor: [0.6, 0.95, 1.0, 1.0],
      },
      {
        id: 'wms-australia-extent-rect',
        name: 'Australia Continental WMS Service Extent',
        show: true,
        positionDegrees: null,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [122.0, -21.0, 128.0, -17.0],
          height: 1000.0,
          extrudedHeight: 20000.0,
        },
        fill: true,
        outline: true,
        color: [0.2, 0.4, 0.65, 0.5],
        outlineColor: [0.5, 0.75, 1.0, 1.0],
      },
    ],
  },
};

for (const [id, data] of Object.entries(batch90Data)) {
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
