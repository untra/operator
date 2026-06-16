// operator-run-step — the node type emitted by `operator workflow export --format agnt`.
//
// Each exported node represents one issuetype step and carries
// { ticket, step, prompt, ... } in its config. This tool reads `ticket` and asks
// Operator to run it via the launch endpoint. Operator sequences its own steps
// internally, so the per-step nodes are a faithful visualization of the ticket's
// shape; executing them drives the one underlying Operator ticket (the launch
// endpoint's relaunch path tolerates a ticket that is already in progress).
import { callOperator } from "./lib/operator-client.js";

class RunStepTool {
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
