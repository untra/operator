const fs = require("node:fs/promises");
const vscode = require("vscode");

async function runScenario() {
  let result;
  try {
    await require(process.env.OPERATOR_E2E_MODULE).run();
    result = { passed: true };
  } catch (error) {
    result = {
      passed: false,
      error: error instanceof Error ? `${error.name} in workbench scenario` : "Scenario failed",
    };
  }
  await fs.writeFile(process.env.OPERATOR_E2E_RESULT, JSON.stringify(result), {
    flag: "wx",
    mode: 0o600,
  });
  await vscode.commands.executeCommand("workbench.action.quit");
}

exports.activate = (context) => {
  if (vscode.env.remoteName) {
    return context.secrets;
  }
  setImmediate(() => {
    void runScenario();
  });
  return context.secrets;
};
