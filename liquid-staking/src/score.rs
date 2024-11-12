use crate::{
    structs::{DelegationContractSelectionInfo, ScoringConfig},
    utils::BPS,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ScoreModule {
    fn calculate_and_update_score(
        &self,
        info: &mut DelegationContractSelectionInfo<Self::Api>,
        is_delegate: bool,
        total_stake: &BigUint,
        config: &ScoringConfig,
    ) -> BigUint {
        let node_score = self.calculate_node_score(info.nr_nodes, is_delegate, config);
        let apy_score = self.calculate_apy_score(info.apy, is_delegate, config);
        let stake_score = self.calculate_stake_score(
            &info.total_staked_from_ls_contract,
            total_stake,
            is_delegate,
            config,
        );

        let final_score = self.combine_scores(node_score, apy_score, stake_score, config);
        info.score = final_score.clone();
        final_score
    }

    fn calculate_node_score(
        &self,
        nr_nodes: u64,
        is_delegate: bool,
        config: &ScoringConfig,
    ) -> BigUint {
        let nodes_biguint = BigUint::from(nr_nodes);
        let min = BigUint::from(config.min_nodes);
        let max = BigUint::from(config.max_nodes);

        // Get base score
        let base_score = self.norm_linear_clamp(&nodes_biguint, &min, &max, is_delegate);

        // Apply aggressive exponential scaling
        let scaled_score = base_score.pow(config.exponential_base as u32);

        // Multiply by max_score_per_category to get final range
        scaled_score * BigUint::from(config.max_score_per_category) / BigUint::from(BPS)
    }

    fn calculate_apy_score(&self, apy: u64, is_delegate: bool, config: &ScoringConfig) -> BigUint {
        let apy_biguint = BigUint::from(apy);
        let min = BigUint::from(config.min_apy);
        let max = BigUint::from(config.max_apy);

        // Get base score - note the !is_delegate for inverse scoring
        let base_score = self.norm_linear_clamp(&apy_biguint, &min, &max, !is_delegate);

        // Apply more aggressive exponential scaling for APY
        let scaled_score = base_score.pow(config.apy_growth_multiplier as u32);

        // Multiply by max_score_per_category to get final range
        scaled_score * BigUint::from(config.max_score_per_category) / BigUint::from(BPS)
    }

    fn calculate_stake_score(
        &self,
        staked: &BigUint,
        total_stake: &BigUint,
        is_delegate: bool,
        config: &ScoringConfig,
    ) -> BigUint {
        return if total_stake > &BigUint::zero() {
            // Get base score
            let base_score =
                self.norm_linear_clamp(staked, &BigUint::zero(), total_stake, is_delegate);

            // Apply exponential scaling
            let scaled_score = base_score.pow(config.exponential_base as u32);

            // Multiply by max_score_per_category to get final range
            scaled_score * BigUint::from(config.max_score_per_category) / BigUint::from(BPS)
        } else {
            // Apply exponential scaling
            let scaled_score =
                BigUint::from(config.max_score_per_category).pow(config.exponential_base as u32);

            // Multiply by max_score_per_category to get final range
            scaled_score * BigUint::from(config.max_score_per_category) / BigUint::from(BPS)
        };
    }

    fn combine_scores(
        &self,
        node_score: BigUint,
        apy_score: BigUint,
        stake_score: BigUint,
        config: &ScoringConfig,
    ) -> BigUint {
        // Apply weights
        let weighted_score = node_score
            .mul(config.nodes_weight)
            .add(&apy_score.mul(config.apy_weight))
            .add(&stake_score.mul(config.stake_weight));

        weighted_score
    }

    fn update_selected_addresses_scores(
        &self,
        selected_addresses: &mut ManagedVec<DelegationContractSelectionInfo<Self::Api>>,
        is_delegate: bool,
        total_stake: &BigUint,
        config: &ScoringConfig,
    ) -> BigUint {
        let mut total_score = BigUint::zero();

        for index in 0..selected_addresses.len() {
            let mut info = selected_addresses.get(index);
            let score =
                self.calculate_and_update_score(&mut info, is_delegate, total_stake, config);
            total_score += &score;
            let _ = selected_addresses.set(index, info);
        }

        total_score
    }

    fn norm_linear_clamp(
        &self,
        value: &BigUint,
        min: &BigUint,
        max: &BigUint,
        down: bool,
    ) -> BigUint {
        let normalized = if value < min {
            BigUint::zero()
        } else if value > max {
            BigUint::from(BPS)
        } else {
            (value - min) * BigUint::from(BPS) / (max - min)
        };

        if down {
            BigUint::from(BPS) - normalized
        } else {
            normalized
        }
    }
}
