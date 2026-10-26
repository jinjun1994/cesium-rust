import vm from 'node:vm';
import assert from 'node:assert/strict';

export const batch28Ids = ['czml-rectangle', 'partial-ellipsoids'];

function quatFromAxisAngle(axis, angle) {
  const half = angle * 0.5;
  const s = Math.sin(half);
  return [axis[0] * s, axis[1] * s, axis[2] * s, Math.cos(half)];
}

function quatMultiply(q1, q2) {
  const [x1, y1, z1, w1] = q1;
  const [x2, y2, z2, w2] = q2;
  return [
    w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2,
    w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2,
    w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2,
    w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2,
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
  const r = [[0, 0, 0], [0, 0, 0], [0, 0, 0]];
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
  const lon = lonDeg * rad, lat = latDeg * rad;
  const sLon = Math.sin(lon), cLon = Math.cos(lon);
  const sLat = Math.sin(lat), cLat = Math.cos(lat);
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

export function extractBatch28(id, source, item, hash, image) {
  let loaded, loadPromise, added, zooms = 0, viewers = 0;
  const records = [];

  class Cartesian3 {
    constructor(x, y, z) {
      Object.assign(this, { x, y, z });
    }
    static fromDegrees(lon, lat, height = 0) {
      const c = new Cartesian3(lon, lat, height);
      c.degrees = [lon, lat, height];
      return c;
    }
  }

  class Color {
    constructor(r, g, b, a = 1) {
      this.rgba = [r, g, b, a];
    }
    withAlpha(a) {
      return new Color(...this.rgba.slice(0, 3), a);
    }
  }
  Object.assign(Color, {
    BLUE: new Color(0, 0, 1),
    RED: new Color(1, 0, 0),
    YELLOW: new Color(1, 1, 0),
    GREEN: new Color(0, 128 / 255, 0),
    DARKCYAN: new Color(0, 139 / 255, 139 / 255),
    WHITE: new Color(1, 1, 1),
    BLACK: new Color(0, 0, 0),
  });

  class HeadingPitchRoll {
    constructor(heading = 0, pitch = 0, roll = 0) {
      Object.assign(this, { heading, pitch, roll });
    }
  }

  const Transforms = {
    headingPitchRollQuaternion(pos, hpr) {
      assert(pos.degrees);
      return headingPitchRollQuaternion(
        pos.degrees[0],
        pos.degrees[1],
        hpr.heading,
        hpr.pitch,
        hpr.roll,
      );
    },
  };

  class CzmlDataSource {
    static load(data) {
      assert(!loaded);
      loaded = JSON.parse(JSON.stringify(data));
      loadPromise = Promise.resolve({ czml: data });
      return loadPromise;
    }
  }

  class Viewer {
    constructor(container) {
      assert.equal(container, 'cesiumContainer');
      viewers++;
      this.entities = {
        add(o) {
          records.push(o);
          return o;
        },
      };
      this.dataSources = {
        add(p) {
          assert.equal(p, loadPromise);
          added = p;
          return p;
        },
      };
    }
    zoomTo(v) {
      assert.equal(v, loaded ? loadPromise : this.entities);
      zooms++;
    }
  }

  const Cesium = {
    Viewer,
    Cartesian3,
    Color,
    HeadingPitchRoll,
    Transforms,
    CzmlDataSource,
    Math: {
      toRadians: n => (n * Math.PI) / 180,
      PI_OVER_TWO: Math.PI / 2,
      PI: Math.PI,
    },
  };

  const executable = source.replace(/^import .*;\s*$/gm, '');
  assert(!/\bimport\b/.test(executable));
  vm.runInNewContext(executable, { Cesium, Math }, { timeout: 1000 });
  assert.equal(viewers, 1);
  assert.equal(zooms, 1);

  if (id === 'czml-rectangle') {
    assert(loaded);
    assert.equal(added, loadPromise);
    assert.equal(loaded[0].version, '1.0');
    assert.equal(loaded.length - 1, 4);

    const entities = loaded.slice(1).map(e => {
      const g = e.rectangle;
      const coords = g.coordinates.wsenDegrees;
      const fill = g.fill ?? true;
      const outline = g.outline ?? false;
      const outlineColor = g.outlineColor ? g.outlineColor.rgba.map(n => n / 255) : [0, 0, 0, 1];
      let color = [1, 1, 1, 1];
      let material;

      if (g.material?.solidColor) {
        color = g.material.solidColor.color.rgba.map(n => n / 255);
      } else if (g.material?.stripe) {
        const s = g.material.stripe;
        material = {
          type: 'stripe',
          evenColor: s.evenColor.rgba.map(n => n / 255),
          oddColor: s.oddColor.rgba.map(n => n / 255),
          repeat: s.repeat ?? 1,
          offset: s.offset ?? 0,
          orientation: s.orientation ?? 'HORIZONTAL',
        };
      } else if (g.material?.image) {
        const img = g.material.image;
        material = {
          type: 'image',
          image: image || img.image.uri || img.image,
          color: img.color ? img.color.rgba.map(n => n / 255) : [1, 1, 1, 1],
          repeat: img.repeat ?? [1, 1],
          transparent: img.transparent ?? false,
        };
      }

      return {
        id: e.id,
        name: e.name || '',
        positionDegrees: null,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: coords,
          height: g.height ?? 0,
          extrudedHeight: g.extrudedHeight ?? null,
          rotation: g.rotation ?? 0,
        },
        color,
        material,
        fill,
        outline,
        outlineColor,
        outlineWidth: 1,
        properties: null,
      };
    });

    return {
      entities: JSON.parse(JSON.stringify(entities)),
      czml: loaded,
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
        shouldAnimate: false,
        requirements: [
          'all four CZML rectangles with extruded height, rotation, and materials',
          'solid color, vertical stripe, and image materials',
          'fill and outline rendering',
          'zoomTo CZML data source',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'partial-ellipsoids') {
    assert.equal(records.length, 12);
    const entities = records.map((e, i) => {
      const g = e.ellipsoid;
      const shape = {
        type: 'ellipsoid',
        radii: [g.radii.x, g.radii.y, g.radii.z],
        ...(g.innerRadii ? { innerRadii: [g.innerRadii.x, g.innerRadii.y, g.innerRadii.z] } : {}),
        ...(g.minimumClock !== undefined ? { minimumClock: g.minimumClock } : {}),
        ...(g.maximumClock !== undefined ? { maximumClock: g.maximumClock } : {}),
        ...(g.minimumCone !== undefined ? { minimumCone: g.minimumCone } : {}),
        ...(g.maximumCone !== undefined ? { maximumCone: g.maximumCone } : {}),
      };

      return {
        id: `partial-ellipsoids-${i + 1}`,
        name: e.name || '',
        positionDegrees: e.position.degrees,
        ...(e.orientation ? { orientation: e.orientation } : {}),
        shape,
        color: g.material.rgba,
        fill: true,
        outline: g.outline ?? false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        properties: { sandcastleCase: id, upstreamEntity: i + 1 },
      };
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
        shouldAnimate: false,
        requirements: [
          'all twelve whole and partial ellipsoids with correct radii and cutouts',
          'inner radii hollow shells, minimum and maximum clock angles, minimum and maximum cone angles',
          'orientation quaternions from headingPitchRoll',
          'outer and inner surfaces, end caps, and outlines',
          'viewer.zoomTo entire collection',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  throw new Error(`unsupported batch28 recipe ${id}`);
}
