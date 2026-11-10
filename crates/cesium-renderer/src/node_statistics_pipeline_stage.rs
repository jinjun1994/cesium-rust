//! NodeStatisticsPipelineStage matching CesiumJS `Scene/Model/NodeStatisticsPipelineStage.js` (Apache-2.0).

use crate::model_components::{Instances, Node};
use crate::model_runtime_node::ModelRuntimeNode;
use crate::model_statistics::ModelStatistics;

/// The node statistics update stage updates memory usage statistics for a Model on the node level.
pub struct NodeStatisticsPipelineStage;

impl NodeStatisticsPipelineStage {
    pub const NAME: &'static str = "NodeStatisticsPipelineStage";

    /// Counts instancing attributes from an instances component.
    pub fn count_instancing_attributes(
        statistics: &mut ModelStatistics,
        instances: Option<&Instances>,
    ) {
        if let Some(instances) = instances {
            for attribute in &instances.attributes {
                if let Some(ref buffer) = attribute.buffer {
                    statistics.add_buffer(buffer, false);
                }
            }
        }
    }

    /// Counts runtime generated instancing buffers on a node.
    pub fn count_generated_buffers(
        statistics: &mut ModelStatistics,
        runtime_node: &ModelRuntimeNode,
    ) {
        if let Some(ref buffer) = runtime_node.instancing_transforms_buffer {
            statistics.add_buffer(buffer, false);
        }
        if let Some(ref buffer) = runtime_node.instancing_transforms_buffer_2d {
            statistics.add_buffer(buffer, false);
        }
        if let Some(ref buffer) = runtime_node.instancing_translation_buffer_2d {
            statistics.add_buffer(buffer, false);
        }
    }

    /// Processes a node, counting its instanced attributes and generated buffers.
    pub fn process(
        statistics: &mut ModelStatistics,
        node: &Node,
        runtime_node: &ModelRuntimeNode,
    ) {
        Self::count_instancing_attributes(statistics, node.instances.as_ref());
        Self::count_generated_buffers(statistics, runtime_node);
    }
}
