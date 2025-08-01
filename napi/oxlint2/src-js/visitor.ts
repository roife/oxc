// Functions to compile 1 or more visitor objects into a single compiled visitor.
//
// # Visitor objects
//
// Visitor objects which are generated by rules' `create` functions have keys being either:
// * Name of an AST type. or
// * Name of an AST type postfixed with `:exit`.
//
// Each property value must be a function that handles that AST node.
//
// e.g.:
//
// ```
// {
//   BinaryExpression(node) {
//     // Do stuff on enter
//   },
//   'BinaryExpression:exit'(node) {
//     // Do stuff on exit
//   },
// }
// ```
//
// # Compiled visitor
//
// Compiled visitor is an array with `NODE_TYPES_COUNT` length, keyed by the ID of the node type.
// `NODE_TYPE_IDS_MAP` maps from type name to ID.
//
// Each element of compiled array is one of:
// * No visitor for this type = `null`.
// * Visitor for leaf node = visit function.
// * Visitor for non-leaf node = object of form `{ enter, exit }`,
//   where each property is either a visitor function or `null`.
//
// e.g.:
//
// ```
// [
//   // Leaf nodes
//   function(node) { /* do stuff */ },
//   // ...
//
//   // Non-leaf nodes
//   {
//     enter: function(node) { /* do stuff */ },
//     exit: null,
//   },
//   // ...
// ]
// ```
//
// # Object reuse
//
// No more than 1 compiled visitor exists at any time, so we reuse a single array `compiledVisitor`,
// rather than creating a new array for each file being linted.
//
// To compile visitors, call:
// * `initCompiledVisitor` once.
// * `addVisitorToCompiled` with each visitor object.
// * `finalizeCompiledVisitor` once.
//
// After this sequence of calls, `compiledVisitor` is ready to be used to walk the AST.
//
// We also recycle:
//
// * `{ enter, exit }` objects which are stored in compiled visitor.
// * Temporary arrays used to store multiple visit functions, which are merged into a single function
//   in `finalizeCompiledVisitor`.
//
// The aim is to reduce pressure on the garbage collector. All these recycled objects are long-lived
// and will graduate to "old space", which leaves as much capacity as possible in "new space"
// for objects created by user code in visitors. If ephemeral user-created objects all fit in new space,
// it will avoid full GC runs, which should greatly improve performance.

// TODO(camc314): we need to generate `.d.ts` file for this module.
// @ts-expect-error
import types from '../dist/parser/generated/lazy/types.cjs';

const { LEAF_NODE_TYPES_COUNT, NODE_TYPE_IDS_MAP, NODE_TYPES_COUNT } = types;

const { isArray } = Array;

// Compiled visitor used for visiting each file.
// Same array is reused for each file.
//
// Initialized with `.push()` to ensure V8 treats the array as "packed" (linear array),
// not "holey" (hash map). This is critical, as looking up elements in this array is a very hot path
// during AST visitation, and holey arrays are much slower.
// https://v8.dev/blog/elements-kinds
export const compiledVisitor: any[] = [];

for (let i = NODE_TYPES_COUNT; i !== 0; i--) {
  compiledVisitor.push(null);
}

// Arrays containing type IDs of types which have multiple visit functions defined for them.
//
// Filled with `0` initially up to maximum size they could ever need to be so:
// 1. These arrays never need to grow.
// 2. V8 treats these arrays as "PACKED_SMI_ELEMENTS".
const mergedLeafVisitorTypeIds: number[] = [],
  mergedEnterVisitorTypeIds: number[] = [],
  mergedExitVisitorTypeIds: number[] = [];

for (let i = LEAF_NODE_TYPES_COUNT; i !== 0; i--) {
  mergedLeafVisitorTypeIds.push(0);
}

for (let i = NODE_TYPES_COUNT - LEAF_NODE_TYPES_COUNT; i !== 0; i--) {
  mergedEnterVisitorTypeIds.push(0);
  mergedExitVisitorTypeIds.push(0);
}

mergedLeafVisitorTypeIds.length = 0;
mergedEnterVisitorTypeIds.length = 0;
mergedExitVisitorTypeIds.length = 0;

// `true` if `addVisitor` has been called with a visitor which visits at least one AST type
let hasActiveVisitors = false;

// Enter+exit object cache.
//
// `compiledVisitor` may contain many `{ enter, exit }` objects.
// Use this cache to reuse those objects across all visitor compilations.
//
// `enterExitObjectCacheNextIndex` is the index of first object in cache which is currently unused.
// It may point to the end of the cache array.
const enterExitObjectCache: any[] = [];
let enterExitObjectCacheNextIndex = 0;

