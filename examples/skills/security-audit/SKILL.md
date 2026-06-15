---
name: security-audit
description: Perform a security audit on code, checking for common vulnerabilities including injection, auth bypass, secrets leakage, and unsafe dependencies
---
# Security Audit

Perform a thorough security review of the target code.

## Scope

- **Injection**: SQL injection, command injection, path traversal
- **Authentication**: Auth bypass, weak token generation, missing rate limits
- **Secrets**: Hardcoded API keys, passwords, tokens in source
- **Dependencies**: Known CVEs in imported packages
- **Data exposure**: PII leakage in logs, error messages, or responses

## Process

1. Read all files in the target scope
2. For each file, check against the vulnerability categories above
3. Classify findings as Critical / High / Medium / Low
4. Provide specific line references and remediation steps
5. Summarize with a risk score (1-10)

## Output Format

For each finding:
- **File**: path/to/file.rs
- **Line**: 42
- **Severity**: High
- **Category**: Injection
- **Description**: User input passed directly to SQL query
- **Fix**: Use parameterized queries
