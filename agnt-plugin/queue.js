// operator-queue-status — GET /api/v1/queue/status
import { callOperator } from "./lib/operator-client.js";

class QueueStatusTool {
  async execute(params, _inputData, _workflowEngine) {
    return callOperator({ params, path: "/api/v1/queue/status" });
  }
}

export default new QueueStatusTool();
