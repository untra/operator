"use strict";
/**
 * Launch dialogs for Operator VS Code extension
 *
 * QuickPick dialogs for selecting tickets and launch options.
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.showLaunchOptionsDialog = showLaunchOptionsDialog;
exports.showTicketPicker = showTicketPicker;
exports.showQuickModelPicker = showQuickModelPicker;
const vscode = __importStar(require("vscode"));
/**
 * Show launch options dialog
 */
async function showLaunchOptionsDialog(ticket, hasExistingSession) {
    // Model selection
    const modelItems = [
        {
            label: 'sonnet',
            description: 'Claude Sonnet (recommended)',
            model: 'sonnet',
        },
        {
            label: 'opus',
            description: 'Claude Opus (most capable)',
            model: 'opus',
        },
        {
            label: 'haiku',
            description: 'Claude Haiku (fastest)',
            model: 'haiku',
        },
    ];
    const modelChoice = await vscode.window.showQuickPick(modelItems, {
        title: `Launch ${ticket.id}: Select Model`,
        placeHolder: 'Choose the model to use',
    });
    if (!modelChoice) {
        return undefined;
    }
    // Options checkboxes
    const optionItems = [
        {
            label: 'YOLO Mode',
            description: 'Auto-accept all permission prompts',
            picked: false,
        },
    ];
    if (hasExistingSession) {
        optionItems.push({
            label: 'Resume Session',
            description: 'Continue from previous session',
            picked: true,
        });
    }
    const optionChoices = await vscode.window.showQuickPick(optionItems, {
        title: `Launch ${ticket.id}: Options`,
        placeHolder: 'Select launch options (Space to toggle)',
        canPickMany: true,
    });
    if (!optionChoices) {
        return undefined;
    }
    const selectedLabels = optionChoices.map((c) => c.label);
    return {
        model: modelChoice.model,
        yoloMode: selectedLabels.includes('YOLO Mode'),
        resumeSession: selectedLabels.includes('Resume Session'),
    };
}
/**
 * Show ticket picker for launch command
 */
async function showTicketPicker(tickets) {
    if (tickets.length === 0) {
        vscode.window.showInformationMessage('No tickets available');
        return undefined;
    }
    const items = tickets.map((t) => ({
        label: t.id,
        description: t.title,
        detail: `${t.type} - ${t.status}`,
        ticket: t,
    }));
    const choice = await vscode.window.showQuickPick(items, {
        title: 'Select Ticket to Launch',
        placeHolder: 'Choose a ticket',
        matchOnDescription: true,
        matchOnDetail: true,
    });
    return choice?.ticket;
}
/**
 * Show quick model picker (for fast launches)
 */
async function showQuickModelPicker() {
    const modelItems = [
        {
            label: '$(sparkle) Sonnet',
            description: 'Recommended balance of speed and capability',
            model: 'sonnet',
        },
        {
            label: '$(star-full) Opus',
            description: 'Most capable, slower',
            model: 'opus',
        },
        {
            label: '$(zap) Haiku',
            description: 'Fastest, simpler tasks',
            model: 'haiku',
        },
    ];
    const choice = await vscode.window.showQuickPick(modelItems, {
        title: 'Select Model',
        placeHolder: 'Choose model for launch',
    });
    return choice?.model;
}
//# sourceMappingURL=launch-dialog.js.map