import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Pick From Ray - Dev
function generatePickFromRayDev() {
  const startSphere = {
    id: 'raypick-start-blue-sphere',
    name: 'Raycast Source Blue Anchor Origin Sphere',
    show: true,
    positionDegrees: [-75.612, 40.041, 30.0],
    shape: {
      type: 'ellipsoid',
      radii: [10.0, 10.0, 10.0],
    },
    color: [0.0, 0.0, 1.0, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.7, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      role: 'RayOrigin',
      radiiMeters: 10.0,
    },
  };

  const targetSphere = {
    id: 'raypick-target-red-sphere',
    name: 'Raycast Target Red Destination Sphere',
    show: true,
    positionDegrees: [-75.611, 40.045, 30.0],
    shape: {
      type: 'ellipsoid',
      radii: [10.0, 10.0, 10.0],
    },
    color: [1.0, 0.0, 0.0, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      role: 'RayTarget',
      radiiMeters: 10.0,
    },
  };

  const hitBox = {
    id: 'raypick-intersection-box',
    name: '3D Tileset Raycast Intersection Marker Volume',
    show: true,
    positionDegrees: [-75.6115, 40.043, 30.0],
    shape: {
      type: 'box',
      dimensions: [8.0, 8.0, 8.0],
    },
    color: [1.0, 0.9, 0.1, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      intersectionPoint: [-75.6115, 40.043, 30.0],
      drillPick: false,
    },
  };

  const extentRect = {
    id: 'raypick-extent-rect',
    name: 'Pick From Ray Inspection Cadastral Bounds',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.620, 40.035, -75.602, 40.052],
    },
    color: [0.2, 0.3, 0.45, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.8, 1.0, 0.8],
    outlineWidth: 1.0,
    properties: {
      tileset: 'Tileset.json',
      depthTestAgainstTerrain: true,
    },
  };

  const entities = [startSphere, targetSphere, hitBox, extentRect];

  const variants = [
    {
      label: 'Baseline: Single Raycast Intersection Mode (scene.pickFromRay, Yellow Hit Highlight)',
      entities: [
        startSphere,
        targetSphere,
        hitBox,
        extentRect,
      ],
    },
    {
      label: 'Multi-Hit Drill-Pick Raycast Mode (scene.drillPickFromRay, Multiple Layer Hits)',
      entities: [
        startSphere,
        targetSphere,
        {
          ...hitBox,
          shape: { type: 'box', dimensions: [12.0, 12.0, 12.0] },
          color: [1.0, 0.5, 0.1, 0.95],
          properties: { ...hitBox.properties, drillPick: true, maxHits: 10 },
        },
        extentRect,
      ],
    },
    {
      label: 'Reverse Azimuth Vector Probe Mode (Swapped Ray Origin and Target Heading)',
      entities: [
        { ...startSphere, positionDegrees: [-75.611, 40.045, 30.0], color: [0.0, 0.8, 0.8, 0.95] },
        { ...targetSphere, positionDegrees: [-75.612, 40.041, 30.0], color: [0.9, 0.2, 0.5, 0.95] },
        hitBox,
        extentRect,
      ],
    },
    {
      label: 'Terrain Grazing Raycast Mode (Low Elevation Grazing Ray Profile)',
      entities: [
        { ...startSphere, positionDegrees: [-75.612, 40.041, 5.0] },
        { ...targetSphere, positionDegrees: [-75.611, 40.045, 5.0] },
        { ...hitBox, positionDegrees: [-75.6115, 40.043, 5.0], color: [0.4, 0.9, 0.3, 0.95] },
        extentRect,
      ],
    },
  ];

  return { entities, variants };
}

