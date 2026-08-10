//! When a synthesis packet qualifies a proposal for assertion.

use crate::model::{AssertionRecord, ProposalCard, SynthesisPacket};
use provenance_macros::rule;

/// The facts the qualification decision runs on, lifted out of the records so
/// that the decision itself is a total function over a finite boolean domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct QualificationFacts {
    /// The packet and the proposal share a scope and a traceability target.
    pub packet_owns_proposal_target: bool,
    /// The packet's suggested artifacts name this proposal.
    pub packet_adjudicates_proposal: bool,
    /// One of the packet's evidence gaps blocks promotion.
    pub blocking_evidence_gap: bool,
    /// One of the packet's required human decisions blocks promotion.
    pub blocking_human_decision: bool,
    /// The packet already carries an assertion, and none of them is for this
    /// proposal.
    pub packet_asserts_another_proposal: bool,
    /// The proposal cites at least one supporting claim.
    pub proposal_has_supporting_claims: bool,
    /// The packet contests one of the proposal's supporting claims.
    pub proposal_claim_contested: bool,
}

/// The conjunct a set of facts failed, named so both readings of the decision
/// can report the same cause in their own words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnmetQualification {
    PacketNotOwnedByProposalTarget,
    PacketDoesNotAdjudicateProposal,
    BlockingEvidenceGap,
    BlockingHumanDecision,
    PacketAssertsAnotherProposal,
    ProposalHasNoSupportingClaims,
    ProposalClaimContested,
}

impl UnmetQualification {
    pub(super) fn describe(self, proposal_id: &str) -> String {
        match self {
            Self::PacketNotOwnedByProposalTarget => {
                "assertion synthesis is not owned by the proposal target".to_owned()
            }
            Self::PacketDoesNotAdjudicateProposal => {
                format!("synthesis packet does not adjudicate proposal {proposal_id}")
            }
            Self::BlockingEvidenceGap => "assertion has a blocking evidence gap".to_owned(),
            Self::BlockingHumanDecision => "assertion has a blocking human decision".to_owned(),
            Self::PacketAssertsAnotherProposal => {
                format!("synthesis packet asserts a proposal other than {proposal_id}")
            }
            Self::ProposalHasNoSupportingClaims => {
                format!("proposal {proposal_id} cites no supporting claim")
            }
            Self::ProposalClaimContested => {
                format!("proposal {proposal_id} cites a contested claim")
            }
        }
    }
}

/// When a synthesis packet qualifies a proposal for assertion.
///
/// A packet qualifies a proposal when it owns that proposal's scope and
/// target, when its suggested artifacts adjudicate that proposal, when nothing
/// in it blocks promotion (no blocking evidence gap, no pending human
/// decision), when it has not already asserted some other proposal, and when
/// the proposal rests on at least one supporting claim that the packet does
/// not contest.
///
/// Both readings of that decision call this function: the must
/// (`ensure_qualifying_assertions`, where a qualified proposal with no
/// assertion is a hole in the run) and the may (`validate_assertion_packet`,
/// where an assertion is accepted only against a packet that qualifies its
/// proposal). They differ only in which facts they establish themselves, which
/// is why the facts arrive as data rather than as a second hand-written
/// conjunction.
#[rule("rule_qualified_proposal_assertion")]
pub const fn qualify_proposal_for_assertion(
    facts: QualificationFacts,
) -> Result<(), UnmetQualification> {
    if !facts.packet_owns_proposal_target {
        return Err(UnmetQualification::PacketNotOwnedByProposalTarget);
    }
    if !facts.packet_adjudicates_proposal {
        return Err(UnmetQualification::PacketDoesNotAdjudicateProposal);
    }
    if facts.blocking_evidence_gap {
        return Err(UnmetQualification::BlockingEvidenceGap);
    }
    if facts.blocking_human_decision {
        return Err(UnmetQualification::BlockingHumanDecision);
    }
    if facts.packet_asserts_another_proposal {
        return Err(UnmetQualification::PacketAssertsAnotherProposal);
    }
    if !facts.proposal_has_supporting_claims {
        return Err(UnmetQualification::ProposalHasNoSupportingClaims);
    }
    if facts.proposal_claim_contested {
        return Err(UnmetQualification::ProposalClaimContested);
    }
    Ok(())
}

/// The must-assert reading, read off the records: does `packet` qualify
/// `proposal` for assertion given the assertions landed so far?
pub fn packet_qualifies_proposal(
    packet: &SynthesisPacket,
    proposal: &ProposalCard,
    assertions: &[AssertionRecord],
) -> bool {
    qualify_proposal_for_assertion(qualification_facts(packet, proposal, assertions)).is_ok()
}

fn qualification_facts(
    packet: &SynthesisPacket,
    proposal: &ProposalCard,
    assertions: &[AssertionRecord],
) -> QualificationFacts {
    let packet_has_assertion = assertions
        .iter()
        .any(|assertion| assertion.synthesis_packet_id == packet.id);
    let packet_asserts_proposal = assertions.iter().any(|assertion| {
        assertion.synthesis_packet_id == packet.id && assertion.proposal_id == proposal.id
    });
    QualificationFacts {
        packet_owns_proposal_target: packet_owns_proposal_target(packet, proposal),
        packet_adjudicates_proposal: packet_adjudicates_proposal(packet, proposal),
        blocking_evidence_gap: blocking_evidence_gap(packet),
        blocking_human_decision: blocking_human_decision(packet),
        packet_asserts_another_proposal: packet_has_assertion && !packet_asserts_proposal,
        proposal_has_supporting_claims: !proposal.traceability.supporting_claim_ids.is_empty(),
        proposal_claim_contested: packet.contested_claims.iter().any(|contested| {
            proposal
                .traceability
                .supporting_claim_ids
                .contains(&contested.claim_id)
        }),
    }
}

