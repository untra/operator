import * as path from 'node:path';
import Mocha from 'mocha';
import { glob } from 'glob';

/**
 * Minimal type for the NYC coverage tool.
 * NYC does not ship its own types, so we define the subset we use.
 */
interface NycInstance {
  reset(): Promise<void>;
  wrap(): Promise<void>;
  exclude: { shouldInstrument(file: string): boolean };
  writeCoverageFile(): Promise<void>;
  report(): Promise<void>;
}

type NycConstructor = new (options: Record<string, unknown>) => NycInstance;

// NYC for coverage instrumentation inside VS Code process
// eslint-disable-next-line @typescript-eslint/no-require-imports, @typescript-eslint/no-var-requires
const NYC: NycConstructor = require('nyc') as NycConstructor;

export async function run(): Promise<void> {
  const testsRoot = path.resolve(__dirname, '.');
  const workspaceRoot = path.join(__dirname, '..', '..', '..');

  // Setup NYC for coverage inside VS Code process
  const nyc: NycInstance = new NYC({
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

  // Clear all out/ module cache so tests load fresh through NYC's hooked require.
  // Extension activation loads src modules before NYC hooks are set up.
  // By clearing both src and test caches, when Mocha loads test files they'll
  // require instrumented src modules through NYC's hooks.
  const outDir = path.join(workspaceRoot, 'out');
  for (const key of Object.keys(require.cache)) {
    if (key.startsWith(outDir) && !key.includes('node_modules')) {
      delete require.cache[key];
    }
  }

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
    mocha.run((failures) => {
      // Write coverage data and report asynchronously, then resolve/reject
      void (async () => {
        // Load any src modules not yet required by tests for `all` coverage.
        // This ensures files like extension.ts appear in the report even if
        // no test imports them directly.
        const srcGlob = await glob('out/src/**/*.js', {
          cwd: workspaceRoot,
          ignore: ['out/src/generated/**'],
        });
        for (const f of srcGlob) {
          const fullPath = path.join(workspaceRoot, f);
          if (!require.cache[fullPath]) {
            try { require(fullPath); } catch { /* ok — some modules need VS Code context */ }
          }
        }

        await nyc.writeCoverageFile();

        // Generate and display coverage report
        console.log('\n--- Coverage Report ---');
        await captureStdout(() => nyc.report());

        if (failures > 0) {
          reject(new Error(`${failures} tests failed.`));
        } else {
          resolve();
        }
      })();
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
