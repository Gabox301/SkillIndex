import { deepStrictEqual, strictEqual, throws } from 'node:assert/strict';
import { describe, it } from 'node:test';
import { computeViewportStart, groupSelectionState, multiSelect, toggleGroupSelection } from '../cli/ui.ts';

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

describe('computeViewportStart', () => {
  it('returns 0 when everything fits in the viewport', () => {
    strictEqual(computeViewportStart({ cursor: 4, total: 5, height: 10, margin: 1, prevStart: 0 }), 0);
  });

  it('keeps start at 0 when the cursor is near the top', () => {
    strictEqual(computeViewportStart({ cursor: 0, total: 20, height: 5, margin: 1, prevStart: 0 }), 0);
    strictEqual(computeViewportStart({ cursor: 1, total: 20, height: 5, margin: 1, prevStart: 0 }), 0);
  });

  it('slides down to keep the cursor visible with margin', () => {
    // cursor 10, height 5, margin 1 => start = 10 - 5 + 1 + 1 = 7
    strictEqual(computeViewportStart({ cursor: 10, total: 20, height: 5, margin: 1, prevStart: 0 }), 7);
  });

  it('slides up to keep the cursor visible with margin', () => {
    // cursor 3 from a scrolled window => start = cursor - margin = 2
    strictEqual(computeViewportStart({ cursor: 3, total: 20, height: 5, margin: 1, prevStart: 7 }), 2);
  });

  it('clamps to the last full window at the bottom', () => {
    // total 20, height 5 => maxStart = 15
    strictEqual(computeViewportStart({ cursor: 19, total: 20, height: 5, margin: 1, prevStart: 0 }), 15);
  });

  it('does not move when the cursor stays within the window and margin', () => {
    // window [7,11], cursor 9 is comfortably inside => start unchanged
    strictEqual(computeViewportStart({ cursor: 9, total: 20, height: 5, margin: 1, prevStart: 7 }), 7);
  });
});
