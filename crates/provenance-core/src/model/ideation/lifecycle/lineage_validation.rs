use super::AssertionRecord;
use crate::model::ProposalCard;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate(
    proposals: &[ProposalCard],
    assertion_records: &[AssertionRecord],
) -> anyhow::Result<()> {
    let assertions = assertion_records
        .iter()
        .map(|assertion| (assertion.id.as_str(), assertion.proposal_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let edges = proposals
        .iter()
        .map(|proposal| {
            let ancestors = proposal
                .builds_on
                .iter()
                .map(|id| {
                    assertions.get(id.as_str()).copied().ok_or_else(|| {
                        anyhow::anyhow!("builds_on assertion {} does not exist", id.as_str())
                    })
                })
                .collect::<anyhow::Result<Vec<_>>>()?;
            Ok((proposal.id.as_str(), ancestors))
        })
        .collect::<anyhow::Result<BTreeMap<_, _>>>()?;
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for proposal in edges.keys() {
        visit(proposal, &edges, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn visit<'a>(
    proposal: &'a str,
    edges: &BTreeMap<&'a str, Vec<&'a str>>,
    visiting: &mut BTreeSet<&'a str>,
    visited: &mut BTreeSet<&'a str>,
) -> anyhow::Result<()> {
    if visited.contains(proposal) {
        return Ok(());
    }
    anyhow::ensure!(
        visiting.insert(proposal),
        "proposal assertion lineage contains a cycle at {proposal}"
    );
    for ancestor in edges.get(proposal).into_iter().flatten() {
        visit(ancestor, edges, visiting, visited)?;
    }
    visiting.remove(proposal);
    visited.insert(proposal);
    Ok(())
}
