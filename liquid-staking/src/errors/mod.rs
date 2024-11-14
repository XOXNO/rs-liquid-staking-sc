pub static ERROR_NOT_ACTIVE: &[u8] = b"Not active";
pub static ERROR_LS_TOKEN_NOT_ISSUED: &[u8] = b"LS token not issued";

pub static ERROR_CLAIM_START: &[u8] = b"Claim operation must be new or pending";
pub static ERROR_OLD_CLAIM_START: &[u8] =
    b"Previous claimed rewards must be redelegated or lesser than 1 EGLD";
pub static ERROR_INSUFFICIENT_REWARDS: &[u8] = b"Existing rewards must be greater than 1 EGLD";
pub static ERROR_RECOMPUTE_RESERVES: &[u8] = b"Claim operation must be in the finished status";
pub static ERROR_CLAIM_EPOCH: &[u8] = b"The rewards were already claimed for this epoch";
pub static ERROR_UNSTAKE_PERIOD_NOT_PASSED: &[u8] = b"The unstake period has not passed";

pub static ERROR_BAD_PAYMENT_TOKEN: &[u8] = b"Bad payment token";
pub static ERROR_BAD_PAYMENT_AMOUNT: &[u8] = b"Insufficient delegated amount";
pub static ERROR_INSUFFICIENT_PENDING_EGLD: &[u8] = b"Insufficient pending EGLD";
pub static ERROR_INSUFFICIENT_UNSTAKE_PENDING_EGLD: &[u8] = b"Insufficient unstake pending EGLD";
pub static ERROR_INSUFFICIENT_UNSTAKE_AMOUNT: &[u8] = b"Insufficient unstake amount";
pub static ERROR_INSUFFICIENT_UNBONDED_AMOUNT: &[u8] = b"Insufficient incoming withdraw amount";
pub static ERROR_INSUFFICIENT_LIQUIDITY: &[u8] = b"Insufficient liquidity minted";
pub static ERROR_INSUFFICIENT_LIQ_BURNED: &[u8] = b"Insufficient liquidity burned";

pub static ERROR_NOT_ENOUGH_LP: &[u8] = b"Not enough LP token supply";

pub static ERROR_BAD_DELEGATION_ADDRESS: &[u8] = b"No delegation contract available";
pub static ERROR_NO_DELEGATION_CONTRACTS: &[u8] = b"There are no delegation contracts whitelisted";
pub static ERROR_FIRST_DELEGATION_NODE: &[u8] = b"The first delegation node is incorrect";
pub static ERROR_ALREADY_WHITELISTED: &[u8] = b"Delegation contract already whitelisted";
pub static ERROR_NOT_WHITELISTED: &[u8] = b"Delegation contract is not whitelisted";
pub static ERROR_DELEGATION_CAP: &[u8] =
    b"Delegation cap must be higher than the total staked amount";
pub static ERROR_ONLY_DELEGATION_ADMIN: &[u8] =
    b"Only the admin of the delegation contract can change the status";
pub static ERROR_MINIMUM_ROUNDS_NOT_PASSED: &[u8] = b"Minimum rounds didn't pass";
pub static ERROR_MAX_DELEGATION_ADDRESSES: &[u8] =
    b"Maximum number of delegation addresses reached";
pub static ERROR_MAX_UN_DELEGATION_ADDRESSES: &[u8] =
    b"Maximum number of un delegation addresses reached";
pub static ERROR_MAX_SELECTED_PROVIDERS: &[u8] = b"Max selected providers must be greater than 0";
pub static ERROR_MAX_CHANGED_DELEGATION_ADDRESSES: &[u8] =
    b"Max delegation addresses must be greater than 0";

pub static ERROR_MIN_EGLD_TO_DELEGATE: &[u8] =
    b"Minimum EGLD to delegate must be greater than 1 EGLD";
pub static ERROR_MIGRATION_SC_NOT_SET: &[u8] = b"Migration SC not set";
pub static ERROR_MIGRATION_NOT_ALLOWED: &[u8] = b"Migration not allowed";

pub static ERROR_NOT_MANAGER: &[u8] = b"Caller is not authorized as a manager";

pub static ERROR_SCORING_CONFIG_NOT_SET: &[u8] = b"Scoring configuration not set";

pub static ERROR_WEIGHTS_MUST_SUM_TO_100: &[u8] = b"Weights must sum to 100";
