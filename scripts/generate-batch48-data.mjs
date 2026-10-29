#!/usr/bin/env node
/**
 * Data generator for Sandcastle Batch 48 recipes:
 * 1. shadows (Shadows on terrain, buildings, 3D models and geometric shapes)
 * 2. atmosphere (Earth atmosphere lighting, Rayleigh & Mie scattering, HDR)
 * 3. fog-post-process (Depth-based distance fog post-process stage)
 * 4. globe-materials (Elevation contours, slope shading, and aspect shading materials)
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const fullDir = path.join(root, 'tests/sandcastle/full');

function generateShadows() {
  const cesiumAir = {
    id: 'cesium-air',
    name: 'Cesium Air',
    show: true,
    positionDegrees: [-75.61208, 40.04252, 94.142],
    shape: {
      type: 'model',
      uri: '../../SampleData/models/CesiumAir/Cesium_Air.glb',
      scale: 1.0,
      minimumPixelSize: 64,
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { shadows: 'ENABLED', category: 'air', heightOffset: 20.0 },
  };

  const groundVehicle = {
    id: 'ground-vehicle',
    name: 'Ground Vehicle',
    show: true,
    positionDegrees: [-75.61178, 40.04252, 74.142],
    shape: {
      type: 'model',
      uri: '../../SampleData/models/GroundVehicle/GroundVehicle.glb',
      scale: 1.0,
      minimumPixelSize: 64,
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { shadows: 'ENABLED', category: 'vehicle', heightOffset: 0.0 },
  };

  const cesiumMan = {
    id: 'cesium-man',
    name: 'Cesium Man',
    show: true,
    positionDegrees: [-75.61148, 40.04252, 74.142],
    shape: {
      type: 'model',
      uri: '../../SampleData/models/CesiumMan/Cesium_Man.glb',
      scale: 1.0,
      minimumPixelSize: 64,
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { shadows: 'ENABLED', category: 'character', heightOffset: 0.0 },
  };

  const woodTower = {
    id: 'wood-tower',
    name: 'Wood Tower',
    show: true,
    positionDegrees: [-75.61208, 40.04222, 74.142],
    shape: {
      type: 'model',
      uri: '../../SampleData/models/WoodTower/Wood_Tower.glb',
      scale: 1.0,
      minimumPixelSize: 64,
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { shadows: 'ENABLED', category: 'structure', heightOffset: 0.0 },
  };

  const simpleCity = {
    id: 'simple-city',
    name: 'Simple City',
    show: true,
    positionDegrees: [-75.61178, 40.04222, 74.142],
    shape: {
      type: 'model',
      uri: '../../SampleData/models/ShadowTester/Shadow_Tester_4.glb',
      scale: 1.0,
      minimumPixelSize: 64,
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { shadows: 'ENABLED', category: 'city', heightOffset: 0.0 },
  };

  const shadowBox = {
    id: 'shadow-box',
    name: 'Red Shadow Box',
    show: true,
    positionDegrees: [-75.61148, 40.04222, 84.142],
    shape: {
      type: 'box',
      dimensions: [10.0, 10.0, 10.0],
    },
    color: [1.0, 0.0, 0.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { shadows: 'ENABLED', material: 'Color.RED' },
  };

  const checkeredBox = {
    id: 'shadow-checkered-box',
    name: 'Checkered Box',
    show: true,
    positionDegrees: [-75.61238, 40.04252, 84.142],
    shape: {
      type: 'box',
      dimensions: [10.0, 10.0, 10.0],
    },
    color: [1.0, 0.2, 0.2, 0.8],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.0, 0.0, 1.0],
    outlineWidth: 1.0,
    properties: { shadows: 'ENABLED', material: 'Checkerboard' },
  };

  const shadowSphere = {
    id: 'shadow-sphere',
    name: 'Translucent Shadow Sphere',
    show: true,
    positionDegrees: [-75.61238, 40.04222, 94.142],
    shape: {
      type: 'ellipsoid',
      radii: [15.0, 15.0, 15.0],
      slicePartitions: 24,
      stackPartitions: 36,
    },
    color: [0.0, 0.4, 1.0, 0.6],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { shadows: 'ENABLED', material: 'Color.BLUE.withAlpha(0.5)' },
  };

  const entities = [
    cesiumAir,
    groundVehicle,
    cesiumMan,
    woodTower,
    simpleCity,
    shadowBox,
    checkeredBox,
    shadowSphere,
  ];

  function relocateEntities(baseLon, baseLat, baseAlt) {
    return entities.map((e) => {
      const orig = e.positionDegrees;
      const dLon = orig[0] - (-75.61208);
      const dLat = orig[1] - 40.04252;
      const dAlt = orig[2] - 74.142;
      return {
        ...e,
        positionDegrees: [baseLon + dLon, baseLat + dLat, baseAlt + dAlt],
      };
    });
  }

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Exton Location (Ground & Terrain Shadows)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Half Dome Location (High Altitude Terrain)',
        entities: relocateEntities(-119.5333, 37.7436, 2640.7),
      },
      {
        label: 'Mount Everest Location (Extreme Mountain Ridge)',
        entities: relocateEntities(86.9248, 27.9882, 8773.18),
      },
      {
        label: 'Space Orbital Location (2000km Low Earth Orbit)',
        entities: relocateEntities(-75.61208, 40.04252, 2000000.0),
      },
    ],
  };

  fs.writeFileSync(
    path.join(fullDir, 'shadows.json'),
    JSON.stringify(entities, null, 2)
  );
  fs.writeFileSync(
    path.join(fullDir, 'shadows.variants.json'),
    JSON.stringify(variants, null, 2)
  );
  console.log('generated shadows.json and .variants.json');
}

function generateAtmosphere() {
  const rayleighSensor = {
    id: 'atmosphere-rayleigh-sensor',
    name: 'Rayleigh Scattering Observation Station',
    show: true,
    positionDegrees: [0.0, 0.0, 1000.0],
    shape: {
      type: 'cylinder',
      length: 4000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.2, 0.6, 1.0, 1.0],
    fill: true,
    outline: true,
    outlineColor: [0.8, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      rayleigh: [5.5e-6, 13.0e-6, 28.4e-6],
      scaleHeight: 8000.0,
      lightIntensity: 20.0,
    },
  };

  const mieProfiler = {
    id: 'atmosphere-mie-profiler',
    name: 'Mie Aerosol Profiler Station',
    show: true,
    positionDegrees: [-30.0, 65.0, 1000.0],
    shape: {
      type: 'box',
      dimensions: [3000.0, 3000.0, 3000.0],
    },
    color: [0.85, 0.9, 0.95, 1.0],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.7, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: {
      mie: [2.0e-5, 2.0e-5, 2.0e-5],
      anisotropy: 0.9,
      scaleHeight: 1200.0,
    },
  };

  const twilightBeacon = {
    id: 'atmosphere-twilight-beacon',
    name: 'Twilight Horizon Terminus Beacon',
    show: true,
    positionDegrees: [85.0, 25.0, 1000.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1200.0,
      bottomRadius: 1800.0,
    },
    color: [1.0, 0.55, 0.15, 1.0],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      hueShift: 0.1,
      saturationShift: 0.2,
      brightnessShift: -0.1,
    },
  };

  const ozoneSphere = {
    id: 'atmosphere-ozone-sphere',
    name: 'Stratospheric Ozone Layer Probe',
    show: true,
    positionDegrees: [-105.0, 40.0, 20000.0],
    shape: {
      type: 'ellipsoid',
      radii: [4000.0, 4000.0, 4000.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.4, 0.1, 0.85, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { altitudeMeters: 20000.0, layer: 'Stratosphere' },
  };

  const entities = [
    rayleighSensor,
    mieProfiler,
    twilightBeacon,
    ozoneSphere,
  ];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Earth Standard Atmosphere (Clear Sky Rayleigh & Mie)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Golden Hour Sunset (Enhanced Red/Amber Rayleigh Shift)',
        entities: entities.map((e) => {
          if (e.id === 'atmosphere-twilight-beacon') {
            return {
              ...e,
              show: true,
              color: [1.0, 0.4, 0.1, 1.0],
              outlineColor: [1.0, 0.8, 0.2, 1.0],
            };
          }
          if (e.id === 'atmosphere-rayleigh-sensor') {
            return {
              ...e,
              show: true,
              color: [0.95, 0.5, 0.3, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Mars Thin Dust Atmosphere (High Aerosol Haze)',
        entities: entities.map((e) => {
          if (e.id === 'atmosphere-mie-profiler') {
            return {
              ...e,
              show: true,
              color: [0.85, 0.45, 0.25, 1.0],
              outlineColor: [1.0, 0.6, 0.3, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'High Dynamic Range (HDR Atmosphere 50.0 Intensity)',
        entities: entities.map((e) => {
          if (e.id === 'atmosphere-ozone-sphere') {
            return {
              ...e,
              show: true,
              color: [0.6, 0.2, 1.0, 0.95],
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(
    path.join(fullDir, 'atmosphere.json'),
    JSON.stringify(entities, null, 2)
  );
  fs.writeFileSync(
    path.join(fullDir, 'atmosphere.variants.json'),
    JSON.stringify(variants, null, 2)
  );
  console.log('generated atmosphere.json and .variants.json');
}

function generateFogPostProcess() {
  const fogNear = {
    id: 'fog-depth-near',
    name: 'Fog Near Boundary Probe (10m)',
    show: true,
    positionDegrees: [-75.597, 39.86, 50.0],
    shape: {
      type: 'box',
      dimensions: [8.0, 8.0, 8.0],
    },
    color: [1.0, 0.2, 0.2, 1.0],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { depthMeters: 10.0, fogAmount: 0.05 },
  };

  const fogMid = {
    id: 'fog-depth-mid',
    name: 'Fog Mid-Range Sensor (60m)',
    show: true,
    positionDegrees: [-75.5964, 39.8603, 50.0],
    shape: {
      type: 'cylinder',
      length: 12.0,
      topRadius: 4.0,
      bottomRadius: 4.0,
    },
    color: [0.2, 0.85, 0.3, 1.0],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { depthMeters: 60.0, fogAmount: 0.3 },
  };

  const fogFar = {
    id: 'fog-depth-far',
    name: 'Fog Far-Range Beacon (140m)',
    show: true,
    positionDegrees: [-75.5958, 39.8607, 50.0],
    shape: {
      type: 'box',
      dimensions: [16.0, 16.0, 16.0],
    },
    color: [0.2, 0.5, 1.0, 1.0],
    fill: true,
    outline: true,
    outlineColor: [0.9, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { depthMeters: 140.0, fogAmount: 0.7 },
  };

  const fogLimit = {
    id: 'fog-depth-limit',
    name: 'Fog Distance Extent Marker (200m)',
    show: true,
    positionDegrees: [-75.5952, 39.8611, 50.0],
    shape: {
      type: 'ellipsoid',
      radii: [18.0, 18.0, 18.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.7, 0.7, 0.8, 0.7],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { depthMeters: 200.0, fogAmount: 1.0 },
  };

  const entities = [fogNear, fogMid, fogFar, fogLimit];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Dense Near Fog (10m - 200m, Black)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Morning Mountain Mist (30m - 500m, Pale Gray)',
        entities: entities.map((e) => {
          if (e.id === 'fog-depth-limit') {
            return {
              ...e,
              show: true,
              color: [0.9, 0.9, 0.95, 0.8],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Hazardous Industrial Smog (15m - 300m, Amber Yellow)',
        entities: entities.map((e) => {
          if (e.id === 'fog-depth-far') {
            return {
              ...e,
              show: true,
              color: [0.95, 0.75, 0.15, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Night Coastal Fog (20m - 250m, Deep Navy)',
        entities: entities.map((e) => {
          if (e.id === 'fog-depth-near') {
            return {
              ...e,
              show: true,
              color: [0.1, 0.2, 0.5, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(
    path.join(fullDir, 'fog-post-process.json'),
    JSON.stringify(entities, null, 2)
  );
  fs.writeFileSync(
    path.join(fullDir, 'fog-post-process.variants.json'),
    JSON.stringify(variants, null, 2)
  );
  console.log('generated fog-post-process.json and .variants.json');
}

function generateGlobeMaterials() {
  const himalayasMarker = {
    id: 'globe-mat-himalayas',
    name: 'Himalayas Peak Elevation Marker',
    show: true,
    positionDegrees: [86.925, 27.988, 8848.0],
    shape: {
      type: 'cylinder',
      length: 800.0,
      topRadius: 250.0,
      bottomRadius: 400.0,
    },
    color: [1.0, 0.2, 0.2, 1.0],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { elevationMeters: 8848.0, region: 'Himalayas' },
  };

  const halfDomeMarker = {
    id: 'globe-mat-halfdome',
    name: 'Half Dome Steep Slope Marker',
    show: true,
    positionDegrees: [-119.533, 37.746, 2694.0],
    shape: {
      type: 'box',
      dimensions: [350.0, 350.0, 350.0],
    },
    color: [0.2, 0.85, 0.35, 1.0],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { slopeDegrees: 65.0, region: 'Half Dome' },
  };

  const vancouverMarker = {
    id: 'globe-mat-vancouver',
    name: 'Vancouver Coastal Datum Marker',
    show: true,
    positionDegrees: [-123.12, 49.28, 50.0],
    shape: {
      type: 'ellipsoid',
      radii: [250.0, 250.0, 250.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.2, 0.55, 1.0, 1.0],
    fill: true,
    outline: true,
    outlineColor: [0.9, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { elevationMeters: 50.0, region: 'Vancouver' },
  };

  const contourBand = {
    id: 'globe-mat-contour-band',
    name: 'Himalayan Elevation Contour Band',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [86.85, 27.9, 87.0, 28.05],
      height: 5000.0,
      extrudedHeight: 5200.0,
      rotation: 0.0,
    },
    color: [0.95, 0.55, 0.1, 0.75],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.3, 1.0],
    outlineWidth: 1.0,
    properties: {
      contourSpacing: 150.0,
      contourWidth: 2.0,
      shading: 'elevation',
    },
  };

  const entities = [
    himalayasMarker,
    halfDomeMarker,
    vancouverMarker,
    contourBand,
  ];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Elevation Ramp & Contours (150m Spacing, 2px Width)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Slope Angle Shading (Steep Terrain Visualization)',
        entities: entities.map((e) => {
          if (e.id === 'globe-mat-halfdome') {
            return {
              ...e,
              show: true,
              color: [0.1, 0.95, 0.5, 1.0],
              outlineColor: [1.0, 1.0, 0.2, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Aspect Orientation Shading (Compass Direction Ramp)',
        entities: entities.map((e) => {
          if (e.id === 'globe-mat-contour-band') {
            return {
              ...e,
              show: true,
              color: [0.3, 0.7, 0.9, 0.75],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'High-Density Elevation Contours (50m Spacing, Red Contours)',
        entities: entities.map((e) => {
          if (e.id === 'globe-mat-himalayas') {
            return {
              ...e,
              show: true,
              color: [1.0, 0.1, 0.1, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(
    path.join(fullDir, 'globe-materials.json'),
    JSON.stringify(entities, null, 2)
  );
  fs.writeFileSync(
    path.join(fullDir, 'globe-materials.variants.json'),
    JSON.stringify(variants, null, 2)
  );
  console.log('generated globe-materials.json and .variants.json');
}

generateShadows();
generateAtmosphere();
generateFogPostProcess();
generateGlobeMaterials();
