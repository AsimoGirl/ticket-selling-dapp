use gear_lib::multitoken::io::*;
use gstd::{prelude::*, ActorId, String};

mod utils;
use utils::*;

//Hacemos un test para crear un concierto
#[test]
fn create_concert() {
    let system = init_system();
    let concert_program = init_concert(&system);
    create(
        &concert_program,
        USER.into(),
        String::from("Stromae"),
        String::from("Stromae en la CDMX 21/06/2023"),
        NUMBER_OF_TICKETS,
        DATE,
        CONCERT_ID,
    );

    //Revisamos que el concierto creado se puede leer en el estado
    check_current_concert(
        &concert_program,
        String::from("Stromae"),
        String::from("Stromae en la CDMX 21/06/2023"),
        DATE,
        NUMBER_OF_TICKETS,
        NUMBER_OF_TICKETS,
    )
}

//Hacemos un test para comprar boletos
#[test]
fn buy_tickets() {
    let system = init_system();
    let concert_program = init_concert(&system);
    create(
        &concert_program,
        USER.into(),
        String::from("Stromae"),
        String::from("Stromae en la CDMX 21/06/2023"),
        NUMBER_OF_TICKETS,
        DATE,
        CONCERT_ID,
    );

    let metadata = vec![Some(TokenMetadata {
        title: Some(String::from("Boleto #1 Stromae en la CDMX 21/06/2023")),
        description: Some(String::from(
            "Boleto #1 Stromae en la CDMX 21/06/2023. Fila 4. Asiento 4.",
        )),
        media: Some(String::from("URL Stromae")),
        reference: Some(String::from("URL JSON con mas info")),
    })];

    buy(
        &concert_program,
        CONCERT_ID,
        AMOUNT,
        metadata.clone(),
        false,
    );
    check_buyers(&concert_program, vec![ActorId::from(USER)]);
    check_user_tickets(&concert_program, ActorId::from(USER), metadata);
}

//Revisamos que la compra de boletos este a prueba de ciertos errores
#[test]
fn buy_tickets_failures() {
    let system = init_system();
    let concert_program = init_concert(&system);
    create(
        &concert_program,
        USER.into(),
        String::from("Stromae"),
        String::from("Stromae en la CDMX 21/06/2023"),
        NUMBER_OF_TICKETS,
        DATE,
        CONCERT_ID,
    );

    // Debe fallar ya que se compra menos de 1 boleto
    buy(&concert_program, CONCERT_ID, 0, vec![None], true);

    // Debe fallar porque queremos comprar mas boletos que los disponibles
    buy(
        &concert_program,
        CONCERT_ID,
        NUMBER_OF_TICKETS + 1,
        vec![None; (NUMBER_OF_TICKETS + 1) as usize],
        true,
    );

    // Debe fallar ya que no se esta dando medata para todos los boletos
    buy(
        &concert_program,
        CONCERT_ID,
        AMOUNT + 3,
        vec![None; (AMOUNT + 1) as usize],
        true,
    );
}

//Se prueba volver a NFTs los tokens
#[test]
fn hold_concert() {
    let system = init_system();
    let concert_program = init_concert(&system);

    create(
        &concert_program,
        USER.into(),
        String::from("Stromae"),
        String::from("Stromae en la CDMX 21/06/2023"),
        NUMBER_OF_TICKETS,
        DATE,
        CONCERT_ID,
    );

    let metadata = vec![Some(TokenMetadata {
        title: Some(String::from("Boleto #1 Stromae en la CDMX 21/06/2023")),
        description: Some(String::from(
            "Boleto #1 Stromae en la CDMX 21/06/2023. Fila 4. Asiento 4.",
        )),
        media: Some(String::from("URL Stromae")),
        reference: Some(String::from("URL JSON con mas info")),
    })];

    buy(&concert_program, CONCERT_ID, AMOUNT, metadata, false);

    hold(&concert_program, CONCERT_ID);
}
