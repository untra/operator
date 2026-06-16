// operator-launch-agent — POST /api/v1/tickets/{id}/launch
import { callOperator } from "./lib/operator-client.js";

class LaunchAgentTool {
  async execute(params, _inputData, _workflowEngine) {
    if (!params || !params.id) {
      return { success: false, result: null, error: "missing required param: id" };
    }
    return callOperator({
      params,
      path: `/api/v1/tickets/${encodeURIComponent(params.id)}/launch`,
      method: "POST",
      body: {
        delegator: params.delegator,
        model: params.model,
        wrapper: params.wrapper,
      },
    });
  }
}

export default new LaunchAgentTool();
