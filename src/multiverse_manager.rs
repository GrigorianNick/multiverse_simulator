use std::sync::mpsc::{Receiver, Sender};

use crate::{handle::Handle, multiverse::{BranchParams, Multiverse, MultiverseNode}, simulation::Universe, timeline::Timeline};

pub enum MultiverseCommand {
    // Node handle
    AdvanceNode((Handle, i32)),
    // Universe handle
    GetUniverse((Handle, Sender<Option<Universe>>)),
    GetNodes(Sender<Vec<Handle>>),
    GetTimneline((Handle, Sender<Timeline>)),
    GetNode((Handle, Sender<Option<MultiverseNode>>)),
    Branch((Handle, Vec<BranchParams>, i32))
}

pub fn start_multiverse(rx: Receiver<MultiverseCommand>) {
    let mut multiverse = Multiverse::new();
    loop {
        let _ = match rx.recv() {
            Ok(cmd) => 
            {
                match cmd {
                    MultiverseCommand::AdvanceNode((handle, duration)) => {multiverse.advance(&handle, duration);},
                    MultiverseCommand::GetUniverse((handle, tx)) => {tx.send(multiverse.get_universe(&handle));},
                    MultiverseCommand::GetNodes(sender) => {sender.send(multiverse.get_nodes()).expect("Failed to send nodes");},
                    MultiverseCommand::GetTimneline((handle, tx)) => {tx.send(multiverse.get_timeline(&handle));},
                    MultiverseCommand::GetNode((handle, tx)) => {tx.send(multiverse.get_node(&handle));}
                    MultiverseCommand::Branch((handle, params, duration)) => {multiverse.branch(&handle, duration, params);}
                };
            },
            Err(_) => break,
        };
    }
}