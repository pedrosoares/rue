# Virtual DOM Implementation for Rue

## Status: ✅ Implemented

The Virtual DOM feature has been fully implemented across 9 phases.

## What Was Built

### New Files Created

| File | Description |
|------|-------------|
| `core/src/node/mount.rs` | `mount_to_dom()` — creates real DOM from VNode tree, supporting parent/anchor insertion |
| `core/src/node/patch.rs` | Core diff/patch engine: `patch_node()`, `patch_element()`, `patch_attributes()`, `patch_event_listeners()`, text patching, element replacement |
| `core/src/node/children.rs` | Children reconciliation: `patch_children()` (key/unkeyed dispatch), `patch_unkeyed_children()`, `patch_keyed_children()`, `longest_increasing_subsequence()` with full test suite |

### Files Modified

| File | Changes |
|------|---------|
| `core/src/node/mod.rs` | Added `VNode::node_type()`, `VNode::same_type()`, `VNode::key()`, `VNode::dom_node()`, `VNode::set_dom_node()`. Added `listener_closures` field to `VElement` for event cleanup. Added `key()` to `VElementBuilder`. Declared new submodules. |
| `core/src/app.rs` | Rewrote `App::update()` to use `patch_node()` instead of destroying/recreating DOM. Added `old_vnode` and `mount_element` fields for diffing. |
| `core/src/lib.rs` | Minor — removed unused re-exports |

## Implementation Details

### Patch Engine Architecture

```
patch_node(old_vnode, new_vnode, parent, anchor)
  │
  ├── Both Empty → nothing to do
  ├── Old Empty → mount_to_dom(new)
  ├── New Empty → parent.remove_child(old.dom_node())
  ├── Different types → replace_node (remove old, mount new)
  └── Same type → dispatch by variant:
       ├── Element → patch_element
       │    ├── patch_attributes (HashMap diff: set/remove)
       │    ├── patch_event_listeners (Closure lifecycle)
       │    └── patch_children
       ├── Text → patch_text_node (set_text_content if changed)
       ├── Fragment → patch_children (transparent container)
       └── Dynamic → re-render, replace content
```

### Keyed Children Reconciliation

The implementation uses the **Longest Increasing Subsequence** (LIS) algorithm, same as Vue 3:

1. Build key→index maps for old and new children
2. Find which old children are reused (matched by key)
3. Compute LIS of reused children → these stay in place
4. Remove children that no longer exist
5. Move non-LIS children to new positions
6. Mount brand-new children
7. Recursively patch each matched old/new pair

The LIS algorithm is O(n log n) using patience sorting.

### Event Listener Lifecycle

Previously, Closures were created with `.forget()` — leaking memory. Now:

1. During mount, each event's `Closure` is stored in `VElement.listener_closures`
2. During patch, old closures are compared to new ones by reference
3. Unchanged listeners keep their closures
4. Removed/changed listeners have their closures properly dropped (after `remove_event_listener`)
5. New listeners get fresh closures stored on the new VElement

## Key Improvements Over Previous Implementation

| Aspect | Before | After |
|--------|--------|-------|
| DOM updates | Destroy + recreate everything | In-place patching of changes only |
| Scroll position | Lost on every update | Preserved |
| Input focus | Lost on every update | Preserved |
| Event listeners | Leaked (forget()) | Properly cleaned up on removal |
| List rendering | N/A (all nodes recreated) | Keyed reconciliation via LIS |
| Performance | O(full tree) per update | O(diff) per update |

## Tests

7 unit tests for the LIS algorithm in `children.rs`:
- Empty input
- Single element
- Already increasing
- Decreasing (any single element)
- Wikipedia example (0,8,4,12,2,10,6,14,1,9,5,13,3,11,7,15) — verifies LIS length = 6
- Filtered (with None entries)
- Filtered out-of-order

## Future Improvements

1. **Dynamic node fine-grained patching** — Store last inner VNode in Dynamic variant so sub-tree updates can diff instead of full replace
2. **Comment anchor fragments** — Use comment markers for precise fragment positioning
3. **Transition support** — Animate element insertion/removal
4. **v-memo / static hoisting** — Skip diffing for static subtrees
