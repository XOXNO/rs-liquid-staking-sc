ADDRESS=erd1qqqqqqqqqqqqqpgq6uzdzy54wnesfnlaycxwymrn9texlnmyah0ssrfvk6
PROXY=https://gateway.xoxno.com
PROJECT="../output-docker/liquid-staking/liquid-staking.wasm"

ACCUMULATOR_SC_ADDRESS=erd1qqqqqqqqqqqqqpgq8538ku69p97lq4eug75y8d6g6yfwhd7c45qs4zvejt
FEES=700
MAX_SELECTED_PROVIDERS=20
MAX_DELEGATION_ADDRESSES=100
UNBOND_PERIOD=10
MIGRATION_SC_ADDRESS="erd1qqqqqqqqqqqqqpgqc0jp2q280xaccqszxwsh5cyl2hv35g79ah0sk4zu5n"

deploy() {
    mxpy --verbose contract deploy --bytecode=${PROJECT}  --metadata-payable-by-sc --arguments ${ACCUMULATOR_SC_ADDRESS} ${FEES} ${MAX_SELECTED_PROVIDERS} ${MAX_DELEGATION_ADDRESSES} ${UNBOND_PERIOD} --recall-nonce \
    --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=150000000 --send --proxy=${PROXY} --chain=1 || return

    echo "New smart contract address: ${ADDRESS}"
}

upgrade() {
    echo "Upgrade smart contract address: ${ADDRESS}"
    mxpy  contract upgrade ${ADDRESS} --metadata-payable-by-sc --bytecode=${PROJECT} --recall-nonce \
    --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=150000000 --send --proxy=${PROXY} --chain=1 || return
}

registerLsToken() {
    mxpy contract call ${ADDRESS} --recall-nonce --function="registerLsToken" \
    --arguments str:StakedEGLD str:XEGLD 0x12 --value 50000000000000000 \
    --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=150000000 --send --proxy=${PROXY} --chain=1 || return
}

registerUnstakeToken() {
    mxpy contract call ${ADDRESS} --recall-nonce --function="registerUnstakeToken" \
    --arguments str:UnbondingEGLD str:UEGLD 0x12 --value 50000000000000000 \
    --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=150000000 --send --proxy=${PROXY} --chain=1 || return
}

setMigrationScAddress() {
    mxpy contract call ${ADDRESS} --recall-nonce \
        --ledger --ledger-account-index=0 --ledger-address-index=0 \
        --proxy=${PROXY} --chain=1 \
        --gas-limit=12000000 \
        --function="setMigrationScAddress" \
        --arguments ${MIGRATION_SC_ADDRESS} \
        --send || return
}

setStateActive() {
    mxpy contract call ${ADDRESS} --recall-nonce --function="setStateActive" \
    --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=15000000 --send --proxy=${PROXY} --chain=1 || return
}

getExchangeRate() {
    mxpy --verbose contract query ${ADDRESS} \
        --proxy=${PROXY} \
        --function="getExchangeRate"
}

getEgldPositionValue() {
    mxpy --verbose contract query ${ADDRESS} \
        --proxy=${PROXY} \
        --function="getEgldPositionValue" --arguments 1000000000000000000
}

getLsValueForPosition() {
    mxpy --verbose contract query ${ADDRESS} \
        --proxy=${PROXY} \
        --function="getLsValueForPosition" --arguments 892262748273425358
}

verifyContract() {
    mxpy --verbose contract verify "${ADDRESS}"  \
    --packaged-src=../output-docker/liquid-staking/liquid-staking-0.0.0.source.json --verifier-url="https://play-api.multiversx.com" \
    --docker-image="multiversx/sdk-rust-contract-builder:v8.0.1" --ledger --ledger-account-index=0 --ledger-address-index=0  || return 
}

buildDocker() {
    mxpy contract reproducible-build --docker-image="multiversx/sdk-rust-contract-builder:v8.0.1"
}

