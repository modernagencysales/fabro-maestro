# Fabro Onboarding — Teaching Guide

A runnable, build-it-up curriculum for getting a new engineer productive with Fabro:
**deterministic workflows, gates, sub-workflows, and evals.** Everything here is
hands-on — the examples in `examples/` are validated and ready to `fabro run`.

> Teaching philosophy: **build, don't lecture.** One new concept per rung, run it,
> and make the determinism *visible* (event log, checkpoints, rewind). Anchor every
> concept to the failure it prevents.

---

## 0. Pre-flight (do this before the session)

Fabro's CLI is a thin client to the server at
`https://fabro-maestro-production.up.railway.app`. Confirm the environment is healthy:

```bash
fabro doctor            # expect Configuration ✓, Location ✓, Crypto ✓, Storage ✓
fabro version           # client + server version
fabro model             # what models/providers the server can run
```

Known gotchas (all real, all hit on this machine):

| Symptom | Cause | Fix |
|---|---|---|
| MCP "Failed to connect" in Claude Code | `~/.local/bin/fabro` symlink pointed at a pruned worktree build | re-point to `target/release/fabro` (or `cargo dev build`) |
| Every `fabro` command errors on settings | `~/.fabro/settings.toml` had an `[llm.providers]/[llm.models]` block; this build has **no top-level `[llm]` key** and uses `deny_unknown_fields` | keep only `[cli.target]` / `[server.*]`; model registry is **server-side** |
| `fabro doctor` shows `[✗] LLM Providers (gemini failed)` | server's Gemini provider is quota/rate-limited | use a non-Gemini model in workflows (Anthropic/OpenAI/OpenRouter all work), or fix the server key |
| `[!] Version parity` | client 0.231 vs server 0.232 | cosmetic; rebuild the CLI when convenient |

**Demo data / nav:** the web UI hides some tabs in live mode by design
(`app-shell.tsx` → `demoOnly`). On the deployed build, **Automations and Insights
only show when demo mode is ON** (the beaker icon). For a clean walkthrough with
populated screens, **turn demo mode on**.

**For live runs:** make sure workflows resolve to a *working* model. The simplest
guard is a graph-level stylesheet (see example 04): `* { model: claude-haiku-4-5; }`.

---

## 1. The one mental model

> **The graph IS the program.** A `.fabro` file is a Graphviz (DOT) directed graph.
> Intelligence lives in *agent nodes* (non-deterministic LLM work). The orchestration
> around them — order, branching, loops, gates, retries — is **deterministic,
> version-controlled, checkpointed, and resumable.**
>
> **Gates are the contract that lets you walk away.** A workflow with no gates is
> just hope with extra steps.

This is the leap from a REPL (Claude Code/Cursor: prompt → review → repeat by hand)
and from no-code automation (n8n: trigger → action). Fabro separates *what the AI
decides* from *when it runs and what must be true for it to proceed*.

---

## 2. Node vocabulary (the cheat sheet)

A node's **shape** is its type. No shape = agent.

| Shape | Type | What it does |
|---|---|---|
| `Mdiamond` | **start** | entry (exactly one) |
| `Msquare` | **exit** | terminal (exactly one) |
| `box` (default) | **agent** | LLM **with tools** (bash, edit, read) — runs an agentic loop |
| `tab` | **prompt** | single LLM call, **no tools** (analysis, summaries) |
| `parallelogram` | **command** | runs a shell script in the sandbox, captures output |
| `hexagon` | **human gate** | pauses for a person; outgoing **edge labels** are the choices |
| `diamond` | **conditional** | routes on `condition=` evaluated against run context |
| `component` | **parallel fan-out** | concurrent branches, isolated context (`join_policy`) |
| `tripleoctagon` | **merge fan-in** | collects branches into `parallel_results.json` |
| `house` | **sub-workflow** | runs a whole child workflow (`stack.child_workflow`) |
| `insulator` | **wait** | pause `duration="30s"` |

