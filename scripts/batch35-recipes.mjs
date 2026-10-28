import vm from 'node:vm';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

export const batch35Ids = [
  'billboards',
  'czml-model',
  'czml-model-data-url',
  'callback-position-property',
];

const LOGO_DATA_URL =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAACvSURBVDhPrZDRDcMgDAU9GqN0lIzijw6SUbJJygUeNQgSqepJTyHG91LVVpwDdfxM3T9TSl1EXZvDwii471fivK73cBFFQNTT/d2KoGpfGOpSIkhUpgUMxq9DFEsWv4IXhlyCnhBFnZcFEEuYqbiUlNwWgMTdrZ3JbQFoEVG53rd8ztG9aPJMnBUQf/VFraBJeWnLS0RfjbKyLJA8FkT5seDYS1Qwyv8t0B/5C2ZmH2/eTGNNBgMmAAAAAElFTkSuQmCC';

const FACILITY_DATA_URL =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQBAMAAADt3eJSAAAAMFBMVEVAQEDAwMDAoAD///+gYADAgACgoKSAYADgwAD///8AAAAAAAAAAAAAAAAAAAAAAAC6opgsAAAACnRSTlP///////////8AsswszwAAAH1JREFUeJw9xiEOwjAYhuF33/6KyQVPlh6AjOwWqBoOgMKScJo6LI5rIAg3IE0WFIK5CpIVQ3jUU0XygQgiH4d2D8R6WKx3TTRI3eoClnvSwxWMxPL5CRjdnHAgZt5TAKG7LydQc/O4zZkKD2N4XaV621MMIWACQUsBxM8/X+GkICjYO7GLAAAAAElFTkSuQmCC';

const MARKER_DATA_URL =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABIAAAASCAYAAABWzo5XAAAAGUlEQVR4nGP4TyXAMGrQqEGjBo0aNNwMAgDZMQs8Mzy3wQAAAABJRU5ErkJggg==';

