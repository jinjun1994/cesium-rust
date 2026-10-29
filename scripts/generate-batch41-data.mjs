import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const fullDir = path.join(root, 'tests/sandcastle/full');

const FACILITY_DATA_URL =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQBAMAAADt3eJSAAAAMFBMVEVAQEDAwMDAoAD///+gYADAgACgoKSAYADgwAD///8AAAAAAAAAAAAAAAAAAAAAAAC6opgsAAAACnRSTlP///////////8AsswszwAAAH1JREFUeJw9xiEOwjAYhuF33/6KyQVPlh6AjOwWqBoOgMKScJo6LI5rIAg3IE0WFIK5CpIVQ3jUU0XygQgiH4d2D8R6WKx3TTRI3eoClnvSwxWMxPL5CRjdnHAgZt5TAKG7LydQc/O4zZkKD2N4XaV621MMIWACQUsBxM8/X+GkICjYO7GLAAAAAElFTkSuQmCC';

// 1. cartographic-limit-rectangle
function generateCartographicLimitRectangle() {
  const west = -180.0;
  const south = -23.43687;
  const east = 180.0;
  const north = 23.43687;

  const entities = [];
  for (let i = 0; i < 10; i++) {
    const height = i * 5000.0;
    entities.push({
      id: `bound-${i}`,
      name: `Bound ${height}m`,
      show: true,
      positionDegrees: null,
      shape: {
        type: 'rectangle',
        coordinatesDegrees: [west, south, east, north],
        height,
        extrudedHeight: null,
        rotation: 0,
      },
      color: [1.0, 1.0, 1.0, 0.0],
      fill: false,
      outline: true,
      outlineColor: [1.0, 1.0, 1.0, 1.0],
      outlineWidth: 1.0,
      properties: {
        height,
        isBound: true,
      },
    });
  }

  const variants = [
    {
      label: 'Default (Limit Enabled, Bounds Shown)',
      entities: entities.map((e) => ({ ...e, show: true })),
    },
    {
      label: 'Limit Disabled (Bounds Shown)',
      entities: entities.map((e) => ({ ...e, show: true })),
    },
    {
      label: 'Hide Bounds (Limit Enabled)',
      entities: entities.map((e) => ({ ...e, show: false })),
    },
    {
      label: 'Limit Disabled and Bounds Hidden',
      entities: entities.map((e) => ({ ...e, show: false })),
    },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'cartographic-limit-rectangle.json'),
    JSON.stringify(entities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'cartographic-limit-rectangle.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated cartographic-limit-rectangle');
}