pub(super) fn packet_owns_proposal_target(
    packet: &SynthesisPacket,
    proposal: &ProposalCard,
) -> bool {
    packet.scope_id == proposal.scope_id && packet.target == proposal.traceability.target
}

pub(super) fn packet_adjudicates_proposal(
    packet: &SynthesisPacket,
    proposal: &ProposalCard,
) -> bool {
    packet.suggested_artifacts.iter().any(|suggestion| {
        suggestion.proposal_id.as_ref() == Some(&proposal.id)
            && suggestion.proposal_key == proposal.proposal_key
            && suggestion.proposal_type == proposal.proposal_type
    })
}

pub(super) fn blocking_evidence_gap(packet: &SynthesisPacket) -> bool {
    packet
        .evidence_gaps
        .iter()
        .any(|gap| gap.blocking_promotion)
}

pub(super) fn blocking_human_decision(packet: &SynthesisPacket) -> bool {
    packet
        .required_human_decisions
        .iter()
        .any(|decision| decision.blocks_promotion)
}

#[cfg(test)]
mod tests {
    use provenance_macros::verifies;

    use super::{qualify_proposal_for_assertion, QualificationFacts, UnmetQualification};

    /// One bit per fact. The struct literal in `all_fact_combinations` forces a
    /// new fact to be wired in here, and the assertion there catches a stale
    /// count, so the domain below stays the whole domain.
    const FACT_COUNT: u32 = 7;

    fn all_fact_combinations() -> Vec<QualificationFacts> {
        (0..1_u32 << FACT_COUNT)
            .map(|bits| {
                let mut index = 0;
                let mut next = || {
                    let value = bits >> index & 1 == 1;
                    index += 1;
                    value
                };
                let facts = QualificationFacts {
                    packet_owns_proposal_target: next(),
                    packet_adjudicates_proposal: next(),
                    blocking_evidence_gap: next(),
                    blocking_human_decision: next(),
                    packet_asserts_another_proposal: next(),
                    proposal_has_supporting_claims: next(),
                    proposal_claim_contested: next(),
                };
                assert_eq!(index, FACT_COUNT, "FACT_COUNT must cover every fact");
                facts
            })
            .collect()
    }

    // Independent restatement of the decision, grouped the way it is argued
    // rather than the way the rule function evaluates it: a packet qualifies a
    // proposal when the packet is about that proposal, when its adjudication
    // has settled, and when the proposal's claims stand. Must not be
    // implemented by calling the rule function.
    fn packet_is_about_proposal(facts: QualificationFacts) -> bool {
        facts.packet_owns_proposal_target
            && facts.packet_adjudicates_proposal
            && !facts.packet_asserts_another_proposal
    }

    fn adjudication_has_settled(facts: QualificationFacts) -> bool {
        !facts.blocking_evidence_gap && !facts.blocking_human_decision
    }

    fn proposal_claims_stand(facts: QualificationFacts) -> bool {
        facts.proposal_has_supporting_claims && !facts.proposal_claim_contested
    }

    fn qualifies_by_oracle(facts: QualificationFacts) -> bool {
        packet_is_about_proposal(facts)
            && adjudication_has_settled(facts)
            && proposal_claims_stand(facts)
    }

    fn fact_is_unmet(facts: QualificationFacts, unmet: UnmetQualification) -> bool {
        match unmet {
            UnmetQualification::PacketNotOwnedByProposalTarget => {
                !facts.packet_owns_proposal_target
            }
            UnmetQualification::PacketDoesNotAdjudicateProposal => {
                !facts.packet_adjudicates_proposal
            }
            UnmetQualification::BlockingEvidenceGap => facts.blocking_evidence_gap,
            UnmetQualification::BlockingHumanDecision => facts.blocking_human_decision,
            UnmetQualification::PacketAssertsAnotherProposal => {
                facts.packet_asserts_another_proposal
            }
            UnmetQualification::ProposalHasNoSupportingClaims => {
                !facts.proposal_has_supporting_claims
            }
            UnmetQualification::ProposalClaimContested => facts.proposal_claim_contested,
        }
    }

    #[test]
    #[verifies("rule_qualified_proposal_assertion", exhaustion)]
    fn qualification_matches_the_decision_on_every_fact_combination() {
        let combinations = all_fact_combinations();
        assert_eq!(combinations.len(), 1 << FACT_COUNT);
        for facts in combinations {
            assert_eq!(
                qualify_proposal_for_assertion(facts).is_ok(),
                qualifies_by_oracle(facts),
                "qualification disagrees with the decision on {facts:?}"
            );
        }
    }

    #[test]
    #[verifies("rule_qualified_proposal_assertion", exhaustion)]
    fn every_refusal_names_a_fact_that_is_actually_unmet() {
        for facts in all_fact_combinations() {
            if let Err(unmet) = qualify_proposal_for_assertion(facts) {
                assert!(
                    fact_is_unmet(facts, unmet),
                    "qualification blamed {unmet:?} on {facts:?}, which meets it"
                );
            }
        }
    }
}
