import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. KHR_mesh_primitive_restart - Dev
function generateKhrMeshPrimitiveRestartDev() {
  const restartBox = {
    id: 'khr-restart-hilbert-box',
    name: 'Hilbert Curve Primitive Restart Topology Anchor Box',
    show: true,
    positionDegrees: [-75.152, 39.947, 50.0],
    shape: {
      type: 'box',
      dimensions: [35.0, 35.0, 50.0],
    },
    color: [0.2, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      extension: 'KHR_mesh_primitive_restart',
      indicesBitDepth: 32,
      restartIndex: '0xFFFFFFFF',
      topology: 'LINE_STRIP',
    },
  };

  const stripCylinder = {
    id: 'khr-restart-strip-cylinder',
    name: 'Batched Line Strip Topology Connector Column',
    show: true,
    positionDegrees: [-75.149, 39.950, 40.0],
    shape: {
      type: 'cylinder',
      length: 55.0,
      topRadius: 7.0,
      bottomRadius: 7.0,
    },
    color: [0.95, 0.45, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      topologyType: 'LineStrips',
      webgl2FixedIndex: true,
    },
  };

  const fanSphere = {
    id: 'khr-restart-fan-sphere',
    name: 'Triangle Fan / Loop Disjoint Mesh Sensor Probe',
    show: true,
    positionDegrees: [-75.155, 39.944, 45.0],
    shape: {
      type: 'ellipsoid',
      radii: [5.5, 5.5, 5.5],
    },
    color: [0.35, 0.9, 0.45, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 1.0, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      fanTopology: 'TriangleFan',
      restartValue16: '0xFFFF',
    },
  };

  const extentRect = {
    id: 'khr-restart-bounds-rect',
    name: 'glTF Batched Primitive Geodetic Boundary Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.165, 39.935, -75.140, 39.958],
    },
    color: [0.25, 0.35, 0.45, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.85, 0.7, 0.95, 0.9],
    outlineWidth: 1.0,
    properties: {
      modelMatrixOrigin: 'FixedFrame',
    },
  };

  const entities = [restartBox, stripCylinder, fanSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Hilbert Curve Primitive Restart (32-bit Indices LINE_STRIP)',
      entities: [restartBox, stripCylinder, fanSphere, extentRect],
    },
    {
      label: 'Line Loops Multiple Disjoint Topologies Mode (Inline Restart 0xFFFF)',
      entities: [
        { ...restartBox, color: [0.95, 0.85, 0.25, 0.95], outlineColor: [1.0, 0.95, 0.7, 1.0] },
        stripCylinder,
        fanSphere,
        extentRect,
      ],
    },
    {
      label: 'Triangle Strips Batched Geometry Mode (Triangulated Strip Restart)',
      entities: [
        restartBox,
        { ...stripCylinder, color: [0.85, 0.25, 0.95, 0.95], outlineColor: [1.0, 0.7, 1.0, 1.0] },
        fanSphere,
        extentRect,
      ],
    },
    {
      label: 'Triangle Fans Disjoint Topologies Inspection (Fixed Index Verification)',
      entities: [
        restartBox,
        stripCylinder,
        { ...fanSphere, color: [0.25, 0.85, 0.85, 0.95], outlineColor: [0.7, 1.0, 1.0, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 2. LensFlare
function generateLensFlare() {
  const sunBox = {
    id: 'lensflare-sun-anchor-box',
    name: 'Solar Flare Direct Sightline Anchor Box',
    show: true,
    positionDegrees: [0.0, 0.0, 2000000.0],
    shape: {
      type: 'box',
      dimensions: [500000.0, 500000.0, 500000.0],
    },
    color: [1.0, 0.95, 0.4, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      postProcessStage: 'LensFlare',
      intensity: 2.0,
      distortion: 10.0,
    },
  };

  const haloCylinder = {
    id: 'lensflare-halo-dispersion-cylinder',
    name: 'Chromatic Ghost Dispersion Ring Datum Column',
    show: true,
    positionDegrees: [15.0, 10.0, 1500000.0],
    shape: {
      type: 'cylinder',
      length: 600000.0,
      topRadius: 120000.0,
      bottomRadius: 120000.0,
    },
    color: [0.95, 0.35, 0.75, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: {
      ghostDispersal: 0.4,
      haloWidth: 0.4,
    },
  };

  const dirtSphere = {
    id: 'lensflare-optical-dirt-sphere',
    name: 'Optical Lens Dirt Screen Artifact Probe',
    show: true,
    positionDegrees: [-15.0, -10.0, 1800000.0],
    shape: {
      type: 'ellipsoid',
      radii: [150000.0, 150000.0, 150000.0],
    },
    color: [0.35, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      dirtAmount: 0.4,
      earthRadius: 6378137.0,
    },
  };

  const extentRect = {
    id: 'lensflare-celestial-view-rect',
    name: 'Deep Space Atmospheric Limb Celestial Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-0.5, -0.5, 0.5, 0.5],
    },
    color: [0.2, 0.25, 0.4, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.95, 0.85, 0.4, 0.9],
    outlineWidth: 1.0,
    properties: {
      cameraPositionMagnitude: 71000000.0,
    },
  };

  const entities = [sunBox, haloCylinder, dirtSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Solar Flare Post-Processing Stage (Intensity 2.0, Distortion 10.0)',
      entities: [sunBox, haloCylinder, dirtSphere, extentRect],
    },
    {
      label: 'High Dispersion Rainbow Halo Mode (Ghost Dispersal 0.8, Halo Width 0.8)',
      entities: [
        { ...sunBox, color: [0.95, 0.55, 0.2, 0.95], outlineColor: [1.0, 0.8, 0.6, 1.0] },
        haloCylinder,
        dirtSphere,
        extentRect,
      ],
    },
    {
      label: 'Heavy Optical Surface Dirt Simulation (Dirt Amount 0.9 Lens Texture)',
      entities: [
        sunBox,
        { ...haloCylinder, color: [0.25, 0.85, 0.55, 0.95], outlineColor: [0.75, 1.0, 0.85, 1.0] },
        dirtSphere,
        extentRect,
      ],
    },
    {
      label: 'Subtle Atmospheric Limb Glare (Low Intensity 0.8 Realistic Bloom)',
      entities: [
        sunBox,
        haloCylinder,
        { ...dirtSphere, color: [0.85, 0.35, 0.95, 0.95], outlineColor: [1.0, 0.75, 1.0, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.35, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 3. Many Clipping Planes - Dev
function generateManyClippingPlanesDev() {
  const centerBox = {
    id: 'many-clip-center-model-box',
    name: '32-Plane Radial Sliced Target Model Anchor Box',
    show: true,
    positionDegrees: [-123.074, 44.050, 300.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 50.0],
    },
    color: [0.25, 0.85, 0.55, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 1.0, 0.85, 1.0],
    outlineWidth: 1.0,
    properties: {
      steps: 32,
      cylinderRadius: -20.0,
      clippingPlanesCount: 32,
    },
  };

  const radialCylinder = {
    id: 'many-clip-radial-cylinder',
    name: 'Radial Clipping Plane Enclosing Cylinder Datum Column',
    show: true,
    positionDegrees: [-123.072, 44.052, 280.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 20.0,
      bottomRadius: 20.0,
    },
    color: [0.95, 0.3, 0.3, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      radiusMultiplier: 1.0,
      clippingModeUnion: false,
    },
  };

  const normalSphere = {
    id: 'many-clip-normal-sphere',
    name: 'Clipping Plane Unit Normal Tangent Direction Probe',
    show: true,
    positionDegrees: [-123.076, 44.048, 290.0],
    shape: {
      type: 'ellipsoid',
      radii: [5.5, 5.5, 5.5],
    },
    color: [0.25, 0.65, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      stepDegrees: 'TWO_PI / 32',
      depthTestAgainstTerrain: true,
    },
  };

  const extentRect = {
    id: 'many-clip-cadastral-rect',
    name: 'Clipping Plane Facility Cadastral Footprint Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.085, 44.040, -123.060, 44.060],
    },
    color: [0.35, 0.35, 0.25, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.95, 0.85, 0.35, 0.9],
    outlineWidth: 1.0,
    properties: {
      clipObjects: ['model', 'b3dm', 'pnts', 'i3dm', 'terrain'],
    },
  };

  const entities = [centerBox, radialCylinder, normalSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: 32 Radial Clipping Planes Interior Cylinder Cross Section',
      entities: [centerBox, radialCylinder, normalSphere, extentRect],
    },
    {
      label: 'High-Density 64 Clipping Planes Faceted Precision Borehole Mode',
      entities: [
        { ...centerBox, color: [0.95, 0.75, 0.2, 0.95], outlineColor: [1.0, 0.9, 0.6, 1.0] },
        radialCylinder,
        normalSphere,
        extentRect,
      ],
    },
    {
      label: 'Clipping Mode Union Outer Shell Inversion (Excavation Inversion Mode)',
      entities: [
        centerBox,
        { ...radialCylinder, color: [0.85, 0.25, 0.95, 0.95], outlineColor: [1.0, 0.7, 1.0, 1.0] },
        normalSphere,
        extentRect,
      ],
    },
    {
      label: 'Terrain Subsurface Radial Slice Envelope (Pivoted Slicing Trench)',
      entities: [
        centerBox,
        radialCylinder,
        { ...normalSphere, color: [0.3, 0.9, 0.8, 0.95], outlineColor: [0.75, 1.0, 0.95, 1.0] },
        { ...extentRect, color: [0.25, 0.35, 0.3, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 4. Mars
function generateMars() {
  const jezeroBox = {
    id: 'mars-jezero-crater-box',
    name: 'Perseverance Jezero Crater Delta Landmark Anchor Box',
    show: true,
    positionDegrees: [77.450, 18.380, 1500.0],
    shape: {
      type: 'box',
      dimensions: [45.0, 45.0, 60.0],
    },
    color: [0.85, 0.35, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.65, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      ellipsoid: 'Ellipsoid.MARS',
      rover: 'Perseverance',
      crater: 'Jezero',
    },
  };

  const atmosphereCylinder = {
    id: 'mars-atmosphere-datum-cylinder',
    name: 'Martian Rayleigh / Mie Atmospheric Radiance Column',
    show: true,
    positionDegrees: [77.460, 18.390, 1200.0],
    shape: {
      type: 'cylinder',
      length: 70.0,
      topRadius: 8.0,
      bottomRadius: 8.0,
    },
    color: [0.95, 0.55, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.8, 0.65, 1.0],
    outlineWidth: 1.0,
    properties: {
      rayleighScaleHeight: 9000,
      mieScaleHeight: 2700.0,
      saturationShift: -0.1,
    },
  };

  const galeSphere = {
    id: 'mars-gale-curiosity-sphere',
    name: 'Curiosity Rover Gale Crater Mt. Sharp Sensor Probe',
    show: true,
    positionDegrees: [137.440, -4.590, 1000.0],
    shape: {
      type: 'ellipsoid',
      radii: [7.0, 7.0, 7.0],
    },
    color: [0.9, 0.2, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      rover: 'Curiosity',
      crater: 'Gale Crater',
      ionAssetId: 3644333,
    },
  };

  const extentRect = {
    id: 'mars-equatorial-survey-rect',
    name: 'NASA Mars Exploration Rover Survey Cadastral Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [77.400, 18.350, 77.500, 18.420],
    },
    color: [0.45, 0.25, 0.2, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.95, 0.45, 0.3, 0.9],
    outlineWidth: 1.0,
    properties: {
      martianJourney: 'The Martian (Andy Weir)',
    },
  };

  const entities = [jezeroBox, atmosphereCylinder, galeSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Mars 3D Tiles Terrain with Martian Atmosphere (Jezero Crater)',
      entities: [jezeroBox, atmosphereCylinder, galeSphere, extentRect],
    },
    {
      label: 'Gale Crater Curiosity Rover Traverse Path (Mt. Sharp Exploration)',
      entities: [
        { ...jezeroBox, color: [0.95, 0.65, 0.15, 0.95], outlineColor: [1.0, 0.85, 0.55, 1.0] },
        atmosphereCylinder,
        galeSphere,
        extentRect,
      ],
    },
    {
      label: 'Perseverance & Ingenuity Aerial Scout Flight Envelope (Jezero Delta)',
      entities: [
        jezeroBox,
        { ...atmosphereCylinder, color: [0.35, 0.75, 0.95, 0.95], outlineColor: [0.75, 0.95, 1.0, 1.0] },
        galeSphere,
        extentRect,
      ],
    },
    {
      label: 'The Martian Ares III to Schiaparelli Cross-Continental Expedition',
      entities: [
        jezeroBox,
        atmosphereCylinder,
        { ...galeSphere, color: [0.85, 0.25, 0.95, 0.95], outlineColor: [1.0, 0.7, 1.0, 1.0] },
        { ...extentRect, color: [0.35, 0.2, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// Generate all Batch 78 fixtures
const datasets = {
  'khr-mesh-primitive-restart-dev': generateKhrMeshPrimitiveRestartDev(),
  'lensflare': generateLensFlare(),
  'many-clipping-planes-dev': generateManyClippingPlanesDev(),
  'mars': generateMars(),
};

for (const [id, { entities, variants }] of Object.entries(datasets)) {
  const jsonPath = path.join(outDir, `${id}.json`);
  const variantsPath = path.join(outDir, `${id}.variants.json`);

  fs.writeFileSync(jsonPath, JSON.stringify(entities, null, 2) + '\n');
  fs.writeFileSync(variantsPath, JSON.stringify({ initial: 0, variants }, null, 2) + '\n');
  console.log(`Generated ${id}.json (${entities.length}) and ${id}.variants.json (${variants.length})`);
}
