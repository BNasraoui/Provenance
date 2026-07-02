mod cli;
mod handlers;
mod output;

use anyhow::Context;
use clap::Parser;
use cli::{
    Cli, Command, ContributionsCommand, CoverageCommand, PromotionDecisionsCommand,
    ProposalsCommand, RequirementsCommand, ResolutionsCommand, RulesCommand, SourceRefCommand,
    SourcesCommand, SynthesisPacketsCommand, ThreadCommand,
};
use output::OutputFormat;
use provenance_core::{
    CanonicalArtifact, CanonicalArtifactType, ClaimChallenge, ConsensusFinding, ContestedClaim,
    ContributionStance, EvidenceGap, IdeationEvidenceReference, IdeationTarget, IdeationTargetType,
    IdentityType, MaterialClaim, MessageRole, MinorityObjection, NodeType, PromotionActor,
    PromotionDecision, PromotionState, ProposalTraceability, ProposalType, RequiredHumanDecision,
    RequirementStatus, ResolutionStatus, RuleModality, RuleSeverity, RuleStatus, RuleType, ScopeId,
    SourceType, StableId, SuggestedArtifact, SuggestedArtifactChange, ThreadParent,
    UncertaintyLevel, UncertaintyRating, UnsupportedRecommendation, UnsupportedSpeculation,
};
use provenance_store::{
    cache,
    layout::ProvenanceLayout,
    merge::{merge_records, read_jsonl_records, MergeOutcome},
    state_store::{
        AddSourceReferenceInput, CreateContributionInput, CreatePromotionDecisionInput,
        CreateProposalCardInput, CreateRequirementInput, CreateResolutionInput, CreateRuleInput,
        CreateSourceInput, CreateSynthesisPacketInput, PostMessageInput, StateStore,
    },
};
use serde::de::DeserializeOwned;

fn parse_json_arg<T: DeserializeOwned>(flag: &str, value: &str) -> anyhow::Result<T> {
    serde_json::from_str(value).with_context(|| format!("--{flag} must be valid JSON"))
}

fn stable_ids(values: Vec<String>) -> anyhow::Result<Vec<StableId>> {
    values.into_iter().map(StableId::new).collect()
}

fn ideation_target(target_type: &str, target_id: String) -> anyhow::Result<IdeationTarget> {
    Ok(IdeationTarget {
        artifact_type: IdeationTargetType::parse(target_type)?,
        artifact_id: StableId::new(target_id)?,
    })
}

