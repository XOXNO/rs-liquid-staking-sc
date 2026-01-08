# Repository Guidelines

## Project Structure & Module Organization
- `liquid-staking/src` holds the production MultiversX smart contract; each module isolates a concern: `delegation` orchestrates provider calls, `liquidity_pool` maintains the xEGLD/EGLD ratio, `selection` + `score` pick fair provider sets, and `manage` guards privileged operations.
- `meta/` crates emit ABI/schema files, `interaction/*.snippets.sh` encode canonical CLI flows, and `output-docker/liquid-staking/*` stores reproducible WASM bundles.
- Scenario suites in `liquid-staking/tests` simulate delegation managers and the accumulator via the mock crates, so changes can be rehearsed end-to-end.

## Architectural Principles
- Liquidity is split between pending staking (`pending_egld_for_delegation`) and pending exits (`pending_egld`); actions always route through these buffers before touching providers.
- xEGLD is a rebasing claim: minting uses `LiquidityPoolModule::pool_add_liquidity`, while rewards enter via `delegateRewards`, increasing the EGLD backing per xEGLD.
- Daily `delegatePending` and `unDelegatePending` jobs respect `MAX_SELECTED_PROVIDERS` and distribute amounts proportionally using `selection::SelectionModule`, keeping the network decentralized.
- Provider metadata (caps, nodes, eligibility) lives in storage mappers; updates must flow through `manage::` endpoints to preserve invariants tracked by the accumulator contract.

## Core Flows & Rules
- `delegate`: accepts EGLD, updates pending buffers, optionally performs instant delegation for whitelisted providers, and mints xEGLD at the current exchange rate.
- `unDelegate`: transfers xEGLD in, burns it, returns instant liquidity if `pending_egld` suffices, and otherwise issues an NFT (`UnstakeTokenAttributes`) representing the 10-epoch unbond period.
- `withdraw`: validates epoch maturity and token identity before releasing unbonded EGLD; the contract must be in `State::Active`.
- `delegatePending`/`unDelegatePending`: callable only by managers; batches must stay within gas limits and respect `max_delegation_addresses`.

## Security Invariants & Review Checklist
- State machine: upgrades set `Inactive`, activation requires `setStateActive`; never assume endpoints run when the state is paused.
- Access control: only managers or governance (`vote::VoteModule`) may change provider lists, fees, or critical config; always add tests covering unauthorized attempts.
- Buffer safety: never bypass `get_action_amount` helpers—doing so risks double-counting pending liquidity or starving instant exits.
- Token hygiene: enforce correct token identifiers (`LS_TOKEN_ID`, `UNSTAKE_TOKEN_ID`), positive amounts, and matching epochs to avoid malicious NFT reuse.
- Config drift: whenever touching `UNBOND_PERIOD`, fee curves, or provider caps, update on-chain storage migrations and the scripts in `interaction/*.sh`.

## Build & Test Touchpoints
- `cargo test -p liquid-staking -- --nocapture` exercises all scenarios; add targeted cases per module touched.
- `cargo run -p liquid-staking-abi` keeps ABI consumers in sync.
- `mxpy contract reproducible-build --docker-image="multiversx/sdk-rust-contract-builder:v11.0.0"` is the blessed path for shipping binaries referenced by deployment scripts.
