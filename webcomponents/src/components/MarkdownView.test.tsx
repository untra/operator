import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

import { MarkdownView } from "./MarkdownView";

const SCRIPT_SCHEME = ["java", "script:"].join("");

function render(source: string): string {
  return renderToStaticMarkup(<MarkdownView source={source} />);
}

describe("MarkdownView", () => {
  test("renders headings, task lists and tables", () => {
    const markup = render(
      "# Feature\n\n- [x] plan\n- [ ] build\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
    );
    expect(markup).toContain("<h1>Feature</h1>");
    expect(markup).toContain('type="checkbox"');
    expect(markup).toContain("<table>");
  });

  test("never renders raw HTML", () => {
    const markup = render("before\n\n<script>alert(1)</script>\n\n<img src=x onerror=alert(1)>\n");
    expect(markup).not.toContain("<script");
    expect(markup).not.toContain("<img");
  });

  test("opens links in a new tab without an opener", () => {
    const markup = render("[docs](https://operator.untra.io)");
    expect(markup).toContain('target="_blank"');
    expect(markup).toContain('rel="noopener noreferrer"');
  });

  test("drops javascript links", () => {
    const markup = render(`[x](${SCRIPT_SCHEME}alert(1))`);
    expect(markup).not.toContain(SCRIPT_SCHEME);
  });
});