fn canonical_artifact(
    artifact_type: Option<String>,
    artifact_id: Option<String>,
) -> anyhow::Result<Option<CanonicalArtifact>> {
    match (artifact_type, artifact_id) {
        (Some(artifact_type), Some(artifact_id)) => Ok(Some(CanonicalArtifact {
            artifact_type: CanonicalArtifactType::parse(&artifact_type)?,
            artifact_id: StableId::new(artifact_id)?,
        })),
        (None, None) => Ok(None),
        _ => anyhow::bail!(
            "--canonical-artifact-type and --canonical-artifact-id must be provided together"
        ),
    }
}

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Init {
            path,
            scope,
            path_prefix,
        } => handlers::init(path, scope, path_prefix)?,
        Command::Check { repo, format } => handlers::check(repo, format)?,
        Command::Materialize { repo, format } => {
            let report = cache::materialize_state(&ProvenanceLayout::new(repo)).await?;
            output::print(format, &report)?;
        }
        Command::Sources { command } => match command {
            SourcesCommand::Create {
                repo,
                scope,
                id,
                name,
                source_type,
                url,
                reference,
                origin_thread,
                origin_message,
                format,
            } => {
                let source = StateStore::new(ProvenanceLayout::new(repo)).create_source(
                    CreateSourceInput {
                        scope_id: ScopeId::new(scope)?,
                        id: StableId::new(id)?,
                        name,
                        source_type: SourceType::parse(&source_type)?,
                        url,
                        reference,
                        origin_thread: origin_thread.map(StableId::new).transpose()?,
                        origin_message: origin_message.map(StableId::new).transpose()?,
                    },
                )?;
                output::print(format, &source)?;
            }
        },
        Command::Requirements { command } => {
            match command {
                RequirementsCommand::Create {
                    repo,
                    scope,
                    id,
                    statement,
                    description,
                    status,
                    origin_thread,
                    origin_message,
                    format,
                } => {
                    let requirement = StateStore::new(ProvenanceLayout::new(repo))
                        .create_requirement(CreateRequirementInput {
                            scope_id: ScopeId::new(scope)?,
                            id: StableId::new(id)?,
                            statement,
                            description,
                            status: RequirementStatus::parse(&status)?,
                            origin_thread: origin_thread.map(StableId::new).transpose()?,
                            origin_message: origin_message.map(StableId::new).transpose()?,
                        })?;
                    output::print(format, &requirement)?;
                }
                RequirementsCommand::SourceRef { command } => match command {
                    SourceRefCommand::Add {
                        repo,
                        scope,
                        requirement_id,
                        source_id,
                        clause,
                        format,
                    } => {
                        let edge = StateStore::new(ProvenanceLayout::new(repo))
                            .add_source_reference(AddSourceReferenceInput {
                                scope_id: ScopeId::new(scope)?,
                                source_id: StableId::new(source_id)?,
                                requirement_id: StableId::new(requirement_id)?,
                                clause,
                            })?;
                        output::print(format, &edge)?;
                    }
                },
            }
        }
        Command::Graph {
            requirement_id,
            repo,
            scope,
            format,
        } => {
            let graph = cache::get_requirement_graph(
                &ProvenanceLayout::new(repo),
                &ScopeId::new(scope)?,
                &StableId::new(requirement_id)?,
            )?;
            output::print(format, &graph)?;
        }
        Command::Resolutions { command } => match command {
            ResolutionsCommand::Create {
                repo,
                scope,
                id,
                title,
                requirement_id,
                position,
                rationale,
                status,
                context,
                enforcement,
                confidence,
                origin_thread,
                origin_message,
                format,
            } => {
                let resolution = StateStore::new(ProvenanceLayout::new(repo)).create_resolution(
                    CreateResolutionInput {
                        scope_id: ScopeId::new(scope)?,
                        id: StableId::new(id)?,
                        title,
                        requirement_id: requirement_id.map(StableId::new).transpose()?,
                        position,
                        rationale,
                        status: ResolutionStatus::parse(&status)?,
                        context,
                        enforcement,
                        confidence,
                        origin_thread: origin_thread.map(StableId::new).transpose()?,
                        origin_message: origin_message.map(StableId::new).transpose()?,
                    },
                )?;
                output::print(format, &resolution)?;
            }
        },
        Command::Rules { command } => match command {
            RulesCommand::Create {
                repo,
                scope,
                id,
                rule_code,
                name,
                description,
                requirement_id,
                resolution_id,
                statement,
                status,
                severity,
                rule_type,
                modality,
                confidence,
                extraction_method,
                source_document,
                source_section,
                origin_thread,
                origin_message,
                format,
            } => {
                let rule =
                    StateStore::new(ProvenanceLayout::new(repo)).create_rule(CreateRuleInput {
                        scope_id: ScopeId::new(scope)?,
                        id: StableId::new(id)?,
                        rule_code,
                        name,
                        description,
                        requirement_id: requirement_id.map(StableId::new).transpose()?,
                        resolution_id: resolution_id.map(StableId::new).transpose()?,
                        statement,
                        status: RuleStatus::parse(&status)?,
                        severity: RuleSeverity::parse(&severity)?,
                        rule_type: rule_type.map(|value| RuleType::parse(&value)).transpose()?,
                        modality: modality
                            .map(|value| RuleModality::parse(&value))
                            .transpose()?,
                        confidence,
                        extraction_method,
                        source_document,
                        source_section,
                        origin_thread: origin_thread.map(StableId::new).transpose()?,
                        origin_message: origin_message.map(StableId::new).transpose()?,
                    })?;
                output::print(format, &rule)?;
            }
        },
        Command::Traceability {
            rule_id,
            repo,
            scope,
            format,
        } => {
            let trace = cache::trace_rule(
                &ProvenanceLayout::new(repo),
                &ScopeId::new(scope)?,
                &StableId::new(rule_id)?,
            )?;
            output::print(format, &trace)?;
        }
        Command::Gaps {
            repo,
            scope,
            format,
        } => {
            let gaps = cache::find_gaps(&ProvenanceLayout::new(repo), &ScopeId::new(scope)?)?;
            output::print(format, &gaps)?;
        }
        Command::Thread { command } => match command {
            ThreadCommand::Post {
                repo,
                scope,
                parent_type,
                parent_id,
                role,
                body,
                format,
            } => {
                let result = StateStore::new(ProvenanceLayout::new(repo)).post_thread_message(
                    PostMessageInput {
                        scope_id: ScopeId::new(scope)?,
                        parent: ThreadParent {
                            node_type: NodeType::parse(&parent_type)?,
                            node_id: StableId::new(parent_id)?,
                        },
                        role: MessageRole::parse(&role)?,
                        body,
                    },
                )?;
                output::print(format, &result)?;
            }
            ThreadCommand::List {
                repo,
                scope,
                format,
            } => {
                let threads = StateStore::new(ProvenanceLayout::new(repo))
                    .list_threads(&ScopeId::new(scope)?)?;
                output::print(format, &threads)?;
            }
        },
        Command::Contributions { command } => match command {
            ContributionsCommand::Create {
                repo,
                scope,
                id,
                target_type,
                target_id,
                participant_slot,
                stance,
                strongest_finding,
                evidence_json,
                claims_json,
                risks_json,
                objections_json,
                challenges_json,
                suggested_changes_json,
                unsupported_recommendations_json,
                uncertainty_level,
                uncertainty_rationale,
                open_questions_json,
                format,
            } => {
                let contribution = StateStore::new(ProvenanceLayout::new(repo))
                    .create_contribution(CreateContributionInput {
                        scope_id: ScopeId::new(scope)?,
                        id: StableId::new(id)?,
                        target: ideation_target(&target_type, target_id)?,
                        participant_slot,
                        stance: ContributionStance::parse(&stance)?,
                        strongest_finding,
                        evidence_references: parse_json_arg::<Vec<IdeationEvidenceReference>>(
                            "evidence-json",
                            &evidence_json,
                        )?,
                        material_claims: parse_json_arg::<Vec<MaterialClaim>>(
                            "claims-json",
                            &claims_json,
                        )?,
                        risks: parse_json_arg::<Vec<String>>("risks-json", &risks_json)?,
                        objections: parse_json_arg::<Vec<String>>(
                            "objections-json",
                            &objections_json,
                        )?,
                        challenges: parse_json_arg::<Vec<ClaimChallenge>>(
                            "challenges-json",
                            &challenges_json,
                        )?,
                        suggested_artifact_changes: parse_json_arg::<Vec<SuggestedArtifactChange>>(
                            "suggested-changes-json",
                            &suggested_changes_json,
                        )?,
                        unsupported_recommendations: parse_json_arg::<
                            Vec<UnsupportedRecommendation>,
                        >(
                            "unsupported-recommendations-json",
                            &unsupported_recommendations_json,
                        )?,
                        uncertainty: UncertaintyRating {
                            level: UncertaintyLevel::parse(&uncertainty_level)?,
                            rationale: uncertainty_rationale,
                        },
                        open_questions: parse_json_arg::<Vec<String>>(
                            "open-questions-json",
                            &open_questions_json,
                        )?,
                    })?;
                output::print(format, &contribution)?;
            }
            ContributionsCommand::List {
                repo,
                scope,
                format,
            } => {
                let contributions = StateStore::new(ProvenanceLayout::new(repo))
                    .list_contributions(&ScopeId::new(scope)?)?;
                output::print(format, &contributions)?;
            }
        },
        Command::SynthesisPackets { command } => match command {
            SynthesisPacketsCommand::Create {
                repo,
                scope,
                id,
                target_type,
                target_id,
                summary,
                consensus_json,
                contested_claims_json,
                minority_objections_json,
                evidence_gaps_json,
                unsupported_speculation_json,
                open_questions_json,
                suggested_artifacts_json,
                required_human_decisions_json,
                format,
            } => {
                let synthesis_packet = StateStore::new(ProvenanceLayout::new(repo))
                    .create_synthesis_packet(CreateSynthesisPacketInput {
                        scope_id: ScopeId::new(scope)?,
                        id: StableId::new(id)?,
                        target: ideation_target(&target_type, target_id)?,
                        summary,
                        consensus: parse_json_arg::<Vec<ConsensusFinding>>(
                            "consensus-json",
                            &consensus_json,
                        )?,
                        contested_claims: parse_json_arg::<Vec<ContestedClaim>>(
                            "contested-claims-json",
                            &contested_claims_json,
                        )?,
                        minority_objections: parse_json_arg::<Vec<MinorityObjection>>(
                            "minority-objections-json",
                            &minority_objections_json,
                        )?,
                        evidence_gaps: parse_json_arg::<Vec<EvidenceGap>>(
                            "evidence-gaps-json",
                            &evidence_gaps_json,
                        )?,
                        unsupported_speculation: parse_json_arg::<Vec<UnsupportedSpeculation>>(
                            "unsupported-speculation-json",
                            &unsupported_speculation_json,
                        )?,
                        open_questions: parse_json_arg::<Vec<String>>(
                            "open-questions-json",
                            &open_questions_json,
                        )?,
                        suggested_artifacts: parse_json_arg::<Vec<SuggestedArtifact>>(
                            "suggested-artifacts-json",
                            &suggested_artifacts_json,
                        )?,
                        required_human_decisions: parse_json_arg::<Vec<RequiredHumanDecision>>(
                            "required-human-decisions-json",
                            &required_human_decisions_json,
                        )?,
                    })?;
                output::print(format, &synthesis_packet)?;
            }
            SynthesisPacketsCommand::List {
                repo,
                scope,
                format,
            } => {
                let synthesis_packets = StateStore::new(ProvenanceLayout::new(repo))
                    .list_synthesis_packets(&ScopeId::new(scope)?)?;
                output::print(format, &synthesis_packets)?;
            }
        },
        Command::Proposals { command } => match command {
            ProposalsCommand::Create {
                repo,
                scope,
                id,
                proposal_key,
                proposal_type,
                title,
                summary,
                target_type,
                target_id,
                source_id,
                evidence_json,
                supporting_claim_id,
                promotion_state,
                duplicate_of,
                superseded_by,
                format,
            } => {
                let proposal = StateStore::new(ProvenanceLayout::new(repo)).create_proposal_card(
                    CreateProposalCardInput {
                        scope_id: ScopeId::new(scope)?,
                        id: StableId::new(id)?,
                        proposal_key,
                        proposal_type: ProposalType::parse(&proposal_type)?,
                        title,
                        summary,
                        traceability: ProposalTraceability {
                            target: ideation_target(&target_type, target_id)?,
                            source_ids: stable_ids(source_id)?,
                            evidence_references: parse_json_arg::<Vec<IdeationEvidenceReference>>(
                                "evidence-json",
                                &evidence_json,
                            )?,
                            supporting_claim_ids: stable_ids(supporting_claim_id)?,
                        },
                        promotion_state: PromotionState::parse(&promotion_state)?,
                        duplicate_of: duplicate_of.map(StableId::new).transpose()?,
                        superseded_by: superseded_by.map(StableId::new).transpose()?,
                    },
                )?;
                output::print(format, &proposal)?;
            }
            ProposalsCommand::List {
                repo,
                scope,
                format,
            } => {
                let proposals = StateStore::new(ProvenanceLayout::new(repo))
                    .list_proposal_cards(&ScopeId::new(scope)?)?;
                output::print(format, &proposals)?;
            }
        },
        Command::PromotionDecisions { command } => match command {
            PromotionDecisionsCommand::Create {
                repo,
                scope,
                id,
                proposal_id,
                decision,
                rationale,
                actor_id,
                actor_type,
                actor_name,
                canonical_artifact_type,
                canonical_artifact_id,
                format,
            } => {
                let promotion_decision = StateStore::new(ProvenanceLayout::new(repo))
                    .create_promotion_decision(CreatePromotionDecisionInput {
                        scope_id: ScopeId::new(scope)?,
                        id: StableId::new(id)?,
                        proposal_id: StableId::new(proposal_id)?,
                        decision: PromotionDecision::parse(&decision)?,
                        rationale,
                        actor: PromotionActor {
                            identity_type: IdentityType::parse(&actor_type)?,
                            id: actor_id,
                            name: actor_name,
                        },
                        canonical_artifact: canonical_artifact(
                            canonical_artifact_type,
                            canonical_artifact_id,
                        )?,
                    })?;
                output::print(format, &promotion_decision)?;
            }
            PromotionDecisionsCommand::List {
                repo,
                scope,
                format,
            } => {
                let decisions = StateStore::new(ProvenanceLayout::new(repo))
                    .list_promotion_decisions(&ScopeId::new(scope)?)?;
                output::print(format, &decisions)?;
            }
        },
        Command::Prime {
            repo,
            scope,
            format,
            include_threads,
        } => {
            let view = cache::prime_context(
                &ProvenanceLayout::new(repo),
                &ScopeId::new(scope)?,
                include_threads,
            )?;
            if matches!(format, OutputFormat::Markdown | OutputFormat::Toon) {
                println!("{}", cache::render_prime_markdown(&view));
            } else {
                output::print(format, &view)?;
            }
        }
        Command::Impact {
            id,
            repo,
            scope,
            node_type,
            max_hops,
            follow_indirect,
            format,
        } => {
            let view = cache::analyze_impact(
                &ProvenanceLayout::new(repo),
                &ScopeId::new(scope)?,
                NodeType::parse(&node_type)?,
                &StableId::new(id)?,
                cache::ImpactOptions {
                    max_hops,
                    follow_indirect,
                },
            )?;
            output::print(format, &view)?;
        }
        Command::Stale {
            repo,
            scope,
            min_age_days: _,
            rule_severities: _,
            min_downstream_rules: _,
            format,
        } => {
            let stale = cache::find_stale(&ProvenanceLayout::new(repo), &ScopeId::new(scope)?)?;
            output::print(format, &stale)?;
        }
        Command::Health {
            repo,
            scope,
            format,
        } => {
            let health =
                cache::coverage_health(&ProvenanceLayout::new(repo), &ScopeId::new(scope)?)?;
            output::print(format, &health)?;
        }
        Command::Orphans {
            repo,
            scope,
            format,
        } => {
            let orphans = cache::orphan_rules(&ProvenanceLayout::new(repo), &ScopeId::new(scope)?)?;
            output::print(format, &orphans)?;
        }
        Command::Coverage { command } => match command {
            CoverageCommand::Scan {
                repo,
                path,
                scope,
                validate_rules,
                format,
                output,
            } => {
                let report = handlers::coverage_scan(repo, &path, scope, validate_rules)?;
                if let Some(output_path) = output {
                    let rendered = handlers::render_coverage(format, &report)?;
                    std::fs::write(output_path, rendered)?;
                } else if matches!(format, OutputFormat::Markdown | OutputFormat::Toon) {
                    print!("{}", handlers::render_coverage(format, &report)?);
                } else {
                    output::print(format, &report)?;
                }
            }
        },
        Command::Export {
            repo,
            scope,
            format,
            output,
        } => {
            let exported = handlers::export_scope(repo, scope)?;
            let rendered = handlers::render_export(format, &exported)?;
            if let Some(output_path) = output {
                std::fs::write(output_path, rendered)?;
            } else {
                print!("{rendered}");
            }
        }
        Command::Import {
            repo,
            scope,
            input,
            dry_run,
            format,
        } => {
            let report = handlers::import_scope(repo, scope, input, dry_run)?;
            output::print(format, &report)?;
        }
        Command::MergeJsonl {
            base,
            ours,
            theirs,
            output,
            format,
        } => {
            let outcome = merge_records(
                &read_jsonl_records(&base)?,
                &read_jsonl_records(&ours)?,
                &read_jsonl_records(&theirs)?,
            )?;
            if let Some(output_path) = output {
                let records = match &outcome {
                    MergeOutcome::Clean { records } => records,
                    MergeOutcome::Conflicted { partial, .. } => partial,
                };
                provenance_store::jsonl::write_jsonl_atomic(&output_path, records)?;
            }
            output::print(format, &outcome)?;
        }
    }
    Ok(())
}
