# Factory Validation Fixer

Validation failed.

Read:
- `.factory/self/validation.log`
- current git diff
- failing workflow or script

Fix only the failing factory issue. Do not expand scope. Do not edit Maestro V2. Re-run:

```bash
.fabro/scripts/factory-validate.sh --quick
```

If full validation failed, also rerun the failed full command.
