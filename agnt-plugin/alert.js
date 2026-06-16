// operator-alert — POST /api/v1/alerts
import { callOperator } from "./lib/operator-client.js";

class AlertTool {
  async execute(params, _inputData, _workflowEngine) {
    if (!params || !params.message) {
      return { success: false, result: null, error: "missing required param: message" };
    }
    return callOperator({
      params,
      path: "/api/v1/alerts",
      method: "POST",
      body: {
        source: params.source || "agnt",
        message: params.message,
        severity: params.severity || "S2",
        project: params.project,
      },
    });
  }
}

export default new AlertTool();
