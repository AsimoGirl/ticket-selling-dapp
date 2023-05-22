//Este crate tiene como funcion determinar como toda la data de entrada y salida será
//codificada y decodificada, permite que el smart contract y el cliente intercambien informacion

//No usamos la biblioteca estandar de Rust en gear por las bibliotecas
#![no_std]

use gear_lib::multitoken::io::*;
use gmeta::{In, InOut, Metadata};
use gstd::{prelude::*, ActorId};

pub struct ContractMetadata;

//Declaramos los tipos de entrada y salidad para las funciones principales del smart contract
impl Metadata for ContractMetadata {
    type Init = In<InitConcert>;
    type Handle = InOut<ConcertAction, ConcertEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = State;
}

//Aqui definimos la estructura del estado del smart contract, es decir lo que guardara.
#[derive(Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct State {
    //La direccion del dueño del smart contract
    pub owner_id: ActorId,
    //La direccion del smart contract donde se realizan las operaciones con los tokens
    //este es un contrato GMT-1155 que se puede encontrar en github.com/gear-dapps/multitoken
    pub contract_id: ActorId,
    //Nombre del evento
    pub name: String,
    //Descripcion del evento
    pub description: String,
    //Es el id del token asociado a los boletos
    pub ticket_ft_id: u128,
    //La direccion del creador del evento
    pub creator: ActorId,
    //El numero de boletos posibles de vender
    pub number_of_tickets: u128,
    //El numero de boletos que sobran
    pub tickets_left: u128,
    //La fecha del evento
    pub date: u128,
    //Las direcciones de los compradores
    pub buyers: Vec<ActorId>,
    //La conatidad de boletos vendidos
    pub id_counter: u128,
    //El id del concierto
    pub concert_id: u128,
    //Dice si todavia se pueden vender boletos
    pub running: bool,
    /// El vector con la metadata que le corresponde a cada comprador
    pub metadata: Vec<(ActorId, Tickets)>,
}

//Aqui se guardaran los boletos
pub type Tickets = Vec<(u128, Option<TokenMetadata>)>;


#[doc(hidden)]
impl State {
    //Actualizamos el estado actual del evento
    pub fn current_concert(self) -> CurrentConcert {
        CurrentConcert {
            name: self.name,
            description: self.description,
            date: self.date,
            number_of_tickets: self.number_of_tickets,
            tickets_left: self.tickets_left,
        }
    }

    //Se hace la asignación de la metadata de los boletos a cada comprador
    pub fn user_tickets(self, user: ActorId) -> Vec<Option<TokenMetadata>> {
        self.metadata
            .into_iter()
            .find_map(|(some_user, tickets)| {
                (some_user == user)
                    .then_some(tickets.into_iter().map(|(_, tickets)| tickets).collect())
            })
            .unwrap_or_default()
    }
}

//La estructura representa el estado actual del evento
#[derive(Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, TypeInfo)]
pub struct CurrentConcert {
    pub name: String,
    pub description: String,
    pub date: u128,
    pub number_of_tickets: u128,
    pub tickets_left: u128,
}

// Definimos las acciones posibles en el smart contract
#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum ConcertAction {
    //Crear un evento
    Create {
        creator: ActorId,
        name: String,
        description: String,
        number_of_tickets: u128,
        date: u128,
    },
    //Convertir los tokens a NFTs
    Hold,
    //Realiza la compra de boletos
    BuyTickets {
        amount: u128,
        metadata: Vec<Option<TokenMetadata>>,
    },
}

//Representa los eventos del smart contract
#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum ConcertEvent {
    //Guarda la informacion de la accion create 
    Creation {
        creator: ActorId,
        concert_id: u128,
        number_of_tickets: u128,
        date: u128,
    },
    //Guarda la informacion de la accion hold
    Hold {
        concert_id: u128,
    },
    //Guarda la información de la accion buytickets
    Purchase {
        concert_id: u128,
        amount: u128,
    },
}

//Son las queries que pueden entrar dentro del estado del smart contract
#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum ConcertStateQuery {
    CurrentConcert,
    Buyers,
    UserTickets { user: ActorId },
}

//Son las respuestas a las posibles queries del estado del smart contract
#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum ConcertStateReply {
    CurrentConcert(CurrentConcert),
    Buyers(Vec<ActorId>),
    UserTickets(Vec<Option<TokenMetadata>>),
}

//Es la estructura que inicializa el smart contract
#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitConcert {
    pub owner_id: ActorId,
    pub mtk_contract: ActorId,
}
