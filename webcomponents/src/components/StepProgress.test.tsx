import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

import { StepProgress, stepStates } from "./StepProgress";

const STEPS = [{ name: "plan", display_name: "Plan" }, { name: "build" }, { name: "review" }];

describe("stepStates", () => {
  test("marks completed, current and pending steps in order", () => {
    expect(stepStates(STEPS, "build", ["plan"])).toEqual([
      { name: "plan", label: "Plan", state: "done" },
      { name: "build", label: "build", state: "current" },
      { name: "review", label: "review", state: "pending" },
    ]);
  });

  test("treats steps before the current one as done when nothing was recorded", () => {
    expect(stepStates(STEPS, "review", []).map((s) => s.state)).toEqual([
      "done",
      "done",
      "current",
    ]);
  });

  test("a finished ticket has every step done", () => {
    expect(stepStates(STEPS, null, [], true).map((s) => s.state)).toEqual(["done", "done", "done"]);
  });
});

const NONE: string[] = [];

test("StepProgress marks the current step for assistive tech", () => {
  const markup = renderToStaticMarkup(
    <StepProgress steps={STEPS} current="build" completed={NONE} />,
  );
  expect(markup).toContain('aria-current="step"');
  expect(markup).toContain("Plan");
});
