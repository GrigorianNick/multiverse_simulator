use serde::{Deserialize, Serialize};

use crate::multiverse::Multiverse;
use crate::multiverse::MultiverseNode;
use crate::simulation::Universe;

#[derive(Serialize, Deserialize)]
pub struct Timeline
{
    pub universes: Vec<Universe>,
}

impl Timeline {
    pub fn new(node: &MultiverseNode, multiverse: &Multiverse) -> Timeline {
        let nodes = node.get_lineage(multiverse);
        Timeline {
            universes: nodes.iter().filter_map(|h| {
                match multiverse.get_node(h) {
                    Some(n) => Some(n.get_universe(&multiverse)),
                    None => None
                }
            }).collect()
        }
    }
}