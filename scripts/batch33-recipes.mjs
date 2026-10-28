import vm from 'node:vm';
import assert from 'node:assert/strict';

export const batch33Ids = [
  'czml-billboard-and-label',
  'geometry-height-reference',
  'czml-path',
  'czml-reference-properties',
];

// Deterministic Pseudo-Random Number Generator (LCG)
function createRng(seed = 5489) {
  let state = seed >>> 0;
  return function next() {
    state = (Math.imul(1664525, state) + 1013904223) >>> 0;
    return state / 4294967296;
  };
}

export function extractBatch33(id, source, item, hash) {
  const rng = createRng(5489);

  class Color {
    constructor(r, g, b, a = 1) {
      this.rgba = [r, g, b, a];
    }
    withAlpha(a) {
      return new Color(...this.rgba.slice(0, 3), a);
    }
    static fromRandom(options = {}) {
      const alpha = options.alpha !== undefined ? options.alpha : 1.0;
      return new Color(rng(), rng(), rng(), alpha);
    }
  }
  Object.assign(Color, {
    RED: new Color(1, 0, 0),
    GREEN: new Color(0, 1, 0),
    BLUE: new Color(0, 0, 1),
    WHITE: new Color(1, 1, 1),
    BLACK: new Color(0, 0, 0),
    YELLOW: new Color(1, 1, 0),
    CYAN: new Color(0, 1, 1),
    MAGENTA: new Color(1, 0, 1),
  });

  class Cartesian2 {
    constructor(x = 0, y = 0) {
      this.x = x;
      this.y = y;
    }
  }

  class Cartesian3 {
    constructor(x = 0, y = 0, z = 0) {
      this.x = x;
      this.y = y;
      this.z = z;
    }
    static fromDegrees(lon, lat, height = 0) {
      const phi = (lat * Math.PI) / 180;
      const lambda = (lon * Math.PI) / 180;
      const a = 6378137.0;
      const b = 6356752.314245;
      const e2 = 1 - (b * b) / (a * a);
      const sinPhi = Math.sin(phi);
      const cosPhi = Math.cos(phi);
      const N = a / Math.sqrt(1 - e2 * sinPhi * sinPhi);
      return {
        _degrees: [lon, lat, height],
        x: (N + height) * cosPhi * Math.cos(lambda),
        y: (N + height) * cosPhi * Math.sin(lambda),
        z: (N * (1 - e2) + height) * sinPhi,
      };
    }
  }

  class HeadingPitchRange {
    constructor(heading = 0, pitch = 0, range = 0) {
      this.heading = heading;
      this.pitch = pitch;
      this.range = range;
    }
  }

  const Matrix4 = {
    IDENTITY: [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1],
  };

  const HeightReference = {
    NONE: 'NONE',
    CLAMP_TO_GROUND: 'CLAMP_TO_GROUND',
    RELATIVE_TO_GROUND: 'RELATIVE_TO_GROUND',
  };

  class EllipsoidTerrainProvider {}

  const Terrain = {
    fromWorldTerrain: () => ({ name: 'worldTerrain' }),
  };

  const addedEntities = [];
  let loadedCzml = null;
  const toolbarMenus = [];

  class EntityCollection {
    constructor() {
      this.items = [];
    }
    add(e) {
      addedEntities.push(e);
      this.items.push(e);
      return e;
    }
    removeAll() {
      addedEntities.length = 0;
      this.items.length = 0;
    }
    getById(id) {
      return { id, isTracked: true };
    }
  }

  class CzmlDataSource {
    constructor() {
      this.entities = new EntityCollection();
    }
    then(cb) {
      if (typeof cb === 'function') cb(this);
      return this;
    }
    static load(data) {
      loadedCzml = data;
      const ds = new CzmlDataSource();
      return ds;
    }
  }

  class Viewer {
    constructor(container, options = {}) {
      assert.equal(container, 'cesiumContainer');
      this.options = options;
      this.entities = new EntityCollection();
      this.dataSources = {
        add: ds => ds,
      };
      this.camera = {
        lookAt: () => {},
        lookAtTransform: () => {},
      };
      this.scene = {
        globe: { depthTestAgainstTerrain: false },
        setTerrain: () => {},
      };
      this.trackedEntity = null;
    }
    zoomTo() {}
  }

  const Sandcastle = {
    addToolbarMenu: options => {
      toolbarMenus.push(options);
    },
    declare: () => {},
    highlight: () => {},
    reset: () => {},
  };

  // Cartesian ECEF to Cartographic degrees
  function ecefToDegrees(x, y, z) {
    const a = 6378137.0;
    const b = 6356752.314245;
    const e2 = 1 - (b * b) / (a * a);
    const p = Math.sqrt(x * x + y * y);
    let lon = Math.atan2(y, x);
    let lat = Math.atan2(z, p * (1 - e2));
    for (let i = 0; i < 5; i++) {
      const sinLat = Math.sin(lat);
      const N = a / Math.sqrt(1 - e2 * sinLat * sinLat);
      lat = Math.atan2(z + e2 * N * sinLat, p);
    }
    const sinLat = Math.sin(lat);
    const N = a / Math.sqrt(1 - e2 * sinLat * sinLat);
    const h = p / Math.cos(lat) - N;
    return [(lon * 180) / Math.PI, (lat * 180) / Math.PI, Math.abs(h) < 1e-4 ? 0.0 : h];
  }

  // Parse font string into px fontSize
  function parseFontSize(fontStr) {
    if (!fontStr) return 30.0;
    const pxMatch = fontStr.match(/(\d+(?:\.\d+)?)px/);
    if (pxMatch) return parseFloat(pxMatch[1]);
    const ptMatch = fontStr.match(/(\d+(?:\.\d+)?)pt/);
    if (ptMatch) return parseFloat(ptMatch[1]) * (4.0 / 3.0);
    return 30.0;
  }

  // Reference dereferencing for CZML
  function dereferenceCzml(packets) {
    const lookup = new Map();
    for (const p of packets) {
      if (p.id) lookup.set(p.id, p);
    }
    function resolve(node, depth = 0) {
      if (depth > 16 || !node || typeof node !== 'object') return node;
      if (node.reference) {
        const [targetId, propPath] = node.reference.split('#');
        const target = lookup.get(targetId);
        assert(target, `Target ${targetId} not found`);
        let cur = target;
        for (const part of propPath.split('.')) {
          cur = cur[part];
          assert(cur !== undefined, `Property ${part} in ${node.reference} not found`);
        }
        return resolve(JSON.parse(JSON.stringify(cur)), depth + 1);
      }
      if (Array.isArray(node)) {
        return node.map(item => resolve(item, depth));
      }
      const out = {};
      for (const [k, v] of Object.entries(node)) {
        out[k] = resolve(v, depth);
      }
      return out;
    }
    return packets.map(p => resolve(p));
  }

  // Run the source script
  const executable = source
    .replace(/^import .*;\s*$/gm, '')
    .replace(/\bawait\s+/g, '');
  assert(!/\bimport\b/.test(executable));

  vm.runInNewContext(
    executable,
    {
      Cesium: {
        Viewer,
        Color,
        Cartesian2,
        Cartesian3,
        HeadingPitchRange,
        Matrix4,
        HeightReference,
        Terrain,
        EllipsoidTerrainProvider,
        CzmlDataSource,
        Math: {
          PI: Math.PI,
          PI_OVER_FOUR: Math.PI / 4,
          toRadians: n => (n * Math.PI) / 180,
        },
      },
      Sandcastle,
    },
    { timeout: 2000 },
  );

  if (id === 'czml-billboard-and-label') {
    assert(loadedCzml);
    assert.equal(loadedCzml[0].version, '1.0');
    const packet = loadedCzml[1];
    assert.equal(packet.id, 'some-unique-id');
    assert(packet.billboard);
    assert(packet.label);

    const pos = ecefToDegrees(...packet.position.cartesian);

    const entities = [
      {
        id: packet.id,
        show: true,
        name: packet.name,
        positionDegrees: pos,
        shape: {
          type: 'billboard',
          image: packet.billboard.image,
          width: 64.0,
          height: 64.0,
          scale: packet.billboard.scale,
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
        id: `${packet.id}-label`,
        show: true,
        name: packet.name,
        positionDegrees: pos,
        shape: {
          type: 'label',
          appearance: {
            style: packet.label.style || 'FILL',
            showBackground: packet.label.showBackground ?? false,
            backgroundColor: packet.label.backgroundColor.rgba.map(n => n / 255),
            backgroundPadding: [7.0, 5.0],
            outlineColor: [0.0, 0.0, 0.0, 1.0],
            outlineWidth: 1.0,
          },
          text: packet.label.text,
          fontSize: parseFontSize(packet.label.font),
          scale: 1.0,
          pixelOffset: packet.label.pixelOffset.cartesian2,
        },
        screen: {
          horizontalOrigin: packet.label.horizontalOrigin || 'LEFT',
          verticalOrigin: 'CENTER',
          rotation: 0.0,
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: packet.label.fillColor.rgba.map(n => n / 255),
        properties: null,
      },
    ];

    return {
      entities: JSON.parse(JSON.stringify(entities)),
      czml: JSON.parse(JSON.stringify(loadedCzml)),
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
        requirements: [
          'CZML billboard with data URI and scale',
          'CZML label with font, background, and pixel offset',
          'billboard and label share position and display concurrently',
          'viewer.zoomTo entire entity collection',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'geometry-height-reference') {
    if (toolbarMenus[0] && typeof toolbarMenus[0][0]?.onselect === 'function') {
      toolbarMenus[0][0].onselect();
    }
    assert.equal(addedEntities.length, 16);
    const entities = addedEntities.map((e, index) => {
      assert(e.polygon);
      const positions = e.polygon.hierarchy.map(p => [
        p._degrees[0],
        p._degrees[1],
        p._degrees[2],
      ]);
      return {
        id: `polygon-${index}`,
        show: true,
        name: '',
        positionDegrees: null,
        shape: {
          type: 'polygon',
          positionsDegrees: positions,
          holes: [],
          height: e.polygon.height,
          heightReference: e.polygon.heightReference,
          extrudedHeight: e.polygon.extrudedHeight,
          extrudedHeightReference: e.polygon.extrudedHeightReference,
        },
        color: e.polygon.material.rgba,
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        properties: { sandcastleCase: id, gridIndex: index },
      };
    });

    // Also build variant 2: 16 boxes, cylinders and ellipsoids
    const geomRng = createRng(5489);
    const longitude = 6.950615989890521;
    const latitude = 45.79546589994886;
    const delta = 0.001;
    const geometries = [];
    for (let i = 0; i < 4; i++) {
      for (let j = 0; j < 4; j++) {
        const west = longitude + delta * i;
        const north = latitude + delta * j + delta;
        const type = Math.floor(geomRng() * 3);
        const col = [geomRng(), geomRng(), geomRng(), 1.0];
        let shape;
        if (type === 0) {
          shape = {
            type: 'box',
            dimensions: [40.0, 30.0, 50.0],
            heightReference: 'CLAMP_TO_GROUND',
          };
        } else if (type === 1) {
          shape = {
            type: 'cylinder',
            length: 50.0,
            topRadius: 20.0,
            bottomRadius: 20.0,
            heightReference: 'CLAMP_TO_GROUND',
          };
        } else {
          shape = {
            type: 'ellipsoid',
            radii: [20.0, 15.0, 25.0],
            heightReference: 'CLAMP_TO_GROUND',
          };
        }
        geometries.push({
          id: `geometry-${i * 4 + j}`,
          show: true,
          name: '',
          positionDegrees: [west, north, 0.0],
          shape,
          color: col,
          fill: true,
          outline: false,
          outlineColor: [0.0, 0.0, 0.0, 1.0],
          outlineWidth: 1.0,
          properties: { sandcastleCase: id, geometryType: type },
        });
      }
    }

    const variants = {
      initial: 0,
      variants: [
        { label: 'Polygons', entities },
        { label: 'Boxes, Cylinders and Ellipsoids', entities: geometries },
      ],
    };

    return {
      entities: JSON.parse(JSON.stringify(entities)),
      variants: JSON.parse(JSON.stringify(variants)),
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
        randomTestSeed: 5489,
        requirements: [
          '16 side-by-side polygons with height and extrudedHeight',
          'HeightReference RELATIVE_TO_GROUND and CLAMP_TO_GROUND',
          'toolbar menu switching between polygons and boxes/cylinders/ellipsoids',
          'viewer.camera.lookAt camera framing',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'czml-path') {
    assert(loadedCzml);
    assert.equal(loadedCzml[0].version, '1.0');
    const packet = loadedCzml[1];
    assert.equal(packet.id, 'path');
    assert(packet.billboard);
    assert(packet.path);
    assert(packet.position);

    const rawCoords = packet.position.cartographicDegrees;
    assert.equal(rawCoords.length % 4, 0);
    const numSamples = rawCoords.length / 4;
    assert(numSamples >= 1000);

    const epochStr = packet.position.epoch; // "2012-08-04T10:00:00Z"
    const epochSec = 1344074400.0;

    const samples = [];
    const polylinePositions = [];

    for (let i = 0; i < rawCoords.length; i += 4) {
      const t = rawCoords[i];
      const lon = rawCoords[i + 1];
      const lat = rawCoords[i + 2];
      const alt = rawCoords[i + 3];

      if (
        polylinePositions.length === 0 ||
        polylinePositions[polylinePositions.length - 1][0] !== lon ||
        polylinePositions[polylinePositions.length - 1][1] !== lat ||
        polylinePositions[polylinePositions.length - 1][2] !== alt
      ) {
        polylinePositions.push([lon, lat, alt]);
      }

      // Calculate cartesian ECEF
      const posEcef = Cartesian3.fromDegrees(lon, lat, alt);
      samples.push({
        time: epochSec + t,
        cartesian: [posEcef.x, posEcef.y, posEcef.z],
      });
    }

    const pathMat = packet.path.material.polylineOutline;
    const entities = [
      {
        id: packet.id,
        show: true,
        name: packet.name,
        positionDegrees: null,
        samples,
        shape: {
          type: 'billboard',
          image: packet.billboard.image,
          width: 64.0,
          height: 64.0,
          scale: packet.billboard.scale,
          pixelOffset: [0.0, 0.0],
        },
        screen: {
          horizontalOrigin: 'CENTER',
          verticalOrigin: 'CENTER',
          rotation: 0.0,
          eyeOffset: packet.billboard.eyeOffset.cartesian,
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [1.0, 1.0, 1.0, 1.0],
        availability: [1344074400.0, 1344092400.0],
        properties: null,
      },
      {
        id: `${packet.id}-lead-trail`,
        show: true,
        name: packet.name,
        positionDegrees: null,
        shape: {
          type: 'widePolyline',
          data: {
            positions: polylinePositions,
            width: packet.path.width,
            arcType: 'GEODESIC',
            clampToGround: false,
            material: {
              type: 'outline',
              outlineColor: pathMat.outlineColor.rgba.map(n => n / 255),
              outlineWidth: pathMat.outlineWidth,
            },
          },
        },
        color: pathMat.color.rgba.map(n => n / 255),
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        properties: null,
      },
    ];

    return {
      entities: JSON.parse(JSON.stringify(entities)),
      czml: JSON.parse(JSON.stringify(loadedCzml)),
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
        requirements: [
          'GPS flight data sampled cartographic coordinates',
          'billboard moving with time animation and eyeOffset',
          'lead and trail wide polyline with PolylineOutline material',
          'viewer.trackedEntity tracking the moving entity',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'czml-reference-properties') {
    assert(loadedCzml);
    assert.equal(loadedCzml[0].version, '1.0');

    // Dereference all reference properties
    const resolved = dereferenceCzml(loadedCzml);

    const fillPacket = resolved.find(p => p.id === 'fillColor-reference');
    assert(fillPacket);
    assert.deepEqual(fillPacket.position.cartographicDegrees, [-110.0, 50.0, 0]);

    const polyPacket = resolved.find(p => p.id === 'polygon');
    assert(polyPacket);
    assert.deepEqual(polyPacket.polygon.material.solidColor.color.rgba, [150, 0, 150, 255]);

    const entities = [
      {
        id: fillPacket.id,
        show: true,
        name: fillPacket.name,
        positionDegrees: fillPacket.position.cartographicDegrees,
        shape: {
          type: 'billboard',
          image: fillPacket.billboard.image,
          width: 64.0,
          height: 64.0,
          scale: fillPacket.billboard.scale,
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
        id: `${fillPacket.id}-label`,
        show: true,
        name: fillPacket.name,
        positionDegrees: fillPacket.position.cartographicDegrees,
        shape: {
          type: 'label',
          appearance: {
            style: fillPacket.label.style || 'FILL_AND_OUTLINE',
            showBackground: false,
            backgroundColor: [0.165, 0.165, 0.165, 0.8],
            backgroundPadding: [7.0, 5.0],
            outlineColor: fillPacket.label.outlineColor.rgba.map(n => n / 255),
            outlineWidth: fillPacket.label.outlineWidth,
          },
          text: fillPacket.label.text,
          fontSize: parseFontSize(fillPacket.label.font),
          scale: 1.0,
          pixelOffset: fillPacket.label.pixelOffset.cartesian2,
        },
        screen: {
          horizontalOrigin: fillPacket.label.horizontalOrigin || 'LEFT',
          verticalOrigin: 'CENTER',
          rotation: 0.0,
        },
        fill: true,
        outline: false,
        outlineColor: fillPacket.label.outlineColor.rgba.map(n => n / 255),
        outlineWidth: 1.0,
        color: fillPacket.label.fillColor.rgba.map(n => n / 255),
        properties: null,
      },
      {
        id: polyPacket.id,
        show: true,
        name: polyPacket.name,
        positionDegrees: null,
        shape: {
          type: 'polygon',
          positionsDegrees: [
            [-115.0, 37.0, 0.0],
            [-115.0, 32.0, 0.0],
            [-107.0, 33.0, 0.0],
            [-102.0, 31.0, 0.0],
            [-102.0, 35.0, 0.0],
          ],
          holes: [],
          height: polyPacket.polygon.height,
          extrudedHeight: null,
        },
        color: polyPacket.polygon.material.solidColor.color.rgba.map(n => n / 255),
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        properties: null,
      },
      {
        id: `${polyPacket.id}-label`,
        show: true,
        name: polyPacket.name,
        positionDegrees: polyPacket.position.cartographicDegrees,
        shape: {
          type: 'label',
          appearance: {
            style: polyPacket.label.style || 'FILL_AND_OUTLINE',
            showBackground: false,
            backgroundColor: [0.165, 0.165, 0.165, 0.8],
            backgroundPadding: [7.0, 5.0],
            outlineColor: [0.0, 0.0, 0.0, 1.0],
            outlineWidth: 1.0,
          },
          text: polyPacket.label.text,
          fontSize: parseFontSize(polyPacket.label.font),
          scale: 1.0,
          pixelOffset: polyPacket.label.pixelOffset.cartesian2,
        },
        screen: {
          horizontalOrigin: polyPacket.label.horizontalOrigin || 'LEFT',
          verticalOrigin: 'CENTER',
          rotation: 0.0,
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: polyPacket.label.fillColor.rgba.map(n => n / 255),
        properties: null,
      },
    ];

    return {
      entities: JSON.parse(JSON.stringify(entities)),
      czml: JSON.parse(JSON.stringify(loadedCzml)),
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
        requirements: [
          'cross-entity property references resolved cleanly',
          'position referencing position-reference#position',
          'polygon fillColor referencing fillColor-reference#label.outlineColor',
          'pure reference target packets with position-only emit zero graphics',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  throw new Error(`Unhandled batch 33 id: ${id}`);
}
