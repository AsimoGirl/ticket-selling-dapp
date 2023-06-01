# proyectoBlockchain

Desarrollo de una aplicación para la venta de boletos de un evento en Gear framework, parachain de Polkadot.


### Si no tiene Rust instalado con el toolchain de Gear en la computadora 

Ejecute el siguiente comando

```shell
make init
```

## Para construir el programa hacer lo siguiente

```shell
make build
```

o

```shell
cargo build --release
```

## Para correr las pruebas

Es necesario correr el siguiente comando porque las pruebas necesitan del binario del smart contract de multi_token

```shell
make test
```

## Para ejecutar todo con un comando

```shell
make all
```

Una vez que se construya el programa, para usarlo en Gear, ir a la página [idea.gear-tech.io][website] y en upload program introducir el archivo venta_boletos.opt.wasm que se encuentra en el directorio target/wasm32-unknown-unknown/release


