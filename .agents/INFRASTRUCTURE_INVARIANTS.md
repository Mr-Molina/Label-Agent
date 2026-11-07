# Infrastructure Invariants & Mechanical Guardrails

Binding safety rules enforced across all sessions and workflows:

> **Numbering convention**: Global invariants use **1–99** and live only in `/home/console-admin/Documents/Ideas/System/Agents/INFRASTRUCTURE_INVARIANTS.md` (byte-identical to `/home/console-admin/.gemini/config/templates/INFRASTRUCTURE_INVARIANTS.md`). Workspace-local `.agents/INFRASTRUCTURE_INVARIANTS.md` entries **MUST** use `P-<n>` (`P-1`, `P-2`) and never reuse global numbers.

## Invariant 13: Debug Artifact Cleanup
- **Rule**: Sanitize or remove temporary debug scripts, scratch logs, and transient build artifacts prior to session handoff.

## Invariant 14: Three Strikes Rule for Troubleshooting
- **Rule**: Max **3 attempts** per error. On the 3rd failure, halt all modifications immediately and compile a Deep Research Request.

## Invariant 24: Credential Leak Scan
- **Rule**: Never commit or persist plain-text API keys, passwords, tokens, or secrets; run a secret scan before session close.

## Invariant 28: Mechanical Mitigations for Recurring Failures
- **Rule**: Mitigate RECURRING failures mechanically (MUST/MUST NOT gates, hooks, or scripts), never via advisory comments.

## Invariant 29: Script Verification Requirement
- **Rule**: After creating or editing helper scripts, run a syntax check and live CLI execution test before completion.

## Invariant 30: Tool Name Schema Integrity
- **Rule**: Validate imported skills against active runtime tool declarations (`search_web`, `run_command`, `invoke_subagent`) before integration.

## Invariant 31: Python Code Generation Safety
- **Rule**: Never embed Python with quotes or regexes inside PowerShell `Set-Content @"..."@` heredocs; use file tools or `.py` files.

## Invariant 32: Strict Verification Exit Criteria
- **Rule**: Never accept non-zero exit codes or `--help` output as verification (Invariant 29); run against valid test input with exit code `0`.

## Invariant 33: Sequential Operations Delegation Threshold
- **Rule**: Max **3 non-exempt main-thread tool calls per wave** (`read_file`/`view_file`, `run_command`, grep/search, dir listings, probes, post-`/wakeup` discovery).
- **Exempt**: Required `SKILL.md`/`references/*.md` reads, delegation/messaging (`invoke_subagent`, `send_message`, `schedule`, `manage_subagents`, `manage_task`), artifact writes, `/goodbye` preflight, SDD scripts, and `parse_transcript.py --heartbeat`. Delegate Op 4+ via `invoke_subagent`.

## Invariant 34: Orchestrator Delegation Mandate & Atomic Micro-Subagent Fan-Out Gate (PM-2026-1102-007 / GLB-020)
- **Rule**: Delegate discovery (`Discovery Scout`), research (`Research Specialist`), CVE scans (`Reconnaissance Lookout`), cleanups (`Cleanup Specialist`), and multi-step debugging via `invoke_subagent` (`TypeName: "self"` or `"research"`).
- **Scope Slicing (`PM-2026-1102-007` / `GLB-020`)**: Scope micro-subagents to `<= 1–3 files` (`<= 400 LOC`), `<= 4 tasks`, `<= 12–15 steps / 90s` (`<= 25 steps / 180s` ceiling), `Model: "flash"` for mechanical edits, wide parallel fan-out, and no post-report `send_message` wrap-ups.

## Invariant 35: Subagent Scope and Reporting
- **Rule**: Subagents operate strictly within assigned scope and report verified findings to the Orchestrator.

## Invariant 36: Invariant and Docstring Preservation
- **Rule**: Preserve existing codebase invariants, comments, and docstrings unless explicitly ordered to mutate them.

## Invariant 37: Subagent Command Exit Verification
- **Rule**: Subagents must verify all command exit codes (`0`) before reporting task completion.

## Invariant 38: Three Strikes Deep Research Transition — Merged into Invariant 14 (Three Strikes)
- **Status**: Merged into Invariant 14 (number retained). Halt mutations on the 3rd failure and compile a Deep Research Request.

## Invariant 39: Evidence Before Assertions (Anti-Guessing)
- **Rule**: Cite concrete command, API, or log evidence before stating any root cause; never state hypotheses as facts or propose fixes before evidence.

## Invariant 40: Live Ground Truth Requirement (Anti-Stale Information)
- **Rule**: Query live infrastructure (SSH, API, CLI, SNMP) for active state, ports, link health, or online status rather than static files or CSV dumps.

## Invariant 41: Anti-Simulated Tool Verification Gate
- **Rule**: Never assert browser rendering, UI actions, or tool runs without live tool responses or image artifacts; report attachment failures immediately.

