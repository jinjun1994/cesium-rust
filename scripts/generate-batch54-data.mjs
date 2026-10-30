import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. 3D Tiles 1.1 CDB Yemen
function generateCdbYemen() {
  const benchmarkPillar = {
    id: 'cdb-crater-benchmark-pillar',
    name: 'Aden Crater CDB Topographic Reference Pillar',
    show: true,
    positionDegrees: [45.042, 12.740, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.85, 0.65, 0.35, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.85, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      ionAssetId: 2389063,
      cdbDataset: 'Elevation and Imagery',
      faccCode: 'AL015',
    },
  };

  const airportBox = {
    id: 'cdb-aden-airport-terminal-box',
    name: 'Aden International Airport CDB Vector Massing Box',
    show: true,
    positionDegrees: [45.028, 12.825, 2000.0],
    shape: {
      type: 'box',
      dimensions: [3000.0, 3000.0, 2500.0],
    },
    color: [0.35, 0.55, 0.85, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.55, 0.75, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      buildingsIonAssetId: 2389064,
      faccCode: 'GB005',
      heightMeters: 28.5,
    },
  };

  const terrainDome = {
    id: 'cdb-yemen-terrain-dome',
    name: 'Yemen Mountainous Terrain Material Surface Dome',
    show: true,
    positionDegrees: [45.055, 12.750, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.75, 0.45, 0.25, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      surfaceSubstrate: 'Sandstone and Basalt',
      cdbMaterialCode: '1.1.2',
    },
  };

  const basinRect = {
    id: 'cdb-aden-port-basin-rect',
    name: 'Gulf of Aden Coastal Port Operations Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [44.95, 12.68, 45.12, 12.86],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.2, 0.7, 0.6, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.3, 0.9, 0.8, 0.9],
    outlineWidth: 1.0,
    properties: {
      geocoderCoverage: 'YEMEN_COASTAL',
      waterClassification: 'Marine Port Basin',
    },
  };

  const entities = [benchmarkPillar, airportBox, terrainDome, basinRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Unstyled CDB 3D Tiles 1.1 Terrain & Buildings',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Style Buildings by Height (Color by Height Thresholds)',
        entities: entities.map((e) => {
          if (e.id === 'cdb-aden-airport-terminal-box') {
            return { ...e, color: [0.9, 0.2, 0.2, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Style Terrain by Materials (CDB Surface Substrate Classification)',
        entities: entities.map((e) => {
          if (e.id === 'cdb-yemen-terrain-dome') {
            return { ...e, color: [0.85, 0.75, 0.3, 0.9] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Enable Feature Picking & Metadata Overlay',
        entities: entities.map((e) => ({
          ...e,
          properties: { ...e.properties, pickingActive: true },
        })),
      },
    ],
  };

  return { entities, variants };
}

// 2. 3D Tiles 1.1 Photogrammetry with KTX2
function generatePhotogrammetry() {
  const beaconTower = {
    id: 'photogrammetry-agi-beacon-tower',
    name: 'AGI HQ Exton Photogrammetry Datum Tower',
    show: true,
    positionDegrees: [-75.628, 40.038, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.2, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      ionAssetId: 40866,
      realityAssetId: 2325106,
      splitDirection: 'LEFT',
    },
  };

  const realityBox = {
    id: 'photogrammetry-reality-tiler-box',
    name: 'Cesium ion Reality Tiler High-Detail Urban Block',
    show: true,
    positionDegrees: [-75.615, 40.045, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.95, 0.45, 0.15, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.65, 0.35, 1.0],
    outlineWidth: 1.0,
    properties: {
      tilerEngine: 'Reality Tiler v1.1',
      ktx2TextureCompression: 'ETC1S / UASTC',
    },
  };

  const textureDome = {
    id: 'photogrammetry-ktx2-texture-dome',
    name: 'KTX2 GPU Texture Compression Diagnostic Hemisphere',
    show: true,
    positionDegrees: [-75.638, 40.030, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.35, 0.85, 0.45, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      textureFormat: 'KTX2_BASIS_UNIVERSAL',
      vramCompressionRatio: '4:1',
    },
  };

  const extonRect = {
    id: 'photogrammetry-exton-facility-rect',
    name: 'Exton Photogrammetry Campus Geographic Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.66, 40.01, -75.59, 40.07],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.55, 0.35, 0.85, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.75, 0.55, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: {
      terrainProvider: 'Cesium World Terrain',
      splitPosition: 0.5,
    },
  };

  const entities = [beaconTower, realityBox, textureDome, extonRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: AGI HQ Dual Split (KTX2 Reality Tiler vs 3D Model Tiler)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Melbourne Photogrammetry (Urban Architecture on Ellipsoid)',
        entities: entities.map((e) => {
          if (e.id === 'photogrammetry-reality-tiler-box') {
            return { ...e, color: [0.85, 0.85, 0.25, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Compute Time to Load (Benchmark Tileset Reload Cycle)',
        entities: entities.map((e) => ({
          ...e,
          properties: { ...e.properties, benchmarkActive: true },
        })),
      },
      {
        label: 'Single Fullscreen Photogrammetry (Split Position 1.0)',
        entities: entities.map((e) => {
          if (e.id === 'photogrammetry-exton-facility-rect') {
            return { ...e, color: [0.2, 0.6, 0.9, 0.8] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 3. 3D Tiles 1.1 Photogrammetry Classification
function generatePhotogrammetryClassification() {
  const clockTower = {
    id: 'ferry-building-clock-tower',
    name: 'San Francisco Ferry Building Clock Tower Spatial Anchor',
    show: true,
    positionDegrees: [-122.394, 37.794, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.95, 0.75, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.85, 0.45, 1.0],
    outlineWidth: 1.0,
    properties: {
      ionAssetId: 2333904,
      modelProvider: 'Aerometrex',
      componentType: 'Clock Tower',
    },
  };

  const embarcaderoBox = {
    id: 'embarcadero-classification-volume',
    name: 'Embarcadero Arcade Classification Volume Massing Box',
    show: true,
    positionDegrees: [-122.385, 37.802, 2000.0],
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
      classificationType: 'CESIUM_3D_TILE',
      featureIdSet: 'architectural_substructures',
    },
  };

  const translucentSphere = {
    id: 'ferry-plaza-translucent-sphere',
    name: 'Ferry Plaza Glass Canopy Translucent Material Sphere',
    show: true,
    positionDegrees: [-122.402, 37.788, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.35, 0.85, 0.75, 0.8],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      pbrAlphaMode: 'BLEND',
      glassTranslucency: 0.7,
    },
  };

  const waterfrontRect = {
    id: 'san-francisco-waterfront-boundary',
    name: 'San Francisco Embarcadero Waterfront Classification Zone',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.42, 37.77, -122.37, 37.82],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.85, 0.35, 0.55, 0.7],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.55, 0.75, 0.9],
    outlineWidth: 1.0,
    properties: {
      maximumScreenSpaceError: 8.0,
      pickTranslucentDepth: true,
    },
  };

  const entities = [clockTower, embarcaderoBox, translucentSphere, waterfrontRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Show Classification (Component Feature Color Overlay)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Translucent Windows (Architectural Glass Stylization)',
        entities: entities.map((e) => {
          if (e.id === 'ferry-plaza-translucent-sphere') {
            return { ...e, color: [0.2, 0.7, 0.95, 0.4] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Stylized PBR Materials (Metallic & Roughness Shader Overrides)',
        entities: entities.map((e) => {
          if (e.id === 'embarcadero-classification-volume') {
            return { ...e, color: [0.95, 0.8, 0.2, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Golden Touch & Multiple Feature ID Sets',
        entities: entities.map((e) => {
          if (e.id === 'ferry-building-clock-tower') {
            return { ...e, color: [1.0, 0.84, 0.0, 1.0] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 4. 3D Tiles 1.1 S2 Globe
function generateS2Globe() {
  const cellPillar = {
    id: 's2-hierarchical-cell-pillar',
    name: 'S2 Cell Spatial Partitioning Axis Datum Pillar',
    show: true,
    positionDegrees: [-88.46, 35.11, 200000.0],
    shape: {
      type: 'cylinder',
      length: 400000.0,
      topRadius: 120000.0,
      bottomRadius: 120000.0,
    },
    color: [0.2, 0.85, 0.75, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.4, 1.0, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: {
      ionAssetId: 1208297,
      s2Projection: 'CELL_HIERARCHY',
      maxarTerrainModel: 'OWT_WFF_1.2',
    },
  };

  const boundingBox = {
    id: 's2-bounding-projection-box',
    name: 'S2 Bounding Volume Cell Extent Alignment Box',
    show: true,
    positionDegrees: [-84.50, 38.00, 180000.0],
    shape: {
      type: 'box',
      dimensions: [250000.0, 250000.0, 200000.0],
    },
    color: [0.95, 0.55, 0.2, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      debugShowBoundingVolume: true,
      s2Face: 1,
      s2Level: 6,
    },
  };

  const globeDome = {
    id: 's2-maxar-terrain-globe-dome',
    name: 'One World Terrain Maxar S2 Planetary Shell Hemisphere',
    show: true,
    positionDegrees: [-92.00, 32.00, 190000.0],
    shape: {
      type: 'ellipsoid',
      radii: [180000.0, 180000.0, 160000.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.35, 0.55, 0.95, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      globeViewerDisabled: true,
      customShaderActive: false,
    },
  };

  const continentalRect = {
    id: 's2-continental-coverage-rect',
    name: 'Continental North America S2 Base Grid Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-96.0, 30.0, -80.0, 42.0],
      height: 10000.0,
      extrudedHeight: 120000.0,
      rotation: 0.0,
    },
    color: [0.45, 0.85, 0.35, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.65, 1.0, 0.55, 0.9],
    outlineWidth: 1.0,
    properties: {
      maximumScreenSpaceError: 4.0,
      landCoverClassification: 'Deciduous Forest and Cropland',
    },
  };

  const entities = [cellPillar, boundingBox, globeDome, continentalRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Globe View (One World Terrain Maxar S2 Base Globe)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Show S2 Bounding Volumes (Debug Hierarchical Cells)',
        entities: entities.map((e) => {
          if (e.id === 's2-bounding-projection-box') {
            return { ...e, color: [1.0, 0.2, 0.2, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Apply Style (Land Cover Semantic Color Coding)',
        entities: entities.map((e) => {
          if (e.id === 's2-continental-coverage-rect') {
            return { ...e, color: [0.2, 0.7, 0.3, 0.85] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Apply Custom Shader (Dynamic Procedural Surface Wave)',
        entities: entities.map((e) => {
          if (e.id === 's2-maxar-terrain-globe-dome') {
            return { ...e, color: [0.85, 0.35, 0.95, 0.9] };
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

// Generate all Batch 54 recipes
console.log('Generating Batch 54 Sandcastle data...');
writeData('3d-tiles-1.1-cdb-yemen', generateCdbYemen());
writeData('3d-tiles-1.1-photogrammetry', generatePhotogrammetry());
writeData('3d-tiles-1.1-photogrammetry-classification', generatePhotogrammetryClassification());
writeData('3d-tiles-1.1-s2-globe', generateS2Globe());
console.log('Batch 54 data generation complete.');