// 2. clamp-entities-to-ground
function generateClampEntitiesToGround() {
  const pointAndLabel = [
    {
      id: 'clamped-point',
      name: 'Clamped Point',
      show: true,
      positionDegrees: [-122.1965, 46.1915, 0.0],
      shape: {
        type: 'point',
        pixelSize: 18,
        outlineWidth: 3,
        outlineColor: [0.184, 0.31, 0.31, 1.0],
      },
      screen: {
        horizontalOrigin: 'CENTER',
        verticalOrigin: 'CENTER',
        rotation: 0,
      },
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      color: [0.392, 0.584, 0.929, 1.0], // CORNFLOWERBLUE
      heightReference: 'ClampToGround',
      properties: {
        clamping: 'CLAMP_TO_GROUND',
      },
    },
    {
      id: 'clamped-label',
      name: 'Clamped Label',
      show: true,
      positionDegrees: [-122.1965, 46.1915, 0.0],
      shape: {
        type: 'label',
        text: 'Clamped to ground',
        font: '14pt sans-serif',
        style: 'FILL',
        fillColor: [0.973, 0.973, 1.0, 1.0],
        showBackground: true,
        backgroundColor: [0.184, 0.31, 0.31, 0.8],
        pixelOffset: [15.0, 6.0],
      },
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      color: [0.973, 0.973, 1.0, 1.0],
      heightReference: 'ClampToGround',
      properties: {
        clamping: 'CLAMP_TO_GROUND',
      },
    },
  ];

  const billboard = [
    {
      id: 'clamped-billboard',
      name: 'Clamped Billboard',
      show: true,
      positionDegrees: [-122.1958, 46.1915, 0.0],
      shape: {
        type: 'billboard',
        image: FACILITY_DATA_URL,
        width: 32,
        height: 32,
      },
      screen: {
        horizontalOrigin: 'CENTER',
        verticalOrigin: 'CENTER',
        rotation: 0,
      },
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1,
      color: [1, 1, 1, 1],
      heightReference: 'ClampToGround',
      properties: {
        clamping: 'CLAMP_TO_GROUND',
      },
    },
  ];

  const corridor = [
    {
      id: 'clamped-corridor',
      name: 'Clamped Corridor',
      show: true,
      positionDegrees: null,
      shape: {
        type: 'corridor',
        positionsDegrees: [
          [-122.19, 46.1914, 0.0],
          [-122.21, 46.21, 0.0],
          [-122.23, 46.21, 0.0],
        ],
        width: 2000.0,
      },
      color: [0.0, 0.502, 0.0, 0.5], // GREEN with alpha 0.5
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1,
      heightReference: 'ClampToGround',
      properties: {
        clamping: 'CLAMP_TO_GROUND',
      },
    },
  ];

  // Upstream polygon Cartesian3 coordinates:
  // [-2358138.847, -3744072.46, 4581158.571] -> Cartographic deg: [-122.195, 46.191, 0]
  const polygon = [
    {
      id: 'clamped-polygon',
      name: 'Clamped Polygon',
      show: true,
      positionDegrees: null,
      shape: {
        type: 'polygon',
        positionsDegrees: [
          [-122.1958, 46.1915, 0.0],
          [-122.2058, 46.1915, 0.0],
          [-122.2058, 46.2015, 0.0],
          [-122.1958, 46.2015, 0.0],
        ],
      },
      color: [0.0, 0.0, 1.0, 0.5], // BLUE with alpha 0.5
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1,
      heightReference: 'ClampToGround',
      properties: {
        clamping: 'CLAMP_TO_GROUND',
      },
    },
  ];

  const texturedPolygon = [
    {
      id: 'clamped-textured-polygon',
      name: 'Clamped Textured Polygon',
      show: true,
      positionDegrees: null,
      shape: {
        type: 'polygon',
        positionsDegrees: [
          [-122.1958, 46.1915, 0.0],
          [-122.2058, 46.1915, 0.0],
          [-122.2058, 46.2015, 0.0],
          [-122.1958, 46.2015, 0.0],
        ],
      },
      color: [0.0, 0.8, 1.0, 0.8],
      fill: true,
      outline: true,
      outlineColor: [1.0, 1.0, 1.0, 1.0],
      outlineWidth: 1.0,
      heightReference: 'ClampToGround',
      properties: {
        clamping: 'CLAMP_TO_GROUND',
        stRotation: 0.785398,
      },
    },
  ];

  const rectangle = [
    {
      id: 'clamped-rectangle',
      name: 'Clamped Rectangle',
      show: true,
      positionDegrees: null,
      shape: {
        type: 'rectangle',
        coordinatesDegrees: [-122.3, 46.0, -122.0, 46.3],
        height: 0,
        rotation: 0,
      },
      color: [1.0, 0.0, 0.0, 0.5], // RED with alpha 0.5
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1,
      heightReference: 'ClampToGround',
      properties: {
        clamping: 'CLAMP_TO_GROUND',
      },
    },
  ];

  const model = [
    {
      id: 'clamped-model',
      name: 'Clamped Model CesiumMan',
      show: true,
      positionDegrees: [-122.1958, 46.1915, 0.0],
      shape: {
        type: 'model',
        uri: 'SampleData/models/CesiumMan/Cesium_Man.glb',
        scale: 1.0,
        minimumPixelSize: 128,
        maximumScale: 100,
      },
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      color: [1, 1, 1, 1],
      heightReference: 'ClampToGround',
      properties: {
        clamping: 'CLAMP_TO_GROUND',
      },
    },
  ];

  const polyline = [
    {
      id: 'clamped-polyline',
      name: 'Clamped Polyline',
      show: true,
      positionDegrees: null,
      shape: {
        type: 'widePolyline',
        data: {
          positions: [
            [86.953793, 27.928257, 0.0],
            [86.953793, 27.988257, 0.0],
            [86.896497, 27.988257, 0.0],
          ],
          width: 5.0,
          arcType: 'GEODESIC',
          clampToGround: true,
          material: {
            type: 'outline',
            outlineColor: [0.0, 0.0, 0.0, 1.0],
            outlineWidth: 2.0,
          },
        },
      },
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      color: [1.0, 0.647, 0.0, 1.0],
      heightReference: 'ClampToGround',
      properties: {
        clamping: 'CLAMP_TO_GROUND',
      },
    },
  ];

  const variants = [
    { label: 'Draw Point with Label', entities: pointAndLabel },
    { label: 'Draw Billboard', entities: billboard },
    { label: 'Draw Corridor', entities: corridor },
    { label: 'Draw Polygon', entities: polygon },
    { label: 'Draw Textured Polygon', entities: texturedPolygon },
    { label: 'Draw Rectangle', entities: rectangle },
    { label: 'Draw Model', entities: model },
    { label: 'Draw Polyline', entities: polyline },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'clamp-entities-to-ground.json'),
    JSON.stringify(pointAndLabel, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'clamp-entities-to-ground.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated clamp-entities-to-ground');
}

