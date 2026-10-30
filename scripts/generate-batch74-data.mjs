import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Ground Primitive Materials - Dev
function generateGroundPrimitiveMaterialsDev() {
  const checkerBox = {
    id: 'ground-mat-checker-box',
    name: 'Checkerboard Draped Ground Primitive Landmark',
    show: true,
    positionDegrees: [-122.195, 46.200, 2000.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 80.0],
    },
    color: [0.95, 0.25, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.65, 0.65, 1.0],
    outlineWidth: 1.0,
    properties: {
      materialType: 'Checkerboard',
      supportsMaterials: true,
      classificationType: 'TERRAIN',
    },
  };

  const bumpCylinder = {
    id: 'ground-mat-bump-cylinder',
    name: 'BumpMap Normal Perturbation Draped Column',
    show: true,
    positionDegrees: [-122.185, 46.208, 1900.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 18.0,
      bottomRadius: 18.0,
    },
    color: [0.25, 0.85, 0.35, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 1.0, 0.75, 1.0],
    outlineWidth: 1.0,
    properties: {
      materialType: 'BumpMap',
      strength: 0.8,
    },
  };

  const waterSphere = {
    id: 'ground-mat-water-sphere',
    name: 'Water Surface Material Normal Wave Probe',
    show: true,
    positionDegrees: [-122.205, 46.192, 1850.0],
    shape: {
      type: 'ellipsoid',
      radii: [16.0, 16.0, 16.0],
    },
    color: [0.2, 0.6, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      materialType: 'Water',
      specularIntensity: 0.8,
    },
  };

  const drapeRect = {
    id: 'ground-mat-drape-rect',
    name: 'Mount St. Helens Ground Primitive Drape Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.22, 46.18, -122.17, 46.22],
    },
    color: [0.2, 0.35, 0.25, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.75, 0.55, 0.8],
    outlineWidth: 1.0,
    properties: {
      appearance: 'EllipsoidSurfaceAppearance',
    },
  };

  const entities = [checkerBox, bumpCylinder, waterSphere, drapeRect];

  const variants = [
    {
      label: 'Baseline: Checkerboard & BumpMap Materials on GroundPrimitives (Draped Grid)',
      entities: [
        checkerBox,
        bumpCylinder,
        waterSphere,
        drapeRect,
      ],
    },
    {
      label: 'Composite HeightField & Specular Terrain Infill (Alpine Bump Shading)',
      entities: [
        { ...checkerBox, color: [0.95, 0.8, 0.2, 0.95], outlineColor: [1.0, 0.9, 0.55, 1.0] },
        bumpCylinder,
        waterSphere,
        drapeRect,
      ],
    },
    {
      label: 'Dynamic Water Normal Ripples & WaterMask Drape (Azure Aquatic Flow)',
      entities: [
        checkerBox,
        { ...bumpCylinder, color: [0.15, 0.8, 0.85, 0.95], outlineColor: [0.6, 1.0, 1.0, 1.0] },
        waterSphere,
        drapeRect,
      ],
    },
    {
      label: 'High-Contrast Ground Elevation Contour Material (Amber Slope Bands)',
      entities: [
        checkerBox,
        bumpCylinder,
        { ...waterSphere, color: [0.95, 0.45, 0.15, 0.95], outlineColor: [1.0, 0.75, 0.5, 1.0] },
        { ...drapeRect, color: [0.35, 0.2, 0.2, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 2. High Dynamic Range
function generateHighDynamicRange() {
  const milktruckBox = {
    id: 'hdr-milktruck-box',
    name: 'Cesium Milk Truck HDR Benchmark Anchor',
    show: true,
    positionDegrees: [-111.822, 36.223, 1250.0],
    shape: {
      type: 'box',
      dimensions: [35.0, 35.0, 50.0],
    },
    color: [0.95, 0.85, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.65, 1.0],
    outlineWidth: 1.0,
    properties: {
      highDynamicRange: true,
      exposure: 1.0,
      tonemapper: 'PBR_NEUTRAL',
    },
  };

  const exposureCylinder = {
    id: 'hdr-exposure-cylinder',
    name: 'Dynamic Exposure Luminescence Column',
    show: true,
    positionDegrees: [-111.820, 36.225, 1230.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 16.0,
      bottomRadius: 16.0,
    },
    color: [0.2, 0.85, 0.75, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 1.0, 0.95, 1.0],
    outlineWidth: 1.0,
    properties: {
      tonemapper: 'ACES',
      shadows: true,
    },
  };

  const tonemapSphere = {
    id: 'hdr-tonemap-sphere',
    name: 'Specular Tone Preservation Probe',
    show: true,
    positionDegrees: [-111.824, 36.221, 1220.0],
    shape: {
      type: 'ellipsoid',
      radii: [15.0, 15.0, 15.0],
    },
    color: [0.95, 0.35, 0.45, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      tonemapper: 'REINHARD',
      gammaCorrected: true,
    },
  };

  const canyonRect = {
    id: 'hdr-canyon-rect',
    name: 'Grand Canyon Scenic HDR Lighting Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-111.84, 36.21, -111.80, 36.24],
    },
    color: [0.35, 0.25, 0.2, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.55, 0.45, 0.8],
    outlineWidth: 1.0,
    properties: {
      location: 'Grand Canyon North Rim',
    },
  };

  const entities = [milktruckBox, exposureCylinder, tonemapSphere, canyonRect];

  const variants = [
    {
      label: 'Baseline: PBR Neutral Tonemapping with HDR Pipeline (Exposure 1.0)',
      entities: [
        milktruckBox,
        exposureCylinder,
        tonemapSphere,
        canyonRect,
      ],
    },
    {
      label: 'ACES Cinematic Tonemapper High Exposure Mode (Vibrant Daylight)',
      entities: [
        { ...milktruckBox, color: [0.25, 0.75, 0.95, 0.95], outlineColor: [0.65, 0.9, 1.0, 1.0] },
        exposureCylinder,
        tonemapSphere,
        canyonRect,
      ],
    },
    {
      label: 'Reinhard Photographic Tonemapping Sunset (Warm Golden Glow)',
      entities: [
        milktruckBox,
        { ...exposureCylinder, color: [0.95, 0.6, 0.15, 0.95], outlineColor: [1.0, 0.85, 0.5, 1.0] },
        tonemapSphere,
        canyonRect,
      ],
    },
    {
      label: 'Filmic Curve Contrast Tone Preservation (Deep Canyon Shadows)',
      entities: [
        milktruckBox,
        exposureCylinder,
        { ...tonemapSphere, color: [0.3, 0.9, 0.5, 0.95], outlineColor: [0.7, 1.0, 0.8, 1.0] },
        { ...canyonRect, color: [0.25, 0.2, 0.3, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 3. Hybrid Snapping - Dev
function generateHybridSnappingDev() {
  const bimBox = {
    id: 'hybrid-snap-bim-box',
    name: 'BIM CAD Database Asset Target Block',
    show: true,
    positionDegrees: [-75.1652, 39.9526, 120.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 80.0],
    },
    color: [0.2, 0.85, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      assetId: 5161569,
      hoverEdgeColor: '#00ffff',
      hoverSurfaceColor: '#ffff00',
    },
  };

  const clientCylinder = {
    id: 'hybrid-snap-client-cylinder',
    name: 'Client-Side GPU Scene Snap Responsive Pillar',
    show: true,
    positionDegrees: [-75.1640, 39.9535, 100.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 18.0,
      bottomRadius: 18.0,
    },
    color: [0.4, 0.95, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 1.0, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      clientColor: '#00ff00',
      snapMode: 'NEAREST',
      instantaneous: true,
    },
  };

  const serverSphere = {
    id: 'hybrid-snap-server-sphere',
    name: 'Authoritative Server Precision Snap Point',
    show: true,
    positionDegrees: [-75.1665, 39.9515, 80.0],
    shape: {
      type: 'ellipsoid',
      radii: [16.0, 16.0, 16.0],
    },
    color: [0.95, 0.2, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      serverColor: '#ff0000',
      precisionSourceGeometry: true,
    },
  };

  const stageRect = {
    id: 'hybrid-snap-stage-rect',
    name: 'Hybrid Snapping Measurement Footprint Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.280, 39.867, -74.956, 40.138],
    },
    color: [0.15, 0.25, 0.35, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.65, 0.85, 0.8],
    outlineWidth: 1.0,
    properties: {
      pickTranslucentDepth: true,
    },
  };

  const entities = [bimBox, clientCylinder, serverSphere, stageRect];

  const variants = [
    {
      label: 'Baseline: Client Hover Snap & Authoritative Server Click Verification (Cyan & Lime)',
      entities: [
        bimBox,
        clientCylinder,
        serverSphere,
        stageRect,
      ],
    },
    {
      label: 'Pure Surface Snapping Diagnostics (Yellow Surface Projection)',
      entities: [
        { ...bimBox, color: [0.95, 0.9, 0.15, 0.95], outlineColor: [1.0, 0.95, 0.55, 1.0] },
        clientCylinder,
        serverSphere,
        stageRect,
      ],
    },
    {
      label: 'Edge-Only Geometry Snapping Mode (High-Precision Boundary Creases)',
      entities: [
        bimBox,
        { ...clientCylinder, color: [0.15, 0.75, 0.95, 0.95], outlineColor: [0.6, 0.9, 1.0, 1.0] },
        serverSphere,
        stageRect,
      ],
    },
    {
      label: 'Deviation Measurement & Tolerance Envelope (Amber Error Vectors)',
      entities: [
        bimBox,
        clientCylinder,
        { ...serverSphere, color: [0.95, 0.5, 0.1, 0.95], outlineColor: [1.0, 0.75, 0.45, 1.0] },
        { ...stageRect, color: [0.35, 0.2, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 4. I3S Consumption Cesium Demo (3D Object Layer)
function generateI3s3dObjectLayer() {
  const sfBox = {
    id: 'i3s-sf-bldg-box',
    name: 'San Francisco Downtown I3S 3D Object Landmark',
    show: true,
    positionDegrees: [-122.401, 37.792, 100.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 80.0],
    },
    color: [0.25, 0.55, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      layerType: '3DObject',
      i3sVersion: '1.7',
      service: 'SanFrancisco_3DObjects_1_7',
    },
  };

  const geoidCylinder = {
    id: 'i3s-geoid-datum-cylinder',
    name: 'EGM2008 Geoid Gravity Elevation Conversion Pillar',
    show: true,
    positionDegrees: [-122.399, 37.794, 90.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 18.0,
      bottomRadius: 18.0,
    },
    color: [0.3, 0.85, 0.45, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 1.0, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      geoidService: 'EGM2008',
      provider: 'ArcGISTiledElevationTerrainProvider',
    },
  };

  const featureSphere = {
    id: 'i3s-feature-probe-sphere',
    name: 'I3S Node Field Attribute Picking Centroid Probe',
    show: true,
    positionDegrees: [-122.403, 37.790, 80.0],
    shape: {
      type: 'ellipsoid',
      radii: [16.0, 16.0, 16.0],
    },
    color: [0.95, 0.7, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.55, 1.0],
    outlineWidth: 1.0,
    properties: {
      infoBoxEnabled: true,
      fieldsCount: 12,
    },
  };

  const extentRect = {
    id: 'i3s-layer-extent-rect',
    name: 'San Francisco Financial District SceneServer Bounds',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.42, 37.78, -122.38, 37.81],
    },
    color: [0.15, 0.25, 0.4, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.65, 0.85, 0.8],
    outlineWidth: 1.0,
    properties: {
      dataProvider: 'I3SDataProvider',
    },
  };

  const entities = [sfBox, geoidCylinder, featureSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: San Francisco 3D Object Scene Layer with EGM2008 Geoid Datum',
      entities: [
        sfBox,
        geoidCylinder,
        featureSphere,
        extentRect,
      ],
    },
    {
      label: 'Attribute Field Inspection & Metadata Picking (Golden Feature Table)',
      entities: [
        { ...sfBox, color: [0.95, 0.8, 0.2, 0.95], outlineColor: [1.0, 0.9, 0.6, 1.0] },
        geoidCylinder,
        featureSphere,
        extentRect,
      ],
    },
    {
      label: 'Integrated Gravity Height Model Conversion (Emerald Vertical Datum)',
      entities: [
        sfBox,
        { ...geoidCylinder, color: [0.2, 0.85, 0.8, 0.95], outlineColor: [0.65, 1.0, 1.0, 1.0] },
        featureSphere,
        extentRect,
      ],
    },
    {
      label: 'Downtown Financial District High-Rise Boundary (Crimson Extent Outline)',
      entities: [
        sfBox,
        geoidCylinder,
        { ...featureSphere, color: [0.9, 0.25, 0.35, 0.95], outlineColor: [1.0, 0.6, 0.7, 1.0] },
        { ...extentRect, color: [0.35, 0.15, 0.25, 0.35] },
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

writeRecipe('ground-primitive-materials-dev', generateGroundPrimitiveMaterialsDev());
writeRecipe('high-dynamic-range', generateHighDynamicRange());
writeRecipe('hybrid-snapping-dev', generateHybridSnappingDev());
writeRecipe('i3s-3d-object-layer', generateI3s3dObjectLayer());
