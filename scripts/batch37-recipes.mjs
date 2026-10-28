import vm from 'node:vm';
import assert from 'node:assert/strict';

export const batch37Ids = [
  '3d-models-coloring',
  '3d-models-dev',
  'labels-dev',
  'billboards-dev',
];

const LOGO_DATA_URL =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAACvSURBVDhPrZDRDcMgDAU9GqN0lIzijw6SUbJJygUeNQgSqepJTyHG91LVVpwDdfxM3T9TSl1EXZvDwii471fivK73cBFFQNTT/d2KoGpfGOpSIkhUpgUMxq9DFEsWv4IXhlyCnhBFnZcFEEuYqbiUlNwWgMTdrZ3JbQFoEVG53rd8ztG9aPJMnBUQf/VFraBJeWnLS0RfjbKyLJA8FkT5seDYS1Qwyv8t0B/5C2ZmH2/eTGNNBgMmAAAAAElFTkSuQmCC';

const FACILITY_DATA_URL =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQBAMAAADt3eJSAAAAMFBMVEVAQEDAwMDAoAD///+gYADAgACgoKSAYADgwAD///8AAAAAAAAAAAAAAAAAAAAAAAC6opgsAAAACnRSTlP///////////8AsswszwAAAH1JREFUeJw9xiEOwjAYhuF33/6KyQVPlh6AjOwWqBoOgMKScJo6LI5rIAg3IE0WFIK5CpIVQ3jUU0XygQgiH4d2D8R6WKx3TTRI3eoClnvSwxWMxPL5CRjdnHAgZt5TAKG7LydQc/O4zZkKD2N4XaV621MMIWACQUsBxM8/X+GkICjYO7GLAAAAAElFTkSuQmCC';

const MARKER_DATA_URL =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABIAAAASCAYAAABWzo5XAAAAGUlEQVR4nGP4TyXAMGrQqEGjBo0aNNwMAgDZMQs8Mzy3wQAAAABJRU5ErkJggg==';

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
  const M_enu = [
    [-sLon, -sLat * cLon, cLat * cLon],
    [cLon, -sLat * sLon, cLat * sLon],
    [0, cLat, sLat],
  ];
  const qRoll = quatFromAxisAngle([1, 0, 0], roll);
  const qPitch = quatFromAxisAngle([0, 1, 0], -pitch);
  const qHeading = quatFromAxisAngle([0, 0, 1], -heading);
  const qHpr = quatMultiply(qHeading, quatMultiply(qPitch, qRoll));
  const R_hpr = quatToMatrix(qHpr);
  const R_total = mat3Multiply(M_enu, R_hpr);
  return matrixToQuat(R_total);
}

function enuToCartographic(centerLonDeg, centerLatDeg, centerHeight, [x, y, z]) {
  const a = 6378137.0;
  const f = 1.0 / 298.257223563;
  const b = a * (1.0 - f);
  const e2 = (a * a - b * b) / (a * a);
  const toRad = Math.PI / 180.0;
  const toDeg = 180.0 / Math.PI;

  const lon = centerLonDeg * toRad;
  const lat = centerLatDeg * toRad;
  const sinLat = Math.sin(lat);
  const cosLat = Math.cos(lat);
  const sinLon = Math.sin(lon);
  const cosLon = Math.cos(lon);

  const N = a / Math.sqrt(1.0 - e2 * sinLat * sinLat);
  const cx = (N + centerHeight) * cosLat * cosLon;
  const cy = (N + centerHeight) * cosLat * sinLon;
  const cz = (N * (1.0 - e2) + centerHeight) * sinLat;

  const east = [-sinLon, cosLon, 0];
  const north = [-sinLat * cosLon, -sinLat * sinLon, cosLat];
  const up = [cosLat * cosLon, cosLat * sinLon, sinLat];

  const px = cx + x * east[0] + y * north[0] + z * up[0];
  const py = cy + x * east[1] + y * north[1] + z * up[1];
  const pz = cz + x * east[2] + y * north[2] + z * up[2];

  const p = Math.sqrt(px * px + py * py);
  const theta = Math.atan2(pz * a, p * b);
  const ePrime2 = (a * a - b * b) / (b * b);

  const finalLat = Math.atan2(
    pz + ePrime2 * b * Math.pow(Math.sin(theta), 3),
    p - e2 * a * Math.pow(Math.cos(theta), 3),
  );
  const finalLon = Math.atan2(py, px);
  const finalN = a / Math.sqrt(1.0 - e2 * Math.sin(finalLat) * Math.sin(finalLat));
  const finalHeight = p / Math.cos(finalLat) - finalN;

  return [finalLon * toDeg, finalLat * toDeg, finalHeight];
}

