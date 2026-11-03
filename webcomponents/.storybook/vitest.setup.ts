import { afterEach, expect } from "vitest";

afterEach(async () => {
  await document.fonts.ready;
  const broken: string[] = [];
  await Promise.all(
    [...document.images].map(async (image) => {
      try {
        await image.decode();
      } catch {
        broken.push(image.src);
      }
    }),
  );
  if (broken.length > 0) {
    throw new Error(`story assets failed to load: ${broken.join(", ")}`);
  }
});

afterEach(() => {
  const roundingTolerance = 1;
  expect(document.documentElement.scrollWidth, "Story overflows the viewport").toBeLessThanOrEqual(
    window.innerWidth + roundingTolerance,
  );
});
