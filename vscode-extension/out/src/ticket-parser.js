"use strict";
/**
 * Ticket metadata parser for YAML frontmatter
 *
 * Parses ticket markdown files to extract session IDs and other
 * metadata stored in YAML frontmatter.
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
exports.parseTicketMetadata = parseTicketMetadata;
exports.parseTicketContent = parseTicketContent;
exports.getCurrentSessionId = getCurrentSessionId;
const fs = __importStar(require("fs/promises"));
/**
 * Parse YAML frontmatter from ticket markdown file
 */
async function parseTicketMetadata(filePath) {
    try {
        const content = await fs.readFile(filePath, 'utf-8');
        return parseTicketContent(content);
    }
    catch {
        return null;
    }
}
/**
 * Parse ticket content string (for testing)
 */
function parseTicketContent(content) {
    // Extract YAML frontmatter between --- markers
    const match = content.match(/^---\n([\s\S]*?)\n---/);
    if (!match) {
        return null;
    }
    const yaml = match[1];
    const metadata = {
        id: '',
        status: '',
        step: '',
        priority: '',
        project: '',
    };
    // Simple YAML parsing for known fields
    for (const line of yaml.split('\n')) {
        // Skip empty lines and lines that start with whitespace (nested)
        if (!line.trim() || line.startsWith(' ') || line.startsWith('\t')) {
            continue;
        }
        const colonIndex = line.indexOf(':');
        if (colonIndex === -1) {
            continue;
        }
        const key = line.slice(0, colonIndex).trim();
        const value = line.slice(colonIndex + 1).trim();
        switch (key) {
            case 'id':
                metadata.id = value;
                break;
            case 'status':
                metadata.status = value;
                break;
            case 'step':
                metadata.step = value;
                break;
            case 'priority':
                metadata.priority = value;
                break;
            case 'project':
                metadata.project = value;
                break;
            case 'worktree_path':
                metadata.worktree_path = value;
                break;
            case 'branch':
                metadata.branch = value;
                break;
        }
    }
    // Parse sessions block (indented key-value pairs under 'sessions:')
    const sessionsMatch = yaml.match(/sessions:\s*\n((?:\s{2}\S+:.*\n?)+)/);
    if (sessionsMatch) {
        metadata.sessions = {};
        for (const line of sessionsMatch[1].split('\n')) {
            const sessionMatch = line.match(/^\s+(\S+):\s*(.+)$/);
            if (sessionMatch) {
                metadata.sessions[sessionMatch[1]] = sessionMatch[2].trim();
            }
        }
    }
    return metadata;
}
/**
 * Get current session ID from ticket metadata
 *
 * Tries the current step first, then falls back to 'initial'
 */
function getCurrentSessionId(metadata) {
    if (!metadata.sessions) {
        return undefined;
    }
    // Try current step first
    if (metadata.step && metadata.sessions[metadata.step]) {
        return metadata.sessions[metadata.step];
    }
    // Fall back to 'initial'
    return metadata.sessions['initial'];
}
//# sourceMappingURL=ticket-parser.js.map