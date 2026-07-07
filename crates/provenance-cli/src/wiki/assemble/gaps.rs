use crate::wiki::model::GapNotice;
use provenance_core::{NodeType, StableId};
use provenance_store::cache::{node_type_word, GapItem, GapKind};

use super::context::Assembler;

impl Assembler<'_> {
    pub(super) fn gap_notice(gap: &GapItem) -> GapNotice {
        GapNotice {
            kind: gap.kind,
            detail: format!("{}: {}", gap.subject(), gap.reason),
        }
    }

    pub(super) fn mirrored_contradiction_notice(
        gap: &GapItem,
        node_type: NodeType,
        node_id: &str,
    ) -> GapNotice {
        GapNotice {
            kind: gap.kind,
            detail: format!(
                "{} {} -> {} {}: {}",
                node_type_word(node_type),
                node_id,
                node_type_word(gap.node_type),
                gap.node_id,
                gap.reason
            ),
        }
    }

    pub(super) fn gaps_for(&self, node_type: NodeType, node_id: &StableId) -> Vec<GapNotice> {
        self.gaps
            .iter()
            .filter_map(|gap| {
                if gap.node_type == node_type && gap.node_id == node_id.as_str() {
                    Some(Self::gap_notice(gap))
                } else if gap.kind == GapKind::UnresolvedContradictsPair
                    && gap.related_node_type == Some(node_type)
                    && gap.related_node_id.as_deref() == Some(node_id.as_str())
                {
                    Some(Self::mirrored_contradiction_notice(
                        gap,
                        node_type,
                        node_id.as_str(),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    pub(super) fn index_gaps(&self) -> Vec<GapNotice> {
        self.gaps
            .iter()
            .filter(|gap| {
                !matches!(
                    gap.kind,
                    GapKind::OrphanRule
                        | GapKind::OrphanResolution
                        | GapKind::UnreferencedSource
                        | GapKind::OpenQuestion
                        | GapKind::UnexploredTopic
                )
            })
            .map(Self::gap_notice)
            .collect()
    }
}