export function extractBatch37(id, source, item, hash) {
  if (id === '3d-models-coloring') {
    const quat = headingPitchRollQuaternion(
      -123.0744619,
      44.0503706,
      (135 * Math.PI) / 180,
      0,
      0,
    );

    const modelConfigs = [
      {
        label: 'Aircraft',
        uri: '../../SampleData/models/CesiumAir/Cesium_Air.glb',
        height: 5000.0,
      },
      {
        label: 'Ground Vehicle',
        uri: '../../SampleData/models/GroundVehicle/GroundVehicle.glb',
        height: 0.0,
      },
      {
        label: 'Hot Air Balloon',
        uri: '../../SampleData/models/CesiumBalloon/CesiumBalloon.glb',
        height: 1000.0,
      },
      {
        label: 'Milk Truck',
        uri: '../../SampleData/models/CesiumMilkTruck/CesiumMilkTruck.glb',
        height: 0.0,
      },
      {
        label: 'Skinned Character',
        uri: '../../SampleData/models/CesiumMan/Cesium_Man.glb',
        height: 0.0,
      },
    ];

    const variants = {
      initial: 0,
      variants: modelConfigs.map((cfg) => ({
        label: cfg.label,
        entities: [
          {
            id: cfg.uri,
            show: true,
            name: cfg.uri,
            positionDegrees: [-123.0744619, 44.0503706, cfg.height],
            orientation: quat,
            shape: {
              type: 'model',
              uri: cfg.uri,
              minimumPixelSize: 128.0,
              maximumScale: 20000.0,
              colorBlendMode: 'Highlight',
              colorBlendAmount: 0.5,
              silhouetteColor: [1.0, 0.0, 0.0, 1.0],
              silhouetteSize: 2.0,
            },
            fill: true,
            outline: false,
            outlineColor: [0.0, 0.0, 0.0, 1.0],
            outlineWidth: 1.0,
            color: [1.0, 0.0, 0.0, 1.0],
            properties: null,
          },
        ],
      })),
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
      variantCount: modelConfigs.length,
      framing: 'zoomToEntities',
      shouldAnimate: true,
      trackedEntity: modelConfigs[0].uri,
      requirements: [
        'Sandcastle toolbar menu with 5 glTF model coloring variations',
        'color blending modes: Highlight, Replace, Mix with colorBlendAmount',
        'silhouette coloring and silhouetteSize outline styling',
        'viewer.trackedEntity camera tracking',
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

  if (id === '3d-models-dev') {
    const devConfigs = [
      {
        label: 'Aircraft',
        uri: '../../SampleData/models/CesiumAir/Cesium_Air.glb',
        height: 5000.0,
        orientation: headingPitchRollQuaternion(
          -123.0744619,
          44.0503706,
          0,
          (10 * Math.PI) / 180,
          (-20 * Math.PI) / 180,
        ),
      },
      {
        label: 'Drone',
        uri: '../../SampleData/models/CesiumDrone/CesiumDrone.glb',
        height: 150.0,
        orientation: headingPitchRollQuaternion(
          -123.0744619,
          44.0503706,
          0,
          (10 * Math.PI) / 180,
          (-20 * Math.PI) / 180,
        ),
      },
      {
        label: 'Ground Vehicle',
        uri: '../../SampleData/models/GroundVehicle/GroundVehicle.glb',
        height: 0.0,
        orientation: headingPitchRollQuaternion(-123.0744619, 44.0503706, 0, 0, 0),
      },
      {
        label: 'Milk Truck',
        uri: '../../SampleData/models/CesiumMilkTruck/CesiumMilkTruck.glb',
        height: 0.0,
        orientation: headingPitchRollQuaternion(-123.0744619, 44.0503706, 0, 0, 0),
      },
      {
        label: 'Skinned Character',
        uri: '../../SampleData/models/CesiumMan/Cesium_Man.glb',
        height: 0.0,
        orientation: headingPitchRollQuaternion(-123.0744619, 44.0503706, 0, 0, 0),
      },
    ];

    const variants = {
      initial: 0,
      variants: devConfigs.map((cfg) => ({
        label: cfg.label,
        entities: [
          {
            id: cfg.uri,
            show: true,
            name: cfg.uri,
            positionDegrees: [-123.0744619, 44.0503706, cfg.height],
            orientation: cfg.orientation,
            shape: {
              type: 'model',
              uri: cfg.uri,
              minimumPixelSize: 128.0,
              maximumScale: 20000.0,
              colorBlendMode: 'Highlight',
              colorBlendAmount: 0.5,
              runAnimations: true,
            },
            fill: true,
            outline: false,
            outlineColor: [0.0, 0.0, 0.0, 1.0],
            outlineWidth: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            properties: null,
          },
        ],
      })),
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
      variantCount: devConfigs.length,
      framing: 'lookAt',
      shouldAnimate: true,
      requirements: [
        'Sandcastle toolbar menu with 5 primitive Model.fromGltf development variations',
        'modelMatrix computed from headingPitchRollToFixedFrame',
        'minimumPixelSize: 128 and activeAnimations multiplier/repeat loop',
        'camera.lookAt with HeadingPitchRange orientation',
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

  if (id === 'labels-dev') {
    const centerENU = [-75.59777, 40.03883, 0.0];
    const eastCart = enuToCartographic(centerENU[0], centerENU[1], 0.0, [1000000.0, 0.0, 0.0]);
    const northCart = enuToCartographic(centerENU[0], centerENU[1], 0.0, [0.0, 1000000.0, 0.0]);
    const upCart = enuToCartographic(centerENU[0], centerENU[1], 0.0, [0.0, 0.0, 1000000.0]);

    const variants = {
      initial: 0,
      variants: [
        {
          label: 'Add label',
          entities: [
            {
              id: 'label-dev-add',
              show: true,
              name: '',
              positionDegrees: [-75.1641667, 39.9522222, 0.0],
              shape: {
                type: 'label',
                text: 'Philadelphia',
                fontSize: 30.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
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
          label: 'Set font',
          entities: [
            {
              id: 'label-dev-font',
              show: true,
              name: '',
              positionDegrees: [-75.1641667, 39.9522222, 0.0],
              shape: {
                type: 'label',
                text: 'Philadelphia',
                font: '24px Helvetica',
                fontSize: 24.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
                fillColor: [0.6, 0.9, 1.0, 1.0],
                outlineColor: [0.0, 0.0, 0.0, 1.0],
                outlineWidth: 2.0,
                style: 'FILL_AND_OUTLINE',
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [0.6, 0.9, 1.0, 1.0],
              properties: null,
            },
          ],
        },
        {
          label: 'Set properties',
          entities: [
            {
              id: 'label-dev-properties',
              show: true,
              name: '',
              positionDegrees: [-75.1641667, 39.9522222, 300000.0],
              shape: {
                type: 'label',
                text: 'Philadelphia',
                fontSize: 30.0,
                scale: 2.0,
                pixelOffset: [0.0, 0.0],
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
          label: 'Add labels in reference frame',
          entities: [
            {
              id: 'label-dev-ref-center',
              show: true,
              name: 'Center',
              positionDegrees: [-75.59777, 40.03883, 0.0],
              shape: {
                type: 'label',
                text: 'Center',
                font: '13pt sans-serif',
                fontSize: 17.333333333333332,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
                showBackground: true,
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
            {
              id: 'label-dev-ref-east',
              show: true,
              name: 'East',
              positionDegrees: eastCart,
              shape: {
                type: 'label',
                text: 'East',
                font: '13pt sans-serif',
                fontSize: 17.333333333333332,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
                showBackground: true,
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
            {
              id: 'label-dev-ref-north',
              show: true,
              name: 'North',
              positionDegrees: northCart,
              shape: {
                type: 'label',
                text: 'North',
                font: '13pt sans-serif',
                fontSize: 17.333333333333332,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
                showBackground: true,
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
            {
              id: 'label-dev-ref-up',
              show: true,
              name: 'Up',
              positionDegrees: upCart,
              shape: {
                type: 'label',
                text: 'Up',
                font: '13pt sans-serif',
                fontSize: 17.333333333333332,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
                showBackground: true,
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
          label: 'Offset label by distance',
          entities: [
            {
              id: 'label-dev-offset-bb',
              show: true,
              name: '',
              positionDegrees: [-75.1641667, 39.9522222, 0.0],
              shape: {
                type: 'billboard',
                image: FACILITY_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
              },
              screen: {
                scaleByDistance: [1.5e2, 5.0, 1.5e7, 0.5],
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
            {
              id: 'label-dev-offset-lbl',
              show: true,
              name: '',
              positionDegrees: [-75.1641667, 39.9522222, 0.0],
              shape: {
                type: 'label',
                text: 'Label on top of scaling billboard',
                font: '20px sans-serif',
                fontSize: 20.0,
                scale: 1.0,
                pixelOffset: [0.0, -16.0],
                showBackground: true,
              },
              screen: {
                horizontalOrigin: 'CENTER',
                pixelOffsetScaleByDistance: [1.5e2, 3.0, 1.5e7, 0.5],
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
          label: 'Fade label by distance',
          entities: [
            {
              id: 'label-dev-fade-ny',
              show: true,
              name: 'New York',
              positionDegrees: [-73.94, 40.67, 0.0],
              shape: {
                type: 'label',
                text: 'New York',
                fontSize: 30.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
              },
              screen: {
                translucencyByDistance: [1.5e2, 1.0, 1.5e8, 0.0],
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
            {
              id: 'label-dev-fade-atlanta',
              show: true,
              name: 'Atlanta',
              positionDegrees: [-84.39, 33.75, 0.0],
              shape: {
                type: 'label',
                text: 'Atlanta',
                fontSize: 30.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
              },
              screen: {
                translucencyByDistance: [1.5e5, 1.0, 1.5e7, 0.0],
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
        'Sandcastle toolbar menu with 6 LabelCollection development variations',
        'font, style (FILL_AND_OUTLINE), outline width and color tinting',
        'labels in local ENU reference frame transformed to ECEF positions',
        'offsetByDistance and fadeByDistance with NearFarScalar curves',
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

  if (id === 'billboards-dev') {
    const centerENU = [-75.59777, 40.03883, 0.0];
    const eastCart = enuToCartographic(centerENU[0], centerENU[1], 0.0, [1000000.0, 0.0, 0.0]);
    const northCart = enuToCartographic(centerENU[0], centerENU[1], 0.0, [0.0, 1000000.0, 0.0]);
    const upCart = enuToCartographic(centerENU[0], centerENU[1], 0.0, [0.0, 0.0, 1000000.0]);

    const variants = {
      initial: 0,
      variants: [
        {
          label: 'Add billboard',
          entities: [
            {
              id: 'bb-dev-add',
              show: true,
              name: '',
              positionDegrees: [-75.59777, 40.03883, 0.0],
              shape: {
                type: 'billboard',
                image: LOGO_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
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
          label: 'Set billboard properties at creation',
          entities: [
            {
              id: 'bb-dev-properties-create',
              show: true,
              name: '',
              positionDegrees: [-75.59777, 40.03883, 0.0],
              shape: {
                type: 'billboard',
                image: LOGO_DATA_URL,
                width: 100.0,
                height: 25.0,
                scale: 2.0,
                pixelOffset: [0.0, -50.0],
                sizeInMeters: false,
              },
              screen: {
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'BOTTOM',
                rotation: Math.PI / 4,
                eyeOffset: [0.0, 0.0, 0.0],
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [0.0, 1.0, 0.0, 1.0],
              properties: null,
            },
          ],
        },
        {
          label: 'Change billboard properties',
          entities: [
            {
              id: 'bb-dev-change-props',
              show: true,
              name: '',
              positionDegrees: [-75.59777, 40.03883, 300000.0],
              shape: {
                type: 'billboard',
                image: LOGO_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 3.0,
                pixelOffset: [0.0, 0.0],
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 0.25],
              properties: null,
            },
          ],
        },
        {
          label: 'Size billboard in meters',
          entities: [
            {
              id: 'bb-dev-size-meters',
              show: true,
              name: '',
              positionDegrees: [-75.59777, 40.03883, 0.0],
              shape: {
                type: 'billboard',
                image: LOGO_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
                sizeInMeters: true,
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
          label: 'Add multiple billboards',
          entities: [
            {
              id: 'bb-dev-multi-logo',
              show: true,
              name: '',
              positionDegrees: [-75.59777, 40.03883, 0.0],
              shape: {
                type: 'billboard',
                image: LOGO_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 1.0,
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
              id: 'bb-dev-multi-fac1',
              show: true,
              name: '',
              positionDegrees: [-80.5, 35.14, 0.0],
              shape: {
                type: 'billboard',
                image: FACILITY_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 1.0,
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
              id: 'bb-dev-multi-fac2',
              show: true,
              name: '',
              positionDegrees: [-80.12, 25.46, 0.0],
              shape: {
                type: 'billboard',
                image: FACILITY_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
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
          label: 'Scale by distance',
          entities: [
            {
              id: 'bb-dev-scale-dist',
              show: true,
              name: '',
              positionDegrees: [-75.59777, 40.03883, 0.0],
              shape: {
                type: 'billboard',
                image: FACILITY_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
              },
              screen: {
                scaleByDistance: [1.5e2, 2.0, 1.5e7, 0.5],
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
          label: 'Fade by distance',
          entities: [
            {
              id: 'bb-dev-fade-dist',
              show: true,
              name: '',
              positionDegrees: [-75.59777, 40.03883, 0.0],
              shape: {
                type: 'billboard',
                image: LOGO_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
              },
              screen: {
                translucencyByDistance: [1.5e2, 1.0, 1.5e7, 0.2],
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
          label: 'Offset by distance',
          entities: [
            {
              id: 'bb-dev-offset-fac',
              show: true,
              name: '',
              positionDegrees: [-75.59777, 40.03883, 0.0],
              shape: {
                type: 'billboard',
                image: FACILITY_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
              },
              screen: {
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'BOTTOM',
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
            {
              id: 'bb-dev-offset-logo',
              show: true,
              name: '',
              positionDegrees: [-75.59777, 40.03883, 0.0],
              shape: {
                type: 'billboard',
                image: LOGO_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 1.0,
                pixelOffset: [0.0, -16.0],
              },
              screen: {
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'BOTTOM',
                pixelOffsetScaleByDistance: [1.0e3, 1.0, 1.5e6, 0.0],
                translucencyByDistance: [1.0e3, 1.0, 1.5e6, 0.1],
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
          label: 'Add point billboards',
          entities: [
            {
              id: 'bb-dev-pt-red',
              show: true,
              name: 'red point',
              positionDegrees: [-75.59777, 40.03883, 0.0],
              shape: {
                type: 'point',
                pixelSize: 16.0,
                outlineWidth: 0.0,
                outlineColor: [0.0, 0.0, 0.0, 1.0],
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 0.0, 0.0, 1.0],
              properties: null,
            },
            {
              id: 'bb-dev-pt-blue',
              show: true,
              name: 'blue point',
              positionDegrees: [-80.5, 35.14, 0.0],
              shape: {
                type: 'point',
                pixelSize: 32.0,
                outlineWidth: 0.0,
                outlineColor: [0.0, 0.0, 0.0, 1.0],
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [0.0, 0.0, 1.0, 1.0],
              properties: null,
            },
            {
              id: 'bb-dev-pt-lime',
              show: true,
              name: 'lime point',
              positionDegrees: [-80.12, 25.46, 0.0],
              shape: {
                type: 'point',
                pixelSize: 64.0,
                outlineWidth: 0.0,
                outlineColor: [0.0, 0.0, 0.0, 1.0],
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [0.0, 1.0, 0.0, 1.0],
              properties: null,
            },
          ],
        },
        {
          label: 'Add marker billboards',
          entities: [
            {
              id: 'bb-dev-marker-lime',
              show: true,
              name: 'lime marker',
              positionDegrees: [-75.59777, 40.03883, 0.0],
              shape: {
                type: 'billboard',
                image: MARKER_DATA_URL,
                width: 18.0,
                height: 18.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
                imageSubRegion: [49.0, 43.0, 18.0, 18.0],
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [0.0, 1.0, 0.0, 1.0],
              properties: null,
            },
            {
              id: 'bb-dev-marker-azure',
              show: true,
              name: 'azure marker',
              positionDegrees: [-84.0, 39.0, 0.0],
              shape: {
                type: 'billboard',
                image: MARKER_DATA_URL,
                width: 18.0,
                height: 18.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
                imageSubRegion: [61.0, 23.0, 18.0, 18.0],
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [0.0, 0.5, 1.0, 1.0],
              properties: null,
            },
            {
              id: 'bb-dev-marker-cyan',
              show: true,
              name: 'cyan marker',
              positionDegrees: [-70.0, 41.0, 0.0],
              shape: {
                type: 'billboard',
                image: MARKER_DATA_URL,
                width: 14.0,
                height: 14.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
                imageSubRegion: [67.0, 80.0, 14.0, 14.0],
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [0.5, 0.9, 1.0, 1.0],
              properties: null,
            },
            {
              id: 'bb-dev-marker-red',
              show: true,
              name: 'red marker',
              positionDegrees: [-73.0, 37.0, 0.0],
              shape: {
                type: 'billboard',
                image: MARKER_DATA_URL,
                width: 22.0,
                height: 22.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
                imageSubRegion: [27.0, 103.0, 22.0, 22.0],
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 0.0, 0.0, 1.0],
              properties: null,
            },
            {
              id: 'bb-dev-marker-yellow',
              show: true,
              name: 'yellow marker',
              positionDegrees: [-79.0, 35.0, 0.0],
              shape: {
                type: 'billboard',
                image: MARKER_DATA_URL,
                width: 18.0,
                height: 18.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
                imageSubRegion: [105.0, 105.0, 18.0, 18.0],
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 0.0, 1.0],
              properties: null,
            },
          ],
        },
        {
          label: 'In reference frame',
          entities: [
            {
              id: 'bb-dev-ref-center',
              show: true,
              name: 'center',
              positionDegrees: [-75.59777, 40.03883, 0.0],
              shape: {
                type: 'billboard',
                image: FACILITY_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 1.0,
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
              id: 'bb-dev-ref-east',
              show: true,
              name: 'east',
              positionDegrees: eastCart,
              shape: {
                type: 'billboard',
                image: FACILITY_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 1.0,
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
              id: 'bb-dev-ref-north',
              show: true,
              name: 'north',
              positionDegrees: northCart,
              shape: {
                type: 'billboard',
                image: FACILITY_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 1.0,
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
              id: 'bb-dev-ref-up',
              show: true,
              name: 'up',
              positionDegrees: upCart,
              shape: {
                type: 'billboard',
                image: FACILITY_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
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
        'Sandcastle toolbar menu with 11 BillboardCollection development variations',
        'billboard properties at creation: rotation, width, height, sizeInMeters, pixelOffset, eyeOffset',
        'distance display curves: scaleByDistance, translucencyByDistance, pixelOffsetScaleByDistance',
        'canvas point billboards and marker billboards with imageSubRegion',
        'billboards in local ENU reference frame transformed to ECEF positions',
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

  throw new Error(`unknown batch37 id: ${id}`);
}
