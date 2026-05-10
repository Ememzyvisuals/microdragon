// Applies the correct output structure based on task type

pub fn format_for_task(task: &str) -> String {
    let t = task.to_lowercase();
    if t.contains("code") || t.contains("function") || t.contains("script") || t.contains("implement") {
        return CODE_FORMAT.to_string();
    }
    if t.contains("security") || t.contains("audit") || t.contains("vulnerability") {
        return SECURITY_FORMAT.to_string();
    }
    if t.contains("plan") || t.contains("strategy") || t.contains("roadmap") {
        return PLAN_FORMAT.to_string();
    }
    DEFAULT_FORMAT.to_string()
}

pub fn apply(response: &str, _task: &str) -> String {
    // Strip common model filler phrases
    let cleaned = response
        .replace("Certainly! ", "")
        .replace("Of course! ", "")
        .replace("Great question! ", "")
        .replace("As an AI language model, ", "")
        .replace("I'd be happy to help with that. ", "");
    cleaned.trim().to_string()
}

const CODE_FORMAT: &str = r#"## OUTPUT FORMAT — CODE TASK
Structure your response:
1. Brief explanation (2-3 sentences max)
2. Complete, working code in a code block
3. One sentence on how to run/test it
Do NOT include placeholder comments like "// TODO" or "// Add your logic here".
Code must be complete and immediately runnable."#;

const SECURITY_FORMAT: &str = r#"## OUTPUT FORMAT — SECURITY TASK
Structure each finding:
SEVERITY: [CRITICAL/HIGH/MEDIUM/LOW]
FILE: path/to/file.py:line_number
CODE: the vulnerable line
DESCRIPTION: what the vulnerability is
CVSS: score (0.0-10.0)
FIX: exact remediation code or steps"#;

const PLAN_FORMAT: &str = r#"## OUTPUT FORMAT — PLANNING TASK
Structure your plan:
GOAL: restate the goal in one sentence
STEPS: numbered, specific, actionable
TIMELINE: realistic estimate per step
RISK: top 1-2 risks and mitigation
NEXT: the single most important first action"#;

const DEFAULT_FORMAT: &str = r#"## OUTPUT FORMAT
Be direct. Lead with the answer. Follow with reasoning.
Use numbered lists for steps. Use code blocks for code.
End with a concrete next action."#;
