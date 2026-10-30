import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Custom Shaders Property Textures (MAXAR OWT Muscatatuk Photogrammetry)
function generateCustomShadersPropertyTextures() {
  const ce90Box = {
    id: 'custom-proptex-ce90-box',
    name: 'Horizontal Uncertainty CE90 Benchmark Volume',
    show: true,
    positionDegrees: [-85.5340, 39.0520, 232.0],
    shape: {
      type: 'box',
      dimensions: [32.0, 32.0, 24.0],
    },
    color: [0.2, 0.65, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      uncertaintyMode: 'UNCERTAINTY_CE90',
      propertyTexture: 'r3dm_uncertainty_ce90sum',
    },
  };

  const le90Pillar = {
    id: 'custom-proptex-le90-pillar',
    name: 'Vertical Uncertainty LE90 Inspection Cylinder',
    show: true,
    positionDegrees: [-85.5332, 39.0526, 235.0],
    shape: {
      type: 'cylinder',
      length: 30.0,
      topRadius: 14.0,
      bottomRadius: 14.0,
    },
    color: [0.85, 0.55, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.8, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      uncertaintyMode: 'UNCERTAINTY_LE90',
      propertyTexture: 'r3dm_uncertainty_le90sum',
    },
  };

  const combinedProbe = {
    id: 'custom-proptex-combined-probe',
    name: 'Combined Dual Uncertainty Assessment Probe',
    show: true,
    positionDegrees: [-85.5348, 39.0514, 230.0],
    shape: {
      type: 'ellipsoid',
      radii: [12.0, 12.0, 12.0],
    },
    color: [0.75, 0.25, 0.9, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.95, 0.65, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      uncertaintyMode: 'UNCERTAINTY',
      metadataChannels: 2,
    },
  };

  const muscatatukRect = {
    id: 'custom-proptex-muscatatuk-rect',
    name: 'Muscatatuk Photogrammetry Property Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-85.538, 39.049, -85.530, 39.055],
    },
    color: [0.15, 0.35, 0.25, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.8, 0.55, 0.8],
    outlineWidth: 1.0,
    properties: {
      dataset: 'MAXAR OWT Muscatatuk Photogrammetry',
      ionAssetId: 2342602,
    },
  };

  const entities = [ce90Box, le90Pillar, combinedProbe, muscatatukRect];

  const variants = [
    {
      label: 'Baseline: Horizontal Uncertainty CE90 (Cyan Diffuse Infill)',
      entities: [
        ce90Box,
        le90Pillar,
        combinedProbe,
        muscatatukRect,
      ],
    },
    {
      label: 'Vertical Uncertainty LE90 Mode (Amber Gradient Profile)',
      entities: [
        { ...ce90Box, color: [0.95, 0.65, 0.1, 0.95], outlineColor: [1.0, 0.85, 0.4, 1.0] },
        le90Pillar,
        combinedProbe,
        muscatatukRect,
      ],
    },
    {
      label: 'Combined Dual Uncertainty Sum (Vibrant Violet Diffuse)',
      entities: [
        ce90Box,
        { ...le90Pillar, color: [0.85, 0.2, 0.95, 0.95], outlineColor: [1.0, 0.65, 1.0, 1.0] },
        combinedProbe,
        muscatatukRect,
      ],
    },
    {
      label: 'Unprocessed Photogrammetry Surface (Clean True-Color Texture)',
      entities: [
        ce90Box,
        le90Pillar,
        { ...combinedProbe, color: [0.35, 0.85, 0.5, 0.95], outlineColor: [0.75, 1.0, 0.85, 1.0] },
        { ...muscatatukRect, color: [0.1, 0.2, 0.4, 0.3] },
      ],
    },
  ];

  return { entities, variants };
}

