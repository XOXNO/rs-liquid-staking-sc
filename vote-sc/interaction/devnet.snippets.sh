ADDRESS=erd1qqqqqqqqqqqqqpgqrhysvaph268e0n27sa6ll67dluhwxdu9ah0suakcq2
PROXY=https://devnet-gateway.multiversx.com
PROJECT="./output-docker/vote-sc/vote-sc.wasm"
LIQUID_STAKING_SC_ADDRESS="erd1qqqqqqqqqqqqqpgqc2d2z4atpxpk7xgucfkc7nrrp5ynscjrah0scsqc35"
ROOT_HASH="0x1279bb47a567171b665f66ad9a411b57c9fba7f8bbbdebf0ec6bdfc0ae666ff3"
PROPOSAL_ID=102

deploy() {
    mxpy --verbose contract deploy --bytecode=${PROJECT} --recall-nonce \
    --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=150000000 --send --proxy=${PROXY} --chain=D || return

    echo "New smart contract address: ${ADDRESS}"
}

upgrade() {
    echo "Upgrade smart contract address: ${ADDRESS}"
    mxpy  contract upgrade ${ADDRESS} --bytecode=${PROJECT} --recall-nonce \
    --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=600000000 --send --proxy=${PROXY} --chain="D" || return
}

setLiquidStakingAddress() {
    mxpy contract call ${ADDRESS} --recall-nonce --function="set_liquid_staking_address" \
    --arguments ${LIQUID_STAKING_SC_ADDRESS} \
    --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=50000000 --send --proxy=${PROXY} --chain=D || return
}

setRootHash() {
    mxpy contract call ${ADDRESS} --recall-nonce --function="set_root_hash" \
    --arguments ${ROOT_HASH} ${PROPOSAL_ID} \
    --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=50000000 --send --proxy=${PROXY} --chain=D || return
}

verifyContract() {
    mxpy --verbose contract verify "${ADDRESS}"  \
    --packaged-src=./output-docker/vote-sc/vote-sc-0.0.0.source.json --verifier-url="https://devnet-play-api.multiversx.com" \
    --docker-image="multiversx/sdk-rust-contract-builder:v11.0.0" --ledger --ledger-account-index=0 --ledger-address-index=0  || return 
}

buildDocker() {
    mxpy contract reproducible-build --docker-image="multiversx/sdk-rust-contract-builder:v11.0.0"
}