import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Panorama
function generatePanorama() {
  const domeSphere = {
    id: 'panorama-equirectangular-dome',
    name: 'Equirectangular 360 Panoramic Projection Sphere',
    show: true,
    positionDegrees: [-122.4175, 37.655, 100.0],
    shape: {
      type: 'ellipsoid',
      radii: [40.0, 40.0, 40.0],
    },
    color: [0.35, 0.75, 0.95, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.8, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      panoramaType: 'EQUIRECTANGULAR',
      sourceImage: 'Laon_Cathedral_Interior_360x180.jpg',
      fovDegrees: 60.0,
    },
  };

  const cubemapBox = {
    id: 'panorama-cubemap-box',
    name: 'CubeMap 6-Sided Cubic Skybox Projection Volume',
    show: true,
    positionDegrees: [-122.4168, 37.6558, 95.0],
    shape: {
      type: 'box',
      dimensions: [50.0, 50.0, 50.0],
    },
    color: [0.95, 0.65, 0.25, 0.85],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.85, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      panoramaType: 'CUBEMAP',
      faces: ['positiveX', 'negativeX', 'positiveY', 'negativeY', 'positiveZ', 'negativeZ'],
      location: 'Phnom Penh Central Market',
    },
  };

  const targetPole = {
    id: 'panorama-camera-target-marker',
    name: 'Panorama Camera Center Azimuth Datum Column',
    show: true,
    positionDegrees: [-122.4172, 37.6553, 90.0],
    shape: {
      type: 'cylinder',
      length: 30.0,
      topRadius: 4.0,
      bottomRadius: 4.0,
    },
    color: [0.85, 0.25, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      cameraPitchRange: [0, 0, 2],
      zoomSpeed: 0.05,
      minFov: 20.0,
      maxFov: 100.0,
    },
  };

  const extentRect = {
    id: 'panorama-extent-envelope-rect',
    name: 'Panorama Interactive Inspection Cadastral Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.430, 37.645, -122.405, 37.665],
    },
    color: [0.2, 0.3, 0.45, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.8, 1.0, 0.85],
    outlineWidth: 1.0,
    properties: {
      enableZoomControl: true,
      enableTranslateControl: false,
    },
  };

  const entities = [domeSphere, cubemapBox, targetPole, extentRect];

  const variants = [
    {
      label: 'Baseline: Equirectangular 360 Panorama (Laon Cathedral Interior, Standard 60° FOV)',
      entities: [
        domeSphere,
        { ...cubemapBox, show: false },
        targetPole,
        extentRect,
      ],
    },
    {
      label: 'Cube Map Skybox Panorama Mode (Phnom Penh Central Market, 6-Sided Cubic Frame)',
      entities: [
        { ...domeSphere, show: false },
        { ...cubemapBox, show: true, color: [1.0, 0.75, 0.35, 0.95] },
        targetPole,
        extentRect,
      ],
    },
    {
      label: 'Narrow Telephoto Optical Zoom Mode (FOV 25°, High-Precision Architectural Detail)',
      entities: [
        {
          ...domeSphere,
          show: true,
          shape: { type: 'ellipsoid', radii: [25.0, 25.0, 25.0] },
          color: [0.25, 0.95, 0.85, 0.95],
        },
        { ...cubemapBox, show: false },
        targetPole,
        extentRect,
      ],
    },
    {
      label: 'Ultra-Wide Immersive Panorama Mode (FOV 95°, Expanded Environmental Peripheral View)',
      entities: [
        {
          ...domeSphere,
          show: true,
          shape: { type: 'ellipsoid', radii: [55.0, 55.0, 55.0] },
          color: [0.45, 0.65, 1.0, 0.9],
        },
        cubemapBox,
        targetPole,
        extentRect,
      ],
    },
  ];

  return { entities, variants };
}

