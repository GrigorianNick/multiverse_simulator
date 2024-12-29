use std::sync::mpsc::{Receiver, Sender};

use crate::{handle::Handle, multiverse::Multiverse, simulation::Universe};

pub enum MultiverseCommand {
    Echo(Sender<String>),
    // Node handle
    AdvanceNode((Handle, u32)),
    // Universe handle
    GetUniverse((Handle, Sender<Option<Universe>>)),
    GetNodeChildren((Handle, Sender<Vec<Handle>>)),
    GetNodes(Sender<Vec<Handle>>),
}

pub fn start_multiverse(rx: Receiver<MultiverseCommand>) {
    let multiverse = Multiverse::new();
    loop {
        let _ = match rx.recv() {
            Ok(cmd) => 
            {
                match cmd {
                    MultiverseCommand::Echo(sender) => {sender.send("echo".to_string()).expect("Failed to send echo");},
                    MultiverseCommand::AdvanceNode(_) => todo!(),
                    MultiverseCommand::GetUniverse((handle, tx)) => {tx.send(multiverse.get_universe(&handle));},
                    MultiverseCommand::GetNodeChildren(_) => todo!(),
                    MultiverseCommand::GetNodes(sender) => {sender.send(multiverse.get_nodes()).expect("Failed to send nodes");},
                };
            },
            Err(_) => break,
        };
    }
}