Useful attributes on **any** node: `label`, `class` (stylesheet targeting),
`reasoning_effort` (`low`/`medium`/`high`), `max_visits` (loop cap),
`goal_gate=true` (run fails if this node fails), `retry_policy`
(`none`/`standard`/`aggressive`/`linear`/`patient`), `fidelity` (how much prior
context this node sees), `timeout`.

---

## 3. Session agenda (~75–90 min)

Run each example, then **look at the run** (`fabro events <id>`, `fabro inspect <id>`,
or the web UI). The goal is for the dev to *see* determinism, not hear about it.

| # | Example | New concept | Do this |
|---|---|---|---|
| A | — (whiteboard, 10 min) | the mental model + shape vocabulary | sketch the graph, name the shapes |
| 1 | `examples/01-hello.fabro` | graph = program; agent node | run it, then `fabro events`, then **`fabro rewind`/`fork`** to prove replay |
| 2 | `examples/02-verify-loop.fabro` | command node + conditional gate + **self-correcting loop** + `max_visits` | this is the keystone — break it on purpose and watch it recover |
| 3 | `examples/03-human-gate.fabro` | human-in-the-loop | answer it 3 ways: web UI, `fabro resume`, and `fabro_run_interact` from Claude |
| 4 | `examples/04-multi-model.fabro` | model routing via `model_stylesheet` | cheap default, expensive only where it counts |
| 5 | `examples/05-parallel-review.fabro` | parallel fan-out / merge | 3 independent reviewers, then synthesize |
| 6 | `examples/06-subworkflow-parent.fabro` (+ child) | composition & reuse (`house` node) | parent delegates to a reusable child; context diff-merges back |
| 7 | `examples/07-capstone-cold-email.fabro` | **everything together** (Maestro-flavored) | draft → lint → loop → human approve → ship |

---

## 4. Gates — the trust layer (deep dive)

Four kinds. Teach *when to reach for each*.

**a) Conditional gate** (`diamond` + `condition=`) — branch on a prior result.
```dot
gate -> exit      [label="Pass", condition="outcome=succeeded"]
gate -> implement [label="Fix",  max_visits=5]   // unconditional = default fallback
```
Condition grammar: `=`, `!=`, `&&`, `!`, numeric (`context.score > 80`), substring
(`context.log contains error`), and `context.internal.node_visit_count >= 5` for
fixed-count loops.

> ⚠️ **Gotcha I verified in the engine** (`handler/command.rs:174`): a command node's
> outcome is **the shell exit code** — `exit 0` ⇒ `outcome=succeeded`. So the common
> `script="pytest ... || true"` makes the node *always* succeed, which means
> `condition="outcome=succeeded"` is *always* true and the loop won't trigger. To make
> a verify-loop actually loop on failure, either (a) **drop `|| true`** so a failing
> test yields `outcome=failed`, or (b) keep `|| true` and **branch on the output**:
> `condition="context.command.output contains PASS"` (example 07 does this). Confirm
> the loop fires live before you trust it.

**b) Verification gate** — a command (tests/lint/typecheck) or an **LLM-as-judge**
prompt node feeding a conditional. This is the everyday "is it actually correct?" gate.

**c) Human gate** (`hexagon`) — for irreversible or judgment calls. Edge labels are
the options; `[A]`/`[S]` prefixes are the CLI hotkeys.

**d) Goal gate** (`goal_gate=true`) — "this node *must* succeed or the whole run
fails." Note the validator warns if a goal gate has no `retry_target` /
`fallback_retry_target` — a goal gate usually points back at a node that can fix the
problem. Use it on the step that defines success (the "ship" / "definition of done").

---

## 5. Sub-workflows (composition)

`shape=house` runs an entire child workflow as one node:

```dot
impl [label="Implement & Test", shape=house,
      stack.child_workflow="06-subworkflow-child.fabro", manager.max_cycles=50]
```

