"""
microdragon/modules/tech_skills/src/engine.py
═══════════════════════════════════════════════════════════════════════════════
MICRODRAGON TECH SKILLS — COMPLETE TECHNOLOGY KNOWLEDGE BASE
═══════════════════════════════════════════════════════════════════════════════

MICRODRAGON possesses expert-level knowledge in every technology domain.
This module defines the knowledge map and builds context prompts that
activate the right expertise for each task.

DOMAINS:
  Languages    — 40+ programming languages with idioms and best practices
  Frameworks   — 80+ frameworks, full stack, mobile, ML
  Cloud        — AWS, GCP, Azure, every service and deployment pattern
  DevOps       — CI/CD, Docker, K8s, Terraform, monitoring
  Databases    — SQL, NoSQL, time-series, graph, vector
  Security     — see security_expert module
  Architecture — distributed systems, microservices, event-driven
  AI/ML        — training, inference, MLOps, prompt engineering
  Mobile       — iOS, Android, React Native, Flutter
  Networking   — TCP/IP, HTTP/3, WebSockets, load balancing

© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
"""

from dataclasses import dataclass, field


@dataclass
class TechDomain:
    name: str
    technologies: list[str]
    common_tasks: list[str]
    best_practices: list[str]
    prompt_context: str


TECH_SKILLS = {

    # ─────────────────────────────────────────────────────────────────────────
    # PROGRAMMING LANGUAGES
    # ─────────────────────────────────────────────────────────────────────────

    "python": TechDomain(
        name="Python",
        technologies=[
            "Python 3.12", "asyncio", "typing", "dataclasses",
            "pytest", "black", "ruff", "mypy", "poetry", "uv"
        ],
        common_tasks=[
            "async/await patterns", "type annotations", "dataclasses",
            "context managers", "decorators", "generators", "list comprehensions",
            "argparse", "pathlib", "logging", "subprocess", "threading"
        ],
        best_practices=[
            "Use type hints everywhere",
            "Prefer pathlib.Path over os.path",
            "Use dataclasses or Pydantic for data models",
            "asyncio for I/O-bound concurrency, multiprocessing for CPU",
            "Never use bare except — catch specific exceptions",
            "Use black + ruff for formatting and linting",
            "Write docstrings in Google format",
        ],
        prompt_context="""Expert Python developer. Use:
- Type hints and dataclasses always
- async/await for I/O bound operations
- Pydantic for data validation
- pytest for tests with fixtures
- pathlib.Path not os.path
- f-strings, not .format() or %
- Context managers for resource management
Always produce production-ready, type-safe code."""
    ),

    "rust": TechDomain(
        name="Rust",
        technologies=[
            "Rust 1.75+", "tokio", "axum", "serde", "anyhow", "thiserror",
            "reqwest", "sqlx", "clap", "rayon", "crossbeam", "dashmap"
        ],
        common_tasks=[
            "ownership and borrowing", "lifetimes", "traits", "enums with data",
            "error handling with Result/Option", "async with tokio",
            "derive macros", "iterators", "closures", "cargo workspace"
        ],
        best_practices=[
            "Use anyhow for app errors, thiserror for library errors",
            "Prefer Arc<Mutex<T>> over unsafe for shared state",
            "Use #[derive(Debug, Clone, Serialize, Deserialize)]",
            "tokio::spawn for async tasks, rayon for CPU parallelism",
            "Never use unwrap() in production — use ? operator",
            "Use clippy and rustfmt in CI",
            "Benchmark with criterion crate",
        ],
        prompt_context="""Expert Rust developer. Use:
- Ownership correctly — no unnecessary clones
- anyhow::Result<T> for error handling, ? operator always
- tokio for async, Arc<RwLock<T>> for shared state
- serde for all serialization
- never use unwrap() in production code
- derive macros: Debug, Clone, Serialize, Deserialize
Produce memory-safe, zero-cost-abstraction code."""
    ),

    "javascript": TechDomain(
        name="JavaScript / TypeScript",
        technologies=[
            "TypeScript 5+", "Node.js 20", "ES2024", "ESM modules",
            "Zod", "ts-node", "tsx", "vitest", "eslint", "prettier"
        ],
        common_tasks=[
            "async/await", "Promise.all", "destructuring", "spread operator",
            "arrow functions", "modules (ESM)", "Map/Set", "WeakRef",
            "optional chaining ?.", "nullish coalescing ??"
        ],
        best_practices=[
            "TypeScript strict mode always",
            "Use Zod for runtime type validation",
            "Prefer async/await over .then() chains",
            "ESM modules, not CommonJS",
            "Use Array.at(-1) not array[array.length-1]",
            "structuredClone() for deep copies",
            "Use const by default, let when needed, never var",
        ],
        prompt_context="""Expert TypeScript developer. Use:
- TypeScript strict mode, typed everything
- Zod for runtime validation
- async/await, never raw callbacks
- ESM modules (import/export, not require)
- functional patterns (map/filter/reduce)
- const by default
Write type-safe, maintainable code."""
    ),

    "golang": TechDomain(
        name="Go",
        technologies=[
            "Go 1.22+", "goroutines", "channels", "context", "net/http",
            "gin", "fiber", "echo", "gorm", "pgx", "testify", "cobra"
        ],
        common_tasks=[
            "goroutines and channels", "context cancellation", "error wrapping",
            "interfaces", "embedding", "defer", "init()", "table-driven tests",
            "HTTP handlers", "JSON marshaling", "file I/O", "CLI with cobra"
        ],
        best_practices=[
            "Errors are values — check every error",
            "Use context.Context for cancellation/timeout",
            "goroutines are cheap — use them liberally",
            "Channels for communication, mutexes for shared state",
            "table-driven tests with t.Run()",
            "go vet and staticcheck in CI",
            "Use fmt.Errorf('...: %w', err) for error wrapping",
        ],
        prompt_context="""Expert Go developer. Use:
- Error handling: check every error, wrap with fmt.Errorf
- goroutines for concurrency, channels for communication
- context.Context for everything that could be cancelled
- interfaces for testability
- table-driven tests
Idiomatic Go: simple, readable, concurrent."""
    ),

    "java": TechDomain(
        name="Java",
        technologies=[
            "Java 21 LTS", "Spring Boot 3", "Spring Security", "Hibernate",
            "Maven/Gradle", "JUnit 5", "Mockito", "Lombok", "MapStruct",
            "Virtual Threads (Project Loom)", "Records", "Sealed Classes"
        ],
        common_tasks=[
            "Spring Boot REST APIs", "dependency injection", "JPA entities",
            "exception handling", "scheduled tasks", "stream API", "Optional<T>",
            "records", "sealed classes", "pattern matching switch"
        ],
        best_practices=[
            "Use records for immutable DTOs",
            "Virtual threads for high-throughput I/O (Java 21+)",
            "Lombok @Value for immutable, @Builder for builders",
            "Spring Security for authentication",
            "JPA with proper lazy/eager loading",
            "Always validate with @Valid and Bean Validation",
        ],
        prompt_context="""Expert Java developer (Java 21 LTS). Use:
- Records for DTOs, Sealed classes for domain modelling
- Spring Boot 3 patterns: @Service, @Repository, @Controller
- Stream API and Optional correctly
- Virtual threads for blocking I/O
Modern Java — avoid legacy patterns."""
    ),

    "kotlin": TechDomain(
        name="Kotlin",
        technologies=[
            "Kotlin 2.0", "Coroutines", "Flow", "Ktor", "Exposed ORM",
            "Spring Boot + Kotlin", "Arrow", "Kotlin Multiplatform"
        ],
        common_tasks=[
            "coroutines and suspend functions", "data classes", "extension functions",
            "null safety", "sealed classes", "when expressions", "Flow for reactive"
        ],
        best_practices=[
            "Data classes for DTOs",
            "Sealed classes for modelling domain states",
            "Extension functions to add functionality without inheritance",
            "Flow over LiveData for reactive streams",
            "Coroutines over RxJava",
        ],
        prompt_context="""Expert Kotlin developer. Use:
- Data classes, sealed classes, extension functions
- Coroutines for async, Flow for reactive streams
- Null safety operators: ?., !!, ?:, let, run, apply
Idiomatic, concise, null-safe Kotlin."""
    ),

    # ─────────────────────────────────────────────────────────────────────────
    # WEB FRAMEWORKS
    # ─────────────────────────────────────────────────────────────────────────

    "react": TechDomain(
        name="React",
        technologies=[
            "React 19", "Next.js 15", "TypeScript", "Vite", "React Query",
            "Zustand", "Tailwind CSS", "shadcn/ui", "Framer Motion", "React Hook Form"
        ],
        common_tasks=[
            "functional components and hooks", "useState/useEffect/useContext",
            "custom hooks", "Server Components", "Suspense", "React Query",
            "form handling", "routing with App Router", "API routes"
        ],
        best_practices=[
            "Server Components by default in Next.js 15",
            "React Query for server state, Zustand for client state",
            "React Hook Form + Zod for forms",
            "Tailwind for styling, shadcn/ui for components",
            "TypeScript strict mode",
            "Use use() hook for promises",
            "Avoid useEffect for data fetching — use React Query",
        ],
        prompt_context="""Expert React/Next.js 15 developer. Use:
- Server Components by default, Client Components when needed
- React Query for data fetching, Zustand for global state
- TypeScript throughout
- Tailwind CSS for styling
- shadcn/ui for component library
Modern React patterns — no class components."""
    ),

    "nextjs": TechDomain(
        name="Next.js",
        technologies=["Next.js 15", "App Router", "Server Components", "Server Actions",
                       "Edge Runtime", "Middleware", "ISR", "Route Handlers"],
        common_tasks=["App Router layouts", "Server Components", "Server Actions",
                       "metadata API", "loading/error states", "parallel routes"],
        best_practices=[
            "App Router over Pages Router",
            "Server Components by default — only use 'use client' when needed",
            "Server Actions for form mutations",
            "Use next/image for all images",
            "Streaming with Suspense",
        ],
        prompt_context="""Expert Next.js 15 App Router developer. Use Server Components by default.
Server Actions for mutations. TypeScript strict. Tailwind CSS."""
    ),

    "vue": TechDomain(
        name="Vue.js",
        technologies=["Vue 3", "Nuxt 3", "Composition API", "Pinia", "VueRouter 4",
                       "Vite", "TypeScript", "VueUse"],
        common_tasks=["Composition API setup()", "ref/reactive/computed", "defineProps",
                       "provide/inject", "Pinia stores", "async components"],
        best_practices=[
            "Composition API always (not Options API)",
            "Pinia for state management",
            "TypeScript with defineComponent",
            "VueUse for utility composables",
        ],
        prompt_context="""Expert Vue 3 developer. Use Composition API with <script setup>.
TypeScript, Pinia for state, VueUse for utilities."""
    ),

    "fastapi": TechDomain(
        name="FastAPI",
        technologies=["FastAPI 0.115", "Pydantic v2", "SQLAlchemy 2", "Alembic",
                       "asyncpg", "httpx", "pytest-asyncio", "uvicorn"],
        common_tasks=["route handlers", "dependency injection", "Pydantic schemas",
                       "async database operations", "authentication", "background tasks"],
        best_practices=[
            "Pydantic v2 with model_validator and field_validator",
            "Dependency injection for auth and database sessions",
            "Async database with asyncpg or aiomysql",
            "Always use response_model= for type safety",
            "Background tasks for non-blocking operations",
        ],
        prompt_context="""Expert FastAPI developer. Use:
- Pydantic v2 for all schemas
- Dependency injection pattern
- async/await for database operations
- JWT authentication via dependencies
- pytest-asyncio for tests"""
    ),

    # ─────────────────────────────────────────────────────────────────────────
    # CLOUD PLATFORMS
    # ─────────────────────────────────────────────────────────────────────────

    "aws": TechDomain(
        name="Amazon Web Services (AWS)",
        technologies=[
            "EC2", "ECS/EKS", "Lambda", "S3", "RDS", "DynamoDB", "SQS/SNS",
            "API Gateway", "CloudFront", "Route 53", "IAM", "VPC",
            "CloudWatch", "CodePipeline", "CDK/Terraform", "Cognito"
        ],
        common_tasks=[
            "Deploy containerised app to ECS Fargate",
            "Serverless function with Lambda + API Gateway",
            "S3 static website + CloudFront CDN",
            "RDS PostgreSQL with Multi-AZ",
            "IAM roles with least privilege",
            "VPC with public/private subnets",
            "CloudWatch alarms and dashboards",
            "CI/CD with CodePipeline",
        ],
        best_practices=[
            "Infrastructure as Code — always use CDK or Terraform",
            "IAM least privilege — no wildcard permissions",
            "Enable CloudTrail and GuardDuty in every account",
            "VPC with private subnets for databases",
            "Encrypt S3 buckets — block all public access",
            "Use SSM Parameter Store or Secrets Manager, not env vars in code",
            "Tag every resource: Environment, Owner, Project, CostCenter",
            "Enable AWS Config for compliance drift detection",
        ],
        prompt_context="""Expert AWS architect. Use:
- CDK (TypeScript) or Terraform for IaC
- ECS Fargate over EC2 for containers
- Lambda for event-driven workloads
- IAM least privilege
- Always private subnets for databases
- CloudWatch for observability
Suggest services by cost efficiency."""
    ),

    "gcp": TechDomain(
        name="Google Cloud Platform (GCP)",
        technologies=[
            "Cloud Run", "GKE", "Cloud Functions", "BigQuery", "Firestore",
            "Cloud SQL", "Pub/Sub", "Cloud Storage", "Load Balancing",
            "IAM", "VPC", "Cloud Build", "Artifact Registry", "Vertex AI"
        ],
        common_tasks=[
            "Deploy containerised app to Cloud Run",
            "BigQuery analytics queries",
            "Firestore real-time database",
            "Pub/Sub event streaming",
            "Cloud Build CI/CD",
            "Vertex AI for ML workloads",
        ],
        best_practices=[
            "Cloud Run for serverless containers — pay per request",
            "BigQuery for analytics — avoid Spark if BigQuery suffices",
            "Workload Identity over service account keys",
            "VPC Service Controls for sensitive data",
            "Use managed services over self-managed (Cloud SQL > VM + PostgreSQL)",
        ],
        prompt_context="""Expert GCP architect. Prefer managed services.
Cloud Run for containers, BigQuery for analytics, Vertex AI for ML."""
    ),

    "azure": TechDomain(
        name="Microsoft Azure",
        technologies=[
            "Azure App Service", "AKS", "Azure Functions", "Azure SQL",
            "Cosmos DB", "Azure Storage", "Azure AD/Entra", "Key Vault",
            "Azure DevOps", "Bicep/Terraform", "Application Insights"
        ],
        common_tasks=[
            "Azure App Service deployment",
            "AKS Kubernetes cluster",
            "Azure Functions serverless",
            "Cosmos DB NoSQL",
            "Azure AD authentication",
            "Key Vault for secrets",
        ],
        best_practices=[
            "Managed Identity over service principals where possible",
            "Key Vault for all secrets and certificates",
            "Application Insights for observability",
            "Bicep for Azure-native IaC",
            "Azure Policy for compliance",
        ],
        prompt_context="""Expert Azure architect. Use Managed Identity, Key Vault for secrets,
Application Insights for monitoring. Bicep or Terraform for IaC."""
    ),

    # ─────────────────────────────────────────────────────────────────────────
    # DEVOPS & INFRASTRUCTURE
    # ─────────────────────────────────────────────────────────────────────────

    "docker": TechDomain(
        name="Docker & Containers",
        technologies=["Docker", "Docker Compose", "BuildKit", "Multi-stage builds",
                       "Docker Scout", "OCI images"],
        common_tasks=[
            "Write optimised Dockerfile",
            "Multi-stage builds for minimal images",
            "docker-compose.yml for local dev",
            "Build, tag, push to registry",
            "Scan images for vulnerabilities",
        ],
        best_practices=[
            "Multi-stage builds to keep production images small",
            "Use specific image tags — never :latest in production",
            "Run as non-root user (USER appuser)",
            ".dockerignore to exclude node_modules etc.",
            "COPY only required files",
            "Use health checks in production",
            "Layer caching: copy dependency files before source code",
        ],
        prompt_context="""Expert Docker engineer. Use:
- Multi-stage builds for small images
- Non-root USER
- Minimal base images (alpine, distroless)
- Optimised layer caching
Write Dockerfiles that build in under 30 seconds with cache."""
    ),

    "kubernetes": TechDomain(
        name="Kubernetes (K8s)",
        technologies=["Kubernetes 1.30", "Helm 3", "kubectl", "kustomize",
                       "ArgoCD", "Istio", "Prometheus/Grafana", "cert-manager"],
        common_tasks=[
            "Deployment + Service + Ingress",
            "ConfigMap and Secret management",
            "HPA horizontal autoscaling",
            "Resource limits and requests",
            "Rolling deployments",
            "Helm chart creation",
            "ArgoCD GitOps",
        ],
        best_practices=[
            "Always set resource requests and limits",
            "Liveness and readiness probes on every pod",
            "Use Secrets for sensitive data, ConfigMaps for config",
            "HPA for auto-scaling based on CPU/memory/custom metrics",
            "Pod Disruption Budgets for high availability",
            "Network policies to restrict pod-to-pod communication",
            "Use namespaces to separate environments",
        ],
        prompt_context="""Expert Kubernetes engineer. Always set:
- resource requests and limits
- readiness/liveness probes
- HPA for autoscaling
- Use Helm for packaging, ArgoCD for GitOps."""
    ),

    "terraform": TechDomain(
        name="Terraform",
        technologies=["Terraform 1.7+", "Terragrunt", "Terraform Cloud",
                       "Provider: AWS/GCP/Azure", "Modules", "OpenTofu"],
        common_tasks=[
            "Define infrastructure as code",
            "Modules for reusability",
            "Remote state in S3/GCS",
            "Workspaces for environments",
            "Import existing resources",
        ],
        best_practices=[
            "Remote state storage with locking (S3+DynamoDB)",
            "One module per service",
            "Variables for everything environment-specific",
            "Use terraform fmt and validate in CI",
            "Protect production with Sentinel policies",
            "terraform plan before every apply",
        ],
        prompt_context="""Expert Terraform engineer. Use:
- Remote state with locking
- Modules for reusable patterns
- Variables for all environment config
- Consistent naming conventions"""
    ),

    # ─────────────────────────────────────────────────────────────────────────
    # DATABASES
    # ─────────────────────────────────────────────────────────────────────────

    "postgresql": TechDomain(
        name="PostgreSQL",
        technologies=["PostgreSQL 16", "pgvector", "pg_trgm", "TimescaleDB",
                       "Citus", "PostGIS", "pg_cron"],
        common_tasks=[
            "Schema design and normalisation",
            "Index optimisation (B-tree, GIN, BRIN)",
            "EXPLAIN ANALYZE for query performance",
            "Full-text search with tsvector",
            "Window functions",
            "CTEs and recursive queries",
            "JSON/JSONB operations",
        ],
        best_practices=[
            "Use UUID as primary key for distributed systems",
            "Index foreign keys",
            "EXPLAIN ANALYZE before every complex query in production",
            "Connection pooling with PgBouncer",
            "Row-Level Security for multi-tenant apps",
            "Partial indexes for filtered queries",
            "VACUUM and ANALYZE schedules",
        ],
        prompt_context="""Expert PostgreSQL DBA. Use:
- EXPLAIN ANALYZE for optimisation
- Proper index types (B-tree for equality/range, GIN for JSON/fulltext, BRIN for time series)
- CTEs for readability
- Window functions for analytics
Write queries that scale to millions of rows."""
    ),

    "redis": TechDomain(
        name="Redis",
        technologies=["Redis 7", "Redis Stack", "RedisJSON", "RediSearch",
                       "Redis Streams", "RedisAI", "Sentinel", "Cluster"],
        common_tasks=[
            "Caching with TTL",
            "Rate limiting with sliding window",
            "Session storage",
            "Pub/Sub messaging",
            "Distributed locks",
            "Sorted sets for leaderboards",
            "Streams for event streaming",
        ],
        best_practices=[
            "Always set TTL on cache keys",
            "Use Lua scripts for atomic operations",
            "Redis Cluster for horizontal scaling",
            "Sentinel for high availability without cluster overhead",
            "keyspace notifications for cache invalidation",
            "Hash data structures for objects (memory efficient)",
        ],
        prompt_context="""Expert Redis engineer. Use appropriate data structures:
- String for simple values with TTL
- Hash for objects
- Sorted Set for leaderboards
- Stream for event logs
Always consider memory and expiry."""
    ),

    "mongodb": TechDomain(
        name="MongoDB",
        technologies=["MongoDB 7", "Mongoose", "Atlas", "Aggregation Pipeline",
                       "Atlas Search", "Atlas Vector Search"],
        common_tasks=[
            "Schema design for document model",
            "Aggregation pipeline",
            "Indexes (compound, text, geospatial)",
            "Change streams",
            "Atlas Search full-text",
            "Atlas Vector Search for AI",
        ],
        best_practices=[
            "Embed for 1-N when N is bounded (e.g., 5 addresses)",
            "Reference for M-N or large unbounded arrays",
            "Compound indexes following ESR rule (Equality, Sort, Range)",
            "Use $lookup sparingly — MongoDB is not a JOIN database",
            "Change streams for real-time reactivity",
        ],
        prompt_context="""Expert MongoDB architect. Design document schemas for:
- Embedding bounded data
- Referencing for M-N relationships
- Optimised aggregation pipelines
- Proper index strategies"""
    ),

    # ─────────────────────────────────────────────────────────────────────────
    # AI/ML
    # ─────────────────────────────────────────────────────────────────────────

    "machine_learning": TechDomain(
        name="Machine Learning & AI",
        technologies=[
            "PyTorch", "TensorFlow", "HuggingFace Transformers", "scikit-learn",
            "LangChain", "LlamaIndex", "FastAI", "Weights & Biases",
            "MLflow", "Ray", "ONNX", "vLLM", "TensorRT"
        ],
        common_tasks=[
            "Fine-tuning LLMs with LoRA/QLoRA",
            "RAG pipeline implementation",
            "Prompt engineering",
            "Model evaluation and benchmarking",
            "Dataset preparation and cleaning",
            "Model serving with vLLM/FastAPI",
            "Experiment tracking with W&B or MLflow",
        ],
        best_practices=[
            "LoRA/QLoRA for efficient fine-tuning (4-bit quantization)",
            "Always evaluate before and after fine-tuning",
            "Use structured outputs for reliable AI pipelines",
            "RAG over fine-tuning for knowledge updates (cheaper, faster)",
            "Prompt caching for cost optimisation",
            "Stream outputs for better UX",
            "Monitor model drift in production",
        ],
        prompt_context="""Expert ML engineer and AI architect. Use:
- HuggingFace ecosystem for open models
- LoRA/QLoRA for efficient fine-tuning
- RAG for knowledge augmentation
- vLLM for high-throughput inference
- Structured outputs for AI pipelines
Optimise for accuracy, latency, and cost."""
    ),

    # ─────────────────────────────────────────────────────────────────────────
    # MOBILE
    # ─────────────────────────────────────────────────────────────────────────

    "react_native": TechDomain(
        name="React Native",
        technologies=["React Native 0.74", "Expo SDK 51", "TypeScript",
                       "React Navigation", "Zustand", "MMKV", "React Query"],
        common_tasks=["Navigation setup", "native modules", "animations with Reanimated",
                       "push notifications", "deep linking", "offline-first"],
        best_practices=[
            "Expo EAS for build and distribution",
            "Reanimated 3 for 60fps animations",
            "MMKV for fast local storage (faster than AsyncStorage)",
            "React Navigation v7 with deep linking",
            "Flipper or Expo DevTools for debugging",
        ],
        prompt_context="""Expert React Native developer. Use Expo SDK, TypeScript,
React Navigation, Reanimated 3 for animations, MMKV for storage."""
    ),

    "flutter": TechDomain(
        name="Flutter",
        technologies=["Flutter 3.22", "Dart 3.4", "Riverpod", "Go Router",
                       "Isar", "Dio", "FlutterFire"],
        common_tasks=["Widget composition", "State management with Riverpod",
                       "Navigation with Go Router", "Firebase integration"],
        best_practices=[
            "Riverpod 2 for state management",
            "Go Router for declarative navigation",
            "Isar for local database (fastest Flutter DB)",
            "const constructors everywhere possible",
            "Extract to widgets to rebuild minimum UI",
        ],
        prompt_context="""Expert Flutter developer. Use:
- Riverpod 2 for state management
- Go Router for navigation
- const wherever possible for performance
- Proper Widget tree decomposition"""
    ),
}


