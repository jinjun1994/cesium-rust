#!/usr/bin/env node
/**
 * Data generator for Sandcastle Batch 42 recipes:
 * 1. materials (20 Material configurations on rectangle, world rectangle, and polyline)
 * 2. bloom (13 CesiumBalloon entities with post-processing bloom parameters)
 * 3. cesium-widget (Cesium_Air model in minimalist widget mode with trackedEntity)
 * 4. cardboard (13 CesiumBalloon entities in circular Hermite flight path around Grand Canyon)
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const fullDir = path.join(root, 'tests/sandcastle/full');

function generateMaterials() {
  const logoPath = path.join(root, 'tests/sandcastle/images/Cesium_Logo_Color.jpg');
  const logoBase64 = `data:image/jpeg;base64,${fs.readFileSync(logoPath).toString('base64')}`;

  const rectEntity = {
    id: 'mat-rectangle',
    name: 'Material Rectangle',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-100.0, 20.0, -70.0, 40.0],
      height: 0,
      extrudedHeight: null,
      rotation: 0,
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    material: {
      type: 'image',
      image: logoBase64,
      color: [1.0, 1.0, 1.0, 1.0],
      repeat: [1.0, 1.0],
      transparent: false,
    },
    properties: { materialType: 'Image' },
  };

  const worldRectEntity = {
    id: 'mat-world-rectangle',
    name: 'Material World Rectangle',
    show: false,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-20.0, -10.0, 20.0, 10.0],
      height: 0,
      extrudedHeight: null,
      rotation: 0,
    },
    color: [0.2, 0.5, 0.8, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    material: {
      type: 'rimLighting',
      color: [0.2, 0.5, 0.8, 1.0],
      rimPower: 4.0,
    },
    properties: { materialType: 'RimLighting' },
  };

  const polylineEntity = {
    id: 'mat-polyline',
    name: 'Material Polyline',
    show: false,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: [
          [-110.0, 42.0, 0.0],
          [-85.0, 36.0, 0.0],
          [-100.0, 25.0, 0.0],
          [-77.0, 12.0, 0.0],
        ],
        width: 10.0,
        arcType: 'GEODESIC',
        clampToGround: false,
        material: {
          type: 'glow',
          color: [0.0, 0.749, 1.0, 1.0],
          glowPower: 0.25,
          taperPower: 1.0,
        },
      },
    },
    color: [0.0, 0.749, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { materialType: 'PolylineGlow' },
  };

  const baseEntities = [rectEntity, worldRectEntity, polylineEntity];

  // Helper to clone entities with specific material / show setup
  function makeVariant(activeId, matType, surfaceMat, polylineMat) {
    return baseEntities.map(e => {
      const clone = structuredClone(e);
      clone.show = clone.id === activeId;
      if (clone.id === activeId) {
        if (surfaceMat) {
          clone.material = surfaceMat;
          clone.properties = { materialType: matType };
        } else if (polylineMat && clone.shape.type === 'widePolyline') {
          clone.shape.data.material = polylineMat;
          clone.properties = { materialType: matType };
        }
      }
      return clone;
    });
  }

  const variants = [
    {
      label: 'Image Material (Default)',
      entities: makeVariant('mat-rectangle', 'Image', {
        type: 'image',
        image: logoBase64,
        color: [1.0, 1.0, 1.0, 1.0],
        repeat: [1.0, 1.0],
        transparent: false,
      }),
    },
    {
      label: 'Grid Material',
      entities: makeVariant('mat-rectangle', 'Grid', {
        type: 'grid',
        color: [1.0, 1.0, 1.0, 1.0],
        cellAlpha: 0.1,
        lineCount: [8, 8],
        lineThickness: [2.0, 2.0],
      }),
    },
    {
      label: 'Stripe Material',
      entities: makeVariant('mat-rectangle', 'Stripe', {
        type: 'stripe',
        orientation: 'HORIZONTAL',
        evenColor: [1.0, 1.0, 1.0, 1.0],
        oddColor: [1.0, 0.0, 0.0, 1.0],
        offset: 0,
        repeat: 16,
      }),
    },
    {
      label: 'Checkerboard Material',
      entities: makeVariant('mat-rectangle', 'Checkerboard', {
        type: 'checkerboard',
        evenColor: [1.0, 0.0, 0.0, 1.0],
        oddColor: [0.0, 1.0, 0.0, 1.0],
        repeat: [4, 4],
      }),
    },
    {
      label: 'Dot Material',
      entities: makeVariant('mat-rectangle', 'Dot', {
        type: 'dot',
        lightColor: [0.75, 1.0, 0.0, 1.0],
        darkColor: [0.0, 0.5, 0.0, 1.0],
        repeat: [10, 10],
      }),
    },
    {
      label: 'RimLighting Material (World)',
      entities: makeVariant('mat-world-rectangle', 'RimLighting', {
        type: 'rimLighting',
        color: [0.2, 0.5, 0.8, 1.0],
        rimPower: 4.0,
      }),
    },
    {
      label: 'Polyline Glow Material',
      entities: makeVariant('mat-polyline', 'PolylineGlow', null, {
        type: 'glow',
        color: [0.0, 0.749, 1.0, 1.0],
        glowPower: 0.25,
        taperPower: 1.0,
      }),
    },
    {
      label: 'Polyline Outline Material',
      entities: makeVariant('mat-polyline', 'PolylineOutline', null, {
        type: 'outline',
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 2.0,
      }),
    },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'materials.json'),
    JSON.stringify(variants[0].entities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'materials.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated materials');
}

function generateBloom() {
  const numberOfBalloons = 13;
  const lonIncrement = 0.00025;
  const initialLon = -122.99875;
  const lat = 44.0503706;
  const height = 100.0;
  const url = '../../SampleData/models/CesiumBalloon/CesiumBalloon.glb';

  const defaultEntities = [];
  for (let i = 0; i < numberOfBalloons; ++i) {
    const lon = initialLon + i * lonIncrement;
    defaultEntities.push({
      id: `balloon-${i}`,
      name: `Balloon ${i}`,
      show: true,
      positionDegrees: [lon, lat, height],
      shape: {
        type: 'model',
        uri: url,
        scale: 1.0,
        minimumPixelSize: 64,
        maximumScale: 200,
      },
      color: [1.0, 1.0, 1.0, 1.0],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      properties: {
        balloonIndex: i,
        modelUri: url,
      },
    });
  }

  const variants = [
    {
      label: 'Default Bloom (Contrast 128, Brightness -0.3)',
      entities: defaultEntities,
    },
    {
      label: 'High Contrast Bloom (Contrast 200)',
      entities: defaultEntities.map(e => ({
        ...e,
        properties: { ...e.properties, contrast: 200 },
      })),
    },
    {
      label: 'Subtle Glow (Brightness -0.5, Contrast 80)',
      entities: defaultEntities.map(e => ({
        ...e,
        properties: { ...e.properties, brightness: -0.5, contrast: 80 },
      })),
    },
    {
      label: 'Glow Only Mode',
      entities: defaultEntities.map(e => ({
        ...e,
        properties: { ...e.properties, glowOnly: true },
      })),
    },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'bloom.json'),
    JSON.stringify(defaultEntities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'bloom.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated bloom');
}

function generateCesiumWidget() {
  const position = [-123.0744619, 44.0503706, 500.0];
  const url = '../../SampleData/models/CesiumAir/Cesium_Air.glb';

  function makeEntity(headingDeg, pitchDeg, rollDeg) {
    return [
      {
        id: 'widget-airplane',
        name: 'Cesium Air (Widget)',
        show: true,
        positionDegrees: position,
        shape: {
          type: 'model',
          uri: url,
          minimumPixelSize: 128,
          maximumScale: 20000,
          scale: 1.0,
        },
        color: [1.0, 1.0, 1.0, 1.0],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 0],
        outlineWidth: 1.0,
        properties: {
          widgetMode: true,
          trackedEntity: true,
          headingDegrees: headingDeg,
          pitchDegrees: pitchDeg,
          rollDegrees: rollDeg,
        },
      },
    ];
  }

  const variants = [
    { label: 'Tracked Aircraft (Heading 135°)', entities: makeEntity(135.0, 0.0, 0.0) },
    { label: 'North Heading (Heading 0°)', entities: makeEntity(0.0, 0.0, 0.0) },
    { label: 'East Heading (Heading 90°)', entities: makeEntity(90.0, 0.0, 0.0) },
    { label: 'Climbing Attitude (Pitch +20°)', entities: makeEntity(135.0, 20.0, 0.0) },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'cesium-widget.json'),
    JSON.stringify(variants[0].entities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'cesium-widget.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated cesium-widget');
}

function generateCardboard() {
  const longitude = -112.110693;
  const latitude = 36.0994841;
  const baseRadius = 0.03;
  const numBalloons = 13;
  const url = '../../SampleData/models/CesiumBalloon/CesiumBalloon.glb';

  // Seeded random helper for repeatable circular flight path samples
  let seed = 3;
  function pseudoRandom() {
    seed = (seed * 9301 + 49297) % 233280;
    return seed / 233280;
  }

  // Generate sampled circular flight samples
  function generateSamples(lon, lat, radius) {
    const samples = [];
    const startAngle = pseudoRandom() * 360.0;
    const endAngle = startAngle + 360.0;
    const increment = (pseudoRandom() * 2.0 - 1.0) * 10.0 + 45.0;

    for (let angle = startAngle; angle <= endAngle; angle += increment) {
      const radians = (angle * Math.PI) / 180.0;
      const timeIncrement = Math.min(360.0, ((angle - startAngle) / 360.0) * 360.0);
      const sampleLon = lon + radius * 1.5 * Math.cos(radians);
      const sampleLat = lat + radius * Math.sin(radians);
      const sampleAlt = pseudoRandom() * 500.0 + 1800.0;

      // Convert to ECEF
      const phi = (sampleLat * Math.PI) / 180.0;
      const lambda = (sampleLon * Math.PI) / 180.0;
      const h = sampleAlt;
      const a = 6378137.0;
      const f = 1.0 / 298.257223563;
      const e2 = 2.0 * f - f * f;
      const N = a / Math.sqrt(1.0 - e2 * Math.sin(phi) * Math.sin(phi));
      const x = (N + h) * Math.cos(phi) * Math.cos(lambda);
      const y = (N + h) * Math.cos(phi) * Math.sin(lambda);
      const z = (N * (1.0 - e2) + h) * Math.sin(phi);

      samples.push({
        time: timeIncrement,
        cartesian: [x, y, z],
      });
    }
    return samples;
  }

  const allBalloons = [];
  for (let i = 0; i < numBalloons; ++i) {
    const balloonRadius = i === 0 ? baseRadius : (pseudoRandom() * 2.0 - 1.0) * 0.01 + baseRadius;
    const samples = generateSamples(longitude, latitude, balloonRadius);
    allBalloons.push({
      id: `vr-balloon-${i}`,
      name: i === 0 ? 'Lead VR Balloon' : `Escort Balloon ${i}`,
      show: true,
      positionDegrees: null,
      availability: [0.0, 360.0],
      samples,
      shape: {
        type: 'model',
        uri: url,
        scale: 1.0,
        minimumPixelSize: 64,
        maximumScale: 200,
      },
      color: [1.0, 1.0, 1.0, 1.0],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      properties: {
        vrCardboard: true,
        balloonIndex: i,
        interpolation: 'HermitePolynomialApproximation',
        degree: 2,
      },
    });
  }

  const variants = [
    { label: 'Default VR Balloon Flight (13 Balloons)', entities: allBalloons },
    { label: 'Single Lead Balloon (Lead Anchor)', entities: [allBalloons[0]] },
    { label: 'Formation Flight (5 Balloons)', entities: allBalloons.slice(0, 5) },
    {
      label: 'High Altitude Offset (+1000m)',
      entities: allBalloons.map(b => ({
        ...b,
        properties: { ...b.properties, altitudeOffset: 1000.0 },
      })),
    },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'cardboard.json'),
    JSON.stringify(allBalloons, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'cardboard.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated cardboard');
}

generateMaterials();
generateBloom();
generateCesiumWidget();
generateCardboard();
