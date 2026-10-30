import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Multifrustum Snapping - Dev
function generateMultifrustumSnappingDev() {
  const farBox = {
    id: 'snap-far-tileset-box',
    name: 'Far Frustum 3D Tileset Snapping Target Anchor Box',
    show: true,
    positionDegrees: [-75.165, 39.952, 60.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 80.0],
    },
    color: [0.25, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      frustum: 'FAR',
      farToNearRatio: 100.0,
      logarithmicDepthBuffer: false,
      ionAssetId: 4665927,
    },
  };

  const nearCylinder = {
    id: 'snap-near-model-cylinder',
    name: 'Near Frustum EdgeVisibility Model Datum Column',
    show: true,
    positionDegrees: [-75.162, 39.955, 45.0],
    shape: {
      type: 'cylinder',
      length: 55.0,
      topRadius: 7.0,
      bottomRadius: 7.0,
    },
    color: [0.95, 0.85, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      frustum: 'NEAR',
      edgeDisplayMode: 'SURFACES_AND_EDGES',
    },
  };

  const markerSphere = {
    id: 'snap-marker-probe-sphere',
    name: 'Snap Hit Diagnostic Screen Space Marker Probe',
    show: true,
    positionDegrees: [-75.168, 39.949, 50.0],
    shape: {
      type: 'ellipsoid',
      radii: [5.5, 5.5, 5.5],
    },
    color: [0.0, 1.0, 1.0, 0.95], // Cyan = edge snap
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      markerEdge: '#00ffff',
      markerSurface: '#ffff00',
      markerMiss: '#ff0000',
    },
  };

  const extentRect = {
    id: 'snap-multifrustum-bounds-rect',
    name: 'Multifrustum Snapping Inspection Cadastral Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.175, 39.945, -75.155, 39.960],
    },
    color: [0.25, 0.35, 0.45, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.75, 0.95, 0.75, 0.9],
    outlineWidth: 1.0,
    properties: {
      twoClickMeasurement: true,
    },
  };

  const entities = [farBox, nearCylinder, markerSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Multifrustum Snapping (Cyan Edge Snap in Near Frustum)',
      entities: [farBox, nearCylinder, markerSphere, extentRect],
    },
    {
      label: 'Surface Snapping Mode (Yellow Marker Surface Intersection in Far Frustum)',
      entities: [
        { ...farBox, color: [0.95, 0.55, 0.15, 0.95], outlineColor: [1.0, 0.8, 0.6, 1.0] },
        nearCylinder,
        markerSphere,
        extentRect,
      ],
    },
    {
      label: 'Miss Fallback Raycast Mode (Red Marker Screen Space pickPosition Fallback)',
      entities: [
        farBox,
        { ...nearCylinder, color: [0.95, 0.25, 0.25, 0.95], outlineColor: [1.0, 0.7, 0.7, 1.0] },
        markerSphere,
        extentRect,
      ],
    },
    {
      label: 'Two-Click Measurement Mode (Orange Distance Vector Line Inspection)',
      entities: [
        farBox,
        nearCylinder,
        { ...markerSphere, color: [1.0, 0.55, 0.0, 0.95], outlineColor: [1.0, 0.85, 0.5, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 2. Multiple Shadows - Dev
function generateMultipleShadowsDev() {
  const modelBox = {
    id: 'shadow-model-caster-box',
    name: 'Shadow Tester Model Geometry Caster Anchor Box',
    show: true,
    positionDegrees: [-76.344, 39.845, 200.0],
    shape: {
      type: 'box',
      dimensions: [30.0, 30.0, 60.0],
    },
    color: [0.3, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      shadowCaster: true,
      animationMultiplier: 0.5,
    },
  };

  const pointLight1Cylinder = {
    id: 'shadow-point-light-1-cylinder',
    name: 'Point Light Camera 1 Omnidirectional Shadow Column',
    show: true,
    positionDegrees: [-76.341, 39.848, 180.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 6.5,
      bottomRadius: 6.5,
    },
    color: [0.95, 0.85, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      isPointLight: true,
      shadowMapDebugShow: true,
    },
  };

  const pointLight2Sphere = {
    id: 'shadow-point-light-2-sphere',
    name: 'Point Light Camera 2 Secondary Shadow Map Probe',
    show: true,
    positionDegrees: [-76.347, 39.842, 190.0],
    shape: {
      type: 'ellipsoid',
      radii: [5.5, 5.5, 5.5],
    },
    color: [0.85, 0.25, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      softShadows: false,
      headingOffset: 'Math.PI',
    },
  };

  const extentRect = {
    id: 'shadow-scene-bounds-rect',
    name: 'Multiple Point Light Shadows Geodetic Footprint Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-76.355, 39.835, -76.335, 39.855],
    },
    color: [0.25, 0.35, 0.3, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.95, 0.85, 0.4, 0.9],
    outlineWidth: 1.0,
    properties: {
      terrain: 'WorldTerrain',
    },
  };

  const entities = [modelBox, pointLight1Cylinder, pointLight2Sphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Dual Point Light Shadows (Light 1 Debug Frustum Active)',
      entities: [modelBox, pointLight1Cylinder, pointLight2Sphere, extentRect],
    },
    {
      label: 'Secondary Light 2 Debug View Toggle Mode (Opposite 180° Shadow Vector)',
      entities: [
        { ...modelBox, color: [0.95, 0.45, 0.25, 0.95], outlineColor: [1.0, 0.75, 0.6, 1.0] },
        pointLight1Cylinder,
        pointLight2Sphere,
        extentRect,
      ],
    },
    {
      label: 'Animated Shadow Oscillation Mode (0.5x Looped Moving Geometry)',
      entities: [
        modelBox,
        { ...pointLight1Cylinder, color: [0.35, 0.9, 0.55, 0.95], outlineColor: [0.75, 1.0, 0.85, 1.0] },
        pointLight2Sphere,
        extentRect,
      ],
    },
    {
      label: 'Terrain Self-Shadowing and Occlusion Mode (World Terrain Depth Buffer)',
      entities: [
        modelBox,
        pointLight1Cylinder,
        { ...pointLight2Sphere, color: [0.25, 0.85, 0.85, 0.95], outlineColor: [0.7, 1.0, 1.0, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 3. Multiple Synced Views
function generateMultipleSyncedViews() {
  const sync3DBox = {
    id: 'sync-3d-perspective-box',
    name: '3D Perspective Globe Camera Center Target Box',
    show: true,
    positionDegrees: [-75.165, 39.952, 60.0],
    shape: {
      type: 'box',
      dimensions: [35.0, 35.0, 70.0],
    },
    color: [0.2, 0.85, 0.55, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 1.0, 0.85, 1.0],
    outlineWidth: 1.0,
    properties: {
      sceneMode: 'SCENE3D',
      sharedClock: true,
    },
  };

  const sync2DCylinder = {
    id: 'sync-2d-ortho-cylinder',
    name: '2D Orthographic Map Synchronized Center Datum Column',
    show: true,
    positionDegrees: [-75.162, 39.955, 45.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 6.5,
      bottomRadius: 6.5,
    },
    color: [0.25, 0.65, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      sceneMode: 'SCENE2D',
      pickEllipsoid: true,
    },
  };

  const focalSphere = {
    id: 'sync-focal-pick-sphere',
    name: 'Viewport Center Screen Space Focal Raycast Probe',
    show: true,
    positionDegrees: [-75.168, 39.949, 50.0],
    shape: {
      type: 'ellipsoid',
      radii: [5.5, 5.5, 5.5],
    },
    color: [0.95, 0.85, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      viewCenter: 'Cartesian2(w/2, h/2)',
      distanceLock: true,
    },
  };

  const extentRect = {
    id: 'sync-viewport-extent-rect',
    name: 'Synchronized Dual Viewport Cadastral Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.175, 39.945, -75.155, 39.960],
    },
    color: [0.25, 0.35, 0.45, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.85, 0.7, 0.95, 0.9],
    outlineWidth: 1.0,
    properties: {
      dualViewerSync: '2D_and_3D',
    },
  };

  const entities = [sync3DBox, sync2DCylinder, focalSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Dual 2D/3D Synchronized Viewports (Shared ClockViewModel)',
      entities: [sync3DBox, sync2DCylinder, focalSphere, extentRect],
    },
    {
      label: '2D Orthographic Map Tracking 3D Ellipsoid Center Focal Point',
      entities: [
        { ...sync3DBox, color: [0.95, 0.45, 0.25, 0.95], outlineColor: [1.0, 0.75, 0.6, 1.0] },
        sync2DCylinder,
        focalSphere,
        extentRect,
      ],
    },
    {
      label: 'Dynamic Distance Offset Synchronization (Zoom In / Out Coupling)',
      entities: [
        sync3DBox,
        { ...sync2DCylinder, color: [0.85, 0.25, 0.95, 0.95], outlineColor: [1.0, 0.7, 1.0, 1.0] },
        focalSphere,
        extentRect,
      ],
    },
    {
      label: 'Side-by-Side Dual Viewport Cadastral Boundary Inspection',
      entities: [
        sync3DBox,
        sync2DCylinder,
        { ...focalSphere, color: [0.25, 0.85, 0.85, 0.95], outlineColor: [0.7, 1.0, 1.0, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 4. PAMAP Terrain
function generatePamapTerrain() {
  const pinnacleBox = {
    id: 'pamap-pinnacle-anchor-box',
    name: 'PASDA Pinnacle Landmark High-Resolution Terrain Box',
    show: true,
    positionDegrees: [-76.344, 39.845, 236.7],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 60.0],
    },
    color: [0.35, 0.85, 0.45, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 1.0, 0.85, 1.0],
    outlineWidth: 1.0,
    properties: {
      provider: 'CesiumTerrainProvider',
      ionAssetId: 3957,
      poi: 'Pinnacle',
    },
  };

  const nittanyCylinder = {
    id: 'pamap-mount-nittany-cylinder',
    name: 'Mount Nittany Ridge Topography Datum Column',
    show: true,
    positionDegrees: [-77.864, 40.803, 550.0],
    shape: {
      type: 'cylinder',
      length: 70.0,
      topRadius: 8.0,
      bottomRadius: 8.0,
    },
    color: [0.25, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      poi: 'Mount Nittany',
      curatedBy: 'Pennsylvania Spatial Data Access (PASDA)',
    },
  };

  const elevationSphere = {
    id: 'pamap-elevation-survey-sphere',
    name: 'Appalachian Ridge Topographic Elevation Survey Probe',
    show: true,
    positionDegrees: [-76.341, 39.848, 240.0],
    shape: {
      type: 'ellipsoid',
      radii: [6.0, 6.0, 6.0],
    },
    color: [0.95, 0.55, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.8, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      elevationModel: 'PAMAP_High_Res_Lidar',
    },
  };

  const extentRect = {
    id: 'pamap-pennsylvania-bounds-rect',
    name: 'Pennsylvania Regional Lidar Terrain Cadastral Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-76.360, 39.830, -76.330, 39.860],
    },
    color: [0.25, 0.35, 0.25, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.75, 0.95, 0.65, 0.9],
    outlineWidth: 1.0,
    properties: {
      state: 'Pennsylvania',
    },
  };

  const entities = [pinnacleBox, nittanyCylinder, elevationSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: PAMAP High-Resolution Pennsylvania Terrain (Pinnacle Viewpoint)',
      entities: [pinnacleBox, nittanyCylinder, elevationSphere, extentRect],
    },
    {
      label: 'Mount Nittany Ridge Geological Topography FlyTo View',
      entities: [
        { ...pinnacleBox, color: [0.2, 0.75, 0.95, 0.95], outlineColor: [0.65, 0.95, 1.0, 1.0] },
        nittanyCylinder,
        elevationSphere,
        extentRect,
      ],
    },
    {
      label: 'Appalachian Mountain Crest Contour Elevation Highlights',
      entities: [
        pinnacleBox,
        { ...nittanyCylinder, color: [0.95, 0.85, 0.25, 0.95], outlineColor: [1.0, 0.95, 0.7, 1.0] },
        elevationSphere,
        extentRect,
      ],
    },
    {
      label: 'PASDA Statewide Lidar Geospatial Coverage Footprint',
      entities: [
        pinnacleBox,
        nittanyCylinder,
        { ...elevationSphere, color: [0.85, 0.25, 0.95, 0.95], outlineColor: [1.0, 0.7, 1.0, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// Generate all Batch 80 fixtures
const datasets = {
  'multifrustum-snapping-dev': generateMultifrustumSnappingDev(),
  'multiple-shadows-dev': generateMultipleShadowsDev(),
  'multiple-synced-views': generateMultipleSyncedViews(),
  'pamap-terrain': generatePamapTerrain(),
};

for (const [id, { entities, variants }] of Object.entries(datasets)) {
  const jsonPath = path.join(outDir, `${id}.json`);
  const variantsPath = path.join(outDir, `${id}.variants.json`);

  fs.writeFileSync(jsonPath, JSON.stringify(entities, null, 2) + '\n');
  fs.writeFileSync(variantsPath, JSON.stringify({ initial: 0, variants }, null, 2) + '\n');
  console.log(`Generated ${id}.json (${entities.length}) and ${id}.variants.json (${variants.length})`);
}
