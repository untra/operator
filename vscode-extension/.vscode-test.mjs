import path from 'node:path';
import { defineConfig } from '@vscode/test-cli';

export default defineConfig([{
  label: 'unitTests',
  files: 'out/test/**/*.test.js',
  version: 'stable',
  useInstallation: { fromPath: process.env.OPERATOR_E2E_VSCODE_PATH },
  launchArgs: process.env.OPERATOR_TEST_PROFILE_DIR ? [
    `--user-data-dir=${path.join(process.env.OPERATOR_TEST_PROFILE_DIR, 'p')}`,
    `--extensions-dir=${path.join(process.env.OPERATOR_TEST_PROFILE_DIR, 'e')}`,
  ] : [],
  mocha: {
    ui: 'tdd',
    timeout: 20000
  }
}]);
