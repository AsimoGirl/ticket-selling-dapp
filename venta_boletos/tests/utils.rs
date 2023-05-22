//Este programa incluye funciones auxiliares para realizar pruebas
use venta_boletos_io::*;
use gear_lib::multitoken::io::{InitConfig, TokenMetadata};
use gstd::{prelude::*, ActorId, Encode};
use gtest::{Program, System};

pub const USER: u64 = 193;
pub const MTK_ID: u64 = 2;
pub const CONCERT_ID: u128 = 0;
pub const NUMBER_OF_TICKETS: u128 = 100;
pub const AMOUNT: u128 = 1;
pub const DATE: u128 = 210623;

//Iniciamos el sistema
pub fn init_system() -> System {
    let system = System::new();
    system.init_logger();

    system
}

//Emulamos el inicio de un concierto, usamos directamente el wasm de multi_token que se obtiene al hacer make test
//Ya que no tenemos un valor de smart contract de multi-token fijo
pub fn init_concert(sys: &System) -> Program {
    let concert_program = Program::current(sys);
    //Obtenemos el binario del smart contract de multi-token
    let mtk_program = Program::from_file(sys, "target/multi_token.wasm");
    //Iniciamos el smart contract de multi-token
    let res = mtk_program.send(
        USER,
        InitConfig {
            name: String::from("Multitoken for a concert"),
            symbol: String::from("MTC"),
            base_uri: String::from(""),
        },
    );
    assert!(res.log().is_empty());
    assert!(concert_program
        .send(
            USER,
            InitConcert {
                owner_id: USER.into(),
                mtk_contract: MTK_ID.into(),
            },
        )
        .log()
        .is_empty());

    concert_program
}

//Creamos un concierto
pub fn create(
    concert_program: &Program,
    creator: ActorId,
    name: String,
    description: String,
    number_of_tickets: u128,
    date: u128,
    concert_id: u128,
) {
    let res = concert_program.send(
        USER,
        ConcertAction::Create {
            creator,
            name,
            description,
            number_of_tickets,
            date,
        },
    );

    assert!(res.contains(&(
        USER,
        ConcertEvent::Creation {
            creator,
            concert_id,
            number_of_tickets,
            date,
        }
        .encode()
    )));
}

//Compramos un boleto
pub fn buy(
    concert_program: &Program,
    concert_id: u128,
    amount: u128,
    metadata: Vec<Option<TokenMetadata>>,
    should_fail: bool,
) {
    let res = concert_program.send(USER, ConcertAction::BuyTickets { amount, metadata });

    if should_fail {
        assert!(res.main_failed());
    } else {
        assert!(res.contains(&(USER, ConcertEvent::Purchase { concert_id, amount }.encode())));
    }
}

//Volvemos el boleto en NFT
pub fn hold(concert_program: &Program, concert_id: u128) {
    let res = concert_program.send(USER, ConcertAction::Hold {});

    assert!(res.contains(&(USER, ConcertEvent::Hold { concert_id }.encode())));
}

//Revisa el estado actual del programa, viendo que los datos que se le envien son los que se pueden ver en el estado
pub fn check_current_concert(
    concert_program: &Program,
    name: String,
    description: String,
    date: u128,
    number_of_tickets: u128,
    tickets_left: u128,
) {
    let state: State = concert_program.read_state().expect("Can't read state");
    let CurrentConcert {
        name: true_name,
        description: true_description,
        date: true_date,
        number_of_tickets: true_number_of_tickets,
        tickets_left: true_tickets_left,
    } = state.current_concert();
    if name != true_name {
        panic!("CONCERT: Concert name differs.");
    }
    if description != true_description {
        panic!("CONCERT: Concert description differs.");
    }
    if date != true_date {
        panic!("CONCERT: Concert date differs.");
    }
    if number_of_tickets != true_number_of_tickets {
        panic!("CONCERT: Concert number of tickets differs.");
    }
    if tickets_left != true_tickets_left {
        panic!("CONCERT: Concert number of tickets left differs.");
    }
}

//Revisa que los boletos de un comprador sean correctos comparandolos con lo que muestra el estado
pub fn check_user_tickets(
    concert_program: &Program,
    user: ActorId,
    tickets: Vec<Option<TokenMetadata>>,
) {
    let state: State = concert_program.read_state().expect("Can't read state");
    let true_tickets = state.user_tickets(user);
    if tickets != true_tickets {
        panic!("CONCERT: User tickets differ.");
    }
}

//Revisa que los compradores sean los mismos que los que estan en el estado del programa
pub fn check_buyers(concert_program: &Program, buyers: Vec<ActorId>) {
    let state: State = concert_program.read_state().expect("Can't read state");
    if buyers != state.buyers {
        panic!("CONCERT: Buyers list differs.");
    }
}
