const { mkdirSync, readFileSync, writeFileSync } = require("node:fs");
const path = require("node:path");

const source = path.resolve(__dirname, "../../scripts/runtime/powershell");
const destination = path.resolve(__dirname, "../src/generated");
const scripts = {
  READ_CREDENTIAL_SCRIPT: "read-credential.ps1",
  PROTECT_REGISTRATION_DIRECTORY_SCRIPT: "protect-registration-directory.ps1",
};

function embedRuntimeScripts() {
  const exportsSource = Object.entries(scripts).map(([name, file]) => {
    const body = readFileSync(path.join(source, file), "utf8");
    return `export const ${name} = ${JSON.stringify(body)};`;
  });

  mkdirSync(destination, { recursive: true });
  writeFileSync(
    path.join(destination, "runtime-scripts.ts"),
    `// Generated from scripts/runtime/powershell.\n${exportsSource.join("\n")}\n`,
  );
}

module.exports = { embedRuntimeScripts };
