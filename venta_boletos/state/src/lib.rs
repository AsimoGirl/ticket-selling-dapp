#![no_std]

//Este crate tiene como funcion mostrar los valores del estado del smart contract

use venta_boletos_io::*;
use gear_lib::multitoken::io::TokenMetadata;
use gmeta::{metawasm, Metadata};
use gstd::{prelude::*, ActorId};

//Con este modulo le decimos al programa que debe mostrar el estado del contrato de forma decodificada
#[metawasm]
pub mod metafns {
    pub type State = <ContractMetadata as Metadata>::State;

    pub fn current_concert(state: State) -> CurrentConcert {
        state.current_concert()
    }

    pub fn buyers(state: State) -> Vec<ActorId> {
        state.buyers
    }

    pub fn user_tickets(state: State, user: ActorId) -> Vec<Option<TokenMetadata>> {
        state.user_tickets(user)
    }
}