## Invariant 42: Staged Deployment & Manual Upload Gate
- **Rule**: Decouple hardware config staging (`--stage-only`) from device restarts with an explicit user confirmation gate between phases.

## Invariant 43: Ground Truth Entity & Acronym Verification
- **Rule**: Verify institutional names, client identities, and acronym expansions against authoritative repo docs or backups before authoring UI/config copy.

## Invariant 50: Production 3D Scene Hygiene & Diagnostic Quarantine
- **Rule**: Isolate or purge debug cameras, temporary shaders, and obsolete armature/mesh datablocks before finalizing 3D deliverables.

## Invariant 51: Windows Interactive Desktop `WinSta0\Default` Routing & Physical HWND Visibility
- **Rule**: Route interactive GUI launches to `WinSta0\Default` and verify `IsWindowVisible(hwnd)` with non-zero `GetWindowRect` bounds before claiming visibility.

## Invariant 52: Fail-Safe Win32 Window Management & Process PID Scoping
- **Rule**: Scope Win32 window calls (`EnumWindows`, `ShowWindow`, `SetForegroundWindow`) to `target_pid > 0`, exclude OS shell classes (`Progman`, `WorkerW`, `Shell_TrayWnd`, `IME`), and abort if matches > 5.

## Invariant 53: PowerShell Inline Python Escaping & Syntax Hardening
- **Rule**: Ban complex inline `python -c` (f-strings, nested quotes, semicolons, multi-line code) and bash `\"` escaping in `pwsh`; use `.py` files.

## Invariant 65: Single-Target Troubleshooting Isolation & Post-Completion Scope Boundary Gate (GLB-012 / PM-2026-1026-007)
- **Rule**: Isolate fixes strictly to the failing unit; never modify working files unprompted. Stop cleanly at 100% completion without pitching out-of-scope follow-ups or artificial decision menus (`PM-2026-1026-007`).

## Invariant 66: Real-Time User Narration, Output Hygiene & Anti-Silent Tool Chaining Gate
- **Rule**: Emit a clean 1–2 sentence status update on every Orchestrator tool-calling turn (including `/wakeup` Turn 1); ban silent tool chaining and thought leakage.

## Invariant 67: Embedded Appliance Boot/GPT, Wi-Fi, Klipper Motion & Vendor IPC Daemon Safety
- **Rule**: Never write raw MBR/GPT sectors (`dd`, `sgdisk`, `sfdisk`, `parted`, `fdisk`) or disable vendor IPC daemons (`app-server`, `upgrade-server`, `master-server`, `log_main`).
- Arm a Dead-Man's Rollback Watchdog and verify `/etc/init.d/*` or Wi-Fi/network edits via cold reboot; on Klipper, verify `G28` after `HOME_CUR`/`driver_SGTHRS` edits, keep `noz_ex_com`, split G-code on `;`, and ban high exhaust on `ASA`/`ABS`/`PC`/`PA`.

## Invariant 68: Bounded Subagent Lifecycle (<= 25 Steps / 180s), Unconditional 60s Heartbeat Timer, Telemetry Wakeup & Scope Slicing Gate (PAT-2026-1014-001 / GLB-017 / PM-2026-1026-001 / PM-2026-1026-008 / PM-2026-1102-007)
- **Rule**: Bound `invoke_subagent` prompts to `<= 25 steps / 180s` (`<= 12–15 steps / 90s` default), `<= 1–3 files`, `<= 4 tasks`, and `Parent Conversation ID: <id>`.
- Arm one `schedule(DurationSeconds=60, TimerCondition="any")` timer on subagent dispatch/message; on 60s wakeup, run `parse_transcript.py --heartbeat` + `manage_subagents(list)` (ban `read_file` on `transcript.jsonl`), and intervene/kill if `>25 steps` or `>180s`.

## Invariant 69: User-Intent Preservation & Anti-Feature-Stripping Gate (PAT-2026-1014-002 / PM-2026-1026-002)
- **Rule**: Never resolve a bug or test failure by silently disabling or downgrading an enabled feature (`ADAPTIVE_BED_MESH`, `transfer_call`, call forwarding) without prior user consent.

## Historical / Retired Global Invariants Index
Retired/merged global numbers (1–99) are never reused:
- **Invariants 1–12**: Consolidated into `/wakeup`, `/goodbye`, `wakeup_bootstrap.py`, and Invariant 13.
- **Invariants 15–23**: Consolidated into `single-target-troubleshooting.md`, `verify-autonomous-work.md`, and Invariants 14, 24, 28, 29.
- **Invariants 25–27**: Consolidated into tool hygiene and verification gates (Invariants 28–32).
- **Invariant 38**: Merged into Invariant 14 (Three Strikes).
- **Invariants 44–49**: Consolidated into `no-guessing.md` and Invariants 39–43.
- **Invariants 54–64**: Consolidated into `powershell-syntax-hardening.md`, `orchestrator-delegation-and-narration.md`, and Invariants 50–53, 65–68.
