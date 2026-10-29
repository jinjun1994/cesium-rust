#!/usr/bin/env node
/**
 * Data generator for Sandcastle Batch 50 recipes:
 * 1. ambient-occlusion (Post-process Screen Space Ambient Occlusion SSAO)
 * 2. blue-marble (NASA Blue Marble Next Generation global imagery)
 * 3. camera-controllers (Modular hybrid pan, tilt-orbit, and zoom controllers)
 * 4. camera-tutorial (First-person keyboard/mouse screen space controller)
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const fullDir = path.join(root, 'tests/sandcastle/full');

function generateAmbientOcclusion() {
  const structureBase = {
    id: 'ao-structure-base',
    name: 'Power Plant Generator Complex',
    show: true,
    positionDegrees: [-76.347, 34.773, 50.0],
    shape: {
      type: 'box',
      dimensions: [100.0, 100.0, 60.0],
    },
    color: [0.85, 0.85, 0.9, 1.0],
    fill: true,
    outline: true,
    outlineColor: [0.3, 0.3, 0.35, 1.0],
    outlineWidth: 1.0,
    properties: { intensity: 3.0, bias: 0.1, directionCount: 8 },
  };

  const coolingTower = {
    id: 'ao-cooling-tower',
    name: 'Hyperboloid Cooling Tower',
    show: true,
    positionDegrees: [-76.3458, 34.773, 60.0],
    shape: {
      type: 'cylinder',
      length: 120.0,
      topRadius: 30.0,
      bottomRadius: 45.0,
    },
    color: [0.9, 0.9, 0.95, 1.0],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.4, 0.45, 1.0],
    outlineWidth: 1.0,
    properties: { lengthCap: 0.26, stepCount: 32 },
  };

  const containmentDome = {
    id: 'ao-containment-dome',
    name: 'Reactor Containment Vessel Dome',
    show: true,
    positionDegrees: [-76.3482, 34.773, 50.0],
    shape: {
      type: 'ellipsoid',
      radii: [40.0, 40.0, 35.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.75, 0.8, 0.85, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { occlusionCrevice: true },
  };

  const perimeterBoundary = {
    id: 'ao-perimeter-boundary',
    name: 'Facility Security Footprint Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-76.35, 34.77, -76.344, 34.776],
      height: 10.0,
      extrudedHeight: 15.0,
      rotation: 0.0,
    },
    color: [0.3, 0.6, 0.9, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.8, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { facilityType: 'nuclear', status: 'active' },
  };

  const entities = [structureBase, coolingTower, containmentDome, perimeterBoundary];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Ambient Occlusion Enabled (Intensity 3.0, Bias 0.1, 32 Steps)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'High Contrast Crevice Shading (Intensity 5.0, Bias 0.05)',
        entities: entities.map((e) => {
          if (e.id === 'ao-cooling-tower') {
            return {
              ...e,
              show: true,
              color: [0.8, 0.82, 0.88, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Ambient Occlusion Only (Depth Mask Clay Render Mode)',
        entities: entities.map((e) => {
          if (e.id === 'ao-containment-dome') {
            return {
              ...e,
              show: true,
              color: [0.65, 0.68, 0.72, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Soft Contact Shadows (LengthCap 0.5, 64 Directional Steps)',
        entities: entities.map((e) => {
          if (e.id === 'ao-structure-base') {
            return {
              ...e,
              show: true,
              color: [0.92, 0.92, 0.95, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(
    path.join(fullDir, 'ambient-occlusion.json'),
    JSON.stringify(entities, null, 2)
  );
  fs.writeFileSync(
    path.join(fullDir, 'ambient-occlusion.variants.json'),
    JSON.stringify(variants, null, 2)
  );
  console.log('generated ambient-occlusion.json and .variants.json');
}

function generateBlueMarble() {
  const equatorDatum = {
    id: 'blue-marble-equator-datum',
    name: 'Prime Meridian Equator Datum',
    show: true,
    positionDegrees: [0.0, 0.0, 25000.0],
    shape: {
      type: 'cylinder',
      length: 50000.0,
      topRadius: 15000.0,
      bottomRadius: 15000.0,
    },
    color: [0.2, 0.6, 1.0, 1.0],
    fill: true,
    outline: true,
    outlineColor: [0.8, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { ionAssetId: 3845, imageryEpoch: '2004-07' },
  };

  const amazonMarker = {
    id: 'blue-marble-amazon-marker',
    name: 'Amazon Tropical Biosphere Core',
    show: true,
    positionDegrees: [-60.0, -3.0, 20000.0],
    shape: {
      type: 'box',
      dimensions: [40000.0, 40000.0, 40000.0],
    },
    color: [0.1, 0.8, 0.3, 1.0],
    fill: true,
    outline: true,
    outlineColor: [0.4, 1.0, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: { biome: 'tropical_rainforest', region: 'South America' },
  };

  const saharaMarker = {
    id: 'blue-marble-sahara-marker',
    name: 'Sahara Arid Plateau Marker',
    show: true,
    positionDegrees: [20.0, 25.0, 20000.0],
    shape: {
      type: 'ellipsoid',
      radii: [35000.0, 35000.0, 35000.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [1.0, 0.75, 0.2, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { biome: 'desert', region: 'North Africa' },
  };

  const arcticBoundary = {
    id: 'blue-marble-arctic-boundary',
    name: 'Arctic Circle Polar Ice Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-45.0, 65.0, -43.0, 67.0],
      height: 10000.0,
      extrudedHeight: 15000.0,
      rotation: 0.0,
    },
    color: [0.85, 0.95, 1.0, 0.6],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: { seaIceExtentMin: 'summer_2004', polarCircleLat: 66.5 },
  };

  const entities = [equatorDatum, amazonMarker, saharaMarker, arcticBoundary];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'July 2004 Summer Foliage (Global Daylight Blue Marble)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Tropical Biosphere Focus (Amazon Rainforest Canopy)',
        entities: entities.map((e) => {
          if (e.id === 'blue-marble-amazon-marker') {
            return {
              ...e,
              show: true,
              color: [0.05, 0.95, 0.35, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Arid & Desert Zones (Sahara Golden Sand Albedo)',
        entities: entities.map((e) => {
          if (e.id === 'blue-marble-sahara-marker') {
            return {
              ...e,
              show: true,
              color: [1.0, 0.85, 0.3, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Arctic Polar Ice Cap Extents (Summer Minimal Ice Shroud)',
        entities: entities.map((e) => {
          if (e.id === 'blue-marble-arctic-boundary') {
            return {
              ...e,
              show: true,
              color: [0.9, 0.98, 1.0, 0.75],
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(
    path.join(fullDir, 'blue-marble.json'),
    JSON.stringify(entities, null, 2)
  );
  fs.writeFileSync(
    path.join(fullDir, 'blue-marble.variants.json'),
    JSON.stringify(variants, null, 2)
  );
  console.log('generated blue-marble.json and .variants.json');
}

function generateCameraControllers() {
  const mineHeadframe = {
    id: 'ctrl-mine-headframe',
    name: 'Parc Lead Mine Surface Headframe',
    show: true,
    positionDegrees: [-3.82518, 53.11728, 50.0],
    shape: {
      type: 'box',
      dimensions: [80.0, 80.0, 120.0],
    },
    color: [0.8, 0.3, 0.2, 1.0],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: { controller: 'HybridPan', site: 'ParcLeadMine' },
  };

  const subsurfaceShaft = {
    id: 'ctrl-subsurface-shaft',
    name: 'Vertical Drainage & Hoisting Shaft',
    show: true,
    positionDegrees: [-3.82518, 53.11728, -200.0],
    shape: {
      type: 'cylinder',
      length: 400.0,
      topRadius: 25.0,
      bottomRadius: 25.0,
    },
    color: [0.2, 0.7, 0.9, 1.0],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { controller: 'TiltOrbit', depthMeters: 400.0 },
  };

  const oreChamber = {
    id: 'ctrl-ore-chamber',
    name: 'Subterranean Stope Ore Chamber',
    show: true,
    positionDegrees: [-3.82518, 53.11728, -450.0],
    shape: {
      type: 'ellipsoid',
      radii: [60.0, 45.0, 40.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.9, 0.7, 0.1, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { controller: 'Zoom', mineral: 'Galena / Sphalerite' },
  };

  const surfaceLease = {
    id: 'ctrl-surface-lease',
    name: 'Mining Concession Surface Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-3.83, 53.114, -3.82, 53.12],
      height: 10.0,
      extrudedHeight: 20.0,
      rotation: 0.0,
    },
    color: [0.3, 0.8, 0.4, 0.6],
    fill: true,
    outline: true,
    outlineColor: [0.7, 1.0, 0.8, 0.9],
    outlineWidth: 1.0,
    properties: { leaseAreaKm2: 0.45, legalStatus: 'heritage' },
  };

  const entities = [mineHeadframe, subsurfaceShaft, oreChamber, surfaceLease];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Hybrid Screen Space Pan Controller (Free Planar Drag)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Tilt Orbit Camera Controller (Subsurface Pit Orbit)',
        entities: entities.map((e) => {
          if (e.id === 'ctrl-subsurface-shaft') {
            return {
              ...e,
              show: true,
              color: [0.1, 0.9, 1.0, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Screen Space Zoom Controller (Deep Shaft Inspection)',
        entities: entities.map((e) => {
          if (e.id === 'ctrl-ore-chamber') {
            return {
              ...e,
              show: true,
              color: [1.0, 0.85, 0.2, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Composite Multi-Controller Mode (All Controllers Active)',
        entities: entities.map((e) => {
          if (e.id === 'ctrl-mine-headframe') {
            return {
              ...e,
              show: true,
              color: [0.95, 0.4, 0.25, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(
    path.join(fullDir, 'camera-controllers.json'),
    JSON.stringify(entities, null, 2)
  );
  fs.writeFileSync(
    path.join(fullDir, 'camera-controllers.variants.json'),
    JSON.stringify(variants, null, 2)
  );
  console.log('generated camera-controllers.json and .variants.json');
}

function generateCameraTutorial() {
  const waypointAlpha = {
    id: 'tutorial-waypoint-alpha',
    name: 'Navigational Beacon Alpha (Start)',
    show: true,
    positionDegrees: [-105.0, 40.0, 25.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 10.0,
      bottomRadius: 10.0,
    },
    color: [0.2, 0.8, 0.3, 1.0],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: { controlScheme: 'WASD_QE', waypoint: 'Alpha' },
  };

  const waypointBeta = {
    id: 'tutorial-waypoint-beta',
    name: 'Orientation Checkpoint Beta',
    show: true,
    positionDegrees: [-104.995, 40.005, 15.0],
    shape: {
      type: 'box',
      dimensions: [30.0, 30.0, 30.0],
    },
    color: [1.0, 0.4, 0.2, 1.0],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.8, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: { controlScheme: 'WASD_QE', waypoint: 'Beta' },
  };

  const observationDome = {
    id: 'tutorial-observation-dome',
    name: 'Target Observation Radar Dome',
    show: true,
    positionDegrees: [-104.99, 40.0, 25.0],
    shape: {
      type: 'ellipsoid',
      radii: [25.0, 25.0, 25.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.3, 0.6, 1.0, 0.9],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { targetHeight: 50.0, targetRange: 200.0 },
  };

  const courseBoundary = {
    id: 'tutorial-course-boundary',
    name: 'First-Person Training Arena Perimeter',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-105.01, 39.99, -104.98, 40.01],
      height: 5.0,
      extrudedHeight: 10.0,
      rotation: 0.0,
    },
    color: [0.9, 0.8, 0.2, 0.6],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.5, 0.9],
    outlineWidth: 1.0,
    properties: { courseMeters: 1000.0, surfaceType: 'gridded' },
  };

  const entities = [waypointAlpha, waypointBeta, observationDome, courseBoundary];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Ground Level First-Person Walk (WASD + Mouse Look)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Low-Altitude Drone Flight (Q/E Vertical Thrust Active)',
        entities: entities.map((e) => {
          if (e.id === 'tutorial-waypoint-alpha') {
            return {
              ...e,
              show: true,
              color: [0.1, 0.95, 0.5, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'High-Speed Terrain Reconnaissance (5x Movement Multiplier)',
        entities: entities.map((e) => {
          if (e.id === 'tutorial-waypoint-beta') {
            return {
              ...e,
              show: true,
              color: [1.0, 0.2, 0.1, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Orbit Point of Interest Inspection (Waypoint Lock)',
        entities: entities.map((e) => {
          if (e.id === 'tutorial-observation-dome') {
            return {
              ...e,
              show: true,
              color: [0.5, 0.8, 1.0, 0.95],
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(
    path.join(fullDir, 'camera-tutorial.json'),
    JSON.stringify(entities, null, 2)
  );
  fs.writeFileSync(
    path.join(fullDir, 'camera-tutorial.variants.json'),
    JSON.stringify(variants, null, 2)
  );
  console.log('generated camera-tutorial.json and .variants.json');
}

generateAmbientOcclusion();
generateBlueMarble();
generateCameraControllers();
generateCameraTutorial();
