import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Imagery Color to Alpha
function generateImageryColorToAlpha() {
  const chromaTarget = {
    id: 'color-alpha-chroma-target',
    name: 'Oceanic Chroma Keying Calibration Datum',
    show: true,
    positionDegrees: [-140.0, 0.0, 20000.0],
    shape: {
      type: 'cylinder',
      length: 40000.0,
      topRadius: 15000.0,
      bottomRadius: 15000.0,
    },
    color: [0.0, 0.016, 0.059, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.2, 0.6, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { colorToAlpha: [0.0, 0.016, 0.059], threshold: 0.2, baseLayer: 'Bing Maps' },
  };

  const bumpMassing = {
    id: 'color-alpha-bump-massing',
    name: 'Single Tile Bump Map Topographic Massing Box',
    show: true,
    positionDegrees: [-138.0, 2.0, 15000.0],
    shape: {
      type: 'box',
      dimensions: [30000.0, 30000.0, 25000.0],
    },
    color: [0.75, 0.75, 0.75, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.95, 0.95, 0.95, 1.0],
    outlineWidth: 1.0,
    properties: { singleTileUri: '../images/earthbump1k.jpg', bumpThreshold: 0.1 },
  };

  const thresholdSphere = {
    id: 'color-alpha-threshold-sphere',
    name: 'Abyssal Keying Tolerance Threshold Sphere',
    show: true,
    positionDegrees: [-142.0, -2.0, 18000.0],
    shape: {
      type: 'ellipsoid',
      radii: [18000.0, 18000.0, 18000.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.1, 0.8, 0.7, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: { knockoutObservable: 'threshold', dynamicThreshold: 0.2 },
  };

  const calibrationZone = {
    id: 'color-alpha-calibration-zone',
    name: 'Pacific Equatorial Keying Footprint Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-144.0, -3.0, -136.0, 3.0],
      height: 3000.0,
      extrudedHeight: 12000.0,
      rotation: 0.0,
    },
    color: [0.2, 0.4, 0.8, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.7, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: { geocoderProvider: 'BING', transparencyChannel: 'Alpha Mask' },
  };

  const entities = [chromaTarget, bumpMassing, thresholdSphere, calibrationZone];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Ocean Keying Threshold 0.2 & Bump Layer Threshold 0.1',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'High Keying Threshold 0.5 (Expanded Transparency Mask)',
        entities: entities.map((e) => {
          if (e.id === 'color-alpha-chroma-target') {
            return { ...e, color: [0.1, 0.2, 0.4, 0.9] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Strict Keying Threshold 0.05 (Fine Oceanic Boundary)',
        entities: entities.map((e) => {
          if (e.id === 'color-alpha-bump-massing') {
            return { ...e, color: [0.9, 0.8, 0.6, 0.95] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Single Tile Bump Layer Isolated (Solid Continental Shading)',
        entities: entities.map((e) => {
          if (e.id === 'color-alpha-calibration-zone') {
            return { ...e, color: [0.3, 0.6, 0.4, 0.8] };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(outDir, 'imagery-color-to-alpha.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(outDir, 'imagery-color-to-alpha.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated imagery-color-to-alpha.json and .variants.json');
}

// 2. Imagery Cutout
function generateImageryCutout() {
  const cutoutAnchor = {
    id: 'cutout-window-anchor',
    name: 'Florida / Gulf Cutout Window Center Benchmark',
    show: true,
    positionDegrees: [-80.0, 27.5, 15000.0],
    shape: {
      type: 'cylinder',
      length: 30000.0,
      topRadius: 10000.0,
      bottomRadius: 10000.0,
    },
    color: [0.85, 0.35, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: { defaultCutout: [-90, 20, -70, 40], baseProvider: 'NaturalEarthII' },
  };

  const logoInset = {
    id: 'cutout-logo-inset-box',
    name: 'Cesium Logo Single Tile Inset Massing Box',
    show: true,
    positionDegrees: [-81.0, 28.5, 12000.0],
    shape: {
      type: 'box',
      dimensions: [25000.0, 25000.0, 20000.0],
    },
    color: [0.25, 0.7, 0.9, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { overlayAsset: '../images/Cesium_Logo_overlay.png', fitWithinCutout: true },
  };

  const nightTravelingDome = {
    id: 'cutout-night-traveling-dome',
    name: 'Earth at Night Dynamic Traveling Cutout Point',
    show: true,
    positionDegrees: [-79.0, 26.5, 14000.0],
    shape: {
      type: 'ellipsoid',
      radii: [14000.0, 14000.0, 14000.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.95, 0.85, 0.25, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: { ionAssetId: 3812, moveIncrement: 0.05, alpha: 0.9 },
  };

  const cutoutBoundary = {
    id: 'cutout-window-boundary',
    name: 'Florida Geographic Cutout Window Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-82.0, 25.0, -78.0, 30.0],
      height: 2000.0,
      extrudedHeight: 8000.0,
      rotation: 0.0,
    },
    color: [0.3, 0.75, 0.45, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.95, 0.7, 0.9],
    outlineWidth: 1.0,
    properties: { cutoutActive: true, keyboardTravel: ['moveNorth', 'moveSouth', 'moveEast', 'moveWest'] },
  };

  const entities = [cutoutAnchor, logoInset, nightTravelingDome, cutoutBoundary];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Florida / Gulf Cutout with Logo Overlay & Night Layer',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Cutout Shifted North (North Atlantic Window Focus)',
        entities: entities.map((e) => {
          if (e.id === 'cutout-window-anchor') {
            return { ...e, color: [0.95, 0.55, 0.3, 0.95] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Cutout Shifted West (Continental Interior Window Focus)',
        entities: entities.map((e) => {
          if (e.id === 'cutout-logo-inset-box') {
            return { ...e, color: [0.4, 0.85, 1.0, 0.95] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Cutout Expansion (Wide Tropical Caribbean Basin)',
        entities: entities.map((e) => {
          if (e.id === 'cutout-window-boundary') {
            return { ...e, color: [0.5, 0.9, 0.6, 0.85] };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(outDir, 'imagery-cutout.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(outDir, 'imagery-cutout.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated imagery-cutout.json and .variants.json');
}

// 3. Imagery Layers
function generateImageryLayers() {
  const compositeDatum = {
    id: 'layers-composite-datum',
    name: 'Multi-layer Stack Origin Composite Benchmark',
    show: true,
    positionDegrees: [-71.0, 29.0, 16000.0],
    shape: {
      type: 'cylinder',
      length: 32000.0,
      topRadius: 12000.0,
      bottomRadius: 12000.0,
    },
    color: [0.2, 0.5, 0.9, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.8, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { baseIonAssetId: 3830183, stackCount: 3 },
  };

  const blackMarbleBox = {
    id: 'layers-black-marble-box',
    name: 'Black Marble Night Luminescence Layer Massing Box',
    show: true,
    positionDegrees: [-72.5, 28.0, 12000.0],
    shape: {
      type: 'box',
      dimensions: [25000.0, 25000.0, 20000.0],
    },
    color: [0.95, 0.8, 0.3, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: { ionAssetId: 3812, alpha: 0.5, brightness: 2.0 },
  };

  const logoOverlaySphere = {
    id: 'layers-logo-overlay-sphere',
    name: 'Single Tile Logo Overlay Spatial Anchor Sphere',
    show: true,
    positionDegrees: [-69.5, 29.5, 14000.0],
    shape: {
      type: 'ellipsoid',
      radii: [14000.0, 14000.0, 14000.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.3, 0.85, 0.6, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: { singleTileRectangle: [-75.0, 28.0, -67.0, 29.75] },
  };

  const atlanticFootprint = {
    id: 'layers-atlantic-footprint',
    name: 'Western Atlantic Layer Stacking Geographic Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.0, 27.0, -70.0, 31.0],
      height: 2000.0,
      extrudedHeight: 8000.0,
      rotation: 0.0,
    },
    color: [0.8, 0.45, 0.25, 0.7],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.5, 0.9],
    outlineWidth: 1.0,
    properties: { baseLayerPicker: false, compositionMode: 'Multi-layer Alpha Blend' },
  };

  const entities = [compositeDatum, blackMarbleBox, logoOverlaySphere, atlanticFootprint];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Tri-Layer Composition (Base + Black Marble Alpha 0.5 + Logo Overlay)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Black Marble Opaque Focus (Alpha 1.0, Brightness 2.5)',
        entities: entities.map((e) => {
          if (e.id === 'layers-black-marble-box') {
            return { ...e, color: [1.0, 0.9, 0.4, 0.95] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Day Base Layer Emphasized (Black Marble Alpha 0.2)',
        entities: entities.map((e) => {
          if (e.id === 'layers-composite-datum') {
            return { ...e, color: [0.35, 0.65, 1.0, 0.95] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Logo Overlay Centered Calibration',
        entities: entities.map((e) => {
          if (e.id === 'layers-atlantic-footprint') {
            return { ...e, color: [0.95, 0.55, 0.35, 0.8] };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(outDir, 'imagery-layers.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(outDir, 'imagery-layers.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated imagery-layers.json and .variants.json');
}

// 4. Imagery Layers Manipulation
function generateImageryLayersManipulation() {
  const stackPivot = {
    id: 'manipulation-stack-pivot',
    name: 'Layer Stack Manipulation Interactive Pivot Cylinder',
    show: true,
    positionDegrees: [-93.5, 41.6, 18000.0],
    shape: {
      type: 'cylinder',
      length: 36000.0,
      topRadius: 12000.0,
      bottomRadius: 12000.0,
    },
    color: [0.85, 0.25, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.5, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: { operations: ['raise', 'lower', 'raiseToTop', 'lowerToBottom'], selectedLayer: 'Natural Earth II' },
  };

  const radarTowerBox = {
    id: 'manipulation-radar-tower-box',
    name: 'NEXRAD Weather Radar Sensor Station Box',
    show: true,
    positionDegrees: [-92.0, 42.5, 14000.0],
    shape: {
      type: 'box',
      dimensions: [25000.0, 25000.0, 22000.0],
    },
    color: [0.25, 0.8, 0.35, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.5, 1.0, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: { wmsLayer: 'nexrad-n0r', defaultAlpha: 0.5, format: 'image/png' },
  };

  const goesInfraredSphere = {
    id: 'manipulation-goes-infrared-sphere',
    name: 'GOES Geostationary Infrared Meteorological Sphere',
    show: true,
    positionDegrees: [-94.5, 42.0, 16000.0],
    shape: {
      type: 'ellipsoid',
      radii: [15000.0, 15000.0, 15000.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.65, 0.3, 0.85, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: { wmsLayer: 'goes_conus_ir', knockoutTracked: ['alpha', 'show', 'name'] },
  };

  const sensorCoverageZone = {
    id: 'manipulation-sensor-coverage-zone',
    name: 'Midwest Meteorological Sensor Network Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-95.0, 40.0, -90.0, 44.0],
      height: 2000.0,
      extrudedHeight: 8000.0,
      rotation: 0.0,
    },
    color: [0.3, 0.55, 0.9, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.8, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: { additionalLayersCount: 6, baseLayersCount: 4 },
  };

  const entities = [stackPivot, radarTowerBox, goesInfraredSphere, sensorCoverageZone];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline Stack: Natural Earth II Base + Weather Radar Alpha 0.5',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Weather Radar Raised to Top (Alpha 0.9 Topmost Stacking)',
        entities: entities.map((e) => {
          if (e.id === 'manipulation-radar-tower-box') {
            return { ...e, color: [0.4, 0.95, 0.5, 0.95] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'GOES Infrared Sensor Emphasized (Alpha 0.8 Middle Stacking)',
        entities: entities.map((e) => {
          if (e.id === 'manipulation-goes-infrared-sphere') {
            return { ...e, color: [0.8, 0.45, 1.0, 0.95] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Weather Radar Inverted Lower Stack (Alpha 0.3 Bottommost Stacking)',
        entities: entities.map((e) => {
          if (e.id === 'manipulation-sensor-coverage-zone') {
            return { ...e, color: [0.45, 0.7, 1.0, 0.8] };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(outDir, 'imagery-layers-manipulation.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(outDir, 'imagery-layers-manipulation.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated imagery-layers-manipulation.json and .variants.json');
}

generateImageryColorToAlpha();
generateImageryCutout();
generateImageryLayers();
generateImageryLayersManipulation();
