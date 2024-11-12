# XOXNO EGLD Liquid Staking Protocol

The **XOXNO EGLD Liquid Staking Protocol** is a decentralized, efficient, and user-friendly liquid staking solution on MultiversX. It empowers users to stake EGLD and receive its liquid representation, xEGLD, which they can utilize across DeFi applications, trade, or hold. The protocol has been designed to resolve issues seen in other liquid staking protocols by optimizing delegation distribution and improving decentralization and liquidity for both providers and users.

---

## 🎉 Why XOXNO Liquid Staking?

### Key Problems Solved

- **Delegation Centralization**: In other protocols, staking often concentrates on a few providers, undermining ecosystem health.
- **Manipulatable Delegation Strategy**: The lack of safeguards allows unfair provider selection, where large players can control delegation towards their own validators.
- **Inefficient Stake Redistribution**: Un-delegation impacts a single provider heavily, destabilizing their nodes.
- **Frequent Updates and Transaction Spam**: Other protocols rely on admin wallets for frequent provider updates, leading to excessive transactions.

### What XOXNO Brings New

XOXNO introduces an efficient, fair approach that accumulates delegations daily, delegating to multiple providers at set intervals to avoid centralization. Un-delegations are processed in batches, reducing the load on individual providers. Providers can join freely with no constraints, and protocol updates happen autonomously using public endpoints, which reduces admin interference and excessive transaction spam.

---

## ✨ Benefits

### For Users

- **Instant Unstaking with 0% Fees**: Users can convert xEGLD back to EGLD instantly when pending EGLD is available, bypassing the unbonding period.
- **Stable and Better APR**: Delegations are efficiently distributed across providers, ensuring a stable APR by balancing the total stake across multiple providers.
- **No Forced Fees**: Unlike other protocols that impose a fixed cut fee, XOXNO allows providers to set their own fees, offering users greater flexibility.

### For Providers

- **Daily Batch Delegations**: Ensures fair distribution of staked EGLD among multiple providers, preventing heavy concentration in one validator.
- **Smoother Un-delegation Impact**: Reduces the impact of large un-delegations on any single provider, keeping node operations more stable.
- **Autonomous Updates**: Providers' data is updated on-chain without admin wallets, making the protocol fairer and easier to manage.

---

## 📜 Protocol Overview

XOXNO’s Liquid Staking Protocol manages delegation distribution across multiple providers, following these steps:

1. **Delegation Pooling**: Instead of delegating immediately, incoming EGLD is accumulated within the contract.
2. **Batch Delegation**: Once per day, pooled EGLD is delegated across selected providers, distributing the amount fairly across up to 15–20 providers per transaction. When the amount allows, delegations can be distributed across the entire provider list in multiple transactions.
3. **Dynamic Un-delegation**: If users request to unstake, pending EGLD from the delegation pool is used for instant withdrawal. If unavailable, users enter a 10-day unbonding period, represented by an NFT.
4. **Reward Compounding**: Rewards are automatically re-delegated to providers, with XOXNO taking a fair cut via fresh xEGLD minting, thus compounding the user’s gains.

---

## 🧑‍🤝‍🧑 Users

Users can interact with the protocol using the following endpoints:

- **`delegate`**: Stake EGLD and receive xEGLD instantly.
- **`unDelegate`**: Redeem xEGLD for EGLD through instant conversion or enter the unbonding period.
- **`withdraw`**: Finalize unbonded EGLD withdrawal after the unbonding period.

### Provider Actions

Providers benefit from the following key endpoints:

- **`delegatePending`**: Delegates accumulated EGLD to selected providers daily.
- **`unDelegatePending`**: Un-delegates EGLD from providers to balance liquidity needs.
- **`withdrawPending`**: Withdraws EGLD from specific contracts to fulfill instant unstaking requests.
- **`claimRewards`**: Claims rewards on behalf of users from the underlying staking providers.
- **`delegateRewards`**: Reinvests claimed rewards back into the staking pool to ensure compounding.

---

## 🤝 Community

Connect with XOXNO and the liquid staking community:

- [Web](https://xoxno.com/)
- [App](https://xoxno.com/defi/liquid-staking)
- [Discord](https://discord.gg/xoxno)
- [X](https://x.com/XoxnoNetwork)
- [Telegram](https://t.me/xoxno)

---

This README highlights the unique benefits of XOXNO’s liquid staking solution for both users and providers, underscoring its decentralized, efficient, and user-centric design.
