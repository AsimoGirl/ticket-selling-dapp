use venta_concierto_io::ContractMetadata;

//Construye los archivoss wasm con la metadata
fn main(){
    gear_wasm_builder::build_with_metadata::<ContractMetadata>();
}