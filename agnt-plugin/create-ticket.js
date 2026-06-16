// operator-create-ticket — POST /api/v1/tickets
import { callOperator } from "./lib/operator-client.js";

class CreateTicketTool {
  async execute(params, _inputData, _workflowEngine) {
    if (!params || !params.template) {
      return { success: false, result: null, error: "missing required param: template" };
    }
    return callOperator({
      params,
      path: "/api/v1/tickets",
      method: "POST",
      body: {
        template: params.template,
        project: params.project,
        summary: params.summary,
      },
    });
  }
}

export default new CreateTicketTool();
