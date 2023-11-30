cd cubes
npm run build
cd -
cd artcx
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-unknown-linux-gnu-gcc cargo build --release --target=x86_64-unknown-linux-gnu
cd -
rm -rf release
mkdir -p release/static
cp artcx/target/x86_64-unknown-linux-gnu/release/artcx release/server
cp cubes/dist/* release/static/

cp artcx/public/* release/static/