def get_tech_context(technology: str) -> str:
    """Get the expert context prompt for a technology."""
    tech_lower = technology.lower().replace(' ', '_').replace('-', '_').replace('.', '')
    # Direct match
    if tech_lower in TECH_SKILLS:
        return TECH_SKILLS[tech_lower].prompt_context
    # Fuzzy match
    for key, domain in TECH_SKILLS.items():
        if tech_lower in key or key in tech_lower:
            return domain.prompt_context
        if technology.lower() in domain.name.lower():
            return domain.prompt_context
    # Not found — return generic expert prompt
    return f"""Expert software engineer with deep knowledge of {technology}.
Apply best practices, modern patterns, and production-ready code standards.
Always consider: correctness, performance, security, and maintainability."""


def build_multi_tech_prompt(technologies: list[str]) -> str:
    """Build a combined expertise prompt for multi-technology tasks."""
    contexts = []
    for tech in technologies:
        ctx = get_tech_context(tech)
        contexts.append(f"[{tech.upper()}]: {ctx}")
    base = "\n\n".join(contexts)
    return f"""Expert full-stack engineer with expertise in: {', '.join(technologies)}.

{base}

When generating code, apply all relevant best practices simultaneously.
For cross-technology interactions, use the most idiomatic approach for each."""


def list_all_skills() -> str:
    """List all tech skills MICRODRAGON possesses."""
    lines = [
        "",
        "  MICRODRAGON TECHNICAL SKILLS",
        "  ════════════════════════════════════════════════",
        "",
        "  LANGUAGES:",
        "  Python, Rust, JavaScript/TypeScript, Go, Java, Kotlin, Swift,",
        "  Dart, C#, C/C++, PHP, Ruby, Scala, Haskell, Elixir,",
        "  Shell/Bash, SQL, HTML/CSS, Lua, R, MATLAB, Solidity",
        "",
        "  WEB FRAMEWORKS:",
        "  React, Next.js, Vue, Nuxt, Angular, Svelte, SvelteKit",
        "  FastAPI, Django, Flask, Express, NestJS, Hono, Gin, Echo",
        "  Spring Boot, Actix-Web, Axum, Laravel, Rails, Phoenix",
        "",
        "  MOBILE:",
        "  React Native, Expo, Flutter, iOS (Swift/SwiftUI), Android (Kotlin)",
        "",
        "  CLOUD:",
        "  AWS (all 200+ services), GCP (all services), Azure (all services)",
        "  Vercel, Netlify, Fly.io, Railway, Render, Cloudflare Workers",
        "",
        "  DEVOPS:",
        "  Docker, Kubernetes, Helm, ArgoCD, Terraform, Pulumi, Ansible",
        "  GitHub Actions, GitLab CI, CircleCI, Jenkins, Drone",
        "  Prometheus, Grafana, Loki, Jaeger, OpenTelemetry",
        "",
        "  DATABASES:",
        "  PostgreSQL, MySQL, SQLite, SQL Server",
        "  MongoDB, DynamoDB, Firestore, Cassandra, CouchDB",
        "  Redis, Memcached, Dragonfly",
        "  Elasticsearch, OpenSearch, Typesense",
        "  InfluxDB, TimescaleDB, ClickHouse",
        "  Neo4j, ArangoDB",
        "  pgvector, Pinecone, Weaviate, Qdrant, Chroma (vector)",
        "",
        "  AI/ML:",
        "  PyTorch, TensorFlow, HuggingFace, scikit-learn",
        "  LangChain, LlamaIndex, DSPy, Semantic Kernel",
        "  Fine-tuning: LoRA, QLoRA, RLHF",
        "  Serving: vLLM, TGI, Ollama, LM Studio",
        "  MLOps: MLflow, Weights & Biases, DVC, BentoML",
        "",
        "  NETWORKING:",
        "  TCP/IP, HTTP/1.1, HTTP/2, HTTP/3 (QUIC)",
        "  WebSockets, SSE, gRPC, GraphQL, REST, tRPC",
        "  DNS, TLS/SSL, Load Balancing, CDN, Service Mesh",
        "",
        "  SECURITY:",
        "  OWASP Top 10, SAST/DAST, Penetration Testing",
        "  Cryptography, PKI, OAuth2/OIDC, SAML, JWT",
        "  GDPR, SOC 2, ISO 27001, PCI-DSS",
        "",
        "  ARCHITECTURE PATTERNS:",
        "  Microservices, Event-Driven, CQRS, Event Sourcing",
        "  Hexagonal Architecture, Clean Architecture, DDD",
        "  Distributed Systems, CAP Theorem, Saga Pattern",
        "  API Gateway, BFF, Strangler Fig, Circuit Breaker",
        "",
    ]
    return '\n'.join(lines)
