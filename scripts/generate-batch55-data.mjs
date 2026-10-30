import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. 3D Native Vector Data
function generate3dNativeVector() {
  const datumCylinder = {
    id: 'native-vector-datum-cylinder',
    name: '3D Vector Network Spatial Reference Cylinder',
    show: true,
    positionDegrees: [-105.27, 40.01, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.0, 0.95, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.3, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      vectorTilesetId: 5135764,
      vectorStyleColor: 'cyan',
      vectorLineWidth: 3,
    },
  };

  const corridorBox = {
    id: 'native-vector-corridor-box',
    name: '3D Vector Corridor Extruded Geometry Box',
    show: true,
    positionDegrees: [-105.26, 40.02, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.85, 0.45, 0.15, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.65, 0.35, 1.0],
    outlineWidth: 1.0,
    properties: {
      baseTilesetId: 5135767,
      featureTopology: 'Polyline Corridor',
    },
  };

  const sensorDome = {
    id: 'native-vector-sensor-dome',
    name: 'Vector Feature Semantic Envelope Hemisphere',
    show: true,
    positionDegrees: [-105.28, 40.00, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.35, 0.75, 0.95, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      terrainProvider: 'Cesium World Terrain',
      vectorFormat: 'Point, Polyline, Polygon in 3D Tiles',
    },
  };

  const boundaryRect = {
    id: 'native-vector-boundary-rect',
    name: '3D Native Vector Geographic Boundary Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-105.31, 39.98, -105.24, 40.04],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.2, 0.8, 0.5, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.4, 1.0, 0.7, 0.9],
    outlineWidth: 1.0,
    properties: {
      ionDefaultStyleActive: true,
      coordinateSystem: 'WGS84 Ellipsoidal',
    },
  };

  const entities = [datumCylinder, corridorBox, sensorDome, boundaryRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Cyan Polylines with Linewidth 3 & Default Style',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'High-Contrast Vector Styling (Orange Polylines & Fill)',
        entities: entities.map((e) => {
          if (e.id === 'native-vector-corridor-box') {
            return { ...e, color: [0.95, 0.55, 0.1, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Vector Geometry Highlight (Selected Features)',
        entities: entities.map((e) => {
          if (e.id === 'native-vector-datum-cylinder') {
            return { ...e, color: [1.0, 0.95, 0.2, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Base Model Only (Vectors Suppressed)',
        entities: entities.map((e) => {
          if (e.id === 'native-vector-boundary-rect') {
            return { ...e, color: [0.5, 0.5, 0.6, 0.7] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 2. Offset 3D Tileset Height
function generateAdjustHeight() {
  const originDatum = {
    id: 'adjust-height-origin-datum',
    name: '3D Tileset Elevation Ground Benchmark Datum',
    show: true,
    positionDegrees: [-75.612, 40.042, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.85, 0.35, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.55, 0.45, 1.0],
    outlineWidth: 1.0,
    properties: {
      sampleDataTileset: 'Tilesets/Tileset/tileset.json',
      elevationOffsetMeters: 0.0,
    },
  };

  const translatedBox = {
    id: 'adjust-height-translated-box',
    name: 'Dynamic Height Translation Testing Massing Box',
    show: true,
    positionDegrees: [-75.605, 40.048, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.25, 0.65, 0.95, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      knockoutModelHeight: 0.0,
      modelMatrixTransformation: 'Matrix4.fromTranslation',
    },
  };

  const envelopeSphere = {
    id: 'adjust-height-envelope-sphere',
    name: 'Tileset Bounding Sphere Envelope Hemisphere',
    show: true,
    positionDegrees: [-75.618, 40.035, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.45, 0.85, 0.35, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      boundingSphereRadius: 104.5,
      cameraHeadingPitchRange: [0.0, -0.5, 209.0],
    },
  };

  const boundingRect = {
    id: 'adjust-height-bounding-rect',
    name: 'Exton Tileset Geometric Extent Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.63, 40.03, -75.59, 40.06],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.65, 0.35, 0.85, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.85, 0.55, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: {
      depthTestAgainstTerrain: true,
      shadowsEnabled: true,
    },
  };

  const entities = [originDatum, translatedBox, envelopeSphere, boundingRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Zero Height Offset (Ground Level Elevation)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Elevated Offset +100m (Floating Massing)',
        entities: entities.map((e) => {
          if (e.id === 'adjust-height-translated-box') {
            return { ...e, color: [0.95, 0.85, 0.25, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Subsurface Offset -50m (Excavation Alignment)',
        entities: entities.map((e) => {
          if (e.id === 'adjust-height-origin-datum') {
            return { ...e, color: [0.3, 0.8, 0.9, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Depth Test Against Terrain Enabled (Strict Occlusion)',
        entities: entities.map((e) => {
          if (e.id === 'adjust-height-bounding-rect') {
            return { ...e, color: [0.2, 0.6, 0.85, 0.75] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 3. 3D Tiles Batch Table Hierarchy
function generateBatchTableHierarchy() {
  const zonePivot = {
    id: 'hierarchy-zone-pivot-cylinder',
    name: 'Cadastral Zone & Building Hierarchy Spatial Pivot',
    show: true,
    positionDegrees: [-75.612, 40.042, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [1.0, 0.65, 0.0, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.85, 0.3, 1.0],
    outlineWidth: 1.0,
    properties: {
      sampleDataTileset: 'Hierarchy/BatchTableHierarchy/tileset.json',
      hierarchyClass: 'zone',
      zoneName: 'zone0',
    },
  };

  const buildingBox = {
    id: 'hierarchy-building-massing-box',
    name: 'Multi-Level Building Massing Structural Box',
    show: true,
    positionDegrees: [-75.605, 40.048, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.65, 0.25, 0.85, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.85, 0.45, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      hierarchyClass: 'building',
      buildingName: 'building0',
      buildingArea: 450.0,
    },
  };

  const architecturalDome = {
    id: 'hierarchy-architectural-dome',
    name: 'Roof & Wall Component Semantic Enclosure Dome',
    show: true,
    positionDegrees: [-75.618, 40.035, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.25, 0.85, 0.55, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      hierarchyChildClasses: 'roof, wall, door, doorknob',
      doorWidth: 1.2,
      doorknobSize: 0.08,
    },
  };

  const cadastralRect = {
    id: 'hierarchy-cadastral-boundary-rect',
    name: 'Batch Table Cadastral Zone Boundary Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.63, 40.03, -75.59, 40.06],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.35, 0.55, 0.95, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.55, 0.75, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: {
      julianDate: 2457522.154792,
      styleCondition: "isExactClass('door') ? color('orange') : color('white')",
    },
  };

  const entities = [zonePivot, buildingBox, architecturalDome, cadastralRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Color all doors (Orange Doors, White Buildings)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Color all features derived from door (Class Inheritance)',
        entities: entities.map((e) => {
          if (e.id === 'hierarchy-architectural-dome') {
            return { ...e, color: [1.0, 0.5, 0.1, 0.9] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Color by building (Zone Hierarchy Partitioning)',
        entities: entities.map((e) => {
          if (e.id === 'hierarchy-building-massing-box') {
            return { ...e, color: [0.9, 0.2, 0.2, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Color features by class name & Style by height',
        entities: entities.map((e) => {
          if (e.id === 'hierarchy-zone-pivot-cylinder') {
            return { ...e, color: [0.8, 0.2, 0.8, 0.95] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 4. BIM Design Model with 3D Tiles
function generateBim() {
  const reactorStack = {
    id: 'bim-reactor-stack-cylinder',
    name: 'Power Plant Cooling & Reactor Stack Benchmark Cylinder',
    show: true,
    positionDegrees: [-75.615, 40.045, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.85, 0.85, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      ionAssetId: 2464651,
      modelProvider: 'Bentley Systems',
      plantFacility: 'Thermal Power Plant',
    },
  };

  const generatorHall = {
    id: 'bim-generator-hall-box',
    name: 'Turbine Generator Hall Structural Massing Box',
    show: true,
    positionDegrees: [-75.608, 40.052, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.35, 0.65, 0.85, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.55, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      globalElementId: 112001,
      structuralClass: 'Generator Enclosure',
    },
  };

  const containmentVessel = {
    id: 'bim-containment-vessel-sphere',
    name: 'Pressure Containment Vessel Hemisphere',
    show: true,
    positionDegrees: [-75.622, 40.038, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.95, 0.65, 0.25, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      colorBlendMode: 'REPLACE',
      hiddenElementsCount: 34,
    },
  };

  const powerPlantRect = {
    id: 'bim-power-plant-boundary-rect',
    name: 'Power Plant Industrial Facility Security Perimeter',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.635, 40.032, -75.595, 40.065],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.45, 0.75, 0.35, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.95, 0.55, 0.9],
    outlineWidth: 1.0,
    properties: {
      cameraHeadingPitchRange: [0.5, -0.2, 4.0],
      simulationDate: '2022-08-01T00:00:00Z',
    },
  };

  const entities = [reactorStack, generatorHall, containmentVessel, powerPlantRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Power Plant Overview with Hidden Elements Filtered',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Per-Feature Selection Active (Yellow Hover & Click Highlight)',
        entities: entities.map((e) => {
          if (e.id === 'bim-generator-hall-box') {
            return { ...e, color: [1.0, 0.9, 0.1, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'All Internal Elements Visible (Unfiltered Structural Piping)',
        entities: entities.map((e) => {
          if (e.id === 'bim-containment-vessel-sphere') {
            return { ...e, color: [0.85, 0.35, 0.25, 0.9] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Color Blend Mode Replace (Strict Photorealistic PBR Rendering)',
        entities: entities.map((e) => {
          if (e.id === 'bim-reactor-stack-cylinder') {
            return { ...e, color: [0.2, 0.6, 0.9, 0.95] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// Write helper
function writeData(id, data) {
  const jsonPath = path.join(outDir, `${id}.json`);
  const variantsPath = path.join(outDir, `${id}.variants.json`);
  fs.writeFileSync(jsonPath, JSON.stringify(data.entities, null, 2) + '\n');
  fs.writeFileSync(variantsPath, JSON.stringify(data.variants, null, 2) + '\n');
  console.log(`Generated ${id}: ${data.entities.length} entities, ${data.variants.variants.length} variants`);
}

// Generate all Batch 55 recipes
console.log('Generating Batch 55 Sandcastle data...');
writeData('3d-tiles-3d-native-vector', generate3dNativeVector());
writeData('3d-tiles-adjust-height', generateAdjustHeight());
writeData('3d-tiles-batch-table-hierarchy', generateBatchTableHierarchy());
writeData('3d-tiles-bim', generateBim());
console.log('Batch 55 data generation complete.');
