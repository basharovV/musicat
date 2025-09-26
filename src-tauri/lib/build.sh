# cd into script dir
cd "$(dirname "$0")"

cd ./portable-vocal-remover/pvr && cargo build -r -p pvr

cd ..
# Rename the binary based on TAURI_ENV_TARGET_TRIPLE
mv ./target/release/pvr ./target/release/pvr-${TAURI_ENV_TARGET_TRIPLE}