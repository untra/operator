import { createReadStream, existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Plugin } from "vite";

/** Repo-root `icons/` is the canonical set; every surface serves it at `/icons/`. */
const ICONS_DIR = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../icons");
const MOUNT = "/icons";
const CONTENT_TYPE = "image/svg+xml";

/**
 * Serves the canonical icons at the production `/icons/` URL in the dev server and
 * the browser test run, and emits them into the static build.
 *
 * Storybook's own `staticDirs` cannot do this: a `to` subpath breaks test discovery in @storybook/addon-vitest.
 */
export function iconAssets(): Plugin {
  return {
    name: "operator-icon-assets",
    configureServer(server) {
      server.middlewares.use(MOUNT, (request, response, next) => {
        const name = path.basename(decodeURIComponent(request.url ?? "").split("?")[0] ?? "");
        const file = path.join(ICONS_DIR, name);
        if (!name.endsWith(".svg") || !existsSync(file)) {
          next();
          return;
        }
        response.setHeader("Content-Type", CONTENT_TYPE);
        createReadStream(file).pipe(response);
      });
    },
    generateBundle() {
      for (const name of readdirSync(ICONS_DIR).filter((entry) => entry.endsWith(".svg"))) {
        this.emitFile({
          type: "asset",
          fileName: `${MOUNT.slice(1)}/${name}`,
          source: readFileSync(path.join(ICONS_DIR, name)),
        });
      }
    },
  };
}
