import { deepStrictEqual, strictEqual, throws } from 'node:assert/strict';
import { describe, it } from 'node:test';
import { groupSelectionState, multiSelect, toggleGroupSelection } from '../cli/ui.ts';

describe('multiSelect', () => {
  it('throws when initialSelected length does not match items length', () => {
    throws(
      () => multiSelect(['a', 'b', 'c'], { labelFn: (x) => x, initialSelected: [true, false] }),
      /initialSelected length \(2\) must match items length \(3\)/,
    );
  });

  it('returns all items when stdin is not a TTY', async () => {
    const prevIsTTY = process.stdin.isTTY;
    // Force the non-TTY branch deterministically; some test runners leave
    // stdin.isTTY truthy, which would make multiSelect wait for key input.
    process.stdin.isTTY = false;
    try {
      const items = [{ name: 'a' }, { name: 'b' }];
      const result = await multiSelect(items, { labelFn: (x) => x.name });
      deepStrictEqual(result, items);
    } finally {
      process.stdin.isTTY = prevIsTTY;
    }
  });
});

describe('groupSelectionState', () => {
  it('returns all when every member is selected', () => {
    strictEqual(groupSelectionState([true, true, true], [0, 1, 2]), 'all');
  });

  it('returns none when no member is selected', () => {
    strictEqual(groupSelectionState([false, false, false], [0, 1, 2]), 'none');
  });

  it('returns partial when members are mixed', () => {
    strictEqual(groupSelectionState([true, false, true], [0, 1, 2]), 'partial');
  });

  it('only considers the given member indices', () => {
    // Indices 0,2 belong to the group; index 1 (selected) is outside it.
    strictEqual(groupSelectionState([false, true, false], [0, 2]), 'none');
    strictEqual(groupSelectionState([true, false, true], [0, 2]), 'all');
  });

  it('returns none for an empty group', () => {
    strictEqual(groupSelectionState([true, true], []), 'none');
  });
});

describe('toggleGroupSelection', () => {
  it('clears the group when all members are selected', () => {
    const selected = [true, true, true];
    toggleGroupSelection(selected, [0, 1, 2]);
    deepStrictEqual(selected, [false, false, false]);
  });

  it('selects the whole group when some members are off', () => {
    const selected = [true, false, true];
    toggleGroupSelection(selected, [0, 1, 2]);
    deepStrictEqual(selected, [true, true, true]);
  });

  it('selects the whole group when none are selected', () => {
    const selected = [false, false, false];
    toggleGroupSelection(selected, [0, 1, 2]);
    deepStrictEqual(selected, [true, true, true]);
  });

  it('only touches the given member indices', () => {
    const selected = [true, true, false, false];
    // Group covers indices 2,3; index 0,1 must be untouched.
    toggleGroupSelection(selected, [2, 3]);
    deepStrictEqual(selected, [true, true, true, true]);
  });
});
