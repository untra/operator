import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import Mocha from 'mocha';
import { glob } from 'glob';

// NYC for coverage instrumentation inside VS Code process
// eslint-disable-next-line @typescript-eslint/no-require-imports
const NYC = require('nyc');

export async function run(): Promise<void> {
  const testsRoot = path.resolve(__dirname, '.');
  const workspaceRoot = path.join(__dirname, '..', '..', '..');

  // Create a minimal config.toml for tests to prevent activation from hanging
  // when showConfigMissingNotification() is triggered
  const configDir = path.join(os.homedir(), '.config', 'operator');
  const configPath = path.join(configDir, 'config.toml');
  let createdConfig = false;

  if (!fs.existsSync(configPath)) {
    try {
      fs.mkdirSync(configDir, { recursive: true });
      fs.writeFileSync(configPath, '# Test config\nworking_directory = "/tmp"\n');
      createdConfig = true;
    } catch {
      // Failed to create config, tests may timeout
      console.warn('Warning: Could not create test config.toml');
    }
  }

  // Setup NYC for coverage inside VS Code process
  const nyc = new NYC({
    cwd: workspaceRoot,
    reporter: ['text', 'lcov', 'html'],
    all: true,
    silent: false,
    instrument: true,
    hookRequire: true,
    hookRunInContext: true,
    hookRunInThisContext: true,
    include: ['out/src/**/*.js'],
    exclude: ['out/test/**', 'out/src/generated/**'],
    reportDir: path.join(workspaceRoot, 'coverage'),
  });

  await nyc.reset();
  await nyc.wrap();

  // Re-require already-loaded modules for instrumentation
  Object.keys(require.cache)
    .filter((f) => nyc.exclude.shouldInstrument(f))
    .forEach((m) => {
      delete require.cache[m];
      require(m);
    });

  // Create the mocha test
  const mocha = new Mocha({
    ui: 'tdd',
    color: true,
  });

  const files = await glob('**/**.test.js', { cwd: testsRoot });

  // Add files to the test suite
  files.forEach((f) => mocha.addFile(path.resolve(testsRoot, f)));

  // Run the mocha test
  return new Promise((resolve, reject) => {
    mocha.run(async (failures) => {
      // Write coverage data
      await nyc.writeCoverageFile();

      // Generate and display coverage report
      console.log('\n--- Coverage Report ---');
      await captureStdout(nyc.report.bind(nyc));

      // Clean up test config if we created it
      if (createdConfig) {
        try {
          fs.unlinkSync(configPath);
        } catch {
          // Ignore cleanup errors
        }
      }

      if (failures > 0) {
        reject(new Error(`${failures} tests failed.`));
      } else {
        resolve();
      }
    });
  });
}

async function captureStdout(fn: () => Promise<void>): Promise<string> {
  const originalWrite = process.stdout.write.bind(process.stdout);
  let buffer = '';
  process.stdout.write = (s: string): boolean => {
    buffer += s;
    originalWrite(s);
    return true;
  };
  await fn();
  process.stdout.write = originalWrite;
  return buffer;
}
