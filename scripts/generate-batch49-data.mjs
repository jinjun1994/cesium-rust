#!/usr/bin/env node
/**
 * Data generator for Sandcastle Batch 49 recipes:
 * 1. lighting (Dynamic sunlight, moonlight, camera flashlight, and directional light)
 * 2. fog (Scene fog density, minimumBrightness, height falloff, visual density)
 * 3. cloud-parameters (Procedural Cumulus cloud parameters, slice, scale, maximum size)
 * 4. clouds (Volumetric CloudCollection mountain cloud cover and atmospheric weather)
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const fullDir = path.join(root, 'tests/sandcastle/full');

function generateLighting() {
  const balloon = {
    id: 'lighting-balloon',
    name: 'Cesium Balloon',
    show: true,
    positionDegrees: [-122.9754, 38.2618, 60.0],
    shape: {
      type: 'model',
      uri: '../../SampleData/models/CesiumBalloon/CesiumBalloon.glb',
      scale: 7.0,
      minimumPixelSize: 64,
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { lightingMode: 'sunLight', modelScale: 7.0 },
  };

  const pedestal = {
    id: 'lighting-pedestal',
    name: 'Lighting Observation Pedestal',
    show: true,
    positionDegrees: [-122.9754, 38.2618, 20.0],
    shape: {
      type: 'cylinder',
      length: 40.0,
      topRadius: 12.0,
      bottomRadius: 12.0,
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: true,
    outlineColor: [0.8, 0.8, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: { lightingMode: 'sunLight', material: 'WHITE' },
  };

  const orb = {
    id: 'lighting-orb',
    name: 'Translucent Lighting Sphere',
    show: true,
    positionDegrees: [-122.9754, 38.2618, 45.0],
    shape: {
      type: 'ellipsoid',
      radii: [16.0, 16.0, 16.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [1.0, 1.0, 1.0, 0.5],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { lightingMode: 'sunLight', material: 'WHITE.withAlpha(0.5)' },
  };

  const beacon = {
    id: 'lighting-beacon',
    name: 'Photometric Reference Beacon',
    show: true,
    positionDegrees: [-122.9746, 38.2618, 20.0],
    shape: {
      type: 'box',
      dimensions: [20.0, 20.0, 20.0],
    },
    color: [1.0, 0.8, 0.2, 1.0],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { lightingMode: 'sunLight', material: 'GOLD' },
  };

  const entities = [balloon, pedestal, orb, beacon];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Sun Light Mode (Dynamic Solar Position & Shadows)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Moon Light Mode (Soft Lunar Light, Blue Hue 0.5 Intensity)',
        entities: entities.map((e) => {
          if (e.id === 'lighting-orb') {
            return {
              ...e,
              show: true,
              color: [0.7, 0.85, 1.0, 0.7],
            };
          }
          if (e.id === 'lighting-beacon') {
            return {
              ...e,
              show: true,
              color: [0.6, 0.75, 1.0, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Flashlight Mode (Camera View-Aligned Headlamp, 3.0 Intensity)',
        entities: entities.map((e) => {
          if (e.id === 'lighting-pedestal') {
            return {
              ...e,
              show: true,
              color: [1.0, 1.0, 0.9, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Custom Directional Light (Warm Amber #deca7c Light)',
        entities: entities.map((e) => {
          if (e.id === 'lighting-beacon') {
            return {
              ...e,
              show: true,
              color: [0.87, 0.79, 0.49, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(
    path.join(fullDir, 'lighting.json'),
    JSON.stringify(entities, null, 2)
  );
  fs.writeFileSync(
    path.join(fullDir, 'lighting.variants.json'),
    JSON.stringify(variants, null, 2)
  );
  console.log('generated lighting.json and .variants.json');
}

function generateFog() {
  const alpsStation = {
    id: 'fog-alps-station',
    name: 'Gotthard Pass Fog Monitoring Post',
    show: true,
    positionDegrees: [8.54, 46.55, 3000.0],
    shape: {
      type: 'box',
      dimensions: [500.0, 500.0, 500.0],
    },
    color: [1.0, 0.3, 0.2, 1.0],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { fogDensity: 0.0002, altitudeMeters: 3000.0 },
  };

  const valleyStation = {
    id: 'fog-valley-station',
    name: 'Alpine Valley Density Station',
    show: true,
    positionDegrees: [8.58, 46.52, 1500.0],
    shape: {
      type: 'cylinder',
      length: 800.0,
      topRadius: 200.0,
      bottomRadius: 200.0,
    },
    color: [0.2, 0.8, 0.4, 1.0],
    fill: true,
    outline: true,
    outlineColor: [0.9, 1.0, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: { fogDensity: 0.0002, altitudeMeters: 1500.0 },
  };

  const horizonBeacon = {
    id: 'fog-horizon-beacon',
    name: 'High Peak Transmittance Beacon',
    show: true,
    positionDegrees: [8.5, 46.6, 4000.0],
    shape: {
      type: 'ellipsoid',
      radii: [300.0, 300.0, 300.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.3, 0.6, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { fogDensity: 0.0002, altitudeMeters: 4000.0 },
  };

  const boundaryContour = {
    id: 'fog-boundary-contour',
    name: 'Alpine Fog Inversion Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [8.48, 46.48, 8.62, 46.62],
      height: 1000.0,
      extrudedHeight: 1200.0,
      rotation: 0.0,
    },
    color: [0.8, 0.85, 0.95, 0.6],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: { fogMaxHeight: 4000.0, heightFalloff: 0.5 },
  };

  const entities = [alpsStation, valleyStation, horizonBeacon, boundaryContour];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Standard Atmospheric Fog (Density 0.0002, Balanced Falloff)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'High Altitude Alpine Fog (Max Height 4000m, Snow Peak Shroud)',
        entities: entities.map((e) => {
          if (e.id === 'fog-horizon-beacon') {
            return {
              ...e,
              show: true,
              color: [0.6, 0.8, 1.0, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Valley Dense Ground Fog (Height Falloff 0.8, Dense Base)',
        entities: entities.map((e) => {
          if (e.id === 'fog-valley-station') {
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
        label: 'Clear Sky Minimal Fog (Density 0.00002, Crisp Alpine Vista)',
        entities: entities.map((e) => {
          if (e.id === 'fog-alps-station') {
            return {
              ...e,
              show: true,
              color: [1.0, 0.5, 0.3, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(
    path.join(fullDir, 'fog.json'),
    JSON.stringify(entities, null, 2)
  );
  fs.writeFileSync(
    path.join(fullDir, 'fog.variants.json'),
    JSON.stringify(variants, null, 2)
  );
  console.log('generated fog.json and .variants.json');
}

function generateCloudParameters() {
  const cumulusCore = {
    id: 'cloud-cumulus-core',
    name: 'Cumulus Cloud Core Ellipsoid',
    show: true,
    positionDegrees: [-123.07446, 44.05037, 50.0],
    shape: {
      type: 'ellipsoid',
      radii: [25.0, 12.0, 15.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [1.0, 1.0, 1.0, 0.9],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { scaleX: 25.0, scaleY: 12.0, slice: 0.36, brightness: 1.0 },
  };

  const baseFluff = {
    id: 'cloud-base-fluff',
    name: 'Cumulus Base Billow',
    show: true,
    positionDegrees: [-123.0741, 44.05037, 45.0],
    shape: {
      type: 'ellipsoid',
      radii: [18.0, 10.0, 10.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.95, 0.95, 1.0, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { scaleX: 18.0, scaleY: 10.0, slice: 0.36, brightness: 1.0 },
  };

  const topDome = {
    id: 'cloud-top-dome',
    name: 'Cumulus Upper Dome',
    show: true,
    positionDegrees: [-123.0748, 44.05037, 55.0],
    shape: {
      type: 'ellipsoid',
      radii: [15.0, 9.0, 12.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [1.0, 1.0, 1.0, 0.95],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { scaleX: 15.0, scaleY: 9.0, slice: 0.36, brightness: 1.0 },
  };

  const cloudShadow = {
    id: 'cloud-shadow-marker',
    name: 'Cloud Ground Shadow Footprint',
    show: true,
    positionDegrees: [-123.07446, 44.05037, 0.0],
    shape: {
      type: 'box',
      dimensions: [50.0, 30.0, 2.0],
    },
    color: [0.3, 0.3, 0.4, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.5, 0.6, 0.9],
    outlineWidth: 1.0,
    properties: { castShadow: true, groundDrape: true },
  };

  const entities = [cumulusCore, baseFluff, topDome, cloudShadow];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'White Cumulus Cloud (Slice 0.36, Brightness 1.0)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Golden Sunset Cloud (Warm Amber Glow, Brightness 1.2)',
        entities: entities.map((e) => {
          if (e.id === 'cloud-cumulus-core') {
            return {
              ...e,
              show: true,
              color: [1.0, 0.75, 0.45, 0.95],
            };
          }
          if (e.id === 'cloud-top-dome') {
            return {
              ...e,
              show: true,
              color: [1.0, 0.85, 0.55, 0.98],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Storm Grey Nimbus Cloud (Dark Silver, Brightness 0.5)',
        entities: entities.map((e) => {
          if (e.id === 'cloud-cumulus-core') {
            return {
              ...e,
              show: true,
              color: [0.5, 0.52, 0.56, 0.95],
            };
          }
          if (e.id === 'cloud-base-fluff') {
            return {
              ...e,
              show: true,
              color: [0.4, 0.42, 0.48, 0.9],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'High Altitude Cirrus (Scaled Flat, Slice 0.5, Brightness 1.0)',
        entities: entities.map((e) => {
          if (e.id === 'cloud-cumulus-core') {
            return {
              ...e,
              show: true,
              color: [0.95, 0.98, 1.0, 0.8],
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(
    path.join(fullDir, 'cloud-parameters.json'),
    JSON.stringify(entities, null, 2)
  );
  fs.writeFileSync(
    path.join(fullDir, 'cloud-parameters.variants.json'),
    JSON.stringify(variants, null, 2)
  );
  console.log('generated cloud-parameters.json and .variants.json');
}

function generateClouds() {
  const stratus1 = {
    id: 'clouds-mtn-stratus-1',
    name: 'Mount Sylvania Cloud Layer 1',
    show: true,
    positionDegrees: [-122.6908, 45.496, 300.0],
    shape: {
      type: 'ellipsoid',
      radii: [300.0, 150.0, 80.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [1.0, 1.0, 1.0, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { scale: [1500.0, 250.0], maximumSize: [50.0, 15.0, 13.0], slice: 0.3 },
  };

  const stratus2 = {
    id: 'clouds-mtn-stratus-2',
    name: 'Portland West Hills Cloud Layer 2',
    show: true,
    positionDegrees: [-122.72, 45.5, 335.0],
    shape: {
      type: 'ellipsoid',
      radii: [350.0, 180.0, 90.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.95, 0.95, 0.98, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { scale: [1500.0, 300.0], maximumSize: [50.0, 12.0, 15.0], slice: 0.36 },
  };

  const stratus3 = {
    id: 'clouds-mtn-stratus-3',
    name: 'Forest Park Cloud Layer 3',
    show: true,
    positionDegrees: [-122.72, 45.51, 260.0],
    shape: {
      type: 'ellipsoid',
      radii: [400.0, 200.0, 100.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.92, 0.94, 0.97, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { scale: [2000.0, 300.0], maximumSize: [50.0, 12.0, 15.0], slice: 0.49 },
  };

  const cumulusLow = {
    id: 'clouds-mtn-cumulus-low',
    name: 'Willamette Valley Low Cloud Formation',
    show: true,
    positionDegrees: [-122.705, 45.52, 250.0],
    shape: {
      type: 'ellipsoid',
      radii: [180.0, 120.0, 70.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [1.0, 1.0, 1.0, 0.9],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { scale: [230.0, 110.0], maximumSize: [13.0, 13.0, 13.0], slice: 0.2 },
  };

  const valleyCoverage = {
    id: 'clouds-valley-coverage',
    name: 'Valley Weather Ceiling Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.74, 45.48, -122.67, 45.54],
      height: 200.0,
      extrudedHeight: 220.0,
      rotation: 0.0,
    },
    color: [0.85, 0.9, 0.95, 0.6],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 0.8],
    outlineWidth: 1.0,
    properties: { layerType: 'ceiling', baseAltitude: 200.0 },
  };

  const entities = [stratus1, stratus2, stratus3, cumulusLow, valleyCoverage];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Cascade Mountain Cloud Layer (5 Cloud Formations, Low Ceiling)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'High Altitude Sea of Clouds (Elevated Stratus +200m)',
        entities: entities.map((e) => {
          if (e.id === 'clouds-mtn-stratus-2') {
            return {
              ...e,
              show: true,
              color: [0.98, 0.98, 1.0, 0.95],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Scattered Fair Weather Cumulus (Low Density, Soft Haze)',
        entities: entities.map((e) => {
          if (e.id === 'clouds-mtn-cumulus-low') {
            return {
              ...e,
              show: true,
              color: [1.0, 0.95, 0.85, 0.8],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Dense Overcast Mountain Weather (High Opacity Nimbus Haze)',
        entities: entities.map((e) => {
          if (e.id === 'clouds-valley-coverage') {
            return {
              ...e,
              show: true,
              color: [0.75, 0.8, 0.85, 0.8],
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(
    path.join(fullDir, 'clouds.json'),
    JSON.stringify(entities, null, 2)
  );
  fs.writeFileSync(
    path.join(fullDir, 'clouds.variants.json'),
    JSON.stringify(variants, null, 2)
  );
  console.log('generated clouds.json and .variants.json');
}

generateLighting();
generateFog();
generateCloudParameters();
generateClouds();