###PARAMS 
### Contracts - erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqplllllscktaww
DELEGATION_ADDRESS="erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqz0llllsup4dew"
ADMIN_ADDRESS="erd1x45vnu7shhecfz0v03qqfmy8srndch50cdx7m763p743tzlwah0sgzewlm"
TOTAL_STAKED=15032555858737269063515
DELEGATION_CAP=28126500000000000000000
NR_NODES=5
APY=1800
whitelistDelegationContract() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce \
        --function="whitelistDelegationContract" \
        --ledger --ledger-account-index=0 --ledger-address-index=0 \
        --gas-limit=10000000 \
        --proxy=${PROXY} --chain=1 \
        --arguments ${DELEGATION_ADDRESS} ${ADMIN_ADDRESS} ${TOTAL_STAKED} ${DELEGATION_CAP} ${NR_NODES} ${APY}\
        --send || return
}

changeDelegationContractParams() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce \
        --function="changeDelegationContractParams" \
        --ledger --ledger-account-index=0 --ledger-address-index=0 \
        --gas-limit=10000000 \
        --proxy=${PROXY} --chain=1 \
        --arguments ${DELEGATION_ADDRESS} ${TOTAL_STAKED} ${DELEGATION_CAP} ${NR_NODES} ${APY} 0x01 \
        --send || return
}

delegate() {
        mxpy contract call ${ADDRESS} --recall-nonce \
        --ledger --ledger-account-index=0 --ledger-address-index=0 \
        --proxy=${PROXY} --chain=1 \
        --gas-limit=10000000 \
        --value=100000000000000000000 \
        --function="delegate" \
        --send || return
}

unDelegate() {
        method_name=str:unDelegate
        my_token=str:XEGLD-c67ed3
        token_amount=300000000000000000
        mxpy contract call ${ADDRESS} --recall-nonce \
        --ledger --ledger-account-index=0 --ledger-address-index=0 \
        --proxy=${PROXY} --chain=1 \
        --gas-limit=10000000 \
        --function="ESDTTransfer" \
        --arguments $my_token $token_amount $method_name \
        --send || return
}

delegatePending() {
        mxpy contract call ${ADDRESS} --recall-nonce \
        --ledger --ledger-account-index=0 --ledger-address-index=0 \
        --proxy=${PROXY} --chain=1 \
        --gas-limit=250000000 \
        --function="delegatePending" \
        --send || return
}

unDelegatePending() {
        mxpy contract call ${ADDRESS} --recall-nonce \
        --ledger --ledger-account-index=0 --ledger-address-index=0 \
        --proxy=${PROXY} --chain=1 \
        --gas-limit=250000000 \
        --function="unDelegatePending" \
        --send || return
}

updateMaxDelegationAddresses() {
    mxpy contract call ${ADDRESS} --recall-nonce \
        --ledger --ledger-account-index=0 --ledger-address-index=0 \
        --proxy=${PROXY} --chain=1 \
        --gas-limit=10000000 \
        --function="updateMaxDelegationAddresses" \
        --arguments 100 \
        --send || return
}

updateMaxSelectedProviders() {
    mxpy contract call ${ADDRESS} --recall-nonce \
        --ledger --ledger-account-index=0 --ledger-address-index=0 \
        --proxy=${PROXY} --chain=1 \
        --gas-limit=10000000 \
        --function="updateMaxSelectedProviders" \
        --arguments ${MAX_SELECTED_PROVIDERS} \
        --send || return
}

setUnbondPeriod() {
    mxpy contract call ${ADDRESS} --recall-nonce \
        --ledger --ledger-account-index=0 --ledger-address-index=0 \
        --proxy=${PROXY} --chain=1 \
        --gas-limit=10000000 \
        --function="setUnbondPeriod" \
        --arguments ${UNBOND_PERIOD} \
        --send || return
}

addManagers() {
    MANAGER_ADDRESS="erd1fmd662htrgt07xxd8me09newa9s0euzvpz3wp0c4pz78f83grt9qm6pn57"
    MANAGER_ADDRESS2="erd1vn9s8uj4e7r6skmqfw5py3hxnluw3ftv6dh47yt449vtvdnn9w2stmwm7l"
    MANAGER_ADDRESS3="erd1cfyadenn4k9wndha0ljhlsdrww9k0jqafqq626hu9zt79urzvzasalgycz"
    mxpy contract call ${ADDRESS} --recall-nonce \
        --ledger --ledger-account-index=0 --ledger-address-index=0 \
        --proxy=${PROXY} --chain=1 \
        --gas-limit=12000000 \
        --function="addManagers" \
        --arguments ${MANAGER_ADDRESS} ${MANAGER_ADDRESS2} ${MANAGER_ADDRESS3} \
        --send || return
}