// 2. Particle System
function generateParticleSystem() {
  const truckBox = {
    id: 'particle-milktruck-carrier-box',
    name: 'Sample Milk Truck Dynamic Model Bounding Hull',
    show: true,
    positionDegrees: [-75.1606, 39.9660, 20.0],
    shape: {
      type: 'box',
      dimensions: [12.0, 6.0, 7.0],
    },
    color: [0.25, 0.85, 0.75, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: {
      modelUri: 'CesiumMilkTruck.glb',
      speedKmh: 45.0,
      sampleStart: '2015-03-25T16:00:00Z',
    },
  };

  const emitterCylinder = {
    id: 'particle-emitter-circle-cylinder',
    name: 'Circle Emitter Physical Nozzle Boundary Column',
    show: true,
    positionDegrees: [-75.1610, 39.9663, 22.0],
    shape: {
      type: 'cylinder',
      length: 8.0,
      topRadius: 4.0,
      bottomRadius: 4.0,
    },
    color: [0.2, 0.8, 0.6, 0.85],
    fill: true,
    outline: true,
    outlineColor: [0.7, 1.0, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      emitterType: 'CircleEmitter',
      radius: 2.0,
      emissionRate: 5.0,
      gravity: 0.0,
    },
  };

  const plumeEllipsoid = {
    id: 'particle-burst-plume-sphere',
    name: 'Multi-Burst Smoke Plume Expansion Volume',
    show: true,
    positionDegrees: [-75.1615, 39.9668, 28.0],
    shape: {
      type: 'ellipsoid',
      radii: [16.0, 16.0, 16.0],
    },
    color: [0.9, 0.95, 0.95, 0.7],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: {
      startScale: 1.0,
      endScale: 5.0,
      particleSize: 25.0,
      bursts: [10, 50, 200],
    },
  };

  const trajectoryRect = {
    id: 'particle-truck-trajectory-rect',
    name: 'Sampled Position Corridor Cadastral Bounds',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.175, 39.945, -75.150, 39.985],
    },
    color: [0.15, 0.35, 0.3, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.8, 0.6, 0.8],
    outlineWidth: 1.0,
    properties: {
      startCoord: [-75.15787, 39.97862],
      stopCoord: [-75.16336, 39.95355],
    },
  };

  const entities = [truckBox, emitterCylinder, plumeEllipsoid, trajectoryRect];

  const variants = [
    {
      label: 'Baseline: Circle Emitter Smoke Plume (Standard Light Sea Green to White Exhaust)',
      entities: [
        truckBox,
        emitterCylinder,
        plumeEllipsoid,
        trajectoryRect,
      ],
    },
    {
      label: 'Volumetric Sphere Emitter Mode (Dense Omni-Directional 25.0 Rate Emission)',
      entities: [
        truckBox,
        {
          ...emitterCylinder,
          shape: { type: 'cylinder', length: 12.0, topRadius: 6.0, bottomRadius: 6.0 },
          color: [0.95, 0.75, 0.2, 0.9],
          properties: { ...emitterCylinder.properties, emitterType: 'SphereEmitter', radius: 2.5, emissionRate: 25.0 },
        },
        {
          ...plumeEllipsoid,
          color: [1.0, 0.85, 0.4, 0.8],
          shape: { type: 'ellipsoid', radii: [22.0, 22.0, 22.0] },
        },
        trajectoryRect,
      ],
    },
    {
      label: 'High-Velocity Directional Cone Jet (Narrow 45° Cone, Maximum Speed 8.0 m/s)',
      entities: [
        truckBox,
        {
          ...emitterCylinder,
          color: [0.3, 0.7, 1.0, 0.9],
          properties: { ...emitterCylinder.properties, emitterType: 'ConeEmitter', angleRad: 0.785, maximumSpeed: 8.0 },
        },
        {
          ...plumeEllipsoid,
          color: [0.6, 0.85, 1.0, 0.75],
          shape: { type: 'ellipsoid', radii: [12.0, 12.0, 26.0] },
        },
        trajectoryRect,
      ],
    },
    {
      label: 'Gravitational Deflection Plume Mode (Downwards Vector Gravity -9.8 m/s²)',
      entities: [
        truckBox,
        emitterCylinder,
        {
          ...plumeEllipsoid,
          positionDegrees: [-75.1615, 39.9668, 16.0],
          color: [0.55, 0.45, 0.75, 0.85],
          properties: { ...plumeEllipsoid.properties, gravity: -9.8 },
        },
        trajectoryRect,
      ],
    },
  ];

  return { entities, variants };
}

