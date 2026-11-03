import { composeStories, setProjectAnnotations } from "@storybook/react";
import { clearAllMocks, within } from "storybook/test";
import { afterEach, beforeEach, describe, expect, test } from "vitest";
import { userEvent } from "vitest/browser";
import * as previewAnnotations from "../.storybook/preview";
import * as AppShellStories from "./components/AppShell.stories";
import * as AsyncStateStories from "./components/AsyncState.stories";
import * as AuthCardStories from "./components/AuthCard.stories";
import * as LaunchFormStories from "./components/LaunchForm.stories";
import * as SectionCardStories from "./components/SectionCard.stories";
import * as TicketCreateFormStories from "./components/TicketCreateForm.stories";
import * as IssueTypesViewStories from "./pages/IssueTypesView.stories";

setProjectAnnotations([previewAnnotations]);

type Canvas = ReturnType<typeof within>;
type RunnableStory = { id: string; run: (context: { canvasElement: HTMLElement }) => Promise<void> };
type StoryModule = Parameters<typeof composeStories>[0];

const AppShell = composeStories(AppShellStories);
const AsyncState = composeStories(AsyncStateStories);
const AuthCard = composeStories(AuthCardStories);
const LaunchForm = composeStories(LaunchFormStories);
const SectionCard = composeStories(SectionCardStories);
const TicketCreateForm = composeStories(TicketCreateFormStories);
const IssueTypesView = composeStories(IssueTypesViewStories);

const STORY_MODULES = import.meta.glob<StoryModule>("./**/*.stories.tsx", { eager: true });
const ALL_STORIES = Object.values(STORY_MODULES).flatMap(
  (module): RunnableStory[] => Object.values(composeStories(module)),
);

const FOCUSABLE_SELECTOR =
  'a[href], button, input, select, textarea, summary, [tabindex]:not([tabindex="-1"])';
/** @untra/naiveworkflow-react renders xyflow's focusable nodes/edges with no focus style. */
const UPSTREAM_GRAPH_SELECTOR = ".react-flow";

let canvasElement: HTMLElement;

beforeEach(() => {
  clearAllMocks();
  canvasElement = document.body.appendChild(document.createElement("div"));
});

afterEach(() => {
  canvasElement.remove();
});

async function render(story: RunnableStory): Promise<Canvas> {
  await story.run({ canvasElement });
  return within(canvasElement);
}

function focusIndicatorShown(element: HTMLElement): boolean {
  const before = getComputedStyle(element);
  const unfocused = {
    boxShadow: before.boxShadow,
    borderColor: before.borderColor,
    backgroundColor: before.backgroundColor,
  };
  element.focus({ focusVisible: true });
  if (document.activeElement !== element) {
    return true;
  }
  const after = getComputedStyle(element);
  const outlined = after.outlineStyle !== "none" && Number.parseFloat(after.outlineWidth) > 0;
  return (
    outlined ||
    after.boxShadow !== unfocused.boxShadow ||
    after.borderColor !== unfocused.borderColor ||
    after.backgroundColor !== unfocused.backgroundColor
  );
}

describe.each(ALL_STORIES.map((story) => [story.id, story] as const))("%s", (_id, story) => {
  test("every focusable control shows a focus indicator", async () => {
    await render(story);
    const missing = [...canvasElement.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)]
      .filter((element) => !element.closest(UPSTREAM_GRAPH_SELECTOR))
      .filter((element) => !focusIndicatorShown(element))
      .map((element) => element.outerHTML.slice(0, 120));
    expect(missing).toEqual([]);
  });
});

const LIVE_REGION_CASES = [
  {
    name: "AppShell sign-out failure",
    story: AppShell.SignOutFailure,
    trigger: { name: "Sign out" },
    text: "Could not sign out.",
  },
  {
    name: "AuthCard rejected submission",
    story: AuthCard.FailedSubmission,
    trigger: { name: "Sign in" },
    text: "Incorrect username or password.",
  },
  {
    name: "LaunchForm failed launch",
    story: LaunchForm.FailedSubmission,
    trigger: { name: /Launch/ },
    text: "Could not launch agent.",
  },
];

test.each(LIVE_REGION_CASES)(
  "$name announces through a persistent alert region",
  async ({ story, trigger, text }) => {
    const canvas = await render(story);
    const region = canvas.getByRole("alert");
    expect(region.textContent).toBe("");
    await userEvent.click(canvas.getByRole("button", trigger));
    expect(canvas.getByRole("alert")).toBe(region);
    expect(region.textContent).toBe(text);
  },
);

