// Injects domain-specific knowledge based on the task type.
// This is how Microdragon teaches a small model to behave like an expert.

pub fn inject_for_task(task: &str) -> String {
    let t = task.to_lowercase();

    if t.contains("rust") || t.contains("cargo") {
        return RUST_CONTEXT.to_string();
    }
    if t.contains("python") || t.contains("pip") || t.contains("django") || t.contains("fastapi") {
        return PYTHON_CONTEXT.to_string();
    }
    if t.contains("security") || t.contains("pentest") || t.contains("vulnerability") || t.contains("owasp") {
        return SECURITY_CONTEXT.to_string();
    }
    if t.contains("sql") || t.contains("database") || t.contains("postgres") || t.contains("mysql") {
        return DATABASE_CONTEXT.to_string();
    }
    if t.contains("docker") || t.contains("kubernetes") || t.contains("k8s") || t.contains("devops") {
        return DEVOPS_CONTEXT.to_string();
    }
    if t.contains("trading") || t.contains("stock") || t.contains("crypto") || t.contains("market") {
        return TRADING_CONTEXT.to_string();
    }
    if t.contains("game") || t.contains("gta") || t.contains("mortal kombat") || t.contains("cs2") {
        return GAMING_CONTEXT.to_string();
    }

    "## DOMAIN CONTEXT\nApply general software engineering best practices. Be specific and actionable.".to_string()
}

const RUST_CONTEXT: &str = r#"## RUST EXPERT CONTEXT
- Use anyhow for app errors, thiserror for library errors
- Prefer ? operator over .unwrap() in all production code  
- Arc<RwLock<T>> for shared state, tokio::spawn for async tasks
- Use #[derive(Debug, Clone, Serialize, Deserialize)] on data structs
- Never use unsafe without thorough justification and a comment
- Cargo clippy --all-targets must pass before suggesting any code is done"#;

const PYTHON_CONTEXT: &str = r#"## PYTHON EXPERT CONTEXT
- Type hints on every function signature, always
- async/await for all I/O operations
- Pydantic for data validation, never raw dicts in APIs
- pathlib.Path not os.path
- f-strings, not .format() or %
- pytest with fixtures for all tests
- ruff for linting, black for formatting"#;

const SECURITY_CONTEXT: &str = r#"## SECURITY EXPERT CONTEXT
- OWASP Top 10 is the baseline, not the ceiling
- SQL injection: parameterised queries ONLY, no exceptions
- Never store passwords in plain text — bcrypt/Argon2 minimum
- Defense in depth: each layer should be secure independently
- Least privilege: every service/user gets minimum required permissions
- All security findings must include: severity, CVSS score, file:line, remediation
- Offensive testing ONLY with written scope authorisation"#;

const DATABASE_CONTEXT: &str = r#"## DATABASE EXPERT CONTEXT
- EXPLAIN ANALYZE before optimising any query
- Index foreign keys — always
- Use transactions for any multi-table write operation
- PostgreSQL: JSONB over JSON for queryable JSON data
- Connection pooling is mandatory in production (PgBouncer/pgpool)
- Never SELECT * in production queries"#;

const DEVOPS_CONTEXT: &str = r#"## DEVOPS EXPERT CONTEXT
- Infrastructure as Code — never manual configuration
- Docker: multi-stage builds, non-root USER, specific image tags
- K8s: always set resource requests/limits and readiness probes
- GitOps: ArgoCD or Flux for deployment, not kubectl apply in CI
- Secrets management: never in env vars in code, use Vault or cloud secrets"#;

const TRADING_CONTEXT: &str = r#"## TRADING CONTEXT
- RSI, MACD, Bollinger Bands are indicators not oracles
- Risk management first: position size = (account * risk %) / (entry - stop)
- Backtesting is required before any live strategy
- Always include: entry, stop-loss, take-profit in any trade plan
- ALWAYS append: "Not financial advice. Past performance ≠ future results.""#;

const GAMING_CONTEXT: &str = r#"## GAMING AUTOMATION CONTEXT
- Game must be in WINDOWED mode for screen capture to work
- mss captures screen by window bounds, not focus — terminal can stay open
- pynput sends inputs to window by title, not to focused window
- OpenCV processes frames at 30fps — reduce to 15fps if CPU is under strain
- Always verify game is running before starting automation loop"#;