// 3. Particle System Fireworks
function generateParticleSystemFireworks() {
  const burstSphere = {
    id: 'fireworks-core-explosion-sphere',
    name: 'Multi-Burst Golden Firework Explosion Core Sphere',
    show: true,
    positionDegrees: [-75.59777, 40.03883, 110.0],
    shape: {
      type: 'ellipsoid',
      radii: [30.0, 30.0, 30.0],
    },
    color: [1.0, 0.85, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      burstCount: 20,
      burstSize: 400,
      particlePixelSize: [7.0, 7.0],
      lifetime: 10.0,
    },
  };

  const salvoClusterBox = {
    id: 'fireworks-salvo-cluster-box',
    name: 'High-Altitude Emerald and Cyan Salvo Cluster Box',
    show: true,
    positionDegrees: [-75.5968, 40.0395, 135.0],
    shape: {
      type: 'box',
      dimensions: [55.0, 55.0, 55.0],
    },
    color: [0.15, 0.95, 0.75, 0.85],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: {
      colorPalette: 'EmeraldCyan',
      minLife: 0.3,
      maxLife: 1.0,
    },
  };

  const launchPadCylinder = {
    id: 'fireworks-launch-pad-cylinder',
    name: 'Ground Level Pyrotechnic Mortar Battery Column',
    show: true,
    positionDegrees: [-75.59777, 40.03883, 20.0],
    shape: {
      type: 'cylinder',
      length: 40.0,
      topRadius: 8.0,
      bottomRadius: 10.0,
    },
    color: [0.85, 0.25, 0.35, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.5, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      launchAngleRad: 1.57,
      mortarCount: 20,
      cameraLookAtDistance: 300.0,
    },
  };

  const firingZoneRect = {
    id: 'fireworks-firing-zone-rect',
    name: 'Pyrotechnic Safety Exclusion Cadastral Zone',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.605, 40.032, -75.590, 40.045],
    },
    color: [0.25, 0.15, 0.35, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.9, 0.5, 1.0, 0.8],
    outlineWidth: 1.0,
    properties: {
      exclusionRadiusMeters: 500.0,
    },
  };

  const entities = [burstSphere, salvoClusterBox, launchPadCylinder, firingZoneRect];

  const variants = [
    {
      label: 'Baseline: Multi-Stage Pyrotechnic Salvo (20 Fireworks with Golden and Magenta Bursts)',
      entities: [
        burstSphere,
        salvoClusterBox,
        launchPadCylinder,
        firingZoneRect,
      ],
    },
    {
      label: 'High-Altitude Emerald & Cyan Starbursts (Z-Axis +150m, Cool Luminescent Hues)',
      entities: [
        { ...burstSphere, show: false },
        {
          ...salvoClusterBox,
          positionDegrees: [-75.5968, 40.0395, 160.0],
          shape: { type: 'box', dimensions: [70.0, 70.0, 70.0] },
          color: [0.2, 1.0, 0.9, 0.95],
        },
        launchPadCylinder,
        firingZoneRect,
      ],
    },
    {
      label: 'Rapid-Fire Staccato Finale Mode (Burst Size 800, Short 4.0s Lifetime, Maximum Density)',
      entities: [
        {
          ...burstSphere,
          shape: { type: 'ellipsoid', radii: [45.0, 45.0, 45.0] },
          color: [1.0, 0.35, 0.15, 0.95],
          properties: { ...burstSphere.properties, burstSize: 800, lifetime: 4.0 },
        },
        {
          ...salvoClusterBox,
          color: [1.0, 0.85, 0.2, 0.95],
        },
        launchPadCylinder,
        firingZoneRect,
      ],
    },
    {
      label: 'Ground-Level Heavy Mortar Single Bloom (Massive 120m Explosion Radius)',
      entities: [
        {
          ...burstSphere,
          positionDegrees: [-75.59777, 40.03883, 75.0],
          shape: { type: 'ellipsoid', radii: [60.0, 60.0, 60.0] },
          color: [0.95, 0.15, 0.85, 0.9],
        },
        { ...salvoClusterBox, show: false },
        launchPadCylinder,
        firingZoneRect,
      ],
    },
  ];

  return { entities, variants };
}

