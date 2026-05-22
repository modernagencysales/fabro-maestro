Implemented the next useful slice from `.factory/self/plan.md` by finishing the **remaining part of Patch 2** (plan artifact enforcement without breaking preflight).

### ✅ Changes made

1. **Enforced planner artifact presence in quick validation**
   - **File:** `.fabro/scripts/factory-validate.sh`
   - Added a check in `--quick` mode:
     - Requires `.factory/self/plan.md` to exist.
     - Fails fast if missing (`missing required planner artifact: .factory/self/plan.md`).

2. **Kept preflight compatible (plan doesn’t exist yet at that stage)**
   - **File:** `.fabro/workflows/factory-self-improve/workflow.fabro`
   - Updated preflight script call to:
     - `FACTORY_VALIDATE_ALLOW_MISSING_PLAN=1 .fabro/scripts/factory-validate.sh --quick`
   - This preserves preflight behavior while still enforcing plan existence in later quick validations.

---

### ✅ Validation run

I ran the required command after workflow/script changes:

- `.fabro/scripts/factory-validate.sh --quick` → **passed**

I also verified the preflight bypass flag behavior:

- `FACTORY_VALIDATE_ALLOW_MISSING_PLAN=1 .fabro/scripts/factory-validate.sh --quick` → **passed** with expected warning.

---

If you want, I can now implement the next follow-on slice from the plan (likely Slice 4: retro schema + retro workflow stub).