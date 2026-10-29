#!/usr/bin/env node
/**
 * Data generator for Sandcastle Batch 44 recipes:
 * 1. czml (Satellites, Vehicle, Both, Vehicle Close-up)
 * 2. multi-part-czml (Vehicle along 4500s Mojave corridor in 3 streamed legs with fuel readout)
 * 3. relative-paths (Two satellites and vehicle/helicopter relative to Earth or moving frame)
 * 4. time-dynamic-wheels (Ground vehicle with accelerating wheel rotation, velocity vector readout, and trajectory)
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const fullDir = path.join(root, 'tests/sandcastle/full');

const logoPng =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAACvSURBVDhPrZDRDcMgDAU9GqN0lIzijw6SUbJJygUeNQgSqepJTyHG91LVVpwDdfxM3T9TSl1EXZvDwii471fivK73cBFFQNTT/d2KoGpfGOpSIkhUpgUMxq9DFEsWv4IXhlyCnhBFnZcFEEuYqbiUlNwWgMTdrZ3JbQFoEVG53rd8ztG9aPJMnBUQf/VFraBJeWnLS0RfjbKyLJA8FkT5seDYS1Qwyv8t0B/5C2ZmH2/eTGNNBgMmAAAAAElFTkSuQmCC';

function generateCzml() {
  const geoeyePath = [
    [-50.0, 20.0, 680000.0],
    [-45.0, 30.0, 700000.0],
    [-40.0, 40.0, 690000.0],
    [-35.0, 50.0, 680000.0],
  ];

  const issPath = [
    [10.0, -10.0, 410000.0],
    [15.0, -20.0, 415000.0],
    [20.0, -30.0, 410000.0],
    [25.0, -40.0, 405000.0],
  ];

  const vehicleRoute = [
    [-116.52, 35.02, 10.0],
    [-116.48, 35.05, 10.0],
    [-116.42, 35.10, 10.0],
    [-116.35, 35.15, 10.0],
  ];

  const geoeye = {
    id: 'czml-geoeye1',
    name: 'GeoEye-1',
    show: true,
    positionDegrees: [-45.0, 30.0, 700000.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.0,
      width: 28,
      height: 28,
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { category: 'satellite' },
  };

  const geoeyePathEntity = {
    id: 'czml-geoeye1-path',
    name: 'GeoEye-1 Orbit Path',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: geoeyePath,
        width: 3.0,
        arcType: 'GEODESIC',
        clampToGround: false,
        material: { type: 'solid' },
      },
    },
    color: [1.0, 1.0, 0.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { category: 'satellite' },
  };

  const iss = {
    id: 'czml-iss',
    name: 'ISS',
    show: true,
    positionDegrees: [15.0, -20.0, 415000.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.2,
      width: 32,
      height: 32,
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { category: 'satellite' },
  };

  const issPathEntity = {
    id: 'czml-iss-path',
    name: 'ISS Orbit Path',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: issPath,
        width: 3.0,
        arcType: 'GEODESIC',
        clampToGround: false,
        material: { type: 'solid' },
      },
    },
    color: [0.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { category: 'satellite' },
  };

  const agi = {
    id: 'czml-agi-facility',
    name: 'AGI Facility',
    show: true,
    positionDegrees: [-75.59777, 40.03883, 100.0],
    shape: {
      type: 'label',
      appearance: {
        style: 'FILL',
        showBackground: true,
        backgroundColor: [0.1, 0.1, 0.1, 0.7],
        backgroundPadding: [6, 4],
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
      },
      text: 'AGI Facility',
      fontSize: 13,
      scale: 1.0,
      pixelOffset: [12, -4],
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { category: 'facility' },
  };

  const paPolygon = {
    id: 'czml-pa-polygon',
    name: 'Pennsylvania Territory',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'polygon',
      positionsDegrees: [
        [-80.5, 39.7, 0.0],
        [-74.7, 39.7, 0.0],
        [-74.7, 42.0, 0.0],
        [-80.5, 42.0, 0.0],
      ],
      height: 0,
      extrudedHeight: null,
    },
    color: [0.6, 0.1, 0.6, 0.4],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.0, 0.0, 1.0],
    outlineWidth: 1.0,
    properties: { category: 'region' },
  };

  const vehicle = {
    id: 'czml-ground-vehicle',
    name: 'Ground Vehicle',
    show: false,
    positionDegrees: [-116.52, 35.02, 10.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.0,
      width: 28,
      height: 28,
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { category: 'vehicle' },
  };

  const vehiclePathEntity = {
    id: 'czml-vehicle-path',
    name: 'Vehicle Route',
    show: false,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: vehicleRoute,
        width: 4.0,
        arcType: 'GEODESIC',
        clampToGround: false,
        material: { type: 'solid' },
      },
    },
    color: [1.0, 0.7, 0.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { category: 'vehicle' },
  };

  const allEntities = [
    geoeye,
    geoeyePathEntity,
    iss,
    issPathEntity,
    agi,
    paPolygon,
    vehicle,
    vehiclePathEntity,
  ];

  function makeVariant(activeIds) {
    return allEntities.map(e => {
      const clone = structuredClone(e);
      clone.show = activeIds.includes(clone.id);
      return clone;
    });
  }

  const satIds = [
    'czml-geoeye1',
    'czml-geoeye1-path',
    'czml-iss',
    'czml-iss-path',
    'czml-agi-facility',
    'czml-pa-polygon',
  ];
  const vehIds = ['czml-ground-vehicle', 'czml-vehicle-path'];
  const bothIds = [...satIds, ...vehIds];

  const variants = [
    {
      label: 'Satellites (simple.czml)',
      entities: makeVariant(satIds),
    },
    {
      label: 'Vehicle (Vehicle.czml)',
      entities: makeVariant(vehIds),
    },
    {
      label: 'Both (Satellites and Vehicle)',
      entities: makeVariant(bothIds),
    },
    {
      label: 'Vehicle Close-Up',
      entities: makeVariant(vehIds),
    },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'czml.json'),
    JSON.stringify(variants[0].entities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'czml.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated czml');
}

function generateMultiPartCzml() {
  const modelUrl = 'SampleData/models/CesiumMilkTruck/CesiumMilkTruck.glb';

  const waypoints = [
    { t: 0, lon: -116.52, lat: 35.02, alt: 0 },
    { t: 1500, lon: -116.35, lat: 35.15, alt: 0 },
    { t: 3000, lon: -116.15, lat: 35.30, alt: 0 },
    { t: 4500, lon: -115.90, lat: 35.45, alt: 0 },
  ];

  const fullPathDegrees = waypoints.map(w => [w.lon, w.lat, w.alt + 2.0]);

  function makeVariants() {
    const configs = [
      { label: 'Part 1 (0 - 1500s)', fuel: 200.0, legWaypoints: waypoints.slice(0, 2), currentWp: waypoints[0] },
      { label: 'Part 2 (1500 - 3000s)', fuel: 133.3, legWaypoints: waypoints.slice(1, 3), currentWp: waypoints[1] },
      { label: 'Part 3 (3000 - 4500s)', fuel: 66.7, legWaypoints: waypoints.slice(2, 4), currentWp: waypoints[2] },
      { label: 'Combined Stream (0 - 4500s)', fuel: 20.0, legWaypoints: waypoints, currentWp: waypoints[3] },
    ];

    return configs.map(c => {
      const legDegrees = c.legWaypoints.map(w => [w.lon, w.lat, w.alt + 2.0]);

      const vehicle = {
        id: 'multipart-vehicle',
        name: 'Transcontinental Vehicle',
        show: true,
        positionDegrees: [c.currentWp.lon, c.currentWp.lat, c.currentWp.alt],
        shape: {
          type: 'model',
          uri: modelUrl,
          scale: 2.0,
          minimumPixelSize: 64,
          maximumScale: 2000,
        },
        color: [1.0, 1.0, 1.0, 1.0],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 0],
        outlineWidth: 1.0,
        properties: { fuel: c.fuel, leg: c.name },
      };

      const fuelLabel = {
        id: 'multipart-fuel-display',
        name: 'Fuel Remaining',
        show: true,
        positionDegrees: [c.currentWp.lon, c.currentWp.lat, c.currentWp.alt + 15.0],
        shape: {
          type: 'label',
          appearance: {
            style: 'FILL',
            showBackground: true,
            backgroundColor: [0.15, 0.15, 0.15, 0.8],
            backgroundPadding: [8, 5],
            outlineColor: [0, 0, 0, 1],
            outlineWidth: 1,
          },
          text: `Fuel: ${c.fuel.toFixed(2)} gal`,
          fontSize: 14,
          scale: 2.0,
          pixelOffset: [0, -25],
        },
        color: [1.0, 1.0, 1.0, 1.0],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 0],
        outlineWidth: 1.0,
        properties: { fuel: c.fuel },
      };

      const pathEntity = {
        id: 'multipart-route-path',
        name: 'Streamed Corridor',
        show: true,
        positionDegrees: null,
        shape: {
          type: 'widePolyline',
          data: {
            positions: legDegrees.length > 1 ? legDegrees : fullPathDegrees,
            width: 4.0,
            arcType: 'GEODESIC',
            clampToGround: false,
            material: { type: 'solid' },
          },
        },
        color: [0.0, 0.8, 1.0, 1.0],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 0],
        outlineWidth: 1.0,
        properties: { leg: c.name },
      };

      return {
        label: c.label,
        entities: [vehicle, fuelLabel, pathEntity],
      };
    });
  }

  const variants = makeVariants();

  fs.writeFileSync(
    path.join(fullDir, 'multi-part-czml.json'),
    JSON.stringify(variants[0].entities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'multi-part-czml.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated multi-part-czml');
}

function generateRelativePaths() {
  const satOrbitEarth = [
    [-70.0, 15.0, 500000.0],
    [-60.0, 20.0, 500000.0],
    [-50.0, 25.0, 500000.0],
    [-40.0, 30.0, 500000.0],
  ];

  const satRelSat2 = [
    [-62.0, 18.0, 500000.0],
    [-60.0, 20.0, 500000.0],
    [-58.0, 22.0, 500000.0],
    [-56.0, 24.0, 500000.0],
  ];

  const earthVehHeliPath = [
    [-116.55, 35.00, 10.0],
    [-116.52, 35.02, 10.0],
    [-116.48, 35.05, 10.0],
    [-116.45, 35.08, 10.0],
  ];

  const heliRelVehPath = [
    [-116.51, 35.03, 300.0],
    [-116.50, 35.04, 300.0],
    [-116.49, 35.05, 300.0],
    [-116.48, 35.06, 300.0],
  ];

  function makeEntitySet(mode) {
    const isSat = mode.startsWith('sat');

    const sat1 = {
      id: 'rel-satellite-1',
      name: 'Satellite 1',
      show: isSat,
      positionDegrees: [-60.0, 20.0, 500000.0],
      shape: {
        type: 'billboard',
        image: logoPng,
        scale: 1.0,
        width: 28,
        height: 28,
      },
      color: [0.0, 1.0, 1.0, 1.0],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      properties: { system: 'satellite' },
    };

    const sat2 = {
      id: 'rel-satellite-2',
      name: 'Satellite 2',
      show: isSat,
      positionDegrees: [-58.0, 22.0, 520000.0],
      shape: {
        type: 'billboard',
        image: logoPng,
        scale: 1.0,
        width: 28,
        height: 28,
      },
      color: [1.0, 0.8, 0.0, 1.0],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      properties: { system: 'satellite' },
    };

    const veh = {
      id: 'rel-ground-vehicle',
      name: 'Ground Truck',
      show: !isSat,
      positionDegrees: [-116.52, 35.02, 10.0],
      shape: {
        type: 'billboard',
        image: logoPng,
        scale: 1.0,
        width: 28,
        height: 28,
      },
      color: [1.0, 0.5, 0.0, 1.0],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      properties: { system: 'vehicle' },
    };

    const heli = {
      id: 'rel-helicopter',
      name: 'Helicopter',
      show: !isSat,
      positionDegrees: [-116.50, 35.04, 300.0],
      shape: {
        type: 'billboard',
        image: logoPng,
        scale: 1.0,
        width: 28,
        height: 28,
      },
      color: [0.2, 0.9, 0.2, 1.0],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      properties: { system: 'vehicle' },
    };

    let chosenPath;
    let pathColor;
    if (mode === 'sat-earth') {
      chosenPath = satOrbitEarth;
      pathColor = [0.0, 0.8, 1.0, 1.0];
    } else if (mode === 'sat-relative') {
      chosenPath = satRelSat2;
      pathColor = [1.0, 0.8, 0.0, 1.0];
    } else if (mode === 'veh-earth') {
      chosenPath = earthVehHeliPath;
      pathColor = [1.0, 0.5, 0.0, 1.0];
    } else {
      chosenPath = heliRelVehPath;
      pathColor = [0.2, 0.9, 0.2, 1.0];
    }

    const trajectory = {
      id: 'rel-trajectory-path',
      name: 'Relative Trajectory Corridor',
      show: true,
      positionDegrees: null,
      shape: {
        type: 'widePolyline',
        data: {
          positions: chosenPath,
          width: 4.0,
          arcType: 'GEODESIC',
          clampToGround: false,
          material: { type: 'solid' },
        },
      },
      color: pathColor,
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      properties: { mode },
    };

    return [sat1, sat2, veh, heli, trajectory];
  }

  const variants = [
    {
      label: 'Two Satellites Relative to Earth',
      entities: makeEntitySet('sat-earth'),
    },
    {
      label: 'Satellite 1 Relative to Satellite 2',
      entities: makeEntitySet('sat-relative'),
    },
    {
      label: 'Vehicle and Helicopter Relative to Earth',
      entities: makeEntitySet('veh-earth'),
    },
    {
      label: 'Helicopter Relative to Vehicle',
      entities: makeEntitySet('veh-relative'),
    },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'relative-paths.json'),
    JSON.stringify(variants[0].entities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'relative-paths.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated relative-paths');
}

function generateTimeDynamicWheels() {
  const modelUrl = 'SampleData/models/GroundVehicle/GroundVehicle.glb';

  // Barstow/Daggett CA start and end points
  const p0 = [-117.02292, 34.89164, 0.0];
  const p3 = [-117.02290, 34.88997, 0.0];

  function lerpPoint(a, b, t) {
    return [
      a[0] + (b[0] - a[0]) * t,
      a[1] + (b[1] - a[1]) * t,
      a[2] + (b[2] - a[2]) * t,
    ];
  }

  const pathPositions = [
    [p0[0], p0[1], p0[2] + 1.0],
    [lerpPoint(p0, p3, 0.33)[0], lerpPoint(p0, p3, 0.33)[1], 1.0],
    [lerpPoint(p0, p3, 0.67)[0], lerpPoint(p0, p3, 0.67)[1], 1.0],
    [p3[0], p3[1], p3[2] + 1.0],
  ];

  function makeVariant(speedKmHr, modeName, currentPos) {
    const vehicle = {
      id: 'dynamic-ground-vehicle',
      name: 'Articulated Ground Vehicle',
      show: true,
      positionDegrees: currentPos,
      shape: {
        type: 'model',
        uri: modelUrl,
        scale: 1.0,
        minimumPixelSize: 64,
        maximumScale: 2000,
      },
      color: [1.0, 1.0, 1.0, 1.0],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      properties: { speedKmHr, mode: modeName },
    };

    const speedLabel = {
      id: 'dynamic-speed-readout',
      name: 'Speed Readout',
      show: true,
      positionDegrees: [currentPos[0], currentPos[1], currentPos[2] + 4.0],
      shape: {
        type: 'label',
        appearance: {
          style: 'FILL',
          showBackground: true,
          backgroundColor: [0.1, 0.1, 0.1, 0.8],
          backgroundPadding: [7, 5],
          outlineColor: [0, 0, 0, 1],
          outlineWidth: 1,
        },
        text: `${speedKmHr} km/hr`,
        fontSize: 16,
        scale: 1.0,
        pixelOffset: [0, -20],
      },
      color: [1.0, 1.0, 1.0, 1.0],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      properties: { speedKmHr },
    };

    const pathEntity = {
      id: 'vehicle-corridor-path',
      name: 'Runway Path',
      show: true,
      positionDegrees: null,
      shape: {
        type: 'widePolyline',
        data: {
          positions: pathPositions,
          width: 4.0,
          arcType: 'GEODESIC',
          clampToGround: false,
          material: { type: 'solid' },
        },
      },
      color: [1.0, 0.6, 0.0, 1.0],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      properties: { mode: modeName },
    };

    return [vehicle, speedLabel, pathEntity];
  }

  const variants = [
    {
      label: 'Accelerating Vehicle (0 -> 45 km/hr)',
      entities: makeVariant(45, 'accelerating', p0),
    },
    {
      label: 'Constant Cruise (30 km/hr)',
      entities: makeVariant(30, 'cruise', lerpPoint(p0, p3, 0.5)),
    },
    {
      label: 'High Speed Dash (60 km/hr)',
      entities: makeVariant(60, 'dash', lerpPoint(p0, p3, 0.75)),
    },
    {
      label: 'Braking Stop (45 -> 0 km/hr)',
      entities: makeVariant(0, 'braking', p3),
    },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'time-dynamic-wheels.json'),
    JSON.stringify(variants[0].entities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'time-dynamic-wheels.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated time-dynamic-wheels');
}

generateCzml();
generateMultiPartCzml();
generateRelativePaths();
generateTimeDynamicWheels();
