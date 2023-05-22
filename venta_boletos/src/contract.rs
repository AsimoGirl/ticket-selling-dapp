//Este es el codigo principal del smart contract

use venta_boletos_io::*;
use gear_lib::multitoken::io::*;
use gstd::{errors::Result, msg, prelude::*, ActorId, MessageId};
use hashbrown::{HashMap, HashSet};
use multitoken_io::MyMTKAction;

const ZERO_ID: ActorId = ActorId::zero();

#[derive(Default)]
//La explicacion de cada elemento de esta estructura esta en io/lib.rs en la estrcutura State
struct Concert {
    owner_id: ActorId,
    contract_id: ActorId,
    name: String,
    description: String,
    ticket_ft_id: u128,
    creator: ActorId,
    number_of_tickets: u128,
    tickets_left: u128,
    date: u128,
    buyers: HashSet<ActorId>,
    id_counter: u128,
    concert_id: u128,
    running: bool,
    //Aqui la metadata sera un hasmap donde se guardaran los boletos correspondientes a cada comprador
    metadata: HashMap<ActorId, HashMap<u128, Option<TokenMetadata>>>,
}

//Creamos una instancia global de la estructura para compartirla entre las diversas funciones
static mut CONTRACT: Option<Concert> = None;

//Inicializamos el smart contract
#[no_mangle]
unsafe extern "C" fn init() {
    let config: InitConcert = msg::load().expect("Unable to decode InitConfig");
    let concert = Concert {
        owner_id: config.owner_id,
        contract_id: config.mtk_contract,
        ..Default::default()
    };
    CONTRACT = Some(concert);
}

//Definimos la funcion principal, aqui manejamos las acciones entrantes
#[gstd::async_main]
async unsafe fn main() {
    //Obtenemos la accion con msg::load
    let action: ConcertAction = msg::load().expect("Could not load Action");
    //Obtenemos una referencia mutable a la instancia de Concert, que es nuestro actor
    let concert: &mut Concert = unsafe { CONTRACT.get_or_insert(Default::default()) };
    match action {
        ConcertAction::Create {
            creator,
            name,
            description,
            number_of_tickets,
            date,
        } => concert.create_concert(name, description, creator, number_of_tickets, date),
        ConcertAction::Hold => concert.hold_concert().await,
        ConcertAction::BuyTickets { amount, metadata } => {
            concert.buy_tickets(amount, metadata).await
        }
    }
}

//La funcion envia mensajes como respuesta al mensaje actualmente siendo procesado
fn reply(payload: impl Encode) -> Result<MessageId> {
    msg::reply(payload, 0)
}

//Implementamos la funcionalidad para la estructura Concert
impl Concert {
    //La funcion en la que creamos un concierto
    fn create_concert(
        &mut self,
        name: String,
        description: String,
        creator: ActorId,
        number_of_tickets: u128,
        date: u128,
    ) {
        //Solo podemos crear un concierto en el smart contract
        if self.running {
            panic!("CONCERT: There is already a concert registered.")
        }
        self.creator = creator;
        self.concert_id = self.id_counter;
        self.ticket_ft_id = self.concert_id;
        self.name = name;
        self.description = description;
        self.number_of_tickets = number_of_tickets;
        self.date = date;
        self.running = true;
        self.tickets_left = number_of_tickets;
        //Le respondemos al programa con ConcertEvent que guarda la informacion de la accion hecha
        reply(ConcertEvent::Creation {
            creator,
            concert_id: self.concert_id,
            number_of_tickets,
            date,
        })
        .expect("Error during a replying with ConcertEvent::Creation");
    }

    //La funcion con la que podemos comprar boletos
    async fn buy_tickets(&mut self, amount: u128, mtd: Vec<Option<TokenMetadata>>) {
        //Aseguramos que el comprador mande un mensaje desde uns direccion valida
        if msg::source() == ZERO_ID {
            panic!("CONCERT: Message from zero address");
        }
        //Aseguramos que se compre al menos un boleto
        if amount < 1 {
            panic!("CONCERT: Can not buy less than 1 ticket");
        }
        //Aseguramos que se compren la cantidad de boletos disponibles
        if self.tickets_left < amount {
            panic!("CONCERT: Not enough tickets");
        }
        //Aseguramos que si se quiere comprar mas de un boleto, se proporcione la informacion de cada uno
        if mtd.len() != amount as usize {
            panic!("CONCERT: Metadata not provided for all the tickets");
        }
        //Por cada boleto que vemos en la metadata hacemos los siguiente
        for meta in mtd {
            //Aumentamos el contador del id de los boletos
            self.id_counter += 1;
            //El hasmap de metadata es accesado por el valor de la direccion del comprador
            //Le agregamos al hasmap la metadata del boleto con su id
            self.metadata
                .entry(msg::source())
                .or_default()
                .insert(self.id_counter + 1, meta);
        }
        //Agregamos al comprador a la lista
        self.buyers.insert(msg::source());
        //Reducimos la cantidad de boletos disponibles
        self.tickets_left -= amount;
        //Mandamos un mensaje al smart contract de multitoken para que cree los tokens q
        //que representan los boletos
        msg::send_for_reply_as::<_, MTKEvent>(
            self.contract_id,
            MyMTKAction::Mint {
                amount,
                token_metadata: None,
            },
            0,
        )
        .expect("Error in async message to MTK contract")
        .await
        .expect("CONCERT: Error minting concert tokens");

        reply(ConcertEvent::Purchase {
            concert_id: self.concert_id,
            amount,
        })
        .expect("Error during a replying with ConcertEvent::Purchase");
    }

