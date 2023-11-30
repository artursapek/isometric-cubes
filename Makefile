#!/bin/sh

cd cubes
npm run build
cd -
cd artcx
cargo build --release
cd -
rm -rf release
mkdir -p release/static
cp artcx/target/release/artcx release/server
cp cubes/dist/* release/static/

