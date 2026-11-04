# Cesium legacy b3dm and pnts fixtures

`cesium-batched-deprecated-20.b3dm` and `cesium-batched-deprecated-24.b3dm` are copied unmodified from the local CesiumJS checkout at commit `77d30241863d30f299e7271a22f95b7d589c1c75`, under `Specs/Data/Cesium3DTiles/Batched/BatchedDeprecated1/` and `BatchedDeprecated2/`, respectively. They are used by CesiumJS `packages/engine/Specs/Scene/B3dmParserSpec.js` and carry the CesiumJS project's Apache-2.0 license.

`cesium-batch-table-binary.b3dm` is copied from `Specs/Data/Cesium3DTiles/Batched/BatchedWithBatchTableBinary/batchedWithBatchTableBinary.b3dm` at the same revision and is referenced by `Cesium3DTileBatchTableSpec.js`.

The `cesium-pointcloud-*.pnts` fixtures (`cesium-pointcloud-rgb.pnts`, `cesium-pointcloud-quantized.pnts`, `cesium-pointcloud-batched.pnts`, `cesium-pointcloud-draco.pnts`, `cesium-pointcloud-draco-partial.pnts`, `cesium-pointcloud-draco-batched.pnts`, and `cesium-pointcloud-draco-invalid.pnts`) are copied from `Specs/Data/Cesium3DTiles/PointCloud/` (`PointCloudRGB`, `PointCloudQuantizedOctEncoded`, `PointCloudBatched`, `PointCloudDraco`, `PointCloudDracoPartial`, `PointCloudDracoBatched`, and `PointCloudDracoInvalid`) at the same revision; they are used by CesiumJS point-cloud and `PntsParserSpec.js` tests.
