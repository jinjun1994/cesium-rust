import vm from 'node:vm';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

export const batch36Ids = [
  'labels',
  '3d-models',
  'czml-model-node-transformations',
  'czml-model-articulations',
];

const FACILITY_DATA_URL =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQBAMAAADt3eJSAAAAMFBMVEVAQEDAwMDAoAD///+gYADAgACgoKSAYADgwAD///8AAAAAAAAAAAAAAAAAAAAAAAC6opgsAAAACnRSTlP///////////8AsswszwAAAH1JREFUeJw9xiEOwjAYhuF33/6KyQVPlh6AjOwWqBoOgMKScJo6LI5rIAg3IE0WFIK5CpIVQ3jUU0XygQgiH4d2D8R6WKx3TTRI3eoClnvSwxWMxPL5CRjdnHAgZt5TAKG7LydQc/O4zZkKD2N4XaV621MMIWACQUsBxM8/X+GkICjYO7GLAAAAAElFTkSuQmCC';

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

export function extractBatch36(id, source, item, hash) {
  if (id === 'labels') {
    const defaultAppearance = {
      style: 'FILL',
      showBackground: false,
      backgroundColor: [0.165, 0.165, 0.165, 0.8],
      backgroundPadding: [7.0, 5.0],
      outlineColor: [0.0, 0.0, 0.0, 1.0],
      outlineWidth: 1.0,
    };

    const variants = {
      variants: [
        {
          label: 'Add label',
          entities: [
            {
              id: 'label-add',
              show: true,
              name: '',
              positionDegrees: [-75.1641667, 39.9522222, 0.0],
              shape: {
                type: 'label',
                appearance: { ...defaultAppearance },
                text: 'Philadelphia',
                fontSize: 30.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
              },
              screen: {
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'CENTER',
                rotation: 0.0,
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
              id: 'label-font',
              show: true,
              name: '',
              positionDegrees: [-75.1641667, 39.9522222, 0.0],
              shape: {
                type: 'label',
                appearance: {
                  ...defaultAppearance,
                  style: 'FILL_AND_OUTLINE',
                  outlineColor: [0.0, 0.0, 0.0, 1.0],
                  outlineWidth: 2.0,
                },
                text: 'Philadelphia',
                fontSize: 24.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
              },
              screen: {
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'CENTER',
                rotation: 0.0,
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [135 / 255, 206 / 255, 235 / 255, 1.0],
              properties: null,
            },
          ],
        },
        {
          label: 'Set properties',
          entities: [
            {
              id: 'label-properties',
              show: true,
              name: '',
              positionDegrees: [-75.1641667, 39.9522222, 300000.0],
              shape: {
                type: 'label',
                appearance: {
                  ...defaultAppearance,
                  showBackground: true,
                },
                text: 'Philadelphia',
                fontSize: 30.0,
                scale: 2.0,
                pixelOffset: [0.0, 0.0],
              },
              screen: {
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'CENTER',
                rotation: 0.0,
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
              id: 'label-offset-billboard',
              show: true,
              name: '',
              positionDegrees: [-75.1641667, 39.9522222, 0.0],
              shape: {
                type: 'billboard',
                image: FACILITY_DATA_URL,
                width: 16.0,
                height: 16.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
              },
              screen: {
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'CENTER',
                rotation: 0.0,
                scaleByDistance: [150.0, 5.0, 15000000.0, 0.5],
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
            {
              id: 'label-offset-label',
              show: true,
              name: '',
              positionDegrees: [-75.1641667, 39.9522222, 0.0],
              shape: {
                type: 'label',
                appearance: {
                  ...defaultAppearance,
                  showBackground: true,
                },
                text: 'Label on top of scaling billboard',
                fontSize: 20.0,
                scale: 1.0,
                pixelOffset: [0.0, -16.0],
              },
              screen: {
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'CENTER',
                rotation: 0.0,
                pixelOffsetScaleByDistance: [150.0, 3.0, 15000000.0, 0.5],
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
              id: 'label-fade-ny',
              show: true,
              name: '',
              positionDegrees: [-73.94, 40.67, 0.0],
              shape: {
                type: 'label',
                appearance: { ...defaultAppearance },
                text: 'New York',
                fontSize: 30.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
              },
              screen: {
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'CENTER',
                rotation: 0.0,
                translucencyByDistance: [150.0, 1.0, 150000000.0, 0.0],
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
            {
              id: 'label-fade-atlanta',
              show: true,
              name: '',
              positionDegrees: [-84.39, 33.75, 0.0],
              shape: {
                type: 'label',
                appearance: { ...defaultAppearance },
                text: 'Atlanta',
                fontSize: 30.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
              },
              screen: {
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'CENTER',
                rotation: 0.0,
                translucencyByDistance: [150000.0, 1.0, 15000000.0, 0.0],
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
          label: 'Scale label by distance',
          entities: [
            {
              id: 'label-scale-by-distance',
              show: true,
              name: '',
              positionDegrees: [-75.1641667, 39.9522222, 0.0],
              shape: {
                type: 'label',
                appearance: { ...defaultAppearance },
                text: 'Philadelphia',
                fontSize: 30.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
              },
              screen: {
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'CENTER',
                rotation: 0.0,
                scaleByDistance: [150.0, 2.0, 15000000.0, 0.5],
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
          label: 'Set label with right-to-left language',
          entities: [
            {
              id: 'label-rtl',
              show: true,
              name: '',
              positionDegrees: [-75.1641667, 39.9522222, 0.0],
              shape: {
                type: 'label',
                appearance: { ...defaultAppearance },
                text: 'Master (אדון): Hello\nתלמיד (student): שלום',
                fontSize: 30.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
              },
              screen: {
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'CENTER',
                rotation: 0.0,
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
          label: 'Animate Label',
          entities: [
            {
              id: 'label-animate',
              show: true,
              name: '',
              positionDegrees: [-75.1641667, 39.9522222, 0.0],
              shape: {
                type: 'label',
                appearance: {
                  ...defaultAppearance,
                  style: 'FILL_AND_OUTLINE',
                  outlineColor: [0.0, 0.0, 0.0, 1.0],
                  outlineWidth: 1.0,
                },
                text: 'Philadelphia',
                fontSize: 20.0,
                scale: 1.0,
                pixelOffset: [0.0, 0.0],
              },
              screen: {
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'CENTER',
                rotation: 0.0,
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
      initial: 0,
    };

    const entities = variants.variants[0].entities;
    return {
      entities: JSON.parse(JSON.stringify(entities)),
      variants: JSON.parse(JSON.stringify(variants)),
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: `https://sandcastle.cesium.com/?id=${id}`,
        format: 'variants',
        path: `../../tests/sandcastle/full/${id}.variants.json`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: entities.length,
        variantCount: variants.variants.length,
        framing: 'viewer-default',
        shouldAnimate: false,
        requirements: [
          'Sandcastle toolbar menu with 8 label styling and distance variations',
          'font, style (FILL_AND_OUTLINE), outline width and color tinting',
          'scaleByDistance, translucencyByDistance, pixelOffsetScaleByDistance curves',
          'Hebrew / right-to-left multilingual text rendering and background styling',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === '3d-models') {
    const quat = headingPitchRollQuaternion(
      -123.0744619,
      44.0503706,
      (135 * Math.PI) / 180,
      0,
      0,
    );

    const modelOptions = [
      {
        text: 'Aircraft',
        url: '../../SampleData/models/CesiumAir/Cesium_Air.glb',
        height: 5000.0,
      },
      {
        text: 'Drone',
        url: '../../SampleData/models/CesiumDrone/CesiumDrone.glb',
        height: 150.0,
      },
      {
        text: 'Ground Vehicle',
        url: '../../SampleData/models/GroundVehicle/GroundVehicle.glb',
        height: 0.0,
      },
      {
        text: 'Hot Air Balloon',
        url: '../../SampleData/models/CesiumBalloon/CesiumBalloon.glb',
        height: 1000.0,
      },
      {
        text: 'Milk Truck',
        url: '../../SampleData/models/CesiumMilkTruck/CesiumMilkTruck.glb',
        height: 0.0,
      },
      {
        text: 'Skinned Character',
        url: '../../SampleData/models/CesiumMan/Cesium_Man.glb',
        height: 0.0,
      },
      {
        text: 'Unlit Box',
        url: '../../SampleData/models/BoxUnlit/BoxUnlit.gltf',
        height: 10.0,
      },
      {
        text: 'Draco Compressed Model',
        url: '../../SampleData/models/DracoCompressed/CesiumMilkTruck.gltf',
        height: 0.0,
      },
      {
        text: 'KTX2 Compressed Balloon',
        url: '../../SampleData/models/CesiumBalloonKTX2/CesiumBalloonKTX2.glb',
        height: 1000.0,
      },
      {
        text: 'Instanced Box',
        url: '../../SampleData/models/BoxInstanced/BoxInstanced.gltf',
        height: 15.0,
      },
    ];

    const variants = {
      variants: modelOptions.map(opt => ({
        label: opt.text,
        entities: [
          {
            id: opt.url,
            show: true,
            name: opt.url,
            positionDegrees: [-123.0744619, 44.0503706, opt.height],
            orientation: quat,
            shape: {
              type: 'model',
              uri: opt.url,
              scale: 1.0,
              minimumPixelSize: 128.0,
              maximumScale: 20000.0,
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
      initial: 0,
    };

    const entities = variants.variants[0].entities;
    return {
      entities: JSON.parse(JSON.stringify(entities)),
      variants: JSON.parse(JSON.stringify(variants)),
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: `https://sandcastle.cesium.com/?id=${id}`,
        format: 'variants',
        path: `../../tests/sandcastle/full/${id}.variants.json`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: entities.length,
        variantCount: variants.variants.length,
        framing: 'zoomToEntities',
        shouldAnimate: true,
        trackedEntity: modelOptions[0].url,
        requirements: [
          'Sandcastle toolbar menu with 10 glTF 3D model entity variations',
          'headingPitchRollQuaternion orientation computed from heading 135 deg at ground/altitude positions',
          'minimumPixelSize: 128 and maximumScale: 20000 dimensions',
          'viewer.trackedEntity tracking the active 3D model entity',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'czml-model-node-transformations') {
    let capturedCzml;
    const Cesium = {
      Viewer: class {
        constructor() {
          this.dataSources = {
            add: p => {
              capturedCzml = p;
              return Promise.resolve({
                entities: {
                  getById: () => ({ id: 'model' }),
                },
              });
            },
          };
          this.trackedEntity = null;
        }
      },
      CzmlDataSource: {
        load: czml => czml,
      },
    };

    const executable = source.replace(/^import .*;\s*$/gm, '');
    const ctx = vm.createContext({
      Cesium,
      window: { alert: () => {} },
      document: { getElementById: () => ({}) },
    });
    vm.runInContext(executable, ctx);
    assert(capturedCzml);

    const doc = capturedCzml[0];
    const packet = capturedCzml[1];
    assert.equal(doc.id, 'document');
    assert.equal(packet.id, 'model');
    assert(packet.model);
    assert(packet.model.nodeTransformations);

    const expectedEntities = [
      {
        id: packet.id,
        show: true,
        name: '',
        positionDegrees: [-77.0, 37.0, 100000.0],
        shape: {
          type: 'model',
          uri: packet.model.gltf,
          scale: 1.0,
          minimumPixelSize: 0.0,
          runAnimations: packet.model.runAnimations,
          nodeTransformations: packet.model.nodeTransformations,
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [1.0, 1.0, 1.0, 1.0],
        viewFrom: packet.viewFrom.cartesian,
        properties: null,
      },
    ];

    return {
      entities: JSON.parse(JSON.stringify(expectedEntities)),
      czml: JSON.parse(JSON.stringify(capturedCzml)),
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: `https://sandcastle.cesium.com/?id=${id}`,
        format: 'czml',
        path: `../../tests/sandcastle/full/${id}.czml`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: 1,
        framing: 'zoomToEntities',
        shouldAnimate: true,
        trackedEntity: 'model',
        requirements: [
          'CZML model packet decoding with CesiumMan glTF URI and viewFrom offset',
          'time-dynamic quaternion rotation tracks in nodeTransformations',
          'viewer.trackedEntity tracking the model',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'czml-model-articulations') {
    let capturedCzml;
    const Cesium = {
      Viewer: class {
        constructor() {
          this.dataSources = {
            add: p => {
              capturedCzml = p;
              return Promise.resolve({
                entities: {
                  getById: () => ({ id: 'test model' }),
                },
              });
            },
          };
          this.trackedEntity = null;
        }
      },
      CzmlDataSource: {
        load: czml => czml,
      },
    };

    const executable = source.replace(/^import .*;\s*$/gm, '');
    const ctx = vm.createContext({
      Cesium,
      console: { error: () => {} },
      document: { getElementById: () => ({}) },
    });
    vm.runInContext(executable, ctx);
    assert(capturedCzml);

    const doc = capturedCzml[0];
    const packet = capturedCzml[1];
    assert.equal(doc.id, 'document');
    assert.equal(packet.id, 'test model');
    assert(packet.model);
    assert(packet.model.articulations);

    const expectedEntities = [
      {
        id: packet.id,
        show: true,
        name: packet.name,
        positionDegrees: [-77.0, 37.0, 10000.0],
        shape: {
          type: 'model',
          uri: packet.model.gltf,
          scale: 2.0,
          minimumPixelSize: 128.0,
          runAnimations: packet.model.runAnimations,
          articulations: packet.model.articulations,
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [1.0, 1.0, 1.0, 1.0],
        properties: null,
      },
    ];

    return {
      entities: JSON.parse(JSON.stringify(expectedEntities)),
      czml: JSON.parse(JSON.stringify(capturedCzml)),
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: `https://sandcastle.cesium.com/?id=${id}`,
        format: 'czml',
        path: `../../tests/sandcastle/full/${id}.czml`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: 1,
        framing: 'zoomToEntities',
        shouldAnimate: true,
        trackedEntity: 'test model',
        requirements: [
          'CZML model packet decoding with launch vehicle glTF and LOOP_STOP clock',
          'Lagrange interpolated degree-2 articulation tracks in model.articulations',
          'viewer.trackedEntity tracking test model',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  throw new Error(`Unhandled batch36 recipe: ${id}`);
}
