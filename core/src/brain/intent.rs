// microdragon-core/src/brain/intent.rs
// Intent parsing system

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedIntent {
    pub intent_type: IntentType,
    pub confidence: f64,
    pub entities: Vec<Entity>,
    pub requires_tools: Vec<String>,
    pub priority: Priority,
    pub raw_input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IntentType {
    Code,       // write/debug/review code
    Research,   // web search, summarize
    File,       // file operations
    System,     // system commands
    Social,     // messaging, emails
    Business,   // trading, analysis
    Creative,   // writing, design
    Task,       // task management
    Config,     // configure microdragon
    Help,       // help/documentation
    Converse,   // general conversation
    Automate,   // automation tasks
}

impl std::fmt::Display for IntentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            IntentType::Code => "code",
            IntentType::Research => "research",
            IntentType::File => "file",
            IntentType::System => "system",
            IntentType::Social => "social",
            IntentType::Business => "business",
            IntentType::Creative => "creative",
            IntentType::Task => "task",
            IntentType::Config => "config",
            IntentType::Help => "help",
            IntentType::Converse => "converse",
            IntentType::Automate => "automate",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_type: EntityType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Language,
    FilePath,
    Url,
    Contact,
    Date,
    Number,
    Keyword,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
}

impl ParsedIntent {
    pub fn requires_planning(&self) -> bool {
        matches!(
            self.intent_type,
            IntentType::Code | IntentType::Research | IntentType::Business | IntentType::Automate
        ) || self.requires_tools.len() > 1
    }
}

pub struct IntentParser;

impl IntentParser {
    pub fn new() -> Self {
        Self
    }

    pub async fn parse(&self, input: &str) -> Result<ParsedIntent> {
        let input_lower = input.to_lowercase();
        let intent_type = self.classify_intent(&input_lower);
        let entities = self.extract_entities(input);
        let requires_tools = self.determine_tools(&intent_type);

        Ok(ParsedIntent {
            intent_type,
            confidence: 0.85,
            entities,
            requires_tools,
            priority: Priority::Normal,
            raw_input: input.to_string(),
        })
    }

    fn classify_intent(&self, input: &str) -> IntentType {
        // Code patterns
        if input.contains("code") || input.contains("write") && (input.contains("function") || input.contains("script") || input.contains("program"))
            || input.contains("debug") || input.contains("fix") && input.contains("bug")
            || input.contains("review") && input.contains("code")
            || input.contains("python") || input.contains("rust") || input.contains("javascript")
            || input.contains("compile") || input.contains("test") && input.contains("code")
        {
            return IntentType::Code;
        }

        // Research patterns
        if input.contains("search") || input.contains("find") || input.contains("research")
            || input.contains("look up") || input.contains("what is") || input.contains("explain")
            || input.contains("summarize") || input.contains("analyze")
        {
            return IntentType::Research;
        }

        // File operations
        if input.contains("file") || input.contains("folder") || input.contains("directory")
            || input.contains("read") || input.contains("write to") || input.contains("create file")
            || input.contains("delete") || input.contains("move") || input.contains("copy")
        {
            return IntentType::File;
        }

        // Social
        if input.contains("send") && (input.contains("message") || input.contains("email") || input.contains("whatsapp") || input.contains("telegram"))
            || input.contains("reply") || input.contains("post on")
        {
            return IntentType::Social;
        }

        // Automation
        if input.contains("automate") || input.contains("automation")
            || input.contains("run") && input.contains("workflow")
            || input.contains("schedule") || input.contains("repeat")
        {
            return IntentType::Automate;
        }

        // Business/Trading
        if input.contains("trade") || input.contains("market") || input.contains("stock")
            || input.contains("crypto") || input.contains("portfolio")
        {
            return IntentType::Business;
        }

        // Config
        if input.contains("configure") || input.contains("setup") || input.contains("set api key")
            || input.contains("settings") || input.contains("provider")
        {
            return IntentType::Config;
        }

        // Help
        if input.contains("help") || input.contains("how do i") || input.contains("how to")
            || input.starts_with("?")
        {
            return IntentType::Help;
        }

        IntentType::Converse
    }

    fn extract_entities(&self, input: &str) -> Vec<Entity> {
        let mut entities = Vec::new();

        // Extract URLs
        let url_re = regex::Regex::new(r"https?://[^\s]+").unwrap();
        for mat in url_re.find_iter(input) {
            entities.push(Entity {
                entity_type: EntityType::Url,
                value: mat.as_str().to_string(),
            });
        }

        // Extract file paths
        let path_re = regex::Regex::new(r"(?:[./~])?(?:/[\w.-]+)+").unwrap();
        for mat in path_re.find_iter(input) {
            entities.push(Entity {
                entity_type: EntityType::FilePath,
                value: mat.as_str().to_string(),
            });
        }

        // Detect programming language mentions
        let languages = ["python", "rust", "javascript", "typescript", "go", "java", "c++", "swift", "kotlin"];
        for lang in &languages {
            if input.to_lowercase().contains(lang) {
                entities.push(Entity {
                    entity_type: EntityType::Language,
                    value: lang.to_string(),
                });
            }
        }

        entities
    }

    fn determine_tools(&self, intent: &IntentType) -> Vec<String> {
        match intent {
            IntentType::Code => vec!["code_generator".to_string(), "git".to_string()],
            IntentType::Research => vec!["web_crawler".to_string(), "summarizer".to_string()],
            IntentType::File => vec!["file_manager".to_string()],
            IntentType::Social => vec!["social_connector".to_string()],
            IntentType::Automate => vec!["playwright".to_string(), "pyautogui".to_string()],
            IntentType::Business => vec!["market_data".to_string(), "portfolio".to_string()],
            _ => vec![],
        }
    }
}