function getEnterExitObject() {
  if (enterExitObjectCacheNextIndex < enterExitObjectCache.length) {
    return enterExitObjectCache[enterExitObjectCacheNextIndex++];
  }

  const enterExit: any = { enter: null, exit: null };
  enterExitObjectCache.push(enterExit);
  enterExitObjectCacheNextIndex++;
  return enterExit;
}

// Visit function arrays cache.
//
// During compilation, many arrays may be used temporarily to store multiple visit functions for same AST type.
// The functions in each array are merged into a single function in `finalizeCompiledVisitor`,
// after which these arrays aren't used again.
//
// Use this cache to reuse these arrays across each visitor compilation.
//
// `visitFnArrayCacheNextIndex` is the index of first array in cache which is currently unused.
// It may point to the end of the cache array.
const visitFnArrayCache: any[][] = [];
let visitFnArrayCacheNextIndex = 0;

function createVisitFnArray(visit1: any, visit2: any): any[] {
  if (visitFnArrayCacheNextIndex < visitFnArrayCache.length) {
    const arr = visitFnArrayCache[visitFnArrayCacheNextIndex++];
    arr.push(visit1, visit2);
    return arr;
  }

  const arr = [visit1, visit2];
  visitFnArrayCache.push(arr);
  visitFnArrayCacheNextIndex++;
  return arr;
}

/**
 * Initialize compiled visitor, ready for calls to `addVisitor`.
 */
export function initCompiledVisitor(): void {
  // Reset `compiledVisitor` array after previous compilation
  for (let i = 0; i < NODE_TYPES_COUNT; i++) {
    compiledVisitor[i] = null;
  }

  // Reset enter+exit objects which were used in previous compilation
  for (let i = 0; i < enterExitObjectCacheNextIndex; i++) {
    const enterExit = enterExitObjectCache[i];
    enterExit.enter = null;
    enterExit.exit = null;
  }
  enterExitObjectCacheNextIndex = 0;
}

/**
 * Add a visitor to compiled visitor.
 *
 * @param visitor - Visitor object
 */
export function addVisitorToCompiled(visitor: any): void {
  if (visitor === null || typeof visitor !== 'object') {
    throw new TypeError(
      'Visitor returned from `create` method must be an object',
    );
  }

  // Exit if is empty visitor
  const keys = Object.keys(visitor);
  if (keys.length === 0) return;

  hasActiveVisitors = true;

  // Populate visitors array from provided object
  for (let name of keys) {
    const visitFn = visitor[name];
    if (typeof visitFn !== 'function') {
      throw new TypeError(
        `'${name}' property of visitor object is not a function`,
      );
    }

    const isExit = name.endsWith(':exit');
    if (isExit) name = name.slice(0, -5);

    const typeId = NODE_TYPE_IDS_MAP.get(name);
    if (typeId === void 0) {
      throw new Error(`Unknown node type '${name}' in visitor object`);
    }

    const existing = compiledVisitor[typeId];
    if (typeId < LEAF_NODE_TYPES_COUNT) {
      // Leaf node - store just 1 function, not enter+exit pair
      if (existing === null) {
        compiledVisitor[typeId] = visitFn;
      } else if (isArray(existing)) {
        if (isExit) {
          existing.push(visitFn);
        } else {
          // Insert before last in array in case last was enter visit function from the current rule,
          // to ensure enter is called before exit.
          // It could also be either an enter or exit visitor function for another rule, but the order
          // rules are called in doesn't matter. We only need to make sure that a rule's exit visitor
          // isn't called before enter visitor *for that same rule*.
          existing.splice(existing.length - 1, 0, visitFn);
        }
      } else {
        // Same as above, enter visitor is put to front of list to make sure enter is called before exit
        compiledVisitor[typeId] = isExit
          ? [existing, visitFn]
          : [visitFn, existing];
        mergedLeafVisitorTypeIds.push(typeId);
      }
    } else {
      // Not leaf node - store enter+exit pair
      if (existing === null) {
        const enterExit = compiledVisitor[typeId] = getEnterExitObject();
        if (isExit) {
          enterExit.exit = visitFn;
        } else {
          enterExit.enter = visitFn;
        }
      } else if (isExit) {
        const enterExitObj = existing;
        let { exit } = enterExitObj;
        if (exit === null) {
          enterExitObj.exit = visitFn;
        } else if (isArray(exit)) {
          exit.push(visitFn);
        } else {
          enterExitObj.exit = createVisitFnArray(exit, visitFn);
          mergedExitVisitorTypeIds.push(typeId);
        }
      } else {
        const enterExitObj = existing;
        let { enter } = enterExitObj;
        if (enter === null) {
          enterExitObj.enter = visitFn;
        } else if (isArray(enter)) {
          enter.push(visitFn);
        } else {
          enterExitObj.enter = createVisitFnArray(enter, visitFn);
          mergedEnterVisitorTypeIds.push(typeId);
        }
      }
    }
  }
}

