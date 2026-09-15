import { afterEach } from "vitest";

/**
 * Fails a story whose local fonts or images never resolved, rather than letting it
 * render as empty space. `decode()` waits for the load to settle and rejects only on
 * a genuine error, so a first-request image is not mistaken for a broken one.
 */
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
