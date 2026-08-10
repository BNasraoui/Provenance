use crate::wiki::model::{CodeScan, RuleFunction, VerificationSite};
use provenance_core::coverage::BindingResult;

use super::context::Assembler;

impl Assembler<'_> {
    /// The scan this build read, so a page can say which code it looked at
    /// and a page built without a scan can say that instead.
    pub(super) fn code_scan(&self) -> Option<CodeScan> {
        self.coverage.map(|report| CodeScan {
            commit: report.commit.clone(),
        })
    }

    pub(super) fn rule_function(&self, rule_id: &str) -> Option<RuleFunction> {
        let binding = self
            .coverage?
            .bindings
            .iter()
            .find(|binding| binding.rule_id == rule_id && binding.verification.is_none())?;
        Some(RuleFunction {
            symbol: binding.item_name.clone(),
            location: self.binding_location(binding),
        })
    }

    pub(super) fn verification_sites(&self, rule_id: &str) -> Vec<VerificationSite> {
        let Some(report) = self.coverage else {
            return Vec::new();
        };
        let defining_file = report
            .bindings
            .iter()
            .find(|binding| binding.rule_id == rule_id && binding.verification.is_none())
            .map(|binding| &binding.file_path);
        report
            .bindings
            .iter()
            .filter(|binding| binding.rule_id == rule_id)
            .filter_map(|binding| {
                binding
                    .verification
                    .as_ref()
                    .map(|method| VerificationSite {
                        method: method.clone(),
                        symbol: binding.item_name.clone(),
                        location: self.binding_location(binding),
                        outside_defining_module: defining_file
                            .is_some_and(|file| file != &binding.file_path),
                    })
            })
            .collect()
    }

    fn binding_location(&self, binding: &BindingResult) -> crate::wiki::links::EvidenceRef {
        let reference = format!("{}:{}", binding.file_path, binding.line);
        self.resolver.resolve_at(
            &reference,
            self.coverage.and_then(|report| report.commit.as_deref()),
        )
    }
}
