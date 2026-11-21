import { expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { AsyncState, type AsyncValue } from "./AsyncState";
import { SectionCard } from "./SectionCard";
import { KanbanBoard } from "./KanbanBoard";
import { healthySection, board, runningTicket } from "../../stories/fixtures/operator";

const LOADING: AsyncValue<never> = { status: "loading" };
const FAILED: AsyncValue<never> = { status: "error", message: "Failed" };
const ATTENTION_BOARD: typeof board = {
  ...board,
  running: [{ ...runningTicket, attention: "review" }],
};

test("loading and failures expose announcement roles", () => {
  expect(renderToStaticMarkup(<AsyncState value={LOADING}>{() => null}</AsyncState>)).toContain(
    "<output",
  );
  expect(renderToStaticMarkup(<AsyncState value={FAILED}>{() => null}</AsyncState>)).toContain(
    'role="alert"',
  );
});

test("section health has a text alternative", () => {
  expect(renderToStaticMarkup(<SectionCard section={healthySection} />)).toContain("Healthy");
});

test("attention is exposed as text instead of a prohibited label", () => {
  const markup = renderToStaticMarkup(<KanbanBoard board={ATTENTION_BOARD} />);
  expect(markup).toContain(">Review pending<");
  expect(markup).not.toContain('aria-label="Review pending"');
});
