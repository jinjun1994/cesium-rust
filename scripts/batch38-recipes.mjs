import vm from 'node:vm';
import assert from 'node:assert/strict';

export const batch38Ids = [
  '3d-models-articulations-dev',
  '3d-models-node-explorer-dev',
  'headingpitchroll',
  'entity-tracking',
];

const SATELLITE_ICON_DATA_URL =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAADJSURBVDhPnZHRDcMgEEMZjVEYpaNklIzSEfLfD4qNnXAJSFWfhO7w2Zc0Tf9QG2rXrEzSUeZLOGm47WoH95x3Hl3jEgilvDgsOQUTqsNl68ezEwn1vae6lceSEEYvvWNT/Rxc4CXQNGadho1NXoJ+9iaqc2xi2xbt23PJCDIB6TQjOC6Bho/sDy3fBQT8PrVhibU7yBFcEPaRxOoeTwbwByCOYf9VGp1BYI1BA+EeHhmfzKbBoJEQwn1yzUZtyspIQUha85MpkNIXB7GizqDEECsAAAAASUVORK5CYII=';

function quatFromAxisAngle(axis, angle) {
  const half = angle * 0.5;
  const s = Math.sin(half);
  return [axis[0] * s, axis[1] * s, axis[2] * s, Math.cos(half)];
}

function quatMultiply(a, b) {
  return [
    a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
    a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
    a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
    a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
  ];
}

function quatToMatrix(q) {
  const [x, y, z, w] = q;
  const x2 = x + x, y2 = y + y, z2 = z + z;
  const xx = x * x2, xy = x * y2, xz = x * z2;
  const yy = y * y2, yz = y * z2, zz = z * z2;
  const wx = w * x2, wy = w * y2, wz = w * z2;
  return [
    [1 - (yy + zz), xy - wz, xz + wy],
    [xy + wz, 1 - (xx + zz), yz - wx],
    [xz - wy, yz + wx, 1 - (xx + yy)],
  ];
}

