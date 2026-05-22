use crate::event_replay::Timed;
use crate::foundation::*;
use crate::opportunity_portfolio::*;

pub type OpportunityRankKey = Money;

#[must_use]
pub const fn opportunity_rank_key(candidate: &DropshipOpportunityCandidate) -> OpportunityRankKey {
    candidate.expected_profit()
}

pub fn opportunity_rank_keys(
    candidates: &[DropshipOpportunityCandidate],
) -> Vec<OpportunityRankKey> {
    candidates.iter().map(opportunity_rank_key).collect()
}

#[must_use]
pub fn rank_opportunity_keys(
    candidates: &[DropshipOpportunityCandidate],
) -> Timed<Vec<OpportunityRankKey>> {
    let mut keys = opportunity_rank_keys(candidates);
    let comparisons = merge_sort_count(&mut keys);
    Timed::new(keys, comparisons)
}

fn merge_sort_count(values: &mut [OpportunityRankKey]) -> Nat {
    let len = values.len();
    if len <= 1 {
        return 0;
    }
    let mid = len / 2;
    let mut left = values[..mid].to_vec();
    let mut right = values[mid..].to_vec();
    let mut comparisons = merge_sort_count(&mut left) + merge_sort_count(&mut right);
    let (mut i, mut j, mut k) = (0, 0, 0);
    while i < left.len() && j < right.len() {
        comparisons += 1;
        if left[i] <= right[j] {
            values[k] = left[i];
            i += 1;
        } else {
            values[k] = right[j];
            j += 1;
        }
        k += 1;
    }
    while i < left.len() {
        values[k] = left[i];
        i += 1;
        k += 1;
    }
    while j < right.len() {
        values[k] = right[j];
        j += 1;
        k += 1;
    }
    comparisons
}
