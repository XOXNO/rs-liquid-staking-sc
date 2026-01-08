ADDRESS=erd1qqqqqqqqqqqqqpgqdnpmeseu3j5t7grds9dfj8ttt70pev66ah0sydkq9x
PROXY=https://gateway.xoxno.com
PROJECT="./output-docker/vote-sc/vote-sc.wasm"
LIQUID_STAKING_SC_ADDRESS="erd1qqqqqqqqqqqqqpgq6uzdzy54wnesfnlaycxwymrn9texlnmyah0ssrfvk6"
ROOT_HASH="0x8d45b99f1b9ccb1eb5abb0c817fc160be792543dcbdbc53a6a2281b047e72ff2"
PROPOSAL_ID=2

deploy() {
    mxpy --verbose contract deploy --bytecode=${PROJECT} --recall-nonce \
    --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=150000000 --send --proxy=${PROXY} --chain=1 || return

    echo "New smart contract address: ${ADDRESS}"
}

upgrade() {
    echo "Upgrade smart contract address: ${ADDRESS}"
    mxpy  contract upgrade ${ADDRESS} --bytecode=${PROJECT} --recall-nonce \
    --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=600000000 --send --proxy=${PROXY} --chain=1 || return
}

setLiquidStakingAddress() {
    mxpy contract call ${ADDRESS} --recall-nonce --function="set_liquid_staking_address" \
    --arguments ${LIQUID_STAKING_SC_ADDRESS} \
    --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=50000000 --send --proxy=${PROXY} --chain=1 || return
}

setRootHash() {
    mxpy contract call ${ADDRESS} --function="set_root_hash" \
    --arguments ${ROOT_HASH} ${PROPOSAL_ID} \
    --ledger \
    --gas-limit=50000000 --send --proxy=${PROXY} --chain=1 || return
}

verifyContract() {
    mxpy --verbose contract verify "${ADDRESS}"  \
    --packaged-src=./output-docker/vote-sc/vote-sc-0.0.0.source.json --verifier-url="https://play-api.multiversx.com" \
    --docker-image="multiversx/sdk-rust-contract-builder:v11.0.0" --ledger --ledger-account-index=0 --ledger-address-index=0  || return 
}

buildDocker() {
    mxpy contract reproducible-build --docker-image="multiversx/sdk-rust-contract-builder:v11.0.0"
}