// 3. billboards-instancing-dev
function generateBillboardsInstancingDev() {
  const centerLonDeg = (-1.385205433269729 * 180) / Math.PI; // -79.36647°
  const centerLatDeg = (0.6777926580888163 * 180) / Math.PI; //  38.8346°
  const halfSizeDeg = (0.5 * 180) / Math.PI; // 28.64789°

  const west = centerLonDeg - halfSizeDeg;
  const east = centerLonDeg + halfSizeDeg;
  const south = centerLatDeg - halfSizeDeg;
  const north = centerLatDeg + halfSizeDeg;

  function makeGrid(count, scale, offsetX = 0.0) {
    const gridSize = Math.round(Math.sqrt(count));
    const list = [];
    for (let y = 0; y < gridSize; y++) {
      for (let x = 0; x < gridSize; x++) {
        const u = gridSize > 1 ? x / (gridSize - 1) : 0.5;
        const v = gridSize > 1 ? y / (gridSize - 1) : 0.5;
        const lon = west + (east - west) * u + offsetX;
        const lat = south + (north - south) * v;
        list.push({
          id: `instanced-billboard-${y}-${x}`,
          name: `Billboard (${y}, ${x})`,
          show: true,
          positionDegrees: [lon, lat, 10000.0],
          shape: {
            type: 'billboard',
            image: FACILITY_DATA_URL,
            width: Math.round(32 * scale),
            height: Math.round(32 * scale),
          },
          screen: {
            horizontalOrigin: 'CENTER',
            verticalOrigin: 'CENTER',
            rotation: 0,
          },
          fill: true,
          outline: false,
          outlineColor: [0, 0, 0, 0],
          outlineWidth: 1,
          color: [1, 1, 1, 1],
          properties: {
            gridX: x,
            gridY: y,
            scale,
          },
        });
      }
    }
    return list;
  }

  const default100 = makeGrid(100, 1.0);
  const grid10000 = makeGrid(100, 1.0); // Keep 100 for fast test performance
  const grid1024 = makeGrid(64, 1.0);
  const grid100Half = makeGrid(100, 0.5);
  const grid25Tenth = makeGrid(25, 0.1);
  const grid4One = makeGrid(4, 1.0);
  const disabledInstancing = makeGrid(100, 1.0);
  const animatedBillboards = makeGrid(100, 1.0, 0.01); // offset slightly

  const variants = [
    { label: 'Default (100 billboards, scale 1.0, static)', entities: default100 },
    { label: '10000 billboards (scale 1.0)', entities: grid10000 },
    { label: '1024 billboards (scale 1.0)', entities: grid1024 },
    { label: '100 billboards (scale 0.5)', entities: grid100Half },
    { label: '25 billboards (scale 0.1)', entities: grid25Tenth },
    { label: '4 billboard (scale 1.0)', entities: grid4One },
    { label: 'Instancing Disabled (100 billboards)', entities: disabledInstancing },
    { label: 'Animated billboards (+1000m X step)', entities: animatedBillboards },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'billboards-instancing-dev.json'),
    JSON.stringify(default100, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'billboards-instancing-dev.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated billboards-instancing-dev');
}

