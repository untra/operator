// operator-run-step - node type emitted by `operator workflow export --format agnt`.
// One node per issuetype step. Launch drives the one Operator ticket; Operator sequences the step.
import { callOperator } from "./lib/operator-client.js";

class RunStepTool {
  constructor() {
    this.name = "operator-run-step";
  }
  async execute(params, _inputData, _workflowEngine) {
    const ticket = params && (params.ticket || params.id);
    if (!ticket) {
      return { success: false, result: null, error: "missing required param: ticket" };
    }
    return callOperator({
      params,
      path: `/api/v1/tickets/${encodeURIComponent(ticket)}/launch`,
      method: "POST",
      body: { model: params.model },
    });
  }
}

export default new RunStepTool();
