// operator-export-workflow — POST /api/v1/tickets/{id}/workflow-export?format=...
import { callOperator } from "./lib/operator-client.js";

class ExportWorkflowTool {
  async execute(params, _inputData, _workflowEngine) {
    if (!params || !params.id) {
      return { success: false, result: null, error: "missing required param: id" };
    }
    const format = params.format || "agnt";
    return callOperator({
      params,
      path: `/api/v1/tickets/${encodeURIComponent(params.id)}/workflow-export?format=${encodeURIComponent(format)}`,
      method: "POST",
    });
  }
}

export default new ExportWorkflowTool();
