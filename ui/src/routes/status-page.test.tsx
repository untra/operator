import { afterEach, describe, expect, mock, spyOn, test } from "bun:test";
import { cleanup, fireEvent, render } from "@testing-library/react";
import { useState } from "react";
import { MemoryRouter } from "react-router-dom";
import type { SectionDto } from "../api-client";
import * as webcomponentMocks from "../test-webcomponents";

const sectionsBox: { current: SectionDto[] | null } = { current: null };

mock.module("@operator/webcomponents", () => webcomponentMocks);
mock.module("../sections-context", () => ({
  useSections: () => ({ sections: sectionsBox.current, error: null }),
}));
mock.module("../components/SectionCard", () => ({
  SectionCard: ({ section }: { section: SectionDto }) => (
    <section id={section.id}>{section.label}</section>
  ),
}));

const { StatusPage } = await import("./StatusPage");

const DEEP_LINK = ["/status?s=git"];
let restoreScroll: (() => void) | undefined;

function gitSection(): SectionDto {
  return {
    id: "git",
    label: "Git",
    health: "green",
    description: "Git repositories",
    prerequisites: [],
    met: true,
    children: [],
  };
}

afterEach(() => {
  cleanup();
  sectionsBox.current = null;
  restoreScroll?.();
  restoreScroll = undefined;
});

function Harness() {
  const [, setTick] = useState(0);
  return (
    <>
      <button type="button" onClick={() => setTick((tick) => tick + 1)}>
        refresh
      </button>
      <StatusPage />
    </>
  );
}

function refresh(view: ReturnType<typeof render>): void {
  fireEvent.click(view.getByRole("button", { name: "refresh" }));
}

describe("StatusPage", () => {
  test("a ?s= deep link scrolls once when sections load, not again on a poll", () => {
    const spy = spyOn(HTMLElement.prototype, "scrollIntoView").mockImplementation(() => undefined);
    restoreScroll = () => {
      spy.mockRestore();
    };

    const view = render(
      <MemoryRouter initialEntries={DEEP_LINK}>
        <Harness />
      </MemoryRouter>,
    );
    expect(spy).not.toHaveBeenCalled();

    sectionsBox.current = [gitSection()];
    refresh(view);
    expect(spy).toHaveBeenCalledTimes(1);

    sectionsBox.current = [gitSection()];
    refresh(view);
    expect(spy).toHaveBeenCalledTimes(1);
  });
});
