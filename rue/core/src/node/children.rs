use std::collections::HashMap;

use super::patch::patch_node;
use super::mount::mount_to_dom;
use super::VNode;

// ---------------------------------------------------------------------------
// Children reconciliation entry point
// ---------------------------------------------------------------------------

/// Patch the children list of a DOM node.
///
/// Decides whether to use keyed or un-keyed reconciliation based on whether
/// the children have `key` attributes.
pub fn patch_children(
    old_children: &[VNode],
    new_children: &[VNode],
    parent: &web_sys::Node,
) {
    let old_has_keys = old_children.iter().any(|c| c.key().is_some());
    let new_has_keys = new_children.iter().any(|c| c.key().is_some());

    if old_has_keys && new_has_keys {
        patch_keyed_children(old_children, new_children, parent);
    } else {
        patch_unkeyed_children(old_children, new_children, parent);
    }
}

// ---------------------------------------------------------------------------
// Un-keyed children (position-based)
// ---------------------------------------------------------------------------

/// Patch children by position — simpler but less efficient for reordering.
fn patch_unkeyed_children(
    old_children: &[VNode],
    new_children: &[VNode],
    parent: &web_sys::Node,
) {
    let old_len = old_children.len();
    let new_len = new_children.len();
    let common_len = old_len.min(new_len);

    // Patch common prefix
    for i in 0..common_len {
        patch_node(&old_children[i], &new_children[i], parent, None);
    }

    if new_len > old_len {
        // Add new children at the end
        for i in old_len..new_len {
            mount_to_dom(&new_children[i], parent, None);
        }
    } else if old_len > new_len {
        // Remove extra old children
        for i in new_len..old_len {
            if let Some(node) = old_children[i].dom_node() {
                let _ = parent.remove_child(&node);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Keyed children — uses Longest Increasing Subsequence
// ---------------------------------------------------------------------------

/// Patch children using keys for optimal reordering.
///
/// Algorithm (inspired by Vue 3):
/// 1. Build key→index maps for old and new children
/// 2. Find which old children can be reused (matched by key)
/// 3. Compute longest increasing subsequence of reusable children
///    → these are the nodes that stay in place
/// 4. Remove children that no longer exist
/// 5. Move and insert children efficiently
/// 6. Recursively patch each matched pair
fn patch_keyed_children(
    old_children: &[VNode],
    new_children: &[VNode],
    parent: &web_sys::Node,
) {
    // Build key→index map for old children
    let old_key_to_idx: HashMap<&str, usize> = old_children
        .iter()
        .enumerate()
        .filter_map(|(i, c)| c.key().map(|k| (k, i)))
        .collect();

    // Build key→index map for new children
    let new_key_to_idx: HashMap<&str, usize> = new_children
        .iter()
        .enumerate()
        .filter_map(|(i, c)| c.key().map(|k| (k, i)))
        .collect();

    // Phase 1: Determine which old children are reused, and at what new index.
    // `new_idx_for_old` maps old_index → new_index for reused children.
    let mut new_idx_for_old: Vec<Option<usize>> = vec![None; old_children.len()];
    let mut old_to_new_key: Vec<Option<&str>> = vec![None; old_children.len()];

    for (old_idx, old_child) in old_children.iter().enumerate() {
        if let Some(key) = old_child.key() {
            if let Some(&new_idx) = new_key_to_idx.get(key) {
                new_idx_for_old[old_idx] = Some(new_idx);
                old_to_new_key[old_idx] = Some(key);
            }
        }
    }

    // Phase 2: Compute the longest increasing subsequence of `new_idx_for_old`
    // (filtering out the None entries). These are the children that can stay
    // in their current relative positions.
    let lis_indices = longest_increasing_subsequence_filtered(&new_idx_for_old);

    // Phase 3: Build a sequence of patch operations.
    // We iterate through new children and determine what to do:
    //   - If a child existed in old and is in the LIS → patch in-place, keep position
    //   - If a child existed in old but is NOT in LIS → patch in-place, but it needs to move
    //   - If a child is new → mount it

    // First pass: remove children that no longer exist in new
    for (old_idx, old_child) in old_children.iter().enumerate() {
        if new_idx_for_old[old_idx].is_none() {
            if let Some(node) = old_child.dom_node() {
                let _ = parent.remove_child(&node);
            }
        }
    }

    // Second pass: process new children in order, patching or mounting as needed.
    // We maintain a cursor position in the DOM to handle inserts correctly.
    // This is a simplified version — for full correctness we'd use DOM anchors.

    // We use a reference child approach: for each new child, we ensure it
    // appears before the next old child that hasn't been processed yet.
    // The LIS tells us which old children are already in the right order.

    let mut lis_set: Vec<bool> = vec![false; old_children.len()];
    for &idx in &lis_indices {
        lis_set[idx] = true;
    }

    // We'll collect the operations and apply them.
    // For simplicity, we do a two-pass: first patch/move, then insert new.

    for (new_idx, new_child) in new_children.iter().enumerate() {
        if let Some(key) = new_child.key() {
            if let Some(&old_idx) = old_key_to_idx.get(key) {
                // This child existed in old — patch it
                let old_child = &old_children[old_idx];
                let is_stable = lis_set[old_idx];

                if is_stable {
                    // Child is already in correct position — just patch
                    patch_node(old_child, new_child, parent, None);
                } else {
                    // Child needs to be moved to its new position
                    // We insert before the next reference node
                    let ref_node = find_reference_node(new_children, new_idx, parent);
                    patch_node(old_child, new_child, parent, ref_node.as_ref());
                }
            } else {
                // Brand new child — mount it
                let ref_node = find_reference_node(new_children, new_idx, parent);
                mount_to_dom(new_child, parent, ref_node.as_ref());
            }
        } else {
            // No key — treat as new and mount (simplified)
            let ref_node = find_reference_node(new_children, new_idx, parent);
            mount_to_dom(new_child, parent, ref_node.as_ref());
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Find a reference node to insert `new_child` before.
/// This looks at the next new child that already exists in the DOM.
fn find_reference_node(
    new_children: &[VNode],
    new_idx: usize,
    _parent: &web_sys::Node,
) -> Option<web_sys::Node> {
    // Look for the next new child after `new_idx` that has a DOM node
    for next in (new_idx + 1)..new_children.len() {
        if let Some(node) = new_children[next].dom_node() {
            return Some(node);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Longest Increasing Subsequence
// ---------------------------------------------------------------------------

/// Compute the longest increasing subsequence of an array, returning the
/// **indices** (into `arr`) of elements that form the LIS.
///
/// Uses the patience sorting algorithm (O(n log n)) and then reconstructs
/// the actual subsequence indices.
pub fn longest_increasing_subsequence(arr: &[usize]) -> Vec<usize> {
    if arr.is_empty() {
        return vec![];
    }

    // `tails[i]` = last element of the best subsequence of length (i+1)
    // We store *indices* into `arr`.
    let mut tails: Vec<usize> = vec![];
    // `prev[i]` = index of the previous element in the best subsequence ending at i
    let mut prev: Vec<isize> = vec![-1; arr.len()];

    for (i, &val) in arr.iter().enumerate() {
        // Binary search for the first tail >= val
        let pos = match tails.binary_search_by(|&tail_idx| arr[tail_idx].cmp(&val)) {
            Ok(p) => p,  // exact match — replace
            Err(p) => p, // insertion point
        };

        if pos == tails.len() {
            tails.push(i);
        } else {
            tails[pos] = i;
        }

        // Set prev pointer (if pos > 0, link to tails[pos-1])
        if pos > 0 {
            prev[i] = tails[pos - 1] as isize;
        }
    }

    // Reconstruct the subsequence from the tails
    let mut result: Vec<usize> = vec![];
    if let Some(&last) = tails.last() {
        let mut idx = last;
        loop {
            result.push(idx);
            if prev[idx] == -1 {
                break;
            }
            idx = prev[idx] as usize;
        }
        result.reverse();
    }

    result
}

/// Compute LIS but only for elements where `arr[i]` is `Some(usize)`,
/// ignoring `None` entries. Returns the indices (into `arr`) of the LIS.
fn longest_increasing_subsequence_filtered(arr: &[Option<usize>]) -> Vec<usize> {
    // Collect the values of Some entries with their original indices
    let filtered: Vec<(usize, usize)> = arr
        .iter()
        .enumerate()
        .filter_map(|(i, &opt)| opt.map(|v| (i, v)))
        .collect();

    if filtered.is_empty() {
        return vec![];
    }

    let values: Vec<usize> = filtered.iter().map(|(_, v)| *v).collect();
    let lis = longest_increasing_subsequence(&values);

    // Map back to original indices
    lis.iter().map(|&idx| filtered[idx].0).collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lis_empty() {
        assert_eq!(longest_increasing_subsequence(&[]), Vec::<usize>::new());
    }

    #[test]
    fn test_lis_single() {
        assert_eq!(longest_increasing_subsequence(&[5]), vec![0]);
    }

    #[test]
    fn test_lis_increasing() {
        assert_eq!(longest_increasing_subsequence(&[1, 2, 3, 4]), vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_lis_decreasing() {
        let result = longest_increasing_subsequence(&[4, 3, 2, 1]);
        // Any single element is a valid LIS of length 1
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_lis_mixed() {
        // From Wikipedia: 0, 8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15
        // LIS: 0, 2, 6, 9, 11, 15 (values), or 0, 4, 6, 9, 13, 15 (indices)
        let arr = [0, 8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15];
        let result = longest_increasing_subsequence(&arr);
        // Verify it's increasing
        for w in result.windows(2) {
            assert!(arr[w[0]] < arr[w[1]], "LIS should be increasing");
        }
        // The known LIS length is 6
        assert_eq!(result.len(), 6);
    }

    #[test]
    fn test_lis_filtered() {
        let arr = vec![Some(0), None, Some(1), None, Some(2)];
        let result = longest_increasing_subsequence_filtered(&arr);
        assert_eq!(result, vec![0, 2, 4]);
    }

    #[test]
    fn test_lis_filtered_out_of_order() {
        let arr = vec![Some(2), None, Some(0), Some(1)];
        let result = longest_increasing_subsequence_filtered(&arr);
        assert_eq!(result.len(), 2);
        // Should be indices 2, 3 (values 0, 1)
        assert!(result.contains(&2));
        assert!(result.contains(&3));
    }
}