// 4. Particle System Tails
function generateParticleSystemTails() {
  const aircraftBox = {
    id: 'tails-aircraft-model-box',
    name: 'Cesium_Air Supersonic Jet Entity Model Boundary Hull',
    show: true,
    positionDegrees: [-75.59777, 40.03883, 800.0],
    shape: {
      type: 'box',
      dimensions: [24.0, 20.0, 8.0],
    },
    color: [0.85, 0.9, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      modelUri: 'Cesium_Air.glb',
      scale: 3.5,
      altitudeMeters: 800.0,
      pitchRollRad: [0.0, 1.570796, 0.0],
    },
  };

  const cometCylinder = {
    id: 'tails-comet-stream-cylinder',
    name: 'Comet Tail Cryogenic Stream Ribbon Boundary',
    show: true,
    positionDegrees: [-75.5979, 40.0392, 795.0],
    shape: {
      type: 'cylinder',
      length: 80.0,
      topRadius: 4.0,
      bottomRadius: 12.0,
    },
    color: [0.55, 0.75, 1.0, 0.85],
    fill: true,
    outline: true,
    outlineColor: [0.8, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      mode: 'Comet Tail',
      numberOfSystems: 100,
      colorPalette: 'IcyBlueSilver',
      speed: 0.00005,
    },
  };

  const rocketThrusterSphere = {
    id: 'tails-rocket-thruster-sphere',
    name: 'Rocket Thruster Thermal Afterburner Core Volume',
    show: true,
    positionDegrees: [-75.5976, 40.0385, 792.0],
    shape: {
      type: 'ellipsoid',
      radii: [16.0, 16.0, 22.0],
    },
    color: [1.0, 0.45, 0.1, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.8, 0.3, 1.0],
    outlineWidth: 1.0,
    properties: {
      mode: 'Rocket Thruster',
      numberOfSystems: 50,
      colorPalette: 'ThermalGoldOrange',
      emissionRate: 30.0,
    },
  };

  const flightCorridorRect = {
    id: 'tails-flight-corridor-rect',
    name: 'Supersonic Aircraft Flight Corridor Cadastral Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.606, 40.030, -75.589, 40.046],
    },
    color: [0.15, 0.25, 0.45, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.7, 1.0, 0.8],
    outlineWidth: 1.0,
    properties: {
      cameraLookAtOffset: [-450, -300, 200],
    },
  };

  const entities = [aircraftBox, cometCylinder, rocketThrusterSphere, flightCorridorRect];

  const variants = [
    {
      label: 'Baseline: Comet Tail Cryogenic Particle Ribbon (100 Icy Blue / Silver Systems)',
      entities: [
        aircraftBox,
        cometCylinder,
        { ...rocketThrusterSphere, show: false },
        flightCorridorRect,
      ],
    },
    {
      label: 'Rocket Thruster Thermal Afterburner Mode (50 Fiery Gold / Crimson High-Temp Systems)',
      entities: [
        aircraftBox,
        { ...cometCylinder, show: false },
        {
          ...rocketThrusterSphere,
          show: true,
          color: [1.0, 0.3, 0.05, 0.95],
          shape: { type: 'ellipsoid', radii: [20.0, 20.0, 30.0] },
        },
        flightCorridorRect,
      ],
    },
    {
      label: 'Dual Wingtip Vortex Contrails Mode (Symmetric Aerodynamic Condensation Lines)',
      entities: [
        aircraftBox,
        {
          ...cometCylinder,
          show: true,
          positionDegrees: [-75.5982, 40.0396, 802.0],
          shape: { type: 'cylinder', length: 100.0, topRadius: 2.0, bottomRadius: 6.0 },
          color: [0.8, 0.9, 1.0, 0.7],
        },
        {
          ...rocketThrusterSphere,
          show: true,
          positionDegrees: [-75.5972, 40.0380, 802.0],
          shape: { type: 'cylinder', length: 100.0, topRadius: 2.0, bottomRadius: 6.0 },
          color: [0.8, 0.9, 1.0, 0.7],
        },
        flightCorridorRect,
      ],
    },
    {
      label: 'Low-Velocity Atmospheric Smoke Ribbon Mode (Broad Dispersion Drift Trail)',
      entities: [
        aircraftBox,
        {
          ...cometCylinder,
          show: true,
          shape: { type: 'cylinder', length: 120.0, topRadius: 8.0, bottomRadius: 25.0 },
          color: [0.7, 0.75, 0.8, 0.6],
        },
        { ...rocketThrusterSphere, show: false },
        flightCorridorRect,
      ],
    },
  ];

  return { entities, variants };
}

const generators = {
  'panorama': generatePanorama,
  'particle-system': generateParticleSystem,
  'particle-system-fireworks': generateParticleSystemFireworks,
  'particle-system-tails': generateParticleSystemTails,
};

for (const [id, gen] of Object.entries(generators)) {
  const { entities, variants } = gen();
  const fullPath = path.join(outDir, `${id}.json`);
  const variantsPath = path.join(outDir, `${id}.variants.json`);

  fs.writeFileSync(fullPath, JSON.stringify(entities, null, 2) + '\n');
  fs.writeFileSync(variantsPath, JSON.stringify({ initial: 0, variants }, null, 2) + '\n');
  console.log(`Generated ${id}: ${entities.length} entities, ${variants.length} variants`);
}
