const fs = require("node:fs/promises");
const path = require("node:path");
const os = require("node:os");
const { generateFonts } = require("fantasticon");
const options = require("../.fantasticonrc.js");

async function main() {
  const check = process.argv.includes("--check");
  const temp = await fs.mkdtemp(path.join(os.tmpdir(), "operator-icons-"));
  try {
    await generateFonts({ ...options, outputDir: temp });
    const original = await fs.readFile("package.json", "utf8");
    const manifest = JSON.parse(original);
    const map = JSON.parse(await fs.readFile(path.join(temp, "operator-icons.json"), "utf8"));
    const icons = manifest.contributes.icons;
    for (const [id, contribution] of Object.entries(icons)) {
      const name = id.replace(/^operator-/, "");
      if (!Object.hasOwn(map, name)) throw new Error("Missing SVG for " + id);
      contribution.default.fontCharacter = "\\" + map[name].toString(16).toUpperCase();
    }
    const outputs = new Map();
    if (JSON.stringify(manifest) !== JSON.stringify(JSON.parse(original)))
      outputs.set("package.json", Buffer.from(JSON.stringify(manifest, null, 2) + "\n"));
    for (const file of await fs.readdir(temp)) {
      outputs.set(path.join(options.outputDir, file), await fs.readFile(path.join(temp, file)));
    }
    for (const asset of [manifest.icon, manifest.contributes.viewsContainers.activitybar[0].icon]) {
      await fs.access(asset);
    }
    if (!check) await fs.mkdir(options.outputDir, { recursive: true });
    for (const [file, contents] of outputs) {
      const current = await fs.readFile(file).catch(() => null);
      if (current?.equals(contents)) continue;
      if (check) throw new Error(file + " is stale. Run npm run generate:icons.");
      await fs.writeFile(file, contents);
    }
  } finally {
    await fs.rm(temp, { recursive: true, force: true });
  }
}
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
