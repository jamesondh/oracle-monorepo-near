  #!/bin/bash
set -e
cd "`dirname $0`"
source flags.sh
cargo build --target wasm32-unknown-unknown --release
cp ./../../target/wasm32-unknown-unknown/release/target-contract.wasm  ../../res
cp ../../res/target-contract.wasm ../../oracle/tests/it/wasm
