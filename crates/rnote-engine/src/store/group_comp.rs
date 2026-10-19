// Imports
use super::{StrokeKey, StrokeStore};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "group_component")]
pub struct GroupComponent {
    #[serde(rename = "group_id")]
    pub group_id: u64,
}

/// Systems that are related to grouping strokes.
impl StrokeStore {
    /// The group id of the stroke, if it is part of a group.
    pub(crate) fn group_id_of(&self, key: StrokeKey) -> Option<u64> {
        self.group_components
            .get(key)
            .map(|group_comp| group_comp.group_id)
    }

    /// All keys that share the group of the given stroke, in chronological
    /// order, including the given key itself. Returns an empty vec if the
    /// stroke is ungrouped.
    ///
    /// Trashed strokes are filtered out, so a group shrinks when a member is
    /// deleted.
    pub(crate) fn group_members(&self, key: StrokeKey) -> Vec<StrokeKey> {
        let Some(group_id) = self.group_id_of(key) else {
            return Vec::new();
        };

        self.keys_sorted_chrono()
            .into_iter()
            .filter(|&k| {
                !(self.trashed(k).unwrap_or(false))
                    && self.group_id_of(k).is_some_and(|id| id == group_id)
            })
            .collect()
    }

    /// Puts all given strokes into one new group and returns the new group id.
    pub(crate) fn group_keys(&mut self, keys: &[StrokeKey]) -> u64 {
        self.group_id_counter += 1;
        let group_id = self.group_id_counter;

        for &key in keys {
            Arc::make_mut(&mut self.group_components).insert(
                key,
                Arc::new(GroupComponent { group_id }),
            );
        }

        group_id
    }

    /// Removes the group membership of the given strokes.
    pub(crate) fn ungroup_keys(&mut self, keys: &[StrokeKey]) {
        for &key in keys {
            Arc::make_mut(&mut self.group_components).remove(key);
        }
    }

    /// Re-creates the grouping for duplicated strokes under fresh group ids.
    ///
    /// `old_keys` and `new_keys` must be 1:1 aligned, as returned by
    /// `duplicate_selection`.
    pub(crate) fn regroup_duplicated(&mut self, old_keys: &[StrokeKey], new_keys: &[StrokeKey]) {
        // Map each duplicated group onto a fresh id.
        let mut id_map = std::collections::HashMap::<u64, u64>::new();

        for (&old_key, &new_key) in old_keys.iter().zip(new_keys.iter()) {
            let Some(old_id) = self.group_id_of(old_key) else {
                continue;
            };
            let new_id = *id_map.entry(old_id).or_insert_with(|| {
                self.group_id_counter += 1;
                self.group_id_counter
            });
            Arc::make_mut(&mut self.group_components).insert(
                new_key,
                Arc::new(GroupComponent { group_id: new_id }),
            );
        }
    }

    /// Expands a hit-tested key to all members of its group (click-select
    /// semantics: clicking one member selects the whole group). An ungrouped
    /// key expands to just itself.
    pub(crate) fn expand_hit_to_group(&self, key: StrokeKey) -> Vec<StrokeKey> {
        let mut members = self.group_members(key);
        if members.is_empty() {
            members.push(key);
        }
        members
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strokes::brushstroke::BrushStroke;
    use crate::strokes::Stroke;
    use rnote_compose::penpath::{Element, PenPath};
    use rnote_compose::style::Style;
    use std::time::Instant;

    fn brush_stroke_at(x: f64) -> Stroke {
        Stroke::BrushStroke(BrushStroke::from_penpath(
            PenPath {
                start: Element::new(p2d::math::Vector2::new(x, 0.0), 0.5),
                segments: Vec::new(),
            },
            Style::default(),
        ))
    }

    fn insert_at(store: &mut StrokeStore, x: f64) -> StrokeKey {
        let key = store.insert_stroke(brush_stroke_at(x), None);
        store.update_geometry_for_stroke(key);
        key
    }

    #[test]
    fn group_members_chrono_sorted_and_ungrouped_empty() {
        let mut store = StrokeStore::default();
        let k1 = insert_at(&mut store, 0.0);
        let k2 = insert_at(&mut store, 100.0);
        let k3 = insert_at(&mut store, 200.0);

        assert!(store.group_members(k1).is_empty());

        store.group_keys(&[k1, k2, k3]);

        assert_eq!(store.group_members(k2), vec![k1, k2, k3]);
        assert_eq!(store.group_id_of(k1), store.group_id_of(k3));
    }

    #[test]
    fn ungroup_keys_clears_membership() {
        let mut store = StrokeStore::default();
        let k1 = insert_at(&mut store, 0.0);
        let k2 = insert_at(&mut store, 100.0);
        store.group_keys(&[k1, k2]);

        store.ungroup_keys(&[k1, k2]);

        assert!(store.group_members(k1).is_empty());
        assert!(store.group_members(k2).is_empty());
    }

    #[test]
    fn expand_hit_to_group_includes_self_when_ungrouped() {
        let mut store = StrokeStore::default();
        let k1 = insert_at(&mut store, 0.0);

        assert_eq!(store.expand_hit_to_group(k1), vec![k1]);
    }

    #[test]
    fn delete_member_shrinks_group() {
        let mut store = StrokeStore::default();
        let k1 = insert_at(&mut store, 0.0);
        let k2 = insert_at(&mut store, 100.0);
        let k3 = insert_at(&mut store, 200.0);
        store.group_keys(&[k1, k2, k3]);

        // Trash k2 like the app does (trashed-filtered in group_members).
        store.set_trashed_keys(&[k2], true);
        assert_eq!(store.group_members(k1), vec![k1, k3]);

        // Remove it permanently; the group stays shrunk.
        store.remove_stroke(k2);
        assert_eq!(store.group_members(k3), vec![k1, k3]);
    }

    #[test]
    fn group_survives_history_undo_redo() {
        let mut store = StrokeStore::default();
        let k1 = insert_at(&mut store, 0.0);
        let k2 = insert_at(&mut store, 100.0);

        let _ = store.record(Instant::now());
        store.group_keys(&[k1, k2]);
        let _ = store.record(Instant::now());
        let grouped_id = store.group_id_of(k1);

        let _ = store.undo(Instant::now());
        assert!(store.group_members(k1).is_empty());

        let _ = store.redo(Instant::now());
        assert_eq!(store.group_id_of(k1), grouped_id);
        assert_eq!(store.group_id_of(k2), grouped_id);
    }

    #[test]
    fn duplicate_selection_regroups_with_fresh_ids() {
        let mut store = StrokeStore::default();
        let k1 = insert_at(&mut store, 0.0);
        let k2 = insert_at(&mut store, 100.0);
        store.group_keys(&[k1, k2]);
        let original_id = store.group_id_of(k1).unwrap();

        store.set_selected(k1, true);
        store.set_selected(k2, true);
        let dups = store.duplicate_selection();

        assert_eq!(dups.len(), 2);
        let dup_id = store.group_id_of(dups[0]).unwrap();
        assert_ne!(dup_id, original_id);
        assert_eq!(store.group_id_of(dups[1]), Some(dup_id));
        // Originals keep their group.
        assert_eq!(store.group_id_of(k1), Some(original_id));
    }
}