// 2. Picking - Dev
function generatePickingDev() {
  const targetBox = {
    id: 'pickingdev-target-box',
    name: 'Primitive and GeometryInstance Target Hull',
    show: true,
    positionDegrees: [-75.59777, 40.03883, 25.0],
    shape: {
      type: 'box',
      dimensions: [15.0, 15.0, 15.0],
    },
    color: [0.25, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      primitiveType: 'BoxGeometry',
      highlightColor: '#ffff00',
    },
  };

  const vectorCylinder = {
    id: 'pickingdev-vector-cylinder',
    name: 'Polyline Collection Linear Vector Datum Column',
    show: true,
    positionDegrees: [-75.5972, 40.0392, 20.0],
    shape: {
      type: 'cylinder',
      length: 30.0,
      topRadius: 5.0,
      bottomRadius: 5.0,
    },
    color: [0.95, 0.85, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      primitiveType: 'PolylineCollection',
      width: 5.0,
    },
  };

  const pointSphere = {
    id: 'pickingdev-screen-point-sphere',
    name: 'Screen Space Point and Billboard Interactive Probe',
    show: true,
    positionDegrees: [-75.5982, 40.0383, 18.0],
    shape: {
      type: 'ellipsoid',
      radii: [7.5, 7.5, 7.5],
    },
    color: [0.85, 0.25, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      pixelSize: 15.0,
      hoverHighlight: true,
    },
  };

  const boundsRect = {
    id: 'pickingdev-bounds-rect',
    name: 'Multi-Primitive Interactive Picking Ground Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.608, 40.030, -75.588, 40.047],
    },
    color: [0.2, 0.35, 0.3, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.9, 0.7, 0.8],
    outlineWidth: 1.0,
    properties: {
      mouseMovePick: true,
    },
  };

  const entities = [targetBox, vectorCylinder, pointSphere, boundsRect];

  const variants = [
    {
      label: 'Baseline: Primitive & Geometry Instance Picking (Hover Highlight Yellow)',
      entities: [
        targetBox,
        vectorCylinder,
        pointSphere,
        boundsRect,
      ],
    },
    {
      label: 'Polyline Collection Screen Space Pick Mode (5.0px Width Linear Stroke Pick)',
      entities: [
        { ...targetBox, show: false },
        {
          ...vectorCylinder,
          shape: { type: 'cylinder', length: 45.0, topRadius: 7.0, bottomRadius: 7.0 },
          color: [1.0, 0.95, 0.3, 0.95],
        },
        pointSphere,
        boundsRect,
      ],
    },
    {
      label: '3D Tiles Feature & Point Cloud Pick Mode (Per-Feature Dynamic Point Styling)',
      entities: [
        targetBox,
        vectorCylinder,
        {
          ...pointSphere,
          shape: { type: 'ellipsoid', radii: [12.0, 12.0, 12.0] },
          color: [0.3, 0.9, 0.95, 0.95],
        },
        boundsRect,
      ],
    },
    {
      label: 'Vector Tile Polygon Multi-Feature Pick Mode (Continuous Terrain Clamped Classification)',
      entities: [
        { ...targetBox, color: [0.95, 0.45, 0.2, 0.95] },
        { ...vectorCylinder, show: false },
        pointSphere,
        boundsRect,
      ],
    },
  ];

  return { entities, variants };
}

// 3. Polylines on 3D Tiles
function generatePolylinesOn3dTiles() {
  const powerplantBox = {
    id: 'polylines3d-powerplant-box',
    name: 'Bentley Power Plant BIM Infrastructure Envelope',
    show: true,
    positionDegrees: [-76.36054, 34.94993, 20.0],
    shape: {
      type: 'box',
      dimensions: [25.0, 20.0, 16.0],
    },
    color: [0.35, 0.75, 0.95, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      ionAssetId: 2464651,
      name: 'Power Plant BIM',
      duplicateFloorElement: 114082,
    },
  };

  const pipeCylinder = {
    id: 'polylines3d-clamped-pipe-cylinder',
    name: 'Clamped Dashed Pipeline Industrial Route Column',
    show: true,
    positionDegrees: [-76.36055, 34.94994, 25.0],
    shape: {
      type: 'cylinder',
      length: 35.0,
      topRadius: 3.0,
      bottomRadius: 3.0,
    },
    color: [1.0, 0.85, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      clampToGround: true,
      classificationType: 'CESIUM_3D_TILE',
      dashLength: 20.0,
    },
  };

  const nodeSphere = {
    id: 'polylines3d-photogrammetry-sphere',
    name: 'Photogrammetric Facade Route Node Datum Sphere',
    show: true,
    positionDegrees: [-76.36050, 34.94990, 18.0],
    shape: {
      type: 'ellipsoid',
      radii: [10.0, 10.0, 10.0],
    },
    color: [0.85, 0.35, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      buildingIonAssetId: 40866,
      width: 6.0,
    },
  };

  const corridorRect = {
    id: 'polylines3d-corridor-rect',
    name: 'Clamped 3D Tile Infrastructure Cadastral Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-76.370, 34.942, -76.350, 34.958],
    },
    color: [0.2, 0.3, 0.45, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.8, 1.0, 0.8],
    outlineWidth: 1.0,
    properties: {
      classificationType: 'CESIUM_3D_TILE',
    },
  };

  const entities = [powerplantBox, pipeCylinder, nodeSphere, corridorRect];

  const variants = [
    {
      label: 'Baseline: BIM Industrial Pipes Mode (Power Plant 3D Tile Clamping, Dashed Yellow)',
      entities: [
        powerplantBox,
        pipeCylinder,
        nodeSphere,
        corridorRect,
      ],
    },
    {
      label: 'Photogrammetry Urban Route Mode (Building Facade Surface Clamping)',
      entities: [
        { ...powerplantBox, show: false },
        {
          ...pipeCylinder,
          color: [0.2, 0.95, 0.85, 0.95],
          properties: { ...pipeCylinder.properties, dashLength: 10.0 },
        },
        {
          ...nodeSphere,
          show: true,
          shape: { type: 'ellipsoid', radii: [14.0, 14.0, 14.0] },
          color: [0.95, 0.75, 0.2, 0.95],
        },
        corridorRect,
      ],
    },
    {
      label: 'High-Density Solid Line Pipeline Mode (Continuous Non-Dashed Clamped Polyline)',
      entities: [
        powerplantBox,
        {
          ...pipeCylinder,
          shape: { type: 'cylinder', length: 50.0, topRadius: 5.0, bottomRadius: 5.0 },
          color: [1.0, 0.4, 0.15, 0.95],
        },
        nodeSphere,
        corridorRect,
      ],
    },
    {
      label: 'Dual Infrastructure Overview Mode (Composite Subsurface and Facade Clamping)',
      entities: [
        powerplantBox,
        pipeCylinder,
        { ...nodeSphere, color: [0.55, 0.45, 0.9, 0.95] },
        corridorRect,
      ],
    },
  ];

  return { entities, variants };
}