const ASYNC_TRANSITIONS = [
  { button: "Load", status: "Loading…", alert: "" },
  { button: "Fail", status: "", alert: "Request failed" },
  { button: "Recover", status: "", alert: "" },
];

test("AsyncState keeps one status and one alert region across transitions", async () => {
  const canvas = await render(AsyncState.Transitions);
  const status = canvas.getByRole("status");
  const alert = canvas.getByRole("alert");
  for (const step of ASYNC_TRANSITIONS) {
    const button = canvas.getByRole("button", { name: step.button });
    await userEvent.click(button);
    expect({ status: status.textContent, alert: alert.textContent }).toEqual({
      status: step.status,
      alert: step.alert,
    });
    expect(canvas.getByRole("status")).toBe(status);
    expect(canvas.getByRole("alert")).toBe(alert);
    expect(document.activeElement).toBe(button);
  }
});

interface KeyboardStep {
  act: (canvas: Canvas) => Promise<unknown>;
  attribute?: { of: (canvas: Canvas) => Element; name: string; value: string | null };
}

interface HandlerExpectation {
  handler: string;
  with: unknown[];
  times?: number;
}

interface KeyboardCase {
  name: string;
  story: RunnableStory & { args: object };
  steps: KeyboardStep[];
  calls: HandlerExpectation[];
}

const press = (target: (canvas: Canvas) => HTMLElement, keys: string) => async (canvas: Canvas) => {
  target(canvas).focus();
  await userEvent.keyboard(keys);
};

function summaryOf(): HTMLElement {
  const summary = canvasElement.querySelector("summary");
  if (!summary) {
    throw new Error("story rendered no <summary>");
  }
  return summary;
}
const detailsOf = () => summaryOf().parentElement as HTMLElement;
const issueTypeButton = (canvas: Canvas) => canvas.getByRole("button", { name: /FEAT/ });
const graphButton = (canvas: Canvas) => canvas.getByRole("button", { name: "Graph" });

const KEYBOARD_CASES: KeyboardCase[] = [
  {
    name: "SectionCard summary toggles with Enter and Space",
    story: SectionCard.KeyboardExpansion,
    steps: [
      {
        act: press(summaryOf, "{Enter}"),
        attribute: { of: detailsOf, name: "open", value: "" },
      },
      {
        act: press(summaryOf, " "),
        attribute: { of: detailsOf, name: "open", value: null },
      },
    ],
    calls: [],
  },
  {
    name: "IssueTypesView selection and mode toggle by keyboard",
    story: IssueTypesView.KeyboardSelection,
    steps: [
      {
        act: press(issueTypeButton, "{Enter}"),
        attribute: { of: issueTypeButton, name: "aria-pressed", value: "true" },
      },
      {
        act: press(graphButton, " "),
        attribute: { of: graphButton, name: "aria-pressed", value: "true" },
      },
      { act: (canvas) => userEvent.click(canvas.getByRole("button", { name: "Steps" })) },
    ],
    calls: [
      { handler: "onSelect", with: ["FEAT"] },
      { handler: "onModeChange", with: ["graph"] },
      { handler: "onModeChange", with: ["steps"] },
    ],
  },
  {
    name: "TicketCreateForm fills and submits by keyboard",
    story: TicketCreateForm.Empty,
    steps: [
      { act: (canvas) => userEvent.selectOptions(canvas.getByLabelText("Issue type"), "FEAT") },
      { act: (canvas) => userEvent.selectOptions(canvas.getByLabelText("Project"), "operator") },
      { act: (canvas) => userEvent.type(canvas.getByLabelText("Summary"), "Make status accessible") },
      { act: () => userEvent.tab() },
      { act: () => userEvent.keyboard("{Enter}") },
    ],
    calls: [
      { handler: "onSubmit", with: [], times: 1 },
      {
        handler: "onChange",
        with: [{ issueType: "FEAT", project: "operator", summary: "Make status accessible" }],
      },
    ],
  },
];

test.each(KEYBOARD_CASES)("$name", async ({ story, steps, calls }) => {
  const canvas = await render(story);
  for (const step of steps) {
    await step.act(canvas);
    if (step.attribute) {
      const { of, name, value } = step.attribute;
      expect(of(canvas).getAttribute(name)).toBe(value);
    }
  }
  for (const call of calls) {
    const handler = (story.args as Record<string, unknown>)[call.handler];
    expect(handler).toHaveBeenCalledWith(...call.with);
    if (call.times !== undefined) {
      expect(handler).toHaveBeenCalledTimes(call.times);
    }
  }
});