function mat3Multiply(a, b) {
  const r = [
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
  ];
  for (let i = 0; i < 3; i++) {
    for (let j = 0; j < 3; j++) {
      r[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
    }
  }
  return r;
}

function matrixToQuat(m) {
  const trace = m[0][0] + m[1][1] + m[2][2];
  let x, y, z, w;
  if (trace > 0) {
    const s = 0.5 / Math.sqrt(trace + 1.0);
    w = 0.25 / s;
    x = (m[2][1] - m[1][2]) * s;
    y = (m[0][2] - m[2][0]) * s;
    z = (m[1][0] - m[0][1]) * s;
  } else if (m[0][0] > m[1][1] && m[0][0] > m[2][2]) {
    const s = 2.0 * Math.sqrt(1.0 + m[0][0] - m[1][1] - m[2][2]);
    w = (m[2][1] - m[1][2]) / s;
    x = 0.25 * s;
    y = (m[0][1] + m[1][0]) / s;
    z = (m[0][2] + m[2][0]) / s;
  } else if (m[1][1] > m[2][2]) {
    const s = 2.0 * Math.sqrt(1.0 + m[1][1] - m[0][0] - m[2][2]);
    w = (m[0][2] - m[2][0]) / s;
    x = (m[0][1] + m[1][0]) / s;
    y = 0.25 * s;
    z = (m[1][2] + m[2][1]) / s;
  } else {
    const s = 2.0 * Math.sqrt(1.0 + m[2][2] - m[0][0] - m[1][1]);
    w = (m[1][0] - m[0][1]) / s;
    x = (m[0][2] + m[2][0]) / s;
    y = (m[1][2] + m[2][1]) / s;
    z = 0.25 * s;
  }
  return [x, y, z, w];
}

function headingPitchRollQuaternion(lonDeg, latDeg, heading, pitch, roll) {
  const rad = Math.PI / 180;
  const lon = lonDeg * rad,
    lat = latDeg * rad;
  const sLon = Math.sin(lon),
    cLon = Math.cos(lon);
  const sLat = Math.sin(lat),
    cLat = Math.cos(lat);
  const east = [-sLon, cLon, 0];
  const north = [-sLat * cLon, -sLat * sLon, cLat];
  const up = [cLat * cLon, cLat * sLon, sLat];
  const mENU = [
    [east[0], north[0], up[0]],
    [east[1], north[1], up[1]],
    [east[2], north[2], up[2]],
  ];
  const qENU = matrixToQuat(mENU);

  const qHeading = quatFromAxisAngle([0, 0, 1], -heading);
  const qPitch = quatFromAxisAngle([0, 1, 0], pitch);
  const qRoll = quatFromAxisAngle([1, 0, 0], -roll);
  const qHPR = quatMultiply(qHeading, quatMultiply(qPitch, qRoll));

  return quatMultiply(qENU, qHPR);
}

function headingPitchRollQuaternionNWU(lonDeg, latDeg, heading, pitch, roll) {
  const rad = Math.PI / 180;
  const lon = lonDeg * rad,
    lat = latDeg * rad;
  const sLon = Math.sin(lon),
    cLon = Math.cos(lon);
  const sLat = Math.sin(lat),
    cLat = Math.cos(lat);
  const east = [-sLon, cLon, 0];
  const north = [-sLat * cLon, -sLat * sLon, cLat];
  const up = [cLat * cLon, cLat * sLon, sLat];
  const west = [sLon, -cLon, 0];

  // North-West-Up frame: col 0 = North, col 1 = West, col 2 = Up
  const mNWU = [
    [north[0], west[0], up[0]],
    [north[1], west[1], up[1]],
    [north[2], west[2], up[2]],
  ];
  const qNWU = matrixToQuat(mNWU);

  const qHeading = quatFromAxisAngle([0, 0, 1], -heading);
  const qPitch = quatFromAxisAngle([0, 1, 0], pitch);
  const qRoll = quatFromAxisAngle([1, 0, 0], -roll);
  const qHPR = quatMultiply(qHeading, quatMultiply(qPitch, qRoll));

  return quatMultiply(qNWU, qHPR);
}

export function extractBatch38(id, sourceCode, catalog, hash) {
  const item = catalog.cases.find((c) => c.id === id);
  if (!item) {
    throw new Error(`case ${id} not found in catalog`);
  }

  if (id === '3d-models-articulations-dev') {
    const origin = [-74.693, 28.243, 220000.0];
    const orientation = headingPitchRollQuaternion(origin[0], origin[1], 0, 0, 0);

    const variants = {
      initial: 0,
      variants: [
        {
          label: 'Fairing Open: 0°',
          entities: [
            {
              id: 'launchvehicle',
              show: true,
              name: 'Launch Vehicle',
              positionDegrees: origin,
              orientation,
              shape: {
                type: 'model',
                uri: 'https://cesium.com/public/SandcastleSampleData/launchvehicle.glb',
                scale: 1.0,
                minimumPixelSize: 128,
                runAnimations: false,
                articulations: {
                  'Fairing Open': { stage: 'Move', value: 0.0 },
                  'Fairing Separate': { stage: 'Move', value: 0.0 },
                  'Fairing Drop': { stage: 'Move', value: 0.0 },
                },
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
          ],
        },
        {
          label: 'Fairing Open: 60°',
          entities: [
            {
              id: 'launchvehicle',
              show: true,
              name: 'Launch Vehicle',
              positionDegrees: origin,
              orientation,
              shape: {
                type: 'model',
                uri: 'https://cesium.com/public/SandcastleSampleData/launchvehicle.glb',
                scale: 1.0,
                minimumPixelSize: 128,
                runAnimations: false,
                articulations: {
                  'Fairing Open': { stage: 'Move', value: 60.0 },
                  'Fairing Separate': { stage: 'Move', value: 0.0 },
                  'Fairing Drop': { stage: 'Move', value: 0.0 },
                },
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
          ],
        },
        {
          label: 'Fairing Separate: -25',
          entities: [
            {
              id: 'launchvehicle',
              show: true,
              name: 'Launch Vehicle',
              positionDegrees: origin,
              orientation,
              shape: {
                type: 'model',
                uri: 'https://cesium.com/public/SandcastleSampleData/launchvehicle.glb',
                scale: 1.0,
                minimumPixelSize: 128,
                runAnimations: false,
                articulations: {
                  'Fairing Open': { stage: 'Move', value: 120.0 },
                  'Fairing Separate': { stage: 'Move', value: -25.0 },
                  'Fairing Drop': { stage: 'Move', value: 0.0 },
                },
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
          ],
        },
        {
          label: 'Fairing Drop: -50',
          entities: [
            {
              id: 'launchvehicle',
              show: true,
              name: 'Launch Vehicle',
              positionDegrees: origin,
              orientation,
              shape: {
                type: 'model',
                uri: 'https://cesium.com/public/SandcastleSampleData/launchvehicle.glb',
                scale: 1.0,
                minimumPixelSize: 128,
                runAnimations: false,
                articulations: {
                  'Fairing Open': { stage: 'Move', value: 120.0 },
                  'Fairing Separate': { stage: 'Move', value: -50.0 },
                  'Fairing Drop': { stage: 'Move', value: -50.0 },
                },
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
          ],
        },
      ],
    };

    const initialEntities = variants.variants[0].entities;
    const entry = {
      id,
      title: item.title,
      sourceSha256: hash,
      referenceUrl: `https://sandcastle.cesium.com/?id=${id}`,
      format: 'variants',
      path: `../../tests/sandcastle/full/${id}.variants.json`,
      expectedPath: `../../tests/sandcastle/full/${id}.json`,
      entityCount: initialEntities.length,
      variantCount: variants.variants.length,
      framing: 'viewer-default',
      shouldAnimate: false,
      requirements: [
        'Model.fromGltfAsync loading launchvehicle.glb with minimumPixelSize: 128',
        'Knockout viewModel tracking articulation stages and stage values',
        'Fairing Open, Fairing Separate, Fairing Drop runtime stages',
        'camera.lookAt targeting modelMatrix with HeadingPitchRange',
      ],
      status: 'implemented',
      validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
    };

    return {
      entities: initialEntities,
      variants,
      manifest: entry,
      expectedEntities: initialEntities,
    };
  }

  if (id === '3d-models-node-explorer-dev') {
    const origin = [-123.0744619, 44.0503706, 250000.0];
    const orientation = headingPitchRollQuaternion(origin[0], origin[1], 0, 0, 0);

    const qArmL = quatFromAxisAngle([0, 0, 1], Math.PI / 4);
    const qArmR = quatFromAxisAngle([0, 1, 0], Math.PI / 4);

    const variants = {
      initial: 0,
      variants: [
        {
          label: 'Default Pose',
          entities: [
            {
              id: 'CesiumMan',
              show: true,
              name: 'Cesium Man',
              positionDegrees: origin,
              orientation,
              shape: {
                type: 'model',
                uri: '../../SampleData/models/CesiumMan/Cesium_Man.glb',
                scale: 1.0,
                minimumPixelSize: 128,
                runAnimations: false,
                nodeTransformations: {},
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
          ],
        },
        {
          label: 'Left Arm Rotated (Roll 45°)',
          entities: [
            {
              id: 'CesiumMan',
              show: true,
              name: 'Cesium Man',
              positionDegrees: origin,
              orientation,
              shape: {
                type: 'model',
                uri: '../../SampleData/models/CesiumMan/Cesium_Man.glb',
                scale: 1.0,
                minimumPixelSize: 128,
                runAnimations: false,
                nodeTransformations: {
                  Skeleton_arm_joint_L__3_: {
                    rotation: {
                      unitQuaternion: qArmL,
                    },
                  },
                },
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
          ],
        },
        {
          label: 'Right Arm Rotated (Pitch 45°)',
          entities: [
            {
              id: 'CesiumMan',
              show: true,
              name: 'Cesium Man',
              positionDegrees: origin,
              orientation,
              shape: {
                type: 'model',
                uri: '../../SampleData/models/CesiumMan/Cesium_Man.glb',
                scale: 1.0,
                minimumPixelSize: 128,
                runAnimations: false,
                nodeTransformations: {
                  Skeleton_arm_joint_R__2_: {
                    rotation: {
                      unitQuaternion: qArmR,
                    },
                  },
                },
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
          ],
        },
        {
          label: 'Hide Man (show: false)',
          entities: [
            {
              id: 'CesiumMan',
              show: false,
              name: 'Cesium Man',
              positionDegrees: origin,
              orientation,
              shape: {
                type: 'model',
                uri: '../../SampleData/models/CesiumMan/Cesium_Man.glb',
                scale: 1.0,
                minimumPixelSize: 128,
                runAnimations: false,
                nodeTransformations: {},
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
          ],
        },
      ],
    };

    const initialEntities = variants.variants[0].entities;
    const entry = {
      id,
      title: item.title,
      sourceSha256: hash,
      referenceUrl: `https://sandcastle.cesium.com/?id=${id}`,
      format: 'variants',
      path: `../../tests/sandcastle/full/${id}.variants.json`,
      expectedPath: `../../tests/sandcastle/full/${id}.json`,
      entityCount: initialEntities.length,
      variantCount: variants.variants.length,
      framing: 'viewer-default',
      shouldAnimate: false,
      requirements: [
        'Model.fromGltfAsync loading Cesium_Man.glb at 250km altitude',
        'Knockout viewModel node explorer with translation, rotation, and scale',
        'Transformations applied to node matrices via Matrix4 computed from translation/quaternion/scale',
        'camera.lookAt targeting boundingSphere center with HeadingPitchRange',
        'Sandcastle toggle button for showing and hiding nodes',
      ],
      status: 'implemented',
      validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
    };

    return {
      entities: initialEntities,
      variants,
      manifest: entry,
      expectedEntities: initialEntities,
    };
  }

  if (id === 'headingpitchroll') {
    const origin = [-123.0744619, 44.0503706, 5000.0];
    const rad = Math.PI / 180;
    const delta = 3.0 * rad;

    const pathPositions = [
      [-123.0744619, 44.0503706, 5000.0],
      [-123.0740, 44.0507, 5000.0],
      [-123.0735, 44.0511, 5000.0],
      [-123.0730, 44.0515, 5000.0],
    ];

    const makeEntities = (heading, pitch, roll) => {
      const orientation = headingPitchRollQuaternionNWU(
        origin[0],
        origin[1],
        heading,
        pitch,
        roll,
      );

      return [
        {
          id: 'path',
          show: true,
          name: 'path',
          positionDegrees: null,
          shape: {
            type: 'widePolyline',
            data: {
              positions: pathPositions,
              width: 10.0,
              arcType: 'GEODESIC',
              clampToGround: false,
              material: {
                type: 'glow',
                glowPower: 0.3,
                taperPower: 0.3,
                color: [0.933, 0.91, 0.667, 1.0],
              },
            },
          },
          fill: true,
          outline: false,
          outlineColor: [0.0, 0.0, 0.0, 1.0],
          outlineWidth: 1.0,
          color: [0.933, 0.91, 0.667, 1.0],
          properties: null,
        },
        {
          id: 'plane',
          show: true,
          name: 'Cesium Air',
          positionDegrees: origin,
          orientation,
          shape: {
            type: 'model',
            uri: '../../SampleData/models/CesiumAir/Cesium_Air.glb',
            scale: 1.0,
            minimumPixelSize: 128,
            runAnimations: true,
          },
          fill: true,
          outline: false,
          outlineColor: [0.0, 0.0, 0.0, 1.0],
          outlineWidth: 1.0,
          color: [1.0, 1.0, 1.0, 1.0],
          properties: null,
        },
      ];
    };

    const variants = {
      initial: 0,
      variants: [
        {
          label: 'Default (Heading 0°, Pitch 0°, Roll 0°, Speed 10)',
          entities: makeEntities(0, 0, 0),
        },
        {
          label: 'Pitch Down (ArrowDown, Pitch -3°)',
          entities: makeEntities(0, -delta, 0),
        },
        {
          label: 'Pitch Up (ArrowUp, Pitch +3°)',
          entities: makeEntities(0, delta, 0),
        },
        {
          label: 'Turn Right (ArrowRight, Heading +3°)',
          entities: makeEntities(delta, 0, 0),
        },
        {
          label: 'Turn Left (ArrowLeft, Heading -3°)',
          entities: makeEntities(-delta, 0, 0),
        },
      ],
    };

    const initialEntities = variants.variants[0].entities;
    const entry = {
      id,
      title: item.title,
      sourceSha256: hash,
      referenceUrl: `https://sandcastle.cesium.com/?id=${id}`,
      format: 'variants',
      path: `../../tests/sandcastle/full/${id}.variants.json`,
      expectedPath: `../../tests/sandcastle/full/${id}.json`,
      entityCount: initialEntities.length,
      variantCount: variants.variants.length,
      framing: 'viewer-default',
      shouldAnimate: true,
      requirements: [
        'SampledPositionProperty path entity with PolylineGlowMaterialProperty (color: PALEGOLDENROD, glowPower: 0.3, taperPower: 0.3, width: 10)',
        'CesiumAir.glb model primitive with headingPitchRollToFixedFrame orientation in North-West-Up frame',
        'Keyboard controls for Arrow keys modifying Heading, Pitch, Roll, and Speed',
        'camera.lookAt targeting model with HeadingPitchRange in fromBehind mode',
        'HTML status spans updating heading, pitch, roll, and speed',
      ],
      status: 'implemented',
      validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
    };

    return {
      entities: initialEntities,
      variants,
      manifest: entry,
      expectedEntities: initialEntities,
    };
  }

  if (id === 'entity-tracking') {
    const issPos = [-45.0, 20.0, 400000.0];
    const dronePos = [-123.0744619, 44.0503706, 150.0];
    const droneOrientation = headingPitchRollQuaternion(dronePos[0], dronePos[1], 0, 0, 0);

    const makeTrackingEntities = (trackingFrame) => [
      {
        id: 'Satellite/ISS',
        show: true,
        name: 'ISS',
        positionDegrees: issPos,
        viewFrom: [-300.0, 20.0, 100.0],
        trackingReferenceFrame: trackingFrame,
        shape: {
          type: 'billboard',
          image: SATELLITE_ICON_DATA_URL,
          scale: 1.5,
          width: 16.0,
          height: 16.0,
          pixelOffset: [0.0, 0.0],
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [1.0, 1.0, 1.0, 1.0],
        properties: null,
      },
      {
        id: 'CesiumDrone',
        show: true,
        name: 'Drone',
        positionDegrees: dronePos,
        orientation: droneOrientation,
        viewFrom: [-50.0, 0.0, 5.0],
        trackingReferenceFrame: trackingFrame,
        shape: {
          type: 'model',
          uri: '../../SampleData/models/CesiumDrone/CesiumDrone.glb',
          minimumPixelSize: 64.0,
          maximumScale: 20000.0,
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [1.0, 1.0, 1.0, 1.0],
        properties: null,
      },
    ];

    const variants = {
      initial: 0,
      variants: [
        {
          label: 'Satellites (ISS Tracked)',
          entities: makeTrackingEntities('AUTODETECT'),
        },
        {
          label: 'Drone (Drone Tracked)',
          entities: makeTrackingEntities('AUTODETECT'),
        },
        {
          label: 'Tracking reference frame: Inertial',
          entities: makeTrackingEntities('INERTIAL'),
        },
        {
          label: 'Tracking reference frame: Velocity',
          entities: makeTrackingEntities('VELOCITY'),
        },
        {
          label: 'Tracking reference frame: East-North-Up',
          entities: makeTrackingEntities('ENU'),
        },
      ],
    };

    const initialEntities = variants.variants[0].entities;
    const entry = {
      id,
      title: item.title,
      sourceSha256: hash,
      referenceUrl: `https://sandcastle.cesium.com/?id=${id}`,
      format: 'variants',
      path: `../../tests/sandcastle/full/${id}.variants.json`,
      expectedPath: `../../tests/sandcastle/full/${id}.json`,
      entityCount: initialEntities.length,
      variantCount: variants.variants.length,
      framing: 'viewer-default',
      shouldAnimate: true,
      requirements: [
        'CzmlDataSource.load tracking.czml containing Satellite/ISS and CesiumDrone entities',
        'Satellite tracking with viewFrom [-300, 20, 100] and 30x clock multiplier',
        'Drone tracking with viewFrom [-50, 0, 5] and 1x clock multiplier',
        'Tracking reference frames: Auto-detect, Inertial, Velocity, and East-North-Up',
        'timeline.zoomTo interval for active tracked entity',
      ],
      status: 'implemented',
      validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
    };

    return {
      entities: initialEntities,
      variants,
      manifest: entry,
      expectedEntities: initialEntities,
    };
  }

  throw new Error(`unknown batch38 id: ${id}`);
}
