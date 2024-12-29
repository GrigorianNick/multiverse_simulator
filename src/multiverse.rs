use std::{collections::HashMap, ops::Mul, thread::panicking};

use serde::{de::{self, DeserializeOwned}, Deserialize, Serialize};

use crate::{handle::{self, Handle}, simulation::{Body, Pos, Universe}, store::{Store, StoreSQL}, timeline::{BodyBranchParams, Timeline}};

pub struct Multiverse
{
    pub root_node: Option<Handle>,
    pub nodes: HashMap<Handle, MultiverseNode>,
    pub node_store: Box<dyn Store<MultiverseNode>>,
    pub universe_store: Box<dyn Store<Universe>>,
}

impl Multiverse {

    pub fn new() -> Multiverse {
        let ns = Box::new(StoreSQL::new(String::from("./multiverse_nodes.sqlite")));
        let us = Box::new(StoreSQL::new(String::from("./universe_store.sqlite")));
        let mut m = Multiverse{
            root_node: None,
            nodes: HashMap::new(),
            node_store: ns,
            universe_store: us,
        };
        println!("Loading nodes from storage");
        for h in m.node_store.get_handles() {
            match m.node_store.get(&h) {
                Some(node) => {
                    if node.parent.is_none() {
                        m.root_node = Some(h);
                    }
                    m.nodes.insert(h, node);
                },
                None => (),
            }
        }
        println!("Root node: {:?}", &m.root_node);
        if m.root_node.is_none() {
            let new_node = MultiverseNode::new(None, 0);
            let new_handle = m.node_store.save(new_node);
            m.root_node = Some(new_handle);
            println!("New root node: {:?}", &m.root_node);
        }
        m
    }

    // Fetch a timeline spanning from the root to some arbitrary node
    pub fn get_timeline(&self, handle: &Handle) -> Timeline
    {
        match self.nodes.get(handle) {
            Some(node) => {
                let mut lineage = node.get_lineage(self);
                lineage.push(*handle);
                todo!()
            },
            None => todo!(),
        }
    }

    pub fn update_multiverse(&mut self, handle: Handle, edits: &mut Vec<BranchParams>) {
        match self.get_node_mut(&handle) {
            Some(node) => {
                match &mut node.delta {
                    Some(deltas) => deltas.append(edits),
                    None => node.delta = Some(edits.to_owned()),
                }
            }
            None => ()
        }
    }

    // handle is the Node handle
    pub fn get_universe(&self, handle: &Handle) -> Option<Universe> {
        Some(self.get_node(handle)?.get_universe(&*self.universe_store, &self))
    }

    pub fn get_nodes(&self) -> Vec<Handle> {
        self.nodes.keys().cloned().collect()
    }

    pub fn get_node(&self, handle: &Handle) -> Option<MultiverseNode> {
        self.node_store.get(handle)
    }

    pub fn get_node_mut(&mut self, handle: &Handle) -> Option<&mut MultiverseNode> {
        self.nodes.get_mut(handle)
    }

    pub fn advance(&mut self, handle: &Handle, duration: i32) -> MultiverseNode {
        let new_node = MultiverseNode::new(Some(*handle), duration);
        new_node
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct BranchParams
{
    target_body: uuid::Uuid,
    // d_ prefix stands for "delta"
    position: Option<Pos>,
    d_position: Option<Pos>,
    velocity: Option<Pos>,
    d_velocity: Option<Pos>,
    mass: Option<f64>,
    d_mas: Option<f64>,
}

impl BranchParams {
    // First set absolute params, then deltas
    pub fn apply_body(&self, target: &mut Body)
    {
        target.position = self.position.unwrap_or(target.position);
        target.velocity = self.velocity.unwrap_or(target.velocity);
        target.mass = self.mass.unwrap_or(target.mass);

        target.position += self.d_position.unwrap_or_default();
        target.velocity += self.d_velocity.unwrap_or_default();
        target.mass += self.d_mas.unwrap_or_default();
    }

    pub fn new_body(&self) -> Body {
        let mut b = Body::new();
        b.position = self.position.unwrap_or_default() + self.d_position.unwrap_or_default();
        b.velocity = self.velocity.unwrap_or_default() + self.d_velocity.unwrap_or_default();
        b.mass = self.mass.unwrap_or_default() + self.d_mas.unwrap_or_default();
        b
    }

    pub fn apply_universe(&self, target: &mut Universe) {
        match target.get_body_mut(self.target_body) {
            Some(body) => self.apply_body(body),
            None => target.add_body(self.new_body()),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MultiverseNode
{
    // The parent we base ourselves off of
    pub parent: Option<Handle>,
    // Did we mutate any data compared to our parent?
    pub delta: Option<Vec<BranchParams>>,
    // The next thing to happen cannonically
    pub next: Option<Handle>,
    // Possible future universes
    pub children: Vec<Handle>,
    // The universe at this moment in time
    pub universe: Handle,
    // How many ticks do we advance relative to our parent?
    pub relative_age: i32
}

impl MultiverseNode {
    pub fn new(parent: Option<Handle>, age: i32) -> MultiverseNode {
        MultiverseNode{
            parent: parent,
            delta: None,
            next: None,
            children: vec![],
            universe: Handle::new(),
            relative_age: age
        }
    }

    pub fn get_lineage(&self, multiverse: &Multiverse) -> Vec<Handle> {
        let mut ancestors = vec![];
        let mut me = self;
        loop {
            match (me.parent) {
                None => return ancestors,
                Some(handle) => ancestors.push(handle),
            }
            match (multiverse.nodes.get(&me.parent.unwrap())) {
                // Really, we should panic because this is an incorrect state
                None => return  ancestors,
                Some(node) => me = node,
            }
        }
    }

    pub fn get_parent(&self, multiverse: &Multiverse) -> Option<MultiverseNode> {
        if self.parent.is_none() {
            return None
        }
        match multiverse.nodes.get(&self.parent.unwrap()) {
            Some(node) => Some(node.clone()),
            _ => None
        }
    }

    pub fn calculate_universe(&self, store: &dyn Store<Universe>, multiverse: &Multiverse) -> Universe {
        let mut new_universe = match self.get_parent(multiverse) {
            None => Universe::new(),
            Some(parent) => {
                parent.get_universe(store, multiverse)
            }
        };
        match &self.delta {
            Some(params) => {
                for param in params {
                    param.apply_universe(&mut new_universe);
                }
            }
            _ => (),
        }
        new_universe.tick_for(self.relative_age);
        store.save_handle(&new_universe, self.universe);
        // TODO: replace this super expensive recalc with a dirty flag for lazy eval
        for child_handle in &self.children {
            match multiverse.nodes.get(&child_handle) {
                Some(child) => {
                    child.calculate_universe(store, multiverse);
                },
                None => (),
            };
        }
        new_universe
    }

    pub fn get_universe(&self, store: &dyn Store<Universe>, multiverse: &Multiverse) -> Universe {
        match store.get(&self.universe) {
            Some(u) => return u,
            None => self.calculate_universe(store, multiverse)
        }
    }
}