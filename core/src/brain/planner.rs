// microdragon-core/src/brain/planner.rs
// Task Planner - Creates structured execution plans for complex tasks

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

use super::intent::{IntentType, ParsedIntent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: String,
    pub title: String,
    pub steps: Vec<PlanStep>,
    pub estimated_duration_secs: u64,
    pub requires_confirmation: bool,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub order: u32,
    pub title: String,
    pub description: String,
    pub tool: Option<String>,
    pub is_critical: bool,
    pub can_parallelize: bool,
    pub depends_on: Vec<String>,
    pub status: StepStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

impl ExecutionPlan {
    pub fn to_prompt(&self) -> String {
        let mut prompt = format!("**Execution Plan: {}**\n\n", self.title);
        for (i, step) in self.steps.iter().enumerate() {
            prompt.push_str(&format!(
                "Step {}: {} {}\n  → {}\n",
                i + 1,
                step.title,
                if step.is_critical { "⚠️" } else { "" },
                step.description
            ));
        }
        prompt
    }

    pub fn pending_steps(&self) -> Vec<&PlanStep> {
        self.steps.iter().filter(|s| s.status == StepStatus::Pending).collect()
    }

    pub fn completed_steps(&self) -> usize {
        self.steps.iter().filter(|s| s.status == StepStatus::Completed).count()
    }

    pub fn progress_pct(&self) -> f64 {
        if self.steps.is_empty() { return 100.0; }
        (self.completed_steps() as f64 / self.steps.len() as f64) * 100.0
    }
}

pub struct TaskPlanner;

impl TaskPlanner {
    pub fn new() -> Self {
        Self
    }

    pub async fn create_plan(&self, intent: &ParsedIntent, input: &str) -> Result<ExecutionPlan> {
        let steps = match intent.intent_type {
            IntentType::Code => self.plan_code_task(input),
            IntentType::Research => self.plan_research_task(input),
            IntentType::Automate => self.plan_automation_task(input),
            IntentType::Business => self.plan_business_task(input),
            _ => self.plan_generic_task(input),
        };

        Ok(ExecutionPlan {
            id: Uuid::new_v4().to_string(),
            title: self.extract_title(input),
            steps,
            estimated_duration_secs: 30,
            requires_confirmation: false,
            created_at: Utc::now(),
        })
    }

    fn plan_code_task(&self, input: &str) -> Vec<PlanStep> {
        vec![
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 1,
                title: "Analyze Requirements".to_string(),
                description: "Understand the coding task and identify requirements".to_string(),
                tool: None,
                is_critical: true,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 2,
                title: "Design Solution".to_string(),
                description: "Plan architecture and approach".to_string(),
                tool: None,
                is_critical: true,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 3,
                title: "Generate Code".to_string(),
                description: "Write the implementation code".to_string(),
                tool: Some("code_generator".to_string()),
                is_critical: true,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 4,
                title: "Write Tests".to_string(),
                description: "Generate unit and integration tests".to_string(),
                tool: Some("test_generator".to_string()),
                is_critical: false,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 5,
                title: "Review & Optimize".to_string(),
                description: "Review code quality and suggest optimizations".to_string(),
                tool: None,
                is_critical: false,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
        ]
    }

    fn plan_research_task(&self, _input: &str) -> Vec<PlanStep> {
        vec![
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 1,
                title: "Query Formulation".to_string(),
                description: "Formulate search queries".to_string(),
                tool: None,
                is_critical: true,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 2,
                title: "Web Research".to_string(),
                description: "Search the web for relevant information".to_string(),
                tool: Some("web_crawler".to_string()),
                is_critical: true,
                can_parallelize: true,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 3,
                title: "Extract & Filter".to_string(),
                description: "Extract relevant information from sources".to_string(),
                tool: Some("extractor".to_string()),
                is_critical: true,
                can_parallelize: true,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 4,
                title: "Synthesize".to_string(),
                description: "Synthesize findings into coherent summary".to_string(),
                tool: Some("summarizer".to_string()),
                is_critical: true,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
        ]
    }

    fn plan_automation_task(&self, _input: &str) -> Vec<PlanStep> {
        vec![
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 1,
                title: "Environment Check".to_string(),
                description: "Verify required tools are available".to_string(),
                tool: Some("system_check".to_string()),
                is_critical: true,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 2,
                title: "Build Automation Script".to_string(),
                description: "Create the automation workflow".to_string(),
                tool: Some("playwright".to_string()),
                is_critical: true,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 3,
                title: "Execute Automation".to_string(),
                description: "Run the automation workflow".to_string(),
                tool: Some("executor".to_string()),
                is_critical: true,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 4,
                title: "Verify Results".to_string(),
                description: "Confirm automation completed successfully".to_string(),
                tool: None,
                is_critical: false,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
        ]
    }

    fn plan_business_task(&self, _input: &str) -> Vec<PlanStep> {
        vec![
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 1,
                title: "Fetch Market Data".to_string(),
                description: "Retrieve current market data".to_string(),
                tool: Some("market_data".to_string()),
                is_critical: true,
                can_parallelize: true,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 2,
                title: "Technical Analysis".to_string(),
                description: "Perform technical indicator analysis".to_string(),
                tool: Some("analyzer".to_string()),
                is_critical: true,
                can_parallelize: true,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 3,
                title: "Risk Assessment".to_string(),
                description: "Evaluate risk parameters".to_string(),
                tool: None,
                is_critical: true,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 4,
                title: "Generate Report".to_string(),
                description: "Synthesize analysis into actionable report".to_string(),
                tool: Some("reporter".to_string()),
                is_critical: false,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
        ]
    }

    fn plan_generic_task(&self, input: &str) -> Vec<PlanStep> {
        vec![
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 1,
                title: "Understand Task".to_string(),
                description: "Parse and understand requirements".to_string(),
                tool: None,
                is_critical: true,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 2,
                title: "Execute".to_string(),
                description: format!("Execute: {}", &input[..input.len().min(50)]),
                tool: None,
                is_critical: true,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
            PlanStep {
                id: Uuid::new_v4().to_string(),
                order: 3,
                title: "Validate Output".to_string(),
                description: "Verify and format output".to_string(),
                tool: None,
                is_critical: false,
                can_parallelize: false,
                depends_on: vec![],
                status: StepStatus::Pending,
            },
        ]
    }

    fn extract_title(&self, input: &str) -> String {
        let words: Vec<&str> = input.split_whitespace().take(8).collect();
        let title = words.join(" ");
        if title.len() < input.len() {
            format!("{}...", title)
        } else {
            title
        }
    }
}
