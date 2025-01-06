#!/bin/sh

rm -rf dist/*
mkdir -p dist/static

if [[ $1 == "prod" ]]; then
  cd artcx
  echo "Building x86_64 release for production"
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-unknown-linux-gnu-gcc cargo build --release --target=x86_64-unknown-linux-gnu
  cd -
  cp artcx/target/x86_64-unknown-linux-gnu/release/artcx dist/server

  cd cubes
  npm run build
  cd -

else
  cd artcx
  echo "Building development server"
  cargo build --release
  cd -
  cp artcx/target/release/artcx dist/server

  cd cubes
  npm run build
  cd -
fi

cp -r cubes/dist/* dist/static/
cp -r artcx/public/* dist/static/
