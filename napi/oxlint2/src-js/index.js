import { createRequire } from 'node:module';
import { lint } from './bindings.js';
import { DATA_POINTER_POS_32, SOURCE_LEN_POS_32 } from './generated/constants.cjs';

// Import lazy visitor from `oxc-parser`.
// Use `require` not `import` as `oxc-parser` uses `require` internally,
// and need to make sure get same instance of modules as it uses internally,
// otherwise `TOKEN` here won't be same `TOKEN` as used within `oxc-parser`.
const require = createRequire(import.meta.url);
const { TOKEN } = require('../../parser/raw-transfer/lazy-common.js'),
  { Visitor, getVisitorsArr } = require('../../parser/raw-transfer/visitor.js'),
  walkProgram = require('../../parser/generated/lazy/walk.js');

// --------------------
// Plugin loading
// --------------------

// Absolute paths of plugins which have been loaded
const registeredPluginPaths = new Set();

// Rule objects for loaded rules.
// Indexed by `ruleId`, passed to `lintFile`.
const registeredRules = [];

/**
 * Load a plugin.
 *
 * Main logic is in separate function `loadPluginImpl`, because V8 cannot optimize functions
 * containing try/catch.
 *
 * @param {string} path - Absolute path of plugin file
 * @returns {string} - JSON result
 */
async function loadPlugin(path) {
  try {
    return await loadPluginImpl(path);
  } catch (error) {
    const errorMessage = 'message' in error && typeof error.message === 'string'
      ? error.message
      : 'An unknown error occurred';
    return JSON.stringify({ Failure: errorMessage });
  }
}

async function loadPluginImpl(path) {
  if (registeredPluginPaths.has(path)) {
    return JSON.stringify({ Failure: 'This plugin has already been registered' });
  }

  const { default: plugin } = await import(path);

  registeredPluginPaths.add(path);

  // TODO: Use a validation library to assert the shape of the plugin, and of rules
  const rules = [];
  const ret = {
    name: plugin.meta.name,
    offset: registeredRules.length,
    rules,
  };

  for (const [ruleName, rule] of Object.entries(plugin.rules)) {
    rules.push(ruleName);
    registeredRules.push(rule);
  }

  return JSON.stringify({ Success: ret });
}

// --------------------
// Running rules
// --------------------

// Buffers cache.
//
// All buffers sent from Rust are stored in this array, indexed by `bufferId` (also sent from Rust).
// Buffers are only added to this array, never removed, so no buffers will be garbage collected
// until the process exits.
const buffers = [];

// Diagnostics array. Reused for every file.
const diagnostics = [];

// Text decoder, for decoding source text from buffer
const textDecoder = new TextDecoder('utf-8', { ignoreBOM: true });

// Run rules on a file.
//
// TODO(camc314): why do we have to destructure here?
// In `./bindings.d.ts`, it doesn't indicate that we have to
// (typed as `(filePath: string, bufferId: number, buffer: Uint8Array | undefined | null, ruleIds: number[])`).
function lintFile([filePath, bufferId, buffer, ruleIds]) {
  // If new buffer, add it to `buffers` array. Otherwise, get existing buffer from array.
  // Do this before checks below, to make sure buffer doesn't get garbage collected when not expected
  // if there's an error.
  // TODO: Is this enough to guarantee soundness?
  if (buffer !== null) {
    const { buffer: arrayBuffer, byteOffset } = buffer;
    buffer.uint32 = new Uint32Array(arrayBuffer, byteOffset);
    buffer.float64 = new Float64Array(arrayBuffer, byteOffset);

    while (buffers.length <= bufferId) {
      buffers.push(null);
    }
    buffers[bufferId] = buffer;
  } else {
    buffer = buffers[bufferId];
  }

  if (typeof filePath !== 'string' || filePath.length === 0) {
    throw new Error('expected filePath to be a non-zero length string');
  }
  if (!Array.isArray(ruleIds) || ruleIds.length === 0) {
    throw new Error('Expected `ruleIds` to be a non-zero len array');
  }

  // Get visitors for this file from all rules
  const visitors = ruleIds.map(
    ruleId => registeredRules[ruleId].create(new Context(ruleId, filePath)),
  );
  const visitor = new Visitor(
    visitors.length === 1 ? visitors[0] : combineVisitors(visitors),
  );

  // Visit AST
  const programPos = buffer.uint32[DATA_POINTER_POS_32],
    sourceByteLen = buffer.uint32[SOURCE_LEN_POS_32];

  const sourceText = textDecoder.decode(buffer.subarray(0, sourceByteLen));
  const sourceIsAscii = sourceText.length === sourceByteLen;
  const ast = { buffer, sourceText, sourceByteLen, sourceIsAscii, nodes: new Map(), token: TOKEN };

  walkProgram(programPos, ast, getVisitorsArr(visitor));

  // Send diagnostics back to Rust
  const ret = JSON.stringify(diagnostics);
  diagnostics.length = 0;
  return ret;
}

/**
 * Combine multiple visitor objects into a single visitor object.
 * @param {Array<Object>} visitors - Array of visitor objects
 * @returns {Object} - Combined visitor object
 */
function combineVisitors(visitors) {
  const combinedVisitor = {};
  for (const visitor of visitors) {
    for (const nodeType of Object.keys(visitor)) {
      if (!(nodeType in combinedVisitor)) {
        combinedVisitor[nodeType] = function(node) {
          for (const v of visitors) {
            if (typeof v[nodeType] === 'function') {
              v[nodeType](node);
            }
          }
        };
      }
    }
  }

  return combinedVisitor;
}

/**
 * Context class.
 *
 * A `Context` is passed to each rule's `create` function.
 */
class Context {
  // Rule ID. Index into `registeredRules` array.
  #ruleId;

  /**
   * @constructor
   * @param {number} ruleId - Rule ID
   * @param {string} filePath - Absolute path of file being linted
   */
  constructor(ruleId, filePath) {
    this.#ruleId = ruleId;
    this.physicalFilename = filePath;
  }

  /**
   * Report error.
   * @param {Object} diagnostic - Diagnostic object
   * @param {string} diagnostic.message - Error message
   * @param {Object} diagnostic.loc - Node or loc object
   * @param {number} diagnostic.loc.start - Start range of diagnostic
   * @param {number} diagnostic.loc.end - End range of diagnostic
   * @returns {undefined}
   */
  report(diagnostic) {
    diagnostics.push({
      message: diagnostic.message,
      loc: { start: diagnostic.node.start, end: diagnostic.node.end },
      externalRuleId: this.#ruleId,
    });
  }
}

// --------------------
// Run linter
// --------------------

// Call Rust, passing `loadPlugin` and `lintFile` as callbacks
const success = await lint(loadPlugin, lintFile);

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;
