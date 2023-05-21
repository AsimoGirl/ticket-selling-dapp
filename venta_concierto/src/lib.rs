//Le dice a rust que compile sin la biblioteca estandar, ya que el entorno de gear no cuenta con una
#![no_std]

#[cfg(not(feature = "binary-vendor"))]
mod contract;

#[cfg(feature = "binary-vendor")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));