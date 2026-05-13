import { describe, expect, test } from "bun:test";
import { getVisibleNavigation } from "./app-shell";

describe("getVisibleNavigation", () => {
  test("shows all nav items in demo mode with Automations first", () => {
    const items = getVisibleNavigation(true);
    const names = items.map((i) => i.name);
    expect(names[0]).toBe("Automations");
    expect(names).toContain("Runs");
    expect(names).toContain("Insights");
    expect(names).toContain("Settings");
  });

  test("shows live Automations while keeping Insights demo-only in production mode", () => {
    const items = getVisibleNavigation(false);
    const names = items.map((i) => i.name);
    expect(names).toContain("Automations");
    expect(names).not.toContain("Insights");
    expect(names).toContain("Runs");
    expect(names).toContain("Settings");
  });
});
