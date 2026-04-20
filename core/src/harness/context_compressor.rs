// Compresses conversation history to fit in context window efficiently

pub fn compress(memory: &str) -> String {
    if memory.is_empty() {
        return "## MEMORY\nNo previous context.".to_string();
    }
    // Truncate to last 2000 chars to stay within context budget
    let truncated = if memory.len() > 2000 {
        &memory[memory.len()-2000..]
    } else {
        memory
    };
    format!("## CONVERSATION MEMORY\n{}", truncated)
}
