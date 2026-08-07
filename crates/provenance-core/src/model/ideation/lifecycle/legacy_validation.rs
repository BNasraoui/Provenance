use super::LegacyProposalPolicy;
use crate::model::{DispositionRecord, PromotionState, ProposalCard};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

const SHIPPED_LEGACY_TERMINAL_DIGEST_V1: &str =
    "f3438033d8dfcab54788c3dad687eca24de9a0278ada120ba1296b2d2b86f843";
const SHIPPED_LEGACY_DISPOSITION_DIGEST_V1: &str =
    "8f25c3f3028b68b8f914efcec3a19e08127a86797b8a01bcd5f796dac90c9cfb";

pub(super) fn validate_records(
    proposals: &[ProposalCard],
    dispositions: &[DispositionRecord],
    policy: LegacyProposalPolicy,
) -> anyhow::Result<()> {
    let mut terminals = proposals
        .iter()
        .filter(|proposal| proposal.promotion_state != PromotionState::Proposed)
        .collect::<Vec<_>>();
    if terminals.is_empty() {
        return Ok(());
    }
    anyhow::ensure!(
        policy == LegacyProposalPolicy::ShippedV1,
        "terminal proposal rows are forbidden by the modern-only lifecycle policy"
    );
    terminals.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    let mut hasher = Sha256::new();
    for proposal in terminals {
        hasher.update(serde_json::to_vec(proposal).expect("proposal serialization is infallible"));
        hasher.update(b"\n");
    }
    anyhow::ensure!(
        format!("{:x}", hasher.finalize()) == SHIPPED_LEGACY_TERMINAL_DIGEST_V1,
        "terminal proposal rows do not match the frozen shipped-v1 fingerprint"
    );
    validate_disposition_audit(proposals, dispositions)
}

fn validate_disposition_audit(
    proposals: &[ProposalCard],
    dispositions: &[DispositionRecord],
) -> anyhow::Result<()> {
    let terminal_ids = proposals
        .iter()
        .filter(|proposal| proposal.promotion_state != PromotionState::Proposed)
        .map(|proposal| proposal.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut audit = dispositions
        .iter()
        .filter(|disposition| terminal_ids.contains(disposition.proposal_id.as_str()))
        .collect::<Vec<_>>();
    audit.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    let mut hasher = Sha256::new();
    for disposition in audit {
        hasher.update(
            serde_json::to_vec(disposition).expect("disposition serialization is infallible"),
        );
        hasher.update(b"\n");
    }
    anyhow::ensure!(
        format!("{:x}", hasher.finalize()) == SHIPPED_LEGACY_DISPOSITION_DIGEST_V1,
        "disposition rows do not match the frozen shipped-v1 disposition audit"
    );
    Ok(())
}