// 2. CZML 3D Tiles (BatchedColors B3DM Dataset)
function generateCzml3dTiles() {
  const batchedBox = {
    id: 'czml-3dtiles-batched-box',
    name: 'BatchedColors B3DM Primary Structure',
    show: true,
    positionDegrees: [-75.1520, 39.9520, 30.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 60.0],
    },
    color: [0.9, 0.25, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      czmlPacketId: 'BatchedColors',
      tilesetUri: 'BatchedColors/tileset.json',
    },
  };

  const anchorPillar = {
    id: 'czml-3dtiles-anchor-pillar',
    name: 'CZML Tileset Spatial Anchor Cylinder',
    show: true,
    positionDegrees: [-75.1512, 39.9527, 25.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 16.0,
      bottomRadius: 16.0,
    },
    color: [0.2, 0.75, 0.4, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      packetType: 'tileset',
      format: 'b3dm',
    },
  };

  const originSphere = {
    id: 'czml-3dtiles-origin-sphere',
    name: 'CZML Document Reference Origin Sphere',
    show: true,
    positionDegrees: [-75.1528, 39.9513, 20.0],
    shape: {
      type: 'ellipsoid',
      radii: [14.0, 14.0, 14.0],
    },
    color: [0.3, 0.55, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      spatialReference: 'WGS84',
      czmlVersion: '1.0',
    },
  };

  const datasetRect = {
    id: 'czml-3dtiles-dataset-rect',
    name: 'BatchedColors CZML Footprint Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.156, 39.949, -75.148, 39.955],
    },
    color: [0.3, 0.25, 0.5, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.55, 0.9, 0.8],
    outlineWidth: 1.0,
    properties: {
      dataSource: 'CzmlDataSource',
      dynamicPackets: 2,
    },
  };

  const entities = [batchedBox, anchorPillar, originSphere, datasetRect];

  const variants = [
    {
      label: 'Baseline: BatchedColors 3D Tiles via CZML (Crimson Highlight)',
      entities: [
        batchedBox,
        anchorPillar,
        originSphere,
        datasetRect,
      ],
    },
    {
      label: 'Isolated Emerald Architectural Feature (Green Infill)',
      entities: [
        { ...batchedBox, color: [0.15, 0.85, 0.35, 0.95], outlineColor: [0.65, 1.0, 0.75, 1.0] },
        anchorPillar,
        originSphere,
        datasetRect,
      ],
    },
    {
      label: 'Translucent Batched Volume Inspection (Semi-Transparent Blue)',
      entities: [
        batchedBox,
        { ...anchorPillar, color: [0.25, 0.5, 0.95, 0.5], outlineColor: [0.6, 0.8, 1.0, 1.0] },
        originSphere,
        datasetRect,
      ],
    },
    {
      label: 'High-Contrast Boundary & Wireframe Shell (Gold Outlines)',
      entities: [
        batchedBox,
        anchorPillar,
        { ...originSphere, color: [0.95, 0.8, 0.2, 0.95], outlineColor: [1.0, 0.95, 0.6, 1.0] },
        { ...datasetRect, color: [0.45, 0.35, 0.1, 0.3] },
      ],
    },
  ];

  return { entities, variants };
}

// 3. Depth of Field (Eugene, Oregon - Balloon Constellation)
function generateDepthOfField() {
  const focalBox = {
    id: 'dof-focal-target-box',
    name: 'Focal Target Model Anchor Box',
    show: true,
    positionDegrees: [-122.9985, 44.05037, 107.5],
    shape: {
      type: 'box',
      dimensions: [24.0, 24.0, 36.0],
    },
    color: [0.95, 0.3, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      postProcessStage: 'DepthOfField',
      focalDistance: 87.0,
      sigma: 3.78,
    },
  };

  const kernelPillar = {
    id: 'dof-sampling-kernel-pillar',
    name: 'Gaussian Sampling Kernel Scale Pillar',
    show: true,
    positionDegrees: [-122.9975, 44.05037, 100.0],
    shape: {
      type: 'cylinder',
      length: 45.0,
      topRadius: 12.0,
      bottomRadius: 12.0,
    },
    color: [0.25, 0.7, 0.9, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      stepSize: 2.46,
      delta: 1.0,
    },
  };

  const blurProbe = {
    id: 'dof-blur-probe-sphere',
    name: 'Circle of Confusion Calibration Sphere',
    show: true,
    positionDegrees: [-122.9960, 44.05037, 100.0],
    shape: {
      type: 'ellipsoid',
      radii: [15.0, 15.0, 15.0],
    },
    color: [0.9, 0.75, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      circleOfConfusion: 'Gaussian16Tap',
      nearBlur: 0.85,
    },
  };

  const corridorRect = {
    id: 'dof-balloons-corridor-rect',
    name: 'Balloons Trajectory Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.001, 44.048, -122.994, 44.053],
    },
    color: [0.2, 0.45, 0.3, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.85, 0.65, 0.8],
    outlineWidth: 1.0,
    properties: {
      modelCount: 13,
      modelUri: 'CesiumBalloon.glb',
    },
  };

  const entities = [focalBox, kernelPillar, blurProbe, corridorRect];

  const variants = [
    {
      label: 'Baseline: Depth of Field Enabled (Focal Distance 87m & Scarlet Focus)',
      entities: [
        focalBox,
        kernelPillar,
        blurProbe,
        corridorRect,
      ],
    },
    {
      label: 'Near Field Focus Mode (Focal Distance 35m, High Blur Sigma)',
      entities: [
        { ...focalBox, color: [0.3, 0.85, 0.4, 0.95], outlineColor: [0.75, 1.0, 0.8, 1.0] },
        kernelPillar,
        blurProbe,
        corridorRect,
      ],
    },
    {
      label: 'Far Field Horizon Focus (Focal Distance 200m, Wide Step)',
      entities: [
        focalBox,
        { ...kernelPillar, color: [0.85, 0.35, 0.95, 0.95], outlineColor: [1.0, 0.7, 1.0, 1.0] },
        blurProbe,
        corridorRect,
      ],
    },
    {
      label: 'Bypass Depth of Field (Stage Disabled Pin-Sharp View)',
      entities: [
        focalBox,
        kernelPillar,
        { ...blurProbe, color: [0.15, 0.6, 0.95, 0.95], outlineColor: [0.65, 0.85, 1.0, 1.0] },
        { ...corridorRect, color: [0.35, 0.25, 0.45, 0.3] },
      ],
    },
  ];

  return { entities, variants };
}