// 4. Post-Processing
function generatePostProcessing() {
  const characterBox = {
    id: 'postprocess-character-box',
    name: 'Sample Cesium_Man Animated Model Bounding Hull',
    show: true,
    positionDegrees: [-123.0744619, 44.0503706, 10.0],
    shape: {
      type: 'box',
      dimensions: [8.0, 6.0, 18.0],
    },
    color: [0.35, 0.75, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      modelUri: 'Cesium_Man.glb',
      trackedEntity: true,
    },
  };

  const haloEllipsoid = {
    id: 'postprocess-silhouette-halo-ellipsoid',
    name: 'Full Screen Silhouette Edge Highlight Outer Shell',
    show: true,
    positionDegrees: [-123.0741, 44.0506, 14.0],
    shape: {
      type: 'ellipsoid',
      radii: [12.0, 12.0, 15.0],
    },
    color: [1.0, 0.9, 0.1, 0.85],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      stage: 'Silhouette',
      color: '#ffff00',
      enabled: true,
    },
  };

  const nightvisionCylinder = {
    id: 'postprocess-nightvision-cylinder',
    name: 'Night Vision Phosphor & Vignette Stage Datum Column',
    show: true,
    positionDegrees: [-123.0748, 44.0501, 8.0],
    shape: {
      type: 'cylinder',
      length: 22.0,
      topRadius: 4.0,
      bottomRadius: 4.0,
    },
    color: [0.15, 0.95, 0.35, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.5, 1.0, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      stage: 'NightVision',
      brightness: 0.5,
      gradations: 5.0,
    },
  };

  const inspectionRect = {
    id: 'postprocess-inspection-rect',
    name: 'Post Process Stage Stage Pipeline Cadastral Bounds',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.084, 44.043, -123.064, 44.057],
    },
    color: [0.2, 0.25, 0.35, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.7, 1.0, 0.8],
    outlineWidth: 1.0,
    properties: {
      multiStageComposite: true,
    },
  };

  const entities = [characterBox, haloEllipsoid, nightvisionCylinder, inspectionRect];

  const variants = [
    {
      label: 'Baseline: Silhouette Post-Process Stage (Yellow Outer Boundary Glow)',
      entities: [
        characterBox,
        haloEllipsoid,
        { ...nightvisionCylinder, show: false },
        inspectionRect,
      ],
    },
    {
      label: 'Black and White Quantization Stage (5.0 Gradations Grayscale Render)',
      entities: [
        characterBox,
        { ...haloEllipsoid, show: false },
        {
          ...nightvisionCylinder,
          show: true,
          color: [0.65, 0.65, 0.65, 0.95],
          properties: { ...nightvisionCylinder.properties, stage: 'BlackAndWhite', gradations: 5.0 },
        },
        inspectionRect,
      ],
    },
    {
      label: 'Night Vision Phosphor Simulation Stage (Luminescent Green Tone & Vignette)',
      entities: [
        characterBox,
        { ...haloEllipsoid, show: false },
        {
          ...nightvisionCylinder,
          show: true,
          color: [0.2, 1.0, 0.3, 0.95],
          properties: { ...nightvisionCylinder.properties, stage: 'NightVision', nightVisionShow: true },
        },
        inspectionRect,
      ],
    },
    {
      label: 'High-Exposure Brightness Boost Stage (Brightness Value 1.5 Solar Flare Mode)',
      entities: [
        characterBox,
        {
          ...haloEllipsoid,
          show: true,
          color: [1.0, 1.0, 0.8, 0.95],
        },
        {
          ...nightvisionCylinder,
          show: true,
          color: [1.0, 0.95, 0.6, 0.9],
          properties: { ...nightvisionCylinder.properties, stage: 'Brightness', brightnessValue: 1.5 },
        },
        inspectionRect,
      ],
    },
  ];

  return { entities, variants };
}

const generators = {
  'pick-from-ray-dev': generatePickFromRayDev,
  'picking-dev': generatePickingDev,
  'polylines-on-3d-tiles': generatePolylinesOn3dTiles,
  'post-processing': generatePostProcessing,
};

for (const [id, gen] of Object.entries(generators)) {
  const { entities, variants } = gen();
  const fullPath = path.join(outDir, `${id}.json`);
  const variantsPath = path.join(outDir, `${id}.variants.json`);

  fs.writeFileSync(fullPath, JSON.stringify(entities, null, 2) + '\n');
  fs.writeFileSync(variantsPath, JSON.stringify({ initial: 0, variants }, null, 2) + '\n');
  console.log(`Generated ${id}: ${entities.length} entities, ${variants.length} variants`);
}
