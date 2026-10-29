import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Bathymetry
function generateBathymetry() {
  const marianaTrench = {
    id: 'bathymetry-mariana-trench',
    name: 'Mariana Trench Challenger Deep Bathymetric Benchmark',
    show: true,
    positionDegrees: [142.2, 11.35, 15000.0],
    shape: {
      type: 'cylinder',
      length: 30000.0,
      topRadius: 10000.0,
      bottomRadius: 10000.0,
    },
    color: [0.16, 0.11, 0.18, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.3, 0.2, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: { maxDepthMeters: -10984.0, oceanFeature: 'Abyssal Trench', cmoceanBand: '#291C2F' },
  };

  const midAtlanticRidge = {
    id: 'bathymetry-mid-atlantic-ridge',
    name: 'Mid-Atlantic Spreading Ridge Geomorphology Box',
    show: true,
    positionDegrees: [-28.5, 25.0, 10000.0],
    shape: {
      type: 'box',
      dimensions: [25000.0, 25000.0, 15000.0],
    },
    color: [0.26, 0.49, 0.6, 0.85],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.7, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: { tectonicBoundary: 'Divergent Plate', depthMeters: -2500.0, cmoceanBand: '#437D9A' },
  };

  const hawaiianSeamounts = {
    id: 'bathymetry-hawaiian-seamounts',
    name: 'Hawaiian-Emperor Seamount Chain Volcanic Peak',
    show: true,
    positionDegrees: [-165.0, 28.0, 12000.0],
    shape: {
      type: 'ellipsoid',
      radii: [15000.0, 15000.0, 15000.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.35, 0.72, 0.64, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: { volcanicOrigin: 'Mantle Plume Hotspot', summitDepthMeters: -1500.0, cmoceanBand: '#5AB7A4' },
  };

  const puertoRicoTrench = {
    id: 'bathymetry-puerto-rico-trench',
    name: 'Puerto Rico Trench Deep Atlantic Basin Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-67.5, 18.5, -65.5, 20.5],
      height: 2000.0,
      extrudedHeight: 8000.0,
      rotation: 0.0,
    },
    color: [0.24, 0.26, 0.5, 0.75],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.5, 0.8, 0.9],
    outlineWidth: 1.0,
    properties: { trenchClassification: 'Subduction Zone', contourSpacingMeters: 500.0 },
  };

  const entities = [marianaTrench, midAtlanticRidge, hawaiianSeamounts, puertoRicoTrench];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Composite Elevation Color Contour (CMOCEAN Deep Bathymetry & Dynamic Hillshade)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Elevation Ramp Only (CMOCEAN Deep Seafloor Topography Gradient)',
        entities: entities.map((e) => {
          if (e.id === 'bathymetry-mariana-trench') {
            return { ...e, color: [0.12, 0.08, 0.16, 0.95] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Contour Lines Only (500m Iso-depth Seafloor Contours)',
        entities: entities.map((e) => {
          if (e.id === 'bathymetry-mid-atlantic-ridge') {
            return { ...e, color: [0.8, 0.9, 1.0, 0.9] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Lighting & Hillshade Suppressed (Flat Bathymetry Base Reference)',
        entities: entities.map((e) => {
          if (e.id === 'bathymetry-puerto-rico-trench') {
            return { ...e, color: [0.3, 0.4, 0.6, 0.85] };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(outDir, 'bathymetry.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(outDir, 'bathymetry.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated bathymetry.json and .variants.json');
}

// 2. Bing Maps Labels Only
function generateBingMapsLabelsOnly() {
  const capitolDome = {
    id: 'bing-labels-capitol-dome',
    name: 'United States Capitol Rotunda and Dome Landmark',
    show: true,
    positionDegrees: [-77.009, 38.8899, 150.0],
    shape: {
      type: 'cylinder',
      length: 300.0,
      topRadius: 60.0,
      bottomRadius: 60.0,
    },
    color: [0.9, 0.85, 0.7, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: { landmarkType: 'Civic Monument', labelLayerAssetId: 2411391, splitPosition: 0.5 },
  };

  const mallCorridor = {
    id: 'bing-labels-mall-corridor',
    name: 'National Mall High-Resolution Imagery Alignment Box',
    show: true,
    positionDegrees: [-77.0365, 38.8895, 80.0],
    shape: {
      type: 'box',
      dimensions: [800.0, 150.0, 100.0],
    },
    color: [0.3, 0.7, 0.4, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.9, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: { imageryAssetId: 3827, parkClassification: 'National Mall and Memorial' },
  };

  const potomacConfluence = {
    id: 'bing-labels-potomac-confluence',
    name: 'Potomac River Navigation Confluence Water Mask',
    show: true,
    positionDegrees: [-77.042, 38.878, 50.0],
    shape: {
      type: 'ellipsoid',
      radii: [120.0, 120.0, 60.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.2, 0.5, 0.85, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: { hydrography: 'Potomac Estuary', bathymetricDraftMeters: 7.2 },
  };

  const dcQuadrant = {
    id: 'bing-labels-dc-quadrant',
    name: 'District of Columbia Core Urban Quad Split Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-77.05, 38.88, -77.0, 38.91],
      height: 20.0,
      extrudedHeight: 250.0,
      rotation: 0.0,
    },
    color: [0.85, 0.65, 0.3, 0.7],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.8, 0.4, 0.9],
    outlineWidth: 1.0,
    properties: { splitDirectionLeft: 'Bing Maps Aerial with Labels', splitDirectionRight: 'Bing Maps Aerial + Labels Only' },
  };

  const entities = [capitolDome, mallCorridor, potomacConfluence, dcQuadrant];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Split View Balanced (Split Position 0.5: Aerial with Labels vs Aerial + Labels Only)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Labels Only Layer Suppressed (Unlabelled High-Resolution DC Imagery)',
        entities: entities.map((e) => {
          if (e.id === 'bing-labels-capitol-dome') {
            return { ...e, color: [0.75, 0.75, 0.75, 0.9] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Left Focus (Split Position 0.2: Expanded Right Panel Labels Overlay)',
        entities: entities.map((e) => {
          if (e.id === 'bing-labels-mall-corridor') {
            return { ...e, color: [0.2, 0.85, 0.5, 0.95] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Right Focus (Split Position 0.8: Expanded Left Panel Monolithic Aerial)',
        entities: entities.map((e) => {
          if (e.id === 'bing-labels-dc-quadrant') {
            return { ...e, color: [0.95, 0.55, 0.2, 0.8] };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(outDir, 'bing-maps-labels-only.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(outDir, 'bing-maps-labels-only.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated bing-maps-labels-only.json and .variants.json');
}

// 3. Azure 2D Tiles
function generateAzure2DTiles() {
  const cityHallTower = {
    id: 'azure-philly-city-hall',
    name: 'Philadelphia City Hall Tower Architectural Benchmark',
    show: true,
    positionDegrees: [-75.1636, 39.9526, 250.0],
    shape: {
      type: 'cylinder',
      length: 500.0,
      topRadius: 75.0,
      bottomRadius: 75.0,
    },
    color: [0.8, 0.75, 0.7, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.85, 1.0],
    outlineWidth: 1.0,
    properties: { ionAssetId: 3891169, heightAboveEllipsoidMeters: 167.0, urbanZone: 'Center City' },
  };

  const libertyPlace = {
    id: 'azure-liberty-place-spire',
    name: 'One Liberty Place Skyscraper Urban Massing Box',
    show: true,
    positionDegrees: [-75.1675, 39.9515, 280.0],
    shape: {
      type: 'box',
      dimensions: [120.0, 120.0, 560.0],
    },
    color: [0.3, 0.6, 0.9, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.8, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { architecturalStyle: 'Postmodern', structuralHeightMeters: 288.0 },
  };

  const independenceMall = {
    id: 'azure-independence-mall',
    name: 'Independence National Historical Park Civic Dome',
    show: true,
    positionDegrees: [-75.15, 39.9496, 60.0],
    shape: {
      type: 'ellipsoid',
      radii: [100.0, 100.0, 80.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.4, 0.8, 0.45, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: { historicalSignificance: 'Declaration of Independence', canopyCoveragePercent: 65.0 },
  };

  const metroFootprint = {
    id: 'azure-metro-footprint',
    name: 'Greater Philadelphia Azure 2D Basemap Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.28, 39.87, -74.96, 40.13],
      height: 50.0,
      extrudedHeight: 400.0,
      rotation: 0.0,
    },
    color: [0.35, 0.5, 0.75, 0.65],
    fill: true,
    outline: true,
    outlineColor: [0.55, 0.75, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: { projectionScheme: 'WebMercator', terrainAssetId: 1, geocoderProvider: 'GOOGLE' },
  };

  const entities = [cityHallTower, libertyPlace, independenceMall, metroFootprint];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Azure Maps 2D World Basemap (Philadelphia Urban Core Overview)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'High-Contrast Shaded Relief with Cesium World Terrain',
        entities: entities.map((e) => {
          if (e.id === 'azure-philly-city-hall') {
            return { ...e, color: [0.95, 0.85, 0.5, 0.95] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'City Hall & Center City Skyscraper Macro Focus',
        entities: entities.map((e) => {
          if (e.id === 'azure-liberty-place-spire') {
            return { ...e, color: [0.2, 0.75, 1.0, 0.95] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Delaware River Maritime Corridor Regional Extent',
        entities: entities.map((e) => {
          if (e.id === 'azure-metro-footprint') {
            return { ...e, color: [0.25, 0.65, 0.85, 0.75] };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(outDir, 'azure-2d-tiles.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(outDir, 'azure-2d-tiles.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated azure-2d-tiles.json and .variants.json');
}

// 4. Imagery Adjustment
function generateImageryAdjustment() {
  const calibrationTarget = {
    id: 'adjustment-calibration-target',
    name: 'Radiometric Calibration Sensor Target Cylinder',
    show: true,
    positionDegrees: [-105.27, 40.01, 1000.0],
    shape: {
      type: 'cylinder',
      length: 2000.0,
      topRadius: 500.0,
      bottomRadius: 500.0,
    },
    color: [0.9, 0.2, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.4, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: { radiometricChannel: 'Red (660nm)', brightness: 0.0, saturation: 3.0 },
  };

  const stepWedge = {
    id: 'adjustment-step-wedge',
    name: 'Optical Contrast Step Wedge Massing Box',
    show: true,
    positionDegrees: [-105.25, 40.03, 800.0],
    shape: {
      type: 'box',
      dimensions: [800.0, 800.0, 800.0],
    },
    color: [0.2, 0.8, 0.3, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.4, 1.0, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: { contrastRatio: 1.2, stepGradation: 16, gammaCorrection: 1.0 },
  };

  const colorWheel = {
    id: 'adjustment-color-wheel',
    name: 'Spectral Hue Chromatic Angle Sphere',
    show: true,
    positionDegrees: [-105.29, 40.03, 1200.0],
    shape: {
      type: 'ellipsoid',
      radii: [600.0, 600.0, 600.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.2, 0.5, 0.95, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: { hueRadians: 0.0, chromaticDispersion: 'RGB to HSV' },
  };

  const calibrationZone = {
    id: 'adjustment-calibration-zone',
    name: 'Dynamic Radiance Calibration Geographic Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-105.32, 39.98, -105.22, 40.05],
      height: 500.0,
      extrudedHeight: 1500.0,
      rotation: 0.0,
    },
    color: [0.9, 0.8, 0.2, 0.7],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.4, 0.9],
    outlineWidth: 1.0,
    properties: { knockoutBinding: true, toolbarControls: ['brightness', 'contrast', 'hue', 'saturation', 'gamma'] },
  };

  const entities = [calibrationTarget, stepWedge, colorWheel, calibrationZone];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'High Saturation Vivid (Saturation 3.0, Contrast 1.2, Gamma 1.0)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Monochromatic High Contrast Noir (Saturation 0.0, Contrast 2.0, Gamma 1.2)',
        entities: entities.map((e) => {
          if (e.id === 'adjustment-calibration-target') {
            return { ...e, color: [0.6, 0.6, 0.6, 0.95] };
          }
          if (e.id === 'adjustment-step-wedge') {
            return { ...e, color: [0.85, 0.85, 0.85, 0.9] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Warm Sunset Chromatic Tint (Hue 0.4, Saturation 1.8, Gamma 0.9)',
        entities: entities.map((e) => {
          if (e.id === 'adjustment-color-wheel') {
            return { ...e, color: [0.95, 0.55, 0.2, 0.9] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Cold Lunar Radiance Shift (Hue -0.5, Contrast 1.5, Brightness -0.2)',
        entities: entities.map((e) => {
          if (e.id === 'adjustment-calibration-zone') {
            return { ...e, color: [0.4, 0.6, 0.9, 0.75] };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(outDir, 'imagery-adjustment.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(outDir, 'imagery-adjustment.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated imagery-adjustment.json and .variants.json');
}

generateBathymetry();
generateBingMapsLabelsOnly();
generateAzure2DTiles();
generateImageryAdjustment();