- **Context flow:** child gets a *clone* of the parent context; only the **diff** it
  produces merges back.
- `manager.max_cycles` caps poll cycles (safety); `manager.stop_condition` cancels the
  child early on an external signal (supervisor pattern).
- **When to use:** reuse the same loop across parents, or encapsulate a noisy
  sub-process so the parent's trace stays clean. For one-offs, just add more nodes.

---

## 6. Evals (measure, don't vibe)

Two levels — teach both:

**In-workflow verification = the gates above.** Every workflow the dev writes should
answer: *"what's the gate that proves this worked?"* (a test pass, a judge verdict, a
human approval). No gate ⇒ no trust.

**Offline benchmarking = `evals/swe-bench/`.** 300 SWE-Bench-Lite bug-fix tasks on
Daytona sandboxes. Three steps: **generate** patches → **evaluate** (apply + run
held-out tests, grade pass/fail) → **record** to `scoreboard/leaderboard.json`.

```bash
cd evals/swe-bench && source .venv/bin/activate
python run_eval.py --model claude-haiku-4-5 --provider anthropic --output-dir results/haiku
python evaluate_daytona.py --predictions results/haiku/predictions.jsonl --output-dir results/haiku/eval
python record_results.py --run-name haiku-$(date +%Y%m%d) --gen-dir results/haiku --eval-dir results/haiku/eval --description "..."
```

The discipline: **before trusting a model/prompt change, run the eval and look at the
resolve-rate delta on the leaderboard.** The existing scoreboards compare
haiku/sonnet/opus/gpt54. Numbers ship; vibes don't.

---

## 7. Driving Fabro from Claude Code (the MCP)

The Fabro MCP turns Claude into a control surface. Tools exposed:

| Tool | Use |
|---|---|
| `fabro_run_create` | kick off a workflow run |
| `fabro_run_events` | stream/inspect the event log |
| `fabro_run_search` | find runs |
| `fabro_run_gather` | collect a run's outputs/artifacts |
| `fabro_run_interact` | answer a human gate / steer a running agent |

> If the tools aren't showing up in Claude Code, the MCP probably failed to connect at
> session start — restart Claude Code after `fabro doctor` is green.

---

## 8. Solo exercise (cements it)

Give the dev 15 minutes, no help:

> Write a workflow from scratch that **(a)** implements a small feature, **(b)** runs a
> verification command, **(c)** loops back on failure with `max_visits`, and **(d)**
> ends on a human approval gate before "shipping." Run it, **make it fail on purpose**,
> watch it self-correct, then approve.

If they can do that unaided, they understand Fabro.

---

## 9. CLI cheat-sheet

```bash
fabro validate <file.fabro>     # structural check — ALWAYS before run
fabro run <file|name>           # execute (name resolves .fabro/workflows/<name>/)
fabro events <run-id>           # event log (the deterministic record)
fabro inspect <run-id>          # stages, tokens, durations
fabro logs <run-id>             # raw worker tracing
fabro resume <run-id>           # continue an interrupted/paused run
fabro rewind <run-id>           # roll back to an earlier checkpoint
fabro fork <run-id>             # branch a new run from a checkpoint
fabro steer <run-id>            # nudge a running agent mid-flight
fabro ps                        # active runs / sandboxes
fabro graph <file.fabro>        # render the graph as SVG
fabro doctor                    # environment + integration health
fabro model                     # list/test models
```

---

## 10. Where things live

- Example workflows: `docs/teaching/examples/` (this kit)
- Named workflows: `.fabro/workflows/<name>/workflow.fabro` (run by name)
- Official tutorials (source of truth, version-matched): `docs/public/tutorials/`
- Official node/gate reference: `docs/public/workflows/stages-and-nodes.mdx`
- Eval harness: `evals/swe-bench/`
- Demos this kit is adapted from: `docs/internal/demo/*.fabro`
