#![no_std]
//Si no se incluye binary-vendor en el Cargo el codigo espera una implementacion local
// del modulo contract
#[cfg(not(feature = "binary-vendor"))]
mod contract;

//Si se incluye binary-vendor en el Cargo el codigo genera un binario al compilarse
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
