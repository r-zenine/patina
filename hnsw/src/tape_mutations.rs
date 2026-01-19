use std::marker::PhantomData;
use thiserror::Error;

use crate::{NodeId, tape::GraphTape};

#[derive(Error, Debug)]
pub enum MutationError {
    #[error("Neighbor {neighbor:?} not found in level {level} of node {node_id:?}")]
    NeighborNotFound {
        node_id: NodeId,
        level: u8,
        neighbor: NodeId,
    },
    #[error("No empty slots available in level {level} of node {node_id:?}")]
    NoEmptySlots { node_id: NodeId, level: u8 },
}

pub enum TapeMutations {
    Unordered(Vec<TapeMutation>),
}

#[derive(Debug)]
pub enum TapeMutation {
    Swap {
        node: NodeId,
        level: u8,
        old: NodeId,
        new: NodeId,
    },
    Add {
        node: NodeId,
        level: u8,
        neighbor: NodeId,
    },
    Drop {
        node: NodeId,
        level: u8,
        neighbor: NodeId,
    },
}

pub trait TapeMutator<K: Sized + Copy, const CHUNK_SIZE: usize> {
    fn apply_mutations(
        tape: &mut GraphTape<K, CHUNK_SIZE>,
        mutations: TapeMutations,
    ) -> Result<(), MutationError>;
}

pub struct SequentialMutator<K: Sized + Copy, const CHUNK_SIZE: usize> {
    _phatom: PhantomData<K>,
}

impl<K: Sized + Copy, const CHUNK_SIZE: usize> SequentialMutator<K, CHUNK_SIZE> {
    fn apply(
        tape: &mut GraphTape<K, CHUNK_SIZE>,
        mutation: TapeMutation,
    ) -> Result<(), MutationError> {
        match mutation {
            TapeMutation::Swap {
                node: node_id,
                level,
                old: old_neighbor,
                new: new_neighbor,
            } => {
                tape.swap_neighbor(node_id, level, old_neighbor, new_neighbor);
                Ok(())
            }
            TapeMutation::Add {
                node: node_id,
                level,
                neighbor,
            } => {
                if tape.swap_neighbor(node_id, level, crate::EMPTY_NEIGHBOR, neighbor) {
                    Ok(())
                } else {
                    // Check what went wrong for better error reporting
                    Self::diagnose_add_neighbor_failure(tape, node_id, level)
                }
            }
            TapeMutation::Drop {
                node: node_id,
                level,
                neighbor,
            } => {
                if tape.swap_neighbor(node_id, level, neighbor, crate::EMPTY_NEIGHBOR) {
                    Ok(())
                } else {
                    // Check what went wrong for better error reporting
                    Self::diagnose_drop_neighbor_failure(tape, node_id, level, neighbor)
                }
            }
        }
    }

    fn diagnose_add_neighbor_failure(
        _tape: &GraphTape<K, CHUNK_SIZE>,
        node_id: NodeId,
        level: u8,
    ) -> Result<(), MutationError> {
        // Simple diagnostic: assume it's either node not found, level not found, or no empty slots
        // For now, just return a generic error - could be enhanced with more detailed checking
        Err(MutationError::NoEmptySlots { node_id, level })
    }

    fn diagnose_drop_neighbor_failure(
        _tape: &GraphTape<K, CHUNK_SIZE>,
        node_id: NodeId,
        level: u8,
        neighbor: NodeId,
    ) -> Result<(), MutationError> {
        // Simple diagnostic: assume it's neighbor not found
        // For now, just return a generic error - could be enhanced with more detailed checking
        Err(MutationError::NeighborNotFound {
            node_id,
            level,
            neighbor,
        })
    }

    fn flatten(mutations: TapeMutations) -> Vec<TapeMutation> {
        match mutations {
            TapeMutations::Unordered(mutations) => mutations,
        }
    }
}

impl<K: Sized + Copy, const CHUNK_SIZE: usize> TapeMutator<K, CHUNK_SIZE>
    for SequentialMutator<K, CHUNK_SIZE>
{
    fn apply_mutations(
        tape: &mut GraphTape<K, CHUNK_SIZE>,
        mutations: TapeMutations,
    ) -> Result<(), MutationError> {
        for mutation in SequentialMutator::<K, CHUNK_SIZE>::flatten(mutations) {
            SequentialMutator::<K, CHUNK_SIZE>::apply(tape, mutation)?;
        }
        Ok(())
    }
}
