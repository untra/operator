import * as vscode from "vscode";
import { discoverApiUrl, OperatorApiClient } from "./api-client";
import type { BatchRequest, WorkflowRecipe } from "./generated";

export async function createTicketsFromSelection(ticketsDir: string | undefined): Promise<void> {
  const api = new OperatorApiClient(await discoverApiUrl(ticketsDir));
  const editor = vscode.window.activeTextEditor;
  let text = editor?.document.getText(editor.selection);
  if (!text) {
    const [file] =
      (await vscode.window.showOpenDialog({
        canSelectMany: false,
        filters: { "Workflow records": ["csv", "json"] },
        openLabel: "Read records",
      })) ?? [];
    if (!file) {
      return;
    }
    text = Buffer.from(await vscode.workspace.fs.readFile(file)).toString("utf8");
  }
  const rows = await api.parseWorkflowRecords(text);
  const recipes = await api.workflowRecipes();
  const collections = await api.listCollections();
  const choose = await vscode.window.showQuickPick(
    [
      ...Object.entries(recipes).map(([label, recipe]) => ({
        label,
        description: "Saved recipe",
        recipe,
      })),
      ...collections.flatMap((collection) =>
        collection.types.map((workflow) => {
          const recipe: WorkflowRecipe = {
            collection: collection.name,
            workflow,
            source: "",
            key_columns: [],
            repeat: {},
            shared: {},
            columns: {},
          };
          return { label: workflow, description: collection.name, recipe };
        }),
      ),
    ],
    {
      title: "Create tickets from records",
      placeHolder: "Choose a saved mapping or match workflow input names",
    },
  );
  if (!choose) {
    return;
  }
  const recipe = choose.recipe;
  if (choose.description !== "Saved recipe") {
    const workflow = await api.getIssueType(recipe.workflow, recipe.collection);
    const headers = Array.from(new Set(rows.flatMap(Object.keys)));
    const fields = workflow.fields
      .filter((field) => field.user_editable)
      .map((field) => field.name);
    if (workflow.project_required && !fields.includes("project")) {
      fields.push("project");
    }
    for (const field of fields) {
      const choices = [
        ...headers.map((header) => ({ label: header, description: "Column", inputKind: "column" })),
        { label: "Use workflow default", inputKind: "default" },
        { label: "Shared value", inputKind: "shared" },
        { label: "Repeat each row", inputKind: "repeat" },
      ];
      choices.sort((a, b) => Number(b.label === field) - Number(a.label === field));
      const input = await vscode.window.showQuickPick(choices, { title: `Map ${field}` });
      if (!input) {
        return;
      }
      if (input.inputKind === "column") {
        recipe.columns[field] = input.label;
      }
      if (input.inputKind === "shared" || input.inputKind === "repeat") {
        const value = await vscode.window.showInputBox({
          title: field,
          prompt:
            input.inputKind === "repeat"
              ? "Repeat values separated by |"
              : "Value to use for every ticket",
        });
        if (value === undefined) {
          return;
        }
        if (input.inputKind === "repeat") {
          recipe.repeat[field] = value.split("|");
        } else {
          recipe.shared[field] = value;
        }
      }
    }
  }
  const request: BatchRequest = {
    campaign: null,
    tickets: [],
    dataset: { recipe, rows },
  };
  const preview = await api.previewWorkflowBatch(request);
  const lines = [
    `# ${choose.label}`,
    `${rows.length} rows / ${preview.tickets.length} planned / ${Object.keys(preview.previously_created).length} previously created`,
    ...preview.errors,
  ];
  for (const ticket of preview.tickets) {
    lines.push(`## ${ticket.ref}`, ...ticket.errors, ticket.markdown);
  }
  const document = await vscode.workspace.openTextDocument({
    language: "markdown",
    content: lines.join("\n\n"),
  });
  await vscode.window.showTextDocument(document);
  if (!preview.valid || !preview.tickets.length) {
    return;
  }
  const confirm = `Create ${preview.tickets.length} tickets`;
  if ((await vscode.window.showInformationMessage(choose.label, confirm)) !== confirm) {
    return;
  }
  const created = await api.createWorkflowBatch({ ...request, confirmation: preview.confirmation });
  void vscode.window.showInformationMessage(`Created ${created.tickets.length} tickets.`);
}