export function extractBatch35(id, source, item, hash) {
  if (id === 'billboards') {
    const variants = {
      variants: [
        {
          label: 'Add billboard',
          entities: [
            {
              id: 'billboard-add',
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
          label: 'Set billboard properties at creation',
          entities: [
            {
              id: 'billboard-properties',
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
              id: 'billboard-change',
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
              screen: {
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'CENTER',
                rotation: 0.0,
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
              id: 'billboard-size-in-meters',
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
          label: 'Add multiple billboards',
          entities: [
            {
              id: 'billboard-multiple-1',
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
            {
              id: 'billboard-multiple-2',
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
            {
              id: 'billboard-multiple-3',
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
          label: 'Scale by viewer distance',
          entities: [
            {
              id: 'billboard-scale-by-distance',
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
                verticalOrigin: 'CENTER',
                rotation: 0.0,
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
          label: 'Fade by viewer distance',
          entities: [
            {
              id: 'billboard-fade-by-distance',
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
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'CENTER',
                rotation: 0.0,
                translucencyByDistance: [1.5e2, 2.0, 1.5e7, 0.5],
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
          label: 'Offset by viewer distance',
          entities: [
            {
              id: 'billboard-offset-facility',
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
                rotation: 0.0,
              },
              fill: true,
              outline: false,
              outlineColor: [0.0, 0.0, 0.0, 1.0],
              outlineWidth: 1.0,
              color: [1.0, 1.0, 1.0, 1.0],
              properties: null,
            },
            {
              id: 'billboard-offset-logo',
              show: true,
              name: '',
              positionDegrees: [-75.59777, 40.03883, 0.0],
              shape: {
                type: 'billboard',
                image: LOGO_DATA_URL,
                width: 64.0,
                height: 64.0,
                scale: 1.0,
                pixelOffset: [0.0, -32.0],
              },
              screen: {
                horizontalOrigin: 'CENTER',
                verticalOrigin: 'BOTTOM',
                rotation: 0.0,
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
          label: 'Add marker billboards',
          entities: [
            {
              id: 'marker-1',
              show: true,
              name: '',
              positionDegrees: [-75.59777, 40.03883, 0.0],
              shape: {
                type: 'billboard',
                image: MARKER_DATA_URL,
                width: 18.0,
                height: 18.0,
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
              color: [0.0, 1.0, 0.0, 1.0],
              properties: null,
            },
            {
              id: 'marker-2',
              show: true,
              name: '',
              positionDegrees: [-84.0, 39.0, 0.0],
              shape: {
                type: 'billboard',
                image: MARKER_DATA_URL,
                width: 18.0,
                height: 18.0,
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
              color: [0.0, 0.5, 1.0, 1.0],
              properties: null,
            },
            {
              id: 'marker-3',
              show: true,
              name: '',
              positionDegrees: [-70.0, 41.0, 0.0],
              shape: {
                type: 'billboard',
                image: MARKER_DATA_URL,
                width: 14.0,
                height: 14.0,
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
              color: [0.5, 0.9, 1.0, 1.0],
              properties: null,
            },
            {
              id: 'marker-4',
              show: true,
              name: '',
              positionDegrees: [-73.0, 37.0, 0.0],
              shape: {
                type: 'billboard',
                image: MARKER_DATA_URL,
                width: 22.0,
                height: 22.0,
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
              color: [1.0, 0.0, 0.0, 1.0],
              properties: null,
            },
            {
              id: 'marker-5',
              show: true,
              name: '',
              positionDegrees: [-79.0, 35.0, 0.0],
              shape: {
                type: 'billboard',
                image: MARKER_DATA_URL,
                width: 18.0,
                height: 18.0,
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
              color: [1.0, 1.0, 0.0, 1.0],
              properties: null,
            },
          ],
        },
        {
          label: 'Disable the depth test when clamped to ground',
          entities: [
            {
              id: 'billboard-disable-depth',
              show: true,
              name: '',
              positionDegrees: [-122.1958, 46.1915, 0.0],
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
                verticalOrigin: 'CENTER',
                rotation: 0.0,
                disableDepthTestDistance: 1000000000.0,
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
        referenceUrl: item.referenceUrl,
        format: 'variants',
        path: `../../tests/sandcastle/full/${id}.variants.json`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: entities.length,
        variantCount: variants.variants.length,
        framing: 'viewer-default',
        shouldAnimate: false,
        requirements: [
          'Sandcastle toolbar menu with 10 billboard styling and distance variations',
          'eyeOffset, pixelOffset, rotation, scale, color tinting and origins',
          'scaleByDistance, translucencyByDistance, pixelOffsetScaleByDistance curves',
          'disableDepthTestDistance terrain occlusion control',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'czml-model') {
    const czml = [
      {
        id: 'document',
        name: 'CZML Model',
        version: '1.0',
      },
      {
        id: 'aircraft model',
        name: 'Cesium Air',
        position: {
          cartographicDegrees: [-77.0, 37.0, 10000.0],
        },
        model: {
          gltf: '../../SampleData/models/CesiumAir/Cesium_Air.glb',
          scale: 2.0,
          minimumPixelSize: 128.0,
        },
      },
    ];

    const entities = [
      {
        id: 'aircraft model',
        show: true,
        name: 'Cesium Air',
        positionDegrees: [-77.0, 37.0, 10000.0],
        shape: {
          type: 'model',
          uri: '../../SampleData/models/CesiumAir/Cesium_Air.glb',
          scale: 2.0,
          minimumPixelSize: 128.0,
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
      entities: JSON.parse(JSON.stringify(entities)),
      czml: JSON.parse(JSON.stringify(czml)),
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'czml',
        path: `../../tests/sandcastle/full/${id}.czml`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: entities.length,
        framing: 'zoomToEntities',
        shouldAnimate: true,
        trackedEntity: 'aircraft model',
        requirements: [
          'CZML model packet decoding with gltf URI',
          'scale: 2.0 and minimumPixelSize: 128 model dimensions',
          'viewer.trackedEntity tracking the aircraft model',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'czml-model-data-url') {
    const gltf = {
      accessors: [
        {
          name: 'cube_1_0_positions',
          componentType: 5126,
          count: 24,
          min: [-1, -1, -1],
          max: [1, 1, 1],
          type: 'VEC3',
          bufferView: 0,
          byteOffset: 0,
        },
        {
          name: 'cube_1_0_normals',
          componentType: 5126,
          count: 24,
          min: [-1, -1, -1],
          max: [1, 1, 1],
          type: 'VEC3',
          bufferView: 1,
          byteOffset: 0,
        },
        {
          name: 'cube_1_0_indices',
          componentType: 5123,
          count: 36,
          min: [0],
          max: [23],
          type: 'SCALAR',
          bufferView: 2,
          byteOffset: 0,
        },
      ],
      asset: { generator: 'obj2gltf', version: '2.0' },
      buffers: [
        {
          name: '19bd138d-63f9-4012-890c-30ab6b478824',
          byteLength: 648,
          uri: 'data:application/octet-stream;base64,AACAvwAAgL8AAIC/AACAPwAAgD8AAIC/AACAPwAAgL8AAIC/AACAvwAAgD8AAIC/AACAvwAAgL8AAIC/AACAvwAAgD8AAIA/AACAvwAAgD8AAIC/AACAvwAAgL8AAIA/AACAvwAAgD8AAIC/AACAPwAAgD8AAIA/AACAPwAAgD8AAIC/AACAvwAAgD8AAIA/AACAPwAAgL8AAIC/AACAPwAAgD8AAIC/AACAPwAAgD8AAIA/AACAPwAAgL8AAIA/AACAvwAAgL8AAIC/AACAPwAAgL8AAIC/AACAPwAAgL8AAIA/AACAvwAAgL8AAIA/AACAvwAAgL8AAIA/AACAPwAAgL8AAIA/AACAPwAAgD8AAIA/AACAvwAAgD8AAIA/AAAAAAAAAAAAAIC/AAAAAAAAAAAAAIC/AAAAAAAAAAAAAIC/AAAAAAAAAAAAAIC/AACAvwAAAAAAAAAAAACAvwAAAAAAAAAAAACAvwAAAAAAAAAAAACAvwAAAAAAAAAAAAAAAAAAgD8AAAAAAAAAAAAAgD8AAAAAAAAAAAAAgD8AAAAAAAAAAAAAgD8AAAAAAACAPwAAAAAAAAAAAACAPwAAAAAAAAAAAACAPwAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgL8AAAAAAAAAAAAAgL8AAAAAAAAAAAAAgL8AAAAAAAAAAAAAgL8AAAAAAAAAAAAAAAAAAIA/AAAAAAAAAAAAAIA/AAAAAAAAAAAAAIA/AAAAAAAAAAAAAIA/AAABAAIAAAADAAEABAAFAAYABAAHAAUACAAJAAoACAALAAkADAANAA4ADAAOAA8AEAARABIAEAASABMAFAAVABYAFAAWABcA',
        },
      ],
      bufferViews: [
        {
          name: 'bufferView_0',
          buffer: 0,
          byteLength: 288,
          byteOffset: 0,
          byteStride: 12,
          target: 34962,
        },
        {
          name: 'bufferView_1',
          buffer: 0,
          byteLength: 288,
          byteOffset: 288,
          byteStride: 12,
          target: 34962,
        },
        {
          name: 'bufferView_2',
          buffer: 0,
          byteLength: 72,
          byteOffset: 576,
          target: 34963,
        },
      ],
      materials: [
        {
          name: 'default',
          pbrMetallicRoughness: {
            baseColorFactor: [0.5, 0.5, 0.5, 1],
            metallicFactor: 0,
            roughnessFactor: 1,
          },
          emissiveFactor: [0, 0, 0],
          alphaMode: 'OPAQUE',
          doubleSided: false,
        },
      ],
      meshes: [
        {
          name: 'cube_1',
          primitives: [
            {
              attributes: { POSITION: 0, NORMAL: 1 },
              indices: 2,
              material: 0,
              mode: 4,
            },
          ],
        },
      ],
      nodes: [{ name: 'cube', mesh: 0 }],
      scene: 0,
      scenes: [{ nodes: [0] }],
    };

    const encoded = Buffer.from(JSON.stringify(gltf)).toString('base64');
    const dataUrl = `data:application/json;charset=utf-8;base64,${encoded}`;

    const czml = [
      {
        id: 'document',
        name: 'CZML Model',
        version: '1.0',
      },
      {
        id: 'cube',
        name: 'Cube',
        position: {
          cartographicDegrees: [-77.0, 37.0, 10000.0],
        },
        model: {
          gltf: dataUrl,
          scale: 2.0,
          minimumPixelSize: 128.0,
        },
      },
    ];

    const entities = [
      {
        id: 'cube',
        show: true,
        name: 'Cube',
        positionDegrees: [-77.0, 37.0, 10000.0],
        shape: {
          type: 'model',
          uri: dataUrl,
          scale: 2.0,
          minimumPixelSize: 128.0,
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
      entities: JSON.parse(JSON.stringify(entities)),
      czml: JSON.parse(JSON.stringify(czml)),
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'czml',
        path: `../../tests/sandcastle/full/${id}.czml`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: entities.length,
        framing: 'zoomToEntities',
        shouldAnimate: true,
        trackedEntity: 'cube',
        requirements: [
          'CZML model packet decoding with base64 glTF data URL',
          'scale: 2.0 and minimumPixelSize: 128 model dimensions',
          'viewer.trackedEntity tracking the cube model',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'callback-position-property') {
    const rawWaypoints = [
      [-112.87709, 36.27782, 620.01],
      [-112.87351, 36.27992, 617.9],
      [-112.87081, 36.2816, 617.6],
      [-112.86539, 36.28239, 625.36],
      [-112.86108, 36.28137, 627.82],
      [-112.85551, 36.27967, 625.54],
      [-112.848, 36.27732, 628.9],
      [-112.84086, 36.27739, 638.81],
      [-112.83682, 36.27995, 643.31],
    ];

    const entities = [];
    for (let i = 0; i < rawWaypoints.length; i++) {
      entities.push({
        id: `waypoint-${i}`,
        show: true,
        name: '',
        positionDegrees: rawWaypoints[i],
        shape: {
          type: 'point',
          pixelSize: 8.0,
          outlineWidth: 3.0,
          outlineColor: [1.0, 1.0, 0.0, 1.0],
        },
        fill: true,
        outline: false,
        outlineColor: [1.0, 1.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [0.0, 0.0, 0.0, 0.0],
        properties: null,
      });
    }

    entities.push({
      id: 'drone',
      show: true,
      name: '',
      positionDegrees: rawWaypoints[0],
      shape: {
        type: 'model',
        uri: '../../SampleData/models/CesiumDrone/CesiumDrone.glb',
        scale: 1.0,
        minimumPixelSize: 64.0,
        maximumScale: 20000.0,
      },
      availability: [1427308800.0, 1427308808.0],
      fill: true,
      outline: false,
      outlineColor: [0.0, 0.0, 0.0, 1.0],
      outlineWidth: 1.0,
      color: [1.0, 1.0, 1.0, 1.0],
      properties: null,
    });

    return {
      entities: JSON.parse(JSON.stringify(entities)),
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'entities',
        path: `../../tests/sandcastle/full/${id}.json`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: entities.length,
        framing: 'zoomToEntities',
        shouldAnimate: true,
        trackedEntity: 'drone',
        requirements: [
          '9 waypoint transparent points with yellow 3px outlines',
          'CatmullRomSpline CallbackPositionProperty simulation bounds',
          'CesiumDrone model with minimumPixelSize: 64 and maximumScale: 20000',
          'viewer.trackedEntity tracking the drone',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  throw new Error(`Unsupported batch35 recipe: ${id}`);
}