    //La funcion con la que volvemos a los boletos en NFTS
    // MINT SEVERAL FOR A USER
    async fn hold_concert(&mut self) {
        //Verificamos que solo el creador del concierto pueda generar los NFTs
        if msg::source() != self.creator {
            panic!("CONCERT: Only creator can hold a concert");
        }
        //Declaramos el vector accounts con los compradores y lo volvemos iterable
        let accounts: Vec<_> = self.buyers.clone().into_iter().collect();
        //Declaramos el vector de tokens con el valor de ticket_ft_id repetido el numero de veces de los compradores
        //Esto se hace para recuperar los saldos de los boletos de los compradores
        let tokens: Vec<TokenId> = iter::repeat(self.ticket_ft_id)
            .take(accounts.len())
            .collect();
        //Buscamos el numero de boletos que cada comprador tiene
        let balance_response: MTKEvent = msg::send_for_reply_as(
            self.contract_id,
            //Obtenemos el numero de tokens que tiene cada usuario
            MyMTKAction::BalanceOfBatch {
                accounts,
                ids: tokens,
            },
            0,
        )
        .expect("Error in async message to MTK contract")
        .await
        .expect("CONCERT: Error getting balances from the contract");
        //Verificamos que los balances(la cantidad de boletos que cada comprador tiene) sea correcto
        let balances: Vec<BalanceReply> =
            if let MTKEvent::BalanceOf(balance_response) = balance_response {
                balance_response
            } else {
                Vec::new()
            };
        // Por cada balance destruimos los tokens asociados
        for balance in &balances {
            msg::send_for_reply_as::<_, MTKEvent>(
                self.contract_id,
                MyMTKAction::Burn {
                    id: balance.id,
                    amount: balance.amount,
                },
                0,
            )
            .expect("Error in async message to MTK contract")
            .await
            .expect("CONCERT: Error burning balances");
        }
        //Creamos NFTs 
        for actor in &self.buyers {
            let mut ids = vec![];
            let mut amounts = vec![];
            let mut meta = vec![];
            //Buscamos la metadata de cada comprador
            let actor_metadata = self.metadata.get(actor);
            //Si tiene metadata se la asignamos a actor_md
            if let Some(actor_md) = actor_metadata.cloned() {
                //Iteramos entre los tokens que tiene el comprador y los metemos en los vectore
                for (token, token_meta) in actor_md {
                    ids.push(token);
                    amounts.push(1);
                    meta.push(token_meta);
                }
                //Convertimos a cada uno de estos tokens en un NFT con la funcion MIntBatch de multitoken
                //Cuando en amounts utilizas 1, la funcion los vuelve NFTs
                msg::send_for_reply_as::<_, MTKEvent>(
                    self.contract_id,
                    MyMTKAction::MintBatch {
                        ids,
                        amounts,
                        tokens_metadata: meta,
                    },
                    0,
                )
                .expect("Error in async message to MTK contract")
                .await
                .expect("CONCERT: Error minting tickets");
            }
        }
        //Declaramos al concierto como finalizado
        self.running = false;
        reply(ConcertEvent::Hold {
            concert_id: self.concert_id,
        })
        .expect("Error during a replying with ConcertEvent::Hold");
    }
}

//Esta funcion obtiene la informacion de la variable global CONTRACT y construye 
//una estructura State que representa el estado actual del concierto
fn common_state() -> State {
    //Obtenemos el estado actual de CONTRACT
    let Concert {
        owner_id,
        contract_id,
        name,
        description,
        ticket_ft_id,
        creator,
        number_of_tickets,
        tickets_left,
        date,
        buyers,
        id_counter,
        concert_id,
        running,
        metadata,
    } = unsafe { CONTRACT.get_or_insert(Default::default()) };

    //Creamos una estructura State como la de lib/io con los valores obtenidos de CONTRACT
    State {
        owner_id: *owner_id,
        contract_id: *contract_id,
        name: name.clone(),
        description: description.clone(),
        ticket_ft_id: *ticket_ft_id,
        creator: *creator,
        number_of_tickets: *number_of_tickets,
        tickets_left: *tickets_left,
        date: *date,
        //Copiamos cada elemento del vector 
        buyers: buyers.iter().copied().collect(),
        id_counter: *id_counter,
        concert_id: *concert_id,
        running: *running,
        //Volvemos el hasmap en un vector
        metadata: metadata
            .iter()
            .map(|(k, v)| (*k, v.iter().map(|(k, v)| (*k, v.clone())).collect()))
            .collect(),
    }
}

//Esta funcion permite que sea posible leer el estado del programa 
#[no_mangle]
extern "C" fn state() {
    reply(common_state()).expect(
        "Failed to encode or reply with `<ContractMetadata as Metadata>::State` from `state()`",
    );
}

//Esta funcion regresa el hash de la metadata
#[no_mangle]
extern "C" fn metahash() {
    let metahash: [u8; 32] = include!("../.metahash");
    reply(metahash).expect("Failed to encode or reply with `[u8; 32]` from `metahash()`");
}


