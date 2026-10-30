import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. 3D Tiles Clipping Planes
function generateClippingPlanes() {
  const pivotCylinder = {
    id: 'clip-planes-pivot-cylinder',
    name: 'Clipping Plane Anchor & Rotation Pivot Cylinder',
    show: true,
    positionDegrees: [-75.612, 40.042, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.95, 0.45, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.3, 1.0],
    outlineWidth: 1.0,
    properties: {
      planeNormal: [0.0, 0.0, -1.0],
      planeDistance: 0.0,
      edgeWidth: 1.0,
    },
  };

  const cutoutBox = {
    id: 'clip-planes-cutout-box',
    name: 'Sectional Excavation & Cutout Massing Box',
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
      targetObject: 'BIM',
      edgeColor: 'white',
      unionClipping: false,
    },
  };

  const normalSphere = {
    id: 'clip-planes-normal-sphere',
    name: 'Clipping Normal Orientation Reference Hemisphere',
    show: true,
    positionDegrees: [-75.618, 40.035, 2200.0],
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
      debugBoundingVolumes: false,
      planeRepresentation: 'PlaneGeometry',
    },
  };

  const sectionRect = {
    id: 'clip-planes-section-rect',
    name: 'Cross-Section Spatial Extent Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.63, 40.03, -75.59, 40.06],
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
      clipObjects: 'BIM, Point Cloud, Instanced, Model',
      edgeStylingEnabled: true,
    },
  };

  const entities = [pivotCylinder, cutoutBox, normalSphere, sectionRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: BIM Model Clipping (Horizontal Normal [0, 0, -1])',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Edge Styling Enabled (Glowing White 1px Cut Edges)',
        entities: entities.map((e) => {
          if (e.id === 'clip-planes-pivot-cylinder') {
            return { ...e, color: [1.0, 1.0, 1.0, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Inverted Clipping Normal ([0, 0, 1] Revealing Interior)',
        entities: entities.map((e) => {
          if (e.id === 'clip-planes-cutout-box') {
            return { ...e, color: [0.95, 0.75, 0.15, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Debug Bounding Volumes Enabled (Tileset Sphere & Box Hierarchy)',
        entities: entities.map((e) => {
          if (e.id === 'clip-planes-section-rect') {
            return { ...e, color: [0.2, 0.75, 0.9, 0.8] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 2. 3D Tiles Compare
function generateCompare() {
  const spireCylinder = {
    id: 'compare-melbourne-spire-cylinder',
    name: 'Melbourne CBD Eureka Tower Spatial Datum',
    show: true,
    positionDegrees: [144.963, -37.813, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.15, 0.65, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.35, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      leftTilesetId: 69380,
      leftTitle: 'Melbourne Photogrammetry',
      splitDirection: 'LEFT',
    },
  };

  const dividerBox = {
    id: 'compare-split-divider-box',
    name: 'Split Screen Central Viewport Divider Boundary',
    show: true,
    positionDegrees: [144.970, -37.808, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.95, 0.55, 0.15, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.35, 1.0],
    outlineWidth: 1.0,
    properties: {
      rightTileset: 'Cesium OSM Buildings',
      splitDirection: 'RIGHT',
      splitPosition: 0.5,
    },
  };

  const photoDome = {
    id: 'compare-photogrammetry-dome',
    name: 'Photogrammetry Reality Mesh Coverage Hemisphere',
    show: true,
    positionDegrees: [144.955, -37.818, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.35, 0.85, 0.65, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      urbanCenter: 'Melbourne CBD',
      coordinateSystem: 'GDA2020 / MGA Zone 55',
    },
  };

  const corridorRect = {
    id: 'compare-urban-corridor-rect',
    name: 'Yarra River Urban Corridor Geographic Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [144.94, -37.83, 144.99, -37.79],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.45, 0.45, 0.85, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.65, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: {
      sliderSynchronized: true,
      splitDirectionLeft: 'Melbourne Reality Mesh',
      splitDirectionRight: 'Procedural OSM Vectors',
    },
  };

  const entities = [spireCylinder, dividerBox, photoDome, corridorRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Dual Split View (Photogrammetry Left vs OSM Buildings Right)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Left Focus (Split Position 0.2: Expanded OSM Buildings)',
        entities: entities.map((e) => {
          if (e.id === 'compare-split-divider-box') {
            return { ...e, color: [0.95, 0.25, 0.25, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Right Focus (Split Position 0.8: Expanded Photogrammetry)',
        entities: entities.map((e) => {
          if (e.id === 'compare-melbourne-spire-cylinder') {
            return { ...e, color: [0.2, 0.95, 0.8, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Single Fullscreen Photogrammetry (Split Disabled)',
        entities: entities.map((e) => {
          if (e.id === 'compare-urban-corridor-rect') {
            return { ...e, color: [0.2, 0.7, 0.4, 0.8] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 3. 3D Tiles Edge Visibility
function generateEdgeVisibility() {
  const datumCylinder = {
    id: 'edge-visibility-datum-cylinder',
    name: 'Architectural Edge Contrast Datum Cylinder',
    show: true,
    positionDegrees: [-75.612, 40.042, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.95, 0.85, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      ionAssetId: 4665927,
      edgeDisplayMode: 'EDGES_ONLY',
    },
  };

  const wireframeBox = {
    id: 'edge-visibility-wireframe-box',
    name: 'Geometric Crease & Silhouette Extraction Box',
    show: true,
    positionDegrees: [-75.605, 40.048, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.15, 0.75, 0.85, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.35, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      creaseAngleDegrees: 30.0,
      silhouetteColor: 'cyan',
    },
  };

  const silhouetteDome = {
    id: 'edge-visibility-silhouette-dome',
    name: 'Boundary Silhouette Enclosure Hemisphere',
    show: true,
    positionDegrees: [-75.618, 40.035, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.75, 0.35, 0.85, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      silhouetteWidth: 2.0,
      meshPbrShading: true,
    },
  };

  const campusRect = {
    id: 'edge-visibility-campus-rect',
    name: 'Exton Edge Visibility Testing Geographic Area',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.63, 40.03, -75.59, 40.06],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.35, 0.65, 0.95, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.55, 0.85, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: {
      cameraHeadingPitchRange: [0.0, -0.5, 0.0],
      renderTargetFormat: 'RGBA8_UNORM',
    },
  };

  const entities = [datumCylinder, wireframeBox, silhouetteDome, campusRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Edges Only (Sharp Geometric Crease Lines)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Edges and Silhouettes (Creases plus Outer Silhouette Boundary)',
        entities: entities.map((e) => {
          if (e.id === 'edge-visibility-wireframe-box') {
            return { ...e, color: [0.95, 0.45, 0.15, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Silhouettes Only (Outer Contour Boundary Lines)',
        entities: entities.map((e) => {
          if (e.id === 'edge-visibility-silhouette-dome') {
            return { ...e, color: [0.2, 0.9, 0.5, 0.9] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Edge Display Off (Standard PBR Shaded Mesh)',
        entities: entities.map((e) => {
          if (e.id === 'edge-visibility-datum-cylinder') {
            return { ...e, color: [0.7, 0.7, 0.7, 0.95] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 4. 3D Tiles Feature Picking
function generateFeaturePicking() {
  const beaconCylinder = {
    id: 'picking-manhattan-beacon-cylinder',
    name: 'Lower Manhattan Financial Center Beacon Cylinder',
    show: true,
    positionDegrees: [-74.006, 40.7128, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.85, 0.25, 0.35, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.45, 0.55, 1.0],
    outlineWidth: 1.0,
    properties: {
      dataSource: 'NYC OpenData 3D Buildings',
      binCode: '1001001',
    },
  };

  const selectedBox = {
    id: 'picking-hover-selected-box',
    name: 'Interactive Hover & Selection Focus Box',
    show: true,
    positionDegrees: [-73.998, 40.719, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.95, 0.85, 0.15, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      hoverOutlineColor: 'yellow',
      selectFillColor: 'lime',
    },
  };

  const inspectionSphere = {
    id: 'picking-feature-inspection-sphere',
    name: 'Building Semantic Metadata Query Sphere',
    show: true,
    positionDegrees: [-74.015, 40.705, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.25, 0.75, 0.95, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      infoBoxEnabled: true,
      batchIdActive: true,
    },
  };

  const cadastralRect = {
    id: 'picking-cadastral-zone-rect',
    name: 'Manhattan Cadastral Survey Boundary Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-74.03, 40.69, -73.98, 40.73],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.45, 0.85, 0.45, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.65, 1.0, 0.65, 0.9],
    outlineWidth: 1.0,
    properties: {
      depthTestAgainstTerrain: true,
      initialHeadingPitchRoll: [0.0, -0.5, 0.0],
    },
  };

  const entities = [beaconCylinder, selectedBox, inspectionSphere, cadastralRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Default Manhattan 3D Buildings View',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Feature Hover Highlight (Yellow Outline on Mouse Over)',
        entities: entities.map((e) => {
          if (e.id === 'picking-hover-selected-box') {
            return { ...e, color: [1.0, 1.0, 0.2, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Feature Selected Active (Green Fill & Metadata Inset)',
        entities: entities.map((e) => {
          if (e.id === 'picking-manhattan-beacon-cylinder') {
            return { ...e, color: [0.2, 0.95, 0.3, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Strict Terrain Occlusion Enabled',
        entities: entities.map((e) => {
          if (e.id === 'picking-cadastral-zone-rect') {
            return { ...e, color: [0.25, 0.55, 0.95, 0.8] };
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

// Generate all Batch 56 recipes
console.log('Generating Batch 56 Sandcastle data...');
writeData('3d-tiles-clipping-planes', generateClippingPlanes());
writeData('3d-tiles-compare', generateCompare());
writeData('3d-tiles-edge-visibility', generateEdgeVisibility());
writeData('3d-tiles-feature-picking', generateFeaturePicking());
console.log('Batch 56 data generation complete.');
