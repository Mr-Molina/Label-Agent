# Multi-Agent Cognitive Architecture (AGENTS.md)

@[INFRASTRUCTURE_INVARIANTS.md](./INFRASTRUCTURE_INVARIANTS.md)

Defines multi-agent roles, responsibilities, and delegation boundaries for this workspace.

## Orchestrator Execution Contract (Invariants 33, 34 & 66)
- **Real-Time Narration (Invariant 66)**: Emit a concise 1–2 sentence status update on every tool turn; silent tool chaining is forbidden.
- **3-Operation Cap (Invariant 33)**: Max 3 non-exempt main-thread tool calls (`read_file`/`view_file`, `run_command`, grep/search, dir listings, probes, post-`/wakeup` discovery) per wave. Exempt: required `SKILL.md` reads, delegation/messaging (`invoke_subagent`, `send_message`, `schedule`, `manage_subagents`, `manage_task`), and artifact writes. Delegate Op 4+ via `invoke_subagent`.
- **Micro-Subagent Delegation (Invariants 34 & 68)**: Delegate discovery (`Discovery Scout`), research (`Research Specialist`), CVE scans (`Reconnaissance Lookout`), and cleanups (`Cleanup Specialist`) to bounded micro-subagents (`<= 1–3 files`, `<= 4 tasks`, `<= 15 steps / 90s`, 60s timer).

## Standard Roles

### 1. Reconnaissance Lookout
- **Function**: Dependency CVE scanning and vulnerability monitoring (`cve_audit_worker.py`). Read-only; never runs build/write commands.

### 2. Security Specialist
- **Function**: Credential leak scanning, secret auditing, and compliance checks (Invariant 24) via read-only pattern scripts.

### 3. Cleanup Specialist
- **Function**: Debug artifact cleanup, temporary log removal, and scratch sanitization (Invariant 13).

### 4. Infrastructure Administrator
- **Function**: Git repository management, branch sync, remote pushing, and deployment operations.

### 5. Conversation Auditor
- **Function**: Read-only `transcript.jsonl` inspection (`parse_transcript.py`) for polling loops, silent chaining, and invariant violations.

### 6. Research Specialist
- **Function**: Deep technical research across official docs, datasheets, RFCs, and upstream repos synthesized into notes.

### 7. Audit Architect
- **Function**: Repository discovery analysis, weighted domain committee synthesis, and multi-agent code audit coordination.

### 8. Discovery Scout
- **Function**: Read-only topology surveys, post-`/wakeup` orientation, and manifest indexing beyond the 3-op cap (Invariants 33 & 34).

### 9. Extension Certifier
- **Function**: Multi-layer verification and certification of network endpoints, directory parity, and hardware state.
