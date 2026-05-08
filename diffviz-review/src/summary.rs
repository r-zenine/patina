use serde::Serialize;

use crate::entities::decision::CodeImpact;

#[derive(Debug, Serialize)]
pub struct ApprovedDecisionEntry {
    pub number: u32,
    pub title: String,
    pub approved_by: String,
    pub approval_timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct UnapprovedDecisionEntry {
    pub number: u32,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    pub code_impacts: Vec<CodeImpact>,
}

#[derive(Debug, Serialize)]
pub struct InstructionEntry {
    pub file: String,
    pub lines: String,
    pub content: String,
    pub author: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct ReviewSummaryStats {
    pub total_decisions: usize,
    pub approved_decisions: usize,
    pub unapproved_decisions: usize,
    pub total_instructions: usize,
    pub active_instructions: usize,
}

#[derive(Debug, Serialize)]
pub struct ReviewSummaryDecisions {
    pub approved: Vec<ApprovedDecisionEntry>,
    pub unapproved: Vec<UnapprovedDecisionEntry>,
}

#[derive(Debug, Serialize)]
pub struct ReviewSummaryInstructions {
    pub active: Vec<InstructionEntry>,
    pub addressed: Vec<InstructionEntry>,
}

#[derive(Debug, Serialize)]
pub struct ReviewSummary {
    pub commit: String,
    pub contribution_folder: String,
    pub decisions: ReviewSummaryDecisions,
    pub instructions: ReviewSummaryInstructions,
    pub summary: ReviewSummaryStats,
}

#[derive(Serialize)]
struct MinimalDecisionEntry<'a> {
    number: u32,
    title: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    rationale: &'a Option<String>,
    code_impacts: &'a Vec<CodeImpact>,
}

#[derive(Serialize)]
struct MinimalOutput<'a> {
    commit: &'a str,
    unapproved_decisions: Vec<MinimalDecisionEntry<'a>>,
    instructions: &'a ReviewSummaryInstructions,
}

impl ReviewSummary {
    pub fn to_yaml_minimal(&self) -> Result<String, serde_yaml::Error> {
        let output = MinimalOutput {
            commit: &self.commit,
            unapproved_decisions: self
                .decisions
                .unapproved
                .iter()
                .map(|d| MinimalDecisionEntry {
                    number: d.number,
                    title: &d.title,
                    rationale: &d.rationale,
                    code_impacts: &d.code_impacts,
                })
                .collect(),
            instructions: &self.instructions,
        };
        serde_yaml::to_string(&output)
    }

    pub fn to_yaml_full(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }
}