/**
 * Finalize compiled visitor.
 *
 * After calling this function, `compiledVisitor` is ready to be used to walk the AST.
 *
 * @returns {boolean} - `true` if compiled visitor visits at least 1 AST type
 */
export function finalizeCompiledVisitor() {
  if (hasActiveVisitors === false) return false;

  // Merge visit functions for node types which have multiple visitors from different rules,
  // or enter+exit functions for leaf nodes
  for (const typeId of mergedLeafVisitorTypeIds) {
    compiledVisitor[typeId] = mergeVisitFns(compiledVisitor[typeId]);
  }
  for (const typeId of mergedEnterVisitorTypeIds) {
    const enterExit = compiledVisitor[typeId];
    enterExit.enter = mergeVisitFns(enterExit.enter);
  }
  for (const typeId of mergedExitVisitorTypeIds) {
    const enterExit = compiledVisitor[typeId];
    enterExit.exit = mergeVisitFns(enterExit.exit);
  }

  // Reset state, ready for next time
  mergedLeafVisitorTypeIds.length = 0;
  mergedEnterVisitorTypeIds.length = 0;
  mergedExitVisitorTypeIds.length = 0;

  // Note: Visit function arrays have been emptied in `mergeVisitFns`, so all arrays in `visitFnArrayCache`
  // are now empty and ready for reuse. We just need to reset the index.
  visitFnArrayCacheNextIndex = 0;

  hasActiveVisitors = false;

  return true;
}

/**
 * Merge array of visit functions into a single function, which calls each of input functions in turn.
 *
 * The array passed is cleared (length set to 0), so the array can be reused.
 *
 * The merged function is statically defined and does not contain a loop, to hopefully allow
 * JS engine to heavily optimize it.
 *
 * `mergers` contains pre-defined functions to merge up to 5 visit functions.
 * Merger functions for merging more than 5 visit functions are created dynamically on demand.
 *
 * @param {Array<function>} visitFns - Array of visit functions
 * @returns {function} - Function which calls all of `visitFns` in turn.
 */
function mergeVisitFns(visitFns: Function[]): any {
  const numVisitFns = visitFns.length;

  // Get or create merger for merging `numVisitFns` functions
  let merger: any;
  if (mergers.length <= numVisitFns) {
    while (mergers.length < numVisitFns) {
      mergers.push(null);
    }
    merger = createMerger(numVisitFns);
    mergers.push(merger);
  } else {
    merger = mergers[numVisitFns];
    if (merger === null) merger = mergers[numVisitFns] = createMerger(numVisitFns);
  }

  // Merge functions
  const mergedFn = merger(...visitFns);

  // Empty `visitFns` array, so it can be reused
  visitFns.length = 0;

  return mergedFn;
}

/**
 * Create a merger function that merges `fnCount` functions.
 *
 * @param {number} fnCount - Number of functions to be merged
 * @returns {function} - Function to merge `fnCount` functions
 */
function createMerger(fnCount: number): any {
  const args = [];
  let body = 'return node=>{';
  for (let i = 1; i <= fnCount; i++) {
    args.push(`visit${i}`);
    body += `visit${i}(node);`;
  }
  body += '}';
  args.push(body);
  return new Function(...args) as any;
}

// Pre-defined mergers for merging up to 5 functions
const mergers = [
  null, // No merger for 0 functions
  null, // No merger for 1 function
  (visit1: any, visit2: any) => (node: any) => {
    visit1(node);
    visit2(node);
  },
  (visit1: any, visit2: any, visit3: any) => (node: any) => {
    visit1(node);
    visit2(node);
    visit3(node);
  },
  (visit1: any, visit2: any, visit3: any, visit4: any) => (node: any) => {
    visit1(node);
    visit2(node);
    visit3(node);
    visit4(node);
  },
  (visit1: any, visit2: any, visit3: any, visit4: any, visit5: any) => (node: any) => {
    visit1(node);
    visit2(node);
    visit3(node);
    visit4(node);
    visit5(node);
  },
];
