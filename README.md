# Ticket Selling DApp

The following is the development of a descentralized application for the sale of tickets for an event in Gear framework, Polkadot parachain.

The comments are in Spanish as they were used for educational reasons.

### If you do not have Rust installed with the Gear toolchain on your computer

Run the following command

```shell
make init
```

## To build the program do the following

```shell
make build
```

either

```shell
charge build --release
```

## To run the tests

It is necessary to run the following command because the tests need the multi_token smart contract binary

```shell
make test
```

## To run everything with one command

``shell
make all
```

Once the program is built, to use it in Gear, go to the page <https://idea.gear-tech.io> and in upload program enter the fileventa_boletos.opt.wasm found in the target/ directory wasm32-unknown-unknown/release

