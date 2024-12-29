use serde::{Deserialize, Serialize};

use std::borrow::Borrow;
use std::collections::HashMap;

use crate::handle::Handle;
use crate::simulation::Universe;
use crate::simulation::Pos;
use crate::simulation::Body;
use crate::store::Store;
use crate::store::{self};

pub struct Timeline
{
    // Earth prime
    root: Handle,
    nodes: Vec<Handle>,
    node_store: Box<dyn Store<TimelineNode>>,
    body_store: Box<dyn Store<Body>>,
}

impl Timeline {
    pub fn new(bodies: Vec<Handle>, body_store: Box<dyn Store<Body>>, node_store: Box<dyn Store<TimelineNode>>) -> Timeline
    {
        let mut universe = Universe::new();
        for body in bodies {
            match body_store.get(&body) {
                Some(b) => universe.add_body(b),
                None => ()
            }
        }
        let root_node = TimelineNode::default();
        let root_handle = node_store.save(root_node);
        Timeline {
            root: root_handle,
            nodes: vec![root_handle],
            node_store: node_store,
            body_store: body_store,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct TimelineNode
{
    // Our id, used in Store
    pub id: uuid::Uuid,
    // How many ticks did it take to get here?
    pub duration: i32,
    // What's the id of the node that preceeds us?
    pub parent: Option<Handle>,
    // What's the id of our next child?
    pub next: Option<Handle>,
    // Branch nodes, where the user nudged something
    pub branches: Vec<uuid::Uuid>,
    // State of our universe
    pub universe_state: Universe,
}

#[derive(Default, Serialize, Deserialize)]
pub struct BodyBranchParams
{
    // d_ prefix stands for "delta"
    position: Option<Pos>,
    d_position: Option<Pos>,
    velocity: Option<Pos>,
    d_velocity: Option<Pos>,
    mass: Option<f64>,
    d_mas: Option<Pos>,
}

impl BodyBranchParams {
    // First set absolute params, then deltas
    pub fn apply(&self, target: &mut Body)
    {
        target.position = self.position.unwrap_or(target.position);
        target.velocity = self.velocity.unwrap_or(target.velocity);
        target.mass = self.mass.unwrap_or(target.mass);

        target.position += self.d_position.unwrap_or_default();
        target.velocity += self.d_velocity.unwrap_or_default();
    }
}

pub struct UniverseBranchParams
{
    pub delta_params: HashMap<Handle, BodyBranchParams>,
    pub new_bodies: Vec<Handle>,
    pub removed_bodies: Vec<Handle>,
}

impl UniverseBranchParams {
    pub fn apply(&self, target: &Universe) -> Universe
    {
        let mut universe = target.clone();
        /*for body in &mut universe.bodies {
            match self.delta_params.get(&body.id) {
                Some(param) => param.apply(body),
                None => (),
            };
        }
        universe.remove_bodies(&self.removed_bodies);
        for body in &self.new_bodies {
            universe.add_body(*body);
        }*/
        universe
    }
}

/*impl TimelineNode { 
    pub fn new(store: Box<dyn Store<Universe>>) -> TimelineNode {
        let mut t = TimelineNode::default();
        t.id = uuid::Uuid::new_v4();
        t.universe_state = Universe::new();
        t
    }

    // New child based off of some source parent. Parent remains unmodified
    pub fn new_child(source: &TimelineNode) -> TimelineNode {
        let mut t = TimelineNode::new();
        t.parent = Some(source.id);
        t.universe_state = source.universe_state.clone();
        t
    }

    pub fn branch(&mut self, parameters: UniverseBranchParams, store: impl UniverseStore) -> TimelineNode
    {
        let mut child = TimelineNode::new_child(self);
        parameters.apply(&child.universe_state);
        self.branches.push(child.id);
        child
    }
}*/