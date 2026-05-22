Goal: Improve the Fabro software factory inside modernagencysales/fabro-maestro using docs/factory/maestro-fabro-perfect-software-factory.md. Do not work on Maestro V2.
Run ID: 01KS6HRJR3E0A5FZDTJP44DGM7
Completed 12 stage(s) so far.

(7 earlier stage(s) omitted)

Recent stages:
- scope_guard: succeeded (Script completed: .fabro/scripts/factory-scope-guard.sh)
  - Script: `.fabro/scripts/factory-scope-guard.sh`
  - Output:
    ```
    {
      "decision": "pass",
      "lines_added": 4,
      "lines_removed": 4
    }
    ```
- validate: succeeded (Script completed: .fabro/scripts/factory-validate.sh)
  - Script: `.fabro/scripts/factory-validate.sh`
  - Output:
    ```
    (27 lines omitted)
    Validation: OK
    === fabro validate .fabro/workflows/implement-plan/workflow.fabro ===
    Workflow: ImplementPlan (12 nodes, 15 edges)
    Graph: .fabro/workflows/implement-plan/workflow.fabro
    Validation: OK
    === fabro validate .fabro/workflows/interview/workflow.fabro ===
    Workflow: Interview (8 nodes, 15 edges)
    Graph: .fabro/workflows/interview/workflow.fabro
    Validation: OK
    === fabro validate .fabro/workflows/sleeper/workflow.fabro ===
    Workflow: Sleeper (3 nodes, 2 edges)
    Graph: .fabro/workflows/sleeper/workflow.fabro
    Validation: OK
    === fabro validate .fabro/workflows/smoke/workflow.fabro ===
    Workflow: Smoke (8 nodes, 7 edges)
    Graph: .fabro/workflows/smoke/workflow.fabro
    Validation: OK
    === cargo fmt check ===
    === cargo check workspace ===
    === fabro-web typecheck ===
    bun install v1.3.6 (d530ed99)
    
    Checked 695 installs across 854 packages (no changes) [27.00ms]
    $ tsc
    validation_passed
    ```
- review_fanout: partially_succeeded (Parallel node dispatched 3 branches (1 succeeded, 2 failed))
- merge_reviews: succeeded (Selected best candidate: qa_review)
- consolidate_reviews: succeeded (Stage completed: consolidate_reviews)
  - Model: claude-sonnet-4-6, 14.1k tokens in / 2.2k out
  - Files: /home/daytona/workspace/.factory/reviews/consolidated.md

## Context
- parallel.branch_count: 3
- parallel.fan_in.best_head_sha: 6cbb029533dfb1aa525bc3c489750e760884d1e0
- parallel.fan_in.best_id: qa_review
- parallel.fan_in.best_outcome: succeeded
- parallel.results: [{"id":"qa_review","status":"succeeded","head_sha":"6cbb029533dfb1aa525bc3c489750e760884d1e0"},{"id":"security_review","status":"failed","head_sha":"ad2afe6df30abf5bb4af055254f845ad62e47449"},{"id":"architecture_review","status":"failed","head_sha":"51c28cb33818d3bfd9a5c1ac1eb81c182e0548f1"}]


# Factory Release Notes

Write `.factory/release/release.md`.

Include:
- summary
- files changed
- validation commands and results
- review findings
- known risks
- follow-up factory improvements
- Fabro run branch

Do not edit source files except `.factory/release/release.md`.