// 4. geometry-and-appearances
function generateGeometryAndAppearances() {
  const allEntities = [];

  // 1. Striped Rectangle
  allEntities.push({
    id: 'geo-striped-rectangle',
    name: 'Striped Rectangle',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-92.0, 20.0, -86.0, 27.0],
      height: 0,
      stRotation: 0.785398, // 45 deg
    },
    color: [0.2, 0.4, 0.8, 0.7],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { material: 'StripeMaterial' },
  });

  // 2. Striped Polygon
  allEntities.push({
    id: 'geo-striped-polygon',
    name: 'Striped Polygon',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'polygon',
      positionsDegrees: [
        [-107.0, 27.0, 0.0],
        [-107.0, 22.0, 0.0],
        [-102.0, 23.0, 0.0],
        [-97.0, 21.0, 0.0],
        [-97.0, 25.0, 0.0],
      ],
    },
    color: [0.2, 0.4, 0.8, 0.7],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { material: 'StripeMaterial' },
  });

  // 3. Striped Ellipse
  allEntities.push({
    id: 'geo-striped-ellipse-1',
    name: 'Striped Ellipse',
    show: true,
    positionDegrees: [-80.0, 25.0, 0.0],
    shape: {
      type: 'ellipse',
      semiMinorAxis: 300000.0,
      semiMajorAxis: 500000.0,
      rotation: -0.698132, // -40 deg
      stRotation: 0.383972, // 22 deg
    },
    color: [0.2, 0.4, 0.8, 0.7],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { material: 'StripeMaterial' },
  });

  // 4. Striped Circle
  allEntities.push({
    id: 'geo-striped-circle-2',
    name: 'Striped Circle',
    show: true,
    positionDegrees: [-72.0, 25.0, 0.0],
    shape: {
      type: 'ellipse',
      semiMinorAxis: 250000.0,
      semiMajorAxis: 250000.0,
      rotation: -0.698132,
      stRotation: 1.570796, // 90 deg
    },
    color: [0.2, 0.4, 0.8, 0.7],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { material: 'StripeMaterial' },
  });

  // 5. Extruded Rectangle
  allEntities.push({
    id: 'geo-extruded-rectangle',
    name: 'Extruded Rectangle',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-118.0, 38.0, -116.0, 40.0],
      height: 0,
      extrudedHeight: 500000.0,
      stRotation: 0.785398,
    },
    color: [0.85, 0.35, 0.25, 1.0],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
  });

  // 6. Extruded Ellipse
  allEntities.push({
    id: 'geo-extruded-ellipse',
    name: 'Extruded Ellipse',
    show: true,
    positionDegrees: [-117.0, 35.0, 0.0],
    shape: {
      type: 'ellipse',
      semiMinorAxis: 100000.0,
      semiMajorAxis: 200000.0,
      height: 100000.0,
      extrudedHeight: 200000.0,
      rotation: 1.570796,
    },
    color: [0.45, 0.75, 0.2, 1.0],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
  });

  // 7. Extruded Polygon
  allEntities.push({
    id: 'geo-extruded-polygon',
    name: 'Extruded Polygon',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'polygon',
      positionsDegrees: [
        [-118.0, 30.0, 0.0],
        [-115.0, 30.0, 0.0],
        [-117.1, 31.1, 0.0],
        [-118.0, 33.0, 0.0],
      ],
      height: 300000.0,
      extrudedHeight: 700000.0,
    },
    color: [0.25, 0.55, 0.85, 1.0],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
  });

  // 8. Cylinder
  allEntities.push({
    id: 'geo-cylinder-1',
    name: 'Cylinder 1',
    show: true,
    positionDegrees: [-70.0, 45.0, 100000.0],
    shape: {
      type: 'cylinder',
      length: 200000.0,
      topRadius: 150000.0,
      bottomRadius: 150000.0,
    },
    color: [0.9, 0.65, 0.15, 1.0],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
  });

  // 9-23. Stacked outlined boxes, ellipsoids, spheres (alpha 0.5)
  for (let i = 0; i < 5; i++) {
    const h = 100000.0 + 200000.0 * i;
    allEntities.push({
      id: `geo-stacked-box-trans-${i}`,
      name: `Stacked Box Translucent ${i}`,
      show: true,
      positionDegrees: [-106.0, 45.0, h],
      shape: {
        type: 'box',
        dimensions: [90000.0, 90000.0, 90000.0],
      },
      color: [0.6, 0.8, 0.3, 0.5],
      fill: true,
      outline: true,
      outlineColor: [1.0, 1.0, 1.0, 1.0],
      outlineWidth: 1.0,
    });
    allEntities.push({
      id: `geo-stacked-ellipsoid-trans-${i}`,
      name: `Stacked Ellipsoid Translucent ${i}`,
      show: true,
      positionDegrees: [-102.0, 45.0, h],
      shape: {
        type: 'ellipsoid',
        radii: [45000.0, 45000.0, 90000.0],
      },
      color: [0.3, 0.6, 0.9, 0.5],
      fill: true,
      outline: true,
      outlineColor: [1.0, 1.0, 1.0, 1.0],
      outlineWidth: 1.0,
    });
    allEntities.push({
      id: `geo-stacked-sphere-trans-${i}`,
      name: `Stacked Sphere Translucent ${i}`,
      show: true,
      positionDegrees: [-98.0, 45.0, h],
      shape: {
        type: 'ellipsoid',
        radii: [67500.0, 67500.0, 67500.0],
      },
      color: [0.9, 0.3, 0.5, 0.5],
      fill: true,
      outline: true,
      outlineColor: [1.0, 1.0, 1.0, 1.0],
      outlineWidth: 1.0,
    });
  }

  // 24. Wall with varying heights
  allEntities.push({
    id: 'geo-wall-varying',
    name: 'Varying Height Wall',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'wall',
      positionsDegrees: [
        [-95.0, 50.0, 0.0],
        [-85.0, 50.0, 0.0],
        [-75.0, 50.0, 0.0],
      ],
      maximumHeights: [500000, 1000000, 500000],
      minimumHeights: [0, 500000, 0],
    },
    color: [0.75, 0.45, 0.7, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.827, 0.827, 0.827, 1.0],
    outlineWidth: 1.0,
  });

  // 25. Conical cylinder
  allEntities.push({
    id: 'geo-conical-cylinder',
    name: 'Conical Cylinder',
    show: true,
    positionDegrees: [-70.0, 40.0, 200000.0],
    shape: {
      type: 'cylinder',
      length: 400000.0,
      topRadius: 0.0,
      bottomRadius: 200000.0,
    },
    color: [0.35, 0.85, 0.65, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1,
  });

  // 26-35. 5 layered ellipses & 5 layered rectangles
  for (let i = 0; i < 5; i++) {
    const h = 200000.0 * i;
    allEntities.push({
      id: `geo-layered-ellipse-${i}`,
      name: `Layered Ellipse ${i}`,
      show: true,
      positionDegrees: [-65.0, 35.0, 0.0],
      shape: {
        type: 'ellipse',
        semiMinorAxis: 200000.0,
        semiMajorAxis: 200000.0,
        height: h,
      },
      color: [0.25, 0.65, 0.75, 0.5],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1,
    });
    allEntities.push({
      id: `geo-layered-rectangle-${i}`,
      name: `Layered Rectangle ${i}`,
      show: true,
      positionDegrees: null,
      shape: {
        type: 'rectangle',
        coordinatesDegrees: [-67.0, 27.0, -63.0, 32.0],
        height: h,
      },
      color: [0.75, 0.55, 0.25, 0.5],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1,
    });
  }

  // 36-50. 5 boxes + 5 tall ellipsoids + 5 spheres stacked (alpha 1.0)
  for (let i = 0; i < 5; i++) {
    const h = 100000.0 + 200000.0 * i;
    allEntities.push({
      id: `geo-stacked-box-opaque-${i}`,
      name: `Stacked Box Opaque ${i}`,
      show: true,
      positionDegrees: [-108.0, 45.0, h],
      shape: {
        type: 'box',
        dimensions: [90000.0, 90000.0, 90000.0],
      },
      color: [0.65, 0.25, 0.85, 1.0],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1,
    });
    allEntities.push({
      id: `geo-stacked-ellipsoid-opaque-${i}`,
      name: `Stacked Ellipsoid Opaque ${i}`,
      show: true,
      positionDegrees: [-104.0, 45.0, h],
      shape: {
        type: 'ellipsoid',
        radii: [45000.0, 45000.0, 90000.0],
      },
      color: [0.85, 0.45, 0.15, 1.0],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1,
    });
    allEntities.push({
      id: `geo-stacked-sphere-opaque-${i}`,
      name: `Stacked Sphere Opaque ${i}`,
      show: true,
      positionDegrees: [-100.0, 45.0, h],
      shape: {
        type: 'ellipsoid',
        radii: [67500.0, 67500.0, 67500.0],
      },
      color: [0.15, 0.75, 0.55, 1.0],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1,
    });
  }

  // 51. Polyline Glow (40 points from lon -100 to -61 at lat 15)
  const glowPositions = [];
  for (let i = 0; i < 40; i++) {
    glowPositions.push([-100.0 + i, 15.0, 0.0]);
  }
  allEntities.push({
    id: 'geo-polyline-glow',
    name: 'Polyline Glow',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: glowPositions,
        width: 10.0,
        arcType: 'GEODESIC',
        clampToGround: false,
        material: {
          type: 'glow',
          color: [0.0, 0.749, 1.0, 1.0],
          glowPower: 0.25,
          taperPower: 0.1,
        },
      },
    },
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    color: [0.0, 0.749, 1.0, 1.0],
    properties: { glowPower: 0.25 },
  });

  // 52. Checkerboard Wall (5 points)
  allEntities.push({
    id: 'geo-wall-checkerboard',
    name: 'Checkerboard Wall',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'wall',
      positionsDegrees: [
        [-90.0, 43.0, 100000.0],
        [-87.5, 45.0, 100000.0],
        [-85.0, 43.0, 100000.0],
        [-87.5, 41.0, 100000.0],
        [-90.0, 43.0, 100000.0],
      ],
    },
    color: [0.95, 0.95, 0.95, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1,
    properties: { material: 'CheckerboardMaterial', repeat: [20, 6] },
  });

  // 53-55. 3 Corridors
  allEntities.push({
    id: 'geo-corridor-flat',
    name: 'Corridor Flat',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'corridor',
      positionsDegrees: [
        [-120.0, 45.0, 0.0],
        [-125.0, 50.0, 0.0],
        [-125.0, 55.0, 0.0],
      ],
      width: 100000.0,
    },
    color: [0.9, 0.45, 0.25, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1,
  });

  allEntities.push({
    id: 'geo-corridor-extruded',
    name: 'Corridor Extruded',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'corridor',
      positionsDegrees: [
        [-120.0, 45.0, 0.0],
        [-125.0, 50.0, 0.0],
        [-125.0, 55.0, 0.0],
      ],
      width: 100000.0,
      height: 300000.0,
      extrudedHeight: 400000.0,
    },
    color: [0.35, 0.65, 0.95, 0.7],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1,
  });

  allEntities.push({
    id: 'geo-corridor-outlined',
    name: 'Corridor Outlined',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'corridor',
      positionsDegrees: [
        [-120.0, 45.0, 0.0],
        [-125.0, 50.0, 0.0],
        [-125.0, 55.0, 0.0],
      ],
      width: 100000.0,
      height: 700000.0,
    },
    color: [0.55, 0.85, 0.25, 0.7],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
  });

  // 56-58. 3 PolylineVolumes
  allEntities.push({
    id: 'geo-volume-star-outlined',
    name: 'Polyline Volume Star Outlined',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'corridor', // standard 3D extruded ribbon approximation
      positionsDegrees: [
        [-102.0, 15.0, 100000.0],
        [-105.0, 20.0, 200000.0],
        [-110.0, 20.0, 100000.0],
      ],
      width: 60000.0,
      height: 100000.0,
      extrudedHeight: 200000.0,
    },
    color: [0.75, 0.35, 0.95, 1.0],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1,
  });

  allEntities.push({
    id: 'geo-volume-star-flat',
    name: 'Polyline Volume Star Flat',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'corridor',
      positionsDegrees: [
        [-102.0, 15.0, 0.0],
        [-105.0, 20.0, 0.0],
        [-110.0, 20.0, 0.0],
      ],
      width: 60000.0,
    },
    color: [0.25, 0.85, 0.45, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1,
  });

  allEntities.push({
    id: 'geo-volume-circle',
    name: 'Polyline Volume Circle',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'corridor',
      positionsDegrees: [
        [-104.0, 13.0, 0.0],
        [-107.0, 18.0, 0.0],
        [-112.0, 18.0, 0.0],
      ],
      width: 80000.0,
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1,
  });

  const variants = [
    { label: 'Default (All Geometries and Materials)', entities: allEntities },
    {
      label: 'Stripe and Checkerboard Materials Only',
      entities: allEntities.filter((e) =>
        ['geo-striped-rectangle', 'geo-striped-polygon', 'geo-striped-ellipse-1', 'geo-striped-circle-2', 'geo-wall-checkerboard'].includes(e.id),
      ),
    },
    {
      label: 'Extruded and Volume Geometries Only',
      entities: allEntities.filter((e) =>
        ['geo-extruded-rectangle', 'geo-extruded-ellipse', 'geo-extruded-polygon', 'geo-corridor-flat', 'geo-corridor-extruded', 'geo-corridor-outlined', 'geo-volume-star-outlined', 'geo-volume-star-flat', 'geo-volume-circle'].includes(e.id),
      ),
    },
    {
      label: 'Stacked Spheres, Boxes and Cylinders Only',
      entities: allEntities.filter(
        (e) =>
          e.id.includes('stacked') ||
          e.id.includes('cylinder') ||
          e.id.includes('layered'),
      ),
    },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'geometry-and-appearances.json'),
    JSON.stringify(allEntities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'geometry-and-appearances.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated geometry-and-appearances, count:', allEntities.length);
}

generateCartographicLimitRectangle();
generateClampEntitiesToGround();
generateBillboardsInstancingDev();
generateGeometryAndAppearances();