// 4. Overlay Imagery on 3D Tiles (Philadelphia Reality Mesh)
function generateDrapeImageryOn3dTiles() {
  const meshBox = {
    id: 'drape-reality-mesh-box',
    name: 'Philadelphia Reality Mesh Architectural Anchor',
    show: true,
    positionDegrees: [-75.1630, 39.9520, 27.5],
    shape: {
      type: 'box',
      dimensions: [38.0, 38.0, 55.0],
    },
    color: [0.3, 0.65, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      realityMesh: 'Bentley iTwin RealityDataId',
      maximumScreenSpaceError: 2.0,
    },
  };

  const labelPillar = {
    id: 'drape-bing-label-pillar',
    name: 'Bing Maps Overlay Layer Calibration Pillar',
    show: true,
    positionDegrees: [-75.1622, 39.9528, 20.0],
    shape: {
      type: 'cylinder',
      length: 40.0,
      topRadius: 15.0,
      bottomRadius: 15.0,
    },
    color: [0.9, 0.6, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.85, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      drapedLayer: 'Bing Maps Labels',
      ionAssetId: 2411391,
    },
  };

  const georefProbe = {
    id: 'drape-georef-probe',
    name: 'Cartographic Drape Projection Probe',
    show: true,
    positionDegrees: [-75.1638, 39.9512, 18.0],
    shape: {
      type: 'ellipsoid',
      radii: [14.0, 14.0, 14.0],
    },
    color: [0.4, 0.85, 0.35, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.8, 1.0, 0.75, 1.0],
    outlineWidth: 1.0,
    properties: {
      projectionDrape: 'EllipsoidSurfaceAppearance',
      imageryLayersCount: 1,
    },
  };

  const phillyRect = {
    id: 'drape-philly-bounds-rect',
    name: 'Philadelphia Reality Data Bounds',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.168, 39.948, -75.158, 39.956],
    },
    color: [0.2, 0.25, 0.5, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.55, 0.9, 0.8],
    outlineWidth: 1.0,
    properties: {
      geographicFocus: 'Philadelphia City Hall',
      skyAtmosphere: true,
    },
  };

  const entities = [meshBox, labelPillar, georefProbe, phillyRect];

  const variants = [
    {
      label: 'Baseline: Philadelphia Reality Mesh with Draped Bing Labels',
      entities: [
        meshBox,
        labelPillar,
        georefProbe,
        phillyRect,
      ],
    },
    {
      label: 'Satellite Imagery Overlay Mode (Emerald Aerial Draping)',
      entities: [
        { ...meshBox, color: [0.2, 0.8, 0.45, 0.95], outlineColor: [0.65, 1.0, 0.8, 1.0] },
        labelPillar,
        georefProbe,
        phillyRect,
      ],
    },
    {
      label: 'Cadastral Vector Drape (Amber Parcel Highlighting)',
      entities: [
        meshBox,
        { ...labelPillar, color: [0.95, 0.45, 0.15, 0.95], outlineColor: [1.0, 0.75, 0.45, 1.0] },
        georefProbe,
        phillyRect,
      ],
    },
    {
      label: 'Untextured Raw Reality Mesh (Labels Only High-Contrast Inspection)',
      entities: [
        meshBox,
        labelPillar,
        { ...georefProbe, color: [0.85, 0.3, 0.8, 0.95], outlineColor: [1.0, 0.7, 0.95, 1.0] },
        { ...phillyRect, color: [0.35, 0.3, 0.15, 0.3] },
      ],
    },
  ];

  return { entities, variants };
}

function writeRecipe(id, data) {
  const jsonPath = path.join(outDir, `${id}.json`);
  const variantsPath = path.join(outDir, `${id}.variants.json`);

  fs.writeFileSync(jsonPath, JSON.stringify(data.entities, null, 2) + '\n');
  fs.writeFileSync(variantsPath, JSON.stringify({ initial: 0, variants: data.variants }, null, 2) + '\n');
  console.log(`Generated ${id}: ${data.entities.length} entities, ${data.variants.length} variants`);
}

writeRecipe('custom-shaders-property-textures', generateCustomShadersPropertyTextures());
writeRecipe('czml-3d-tiles', generateCzml3dTiles());
writeRecipe('depth-of-field', generateDepthOfField());
writeRecipe('drape-imagery-on-3d-tiles', generateDrapeImageryOn3dTiles());
