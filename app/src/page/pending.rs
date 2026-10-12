//! What the results area shows while a search is pending: a pure state
//! function, so the rule is tested without a browser.

use crate::search::SearchOutcome;

/// The class that dims the results while a new search is pending.
pub(super) const DIM: &str = "opacity-60 transition-opacity";

/// What the results area shows and whether it is busy.
#[derive(Debug, PartialEq)]
pub(super) struct Area<'a> {
    /// The outcome to render, `None` for nothing.
    pub(super) outcome: Option<&'a SearchOutcome>,
    /// A search is pending: `aria-busy` and the dim class.
    pub(super) busy: bool,
}

impl Area<'_> {
    /// The `aria-busy` attribute: `true` while busy, absent otherwise.
    pub(super) fn aria_busy(&self) -> Option<&'static str> {
        self.busy.then_some("true")
    }

    /// The area's class: dimmed while busy, nothing otherwise.
    pub(super) fn class(&self) -> &'static str {
        if self.busy { DIM } else { "" }
    }
}

/// The area for the outcome the `Resource` holds (`held`: the previous
/// search's while a new one is pending) and whether one is `pending`.
/// Pending, only stale hits stay (dimmed, `aria-busy`): a stale failure is
/// hidden, as today's site hides it. Settled, the held outcome is the
/// landed one and replaces whatever was stale.
pub(super) fn pending_view(held: Option<&SearchOutcome>, pending: bool) -> Area<'_> {
    let outcome = match held {
        Some(SearchOutcome::Hits(_)) | None => held,
        Some(_) if pending => None,
        Some(_) => held,
    };
    Area {
        outcome,
        busy: pending,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::SearchResponse;

    fn hits(query: &str) -> SearchOutcome {
        SearchOutcome::Hits(SearchResponse::empty(query))
    }

    /// Settled: the outcome, not busy, no `aria-busy`, no dim.
    #[test]
    fn a_settled_area_is_neither_busy_nor_dim() {
        let held = hits("a");
        let area = pending_view(Some(&held), false);
        assert_eq!(area.outcome, Some(&held));
        assert!(!area.busy);
        assert_eq!(area.aria_busy(), None);
        assert!(!area.class().contains(DIM), "{}", area.class());
    }

    /// Pending: the stale hits stay, `aria-busy="true"` and dimmed.
    #[test]
    fn pending_keeps_the_stale_hits_busy_and_dim() {
        let held = hits("a");
        let area = pending_view(Some(&held), true);
        assert_eq!(area.outcome, Some(&held));
        assert!(area.busy);
        assert_eq!(area.aria_busy(), Some("true"));
        assert!(area.class().contains(DIM), "{}", area.class());
    }

    /// An error is not kept as stale: while pending it is hidden (today's
    /// site: an error or "starting" hides the previous result).
    #[test]
    fn pending_hides_a_stale_failure() {
        for held in [
            SearchOutcome::Cold { retry_after: None },
            SearchOutcome::RateLimited {
                retry_after: Some(3),
            },
            SearchOutcome::Failed,
        ] {
            let area = pending_view(Some(&held), true);
            assert_eq!(area.outcome, None, "{held:?}");
            assert!(area.busy, "{held:?}");
        }
    }

    /// Δ3 (review 005, round 2), ruled as intended: a stale cold,
    /// rate-limited or failed alert is not kept while the next search is
    /// pending — the area is empty, busy and dim — and the same outcome is
    /// shown once it is the settled one (the alert is not lost, only held
    /// back while it is stale).
    #[test]
    fn a_stale_alert_is_not_kept_while_pending() {
        for held in [
            SearchOutcome::Cold {
                retry_after: Some(7),
            },
            SearchOutcome::RateLimited { retry_after: None },
            SearchOutcome::Failed,
        ] {
            let pending = pending_view(Some(&held), true);
            assert_eq!(pending.outcome, None, "{held:?}: nothing while pending");
            assert_eq!(pending.aria_busy(), Some("true"), "{held:?}");
            assert!(pending.class().contains(DIM), "{held:?}");
            let settled = pending_view(Some(&held), false);
            assert_eq!(settled.outcome, Some(&held), "{held:?}: shown once settled");
        }
    }

    /// Nothing held yet (the first search of a visit): nothing to show,
    /// busy while it is pending.
    #[test]
    fn nothing_held_shows_nothing() {
        assert_eq!(pending_view(None, true).outcome, None);
        assert!(pending_view(None, true).busy);
        assert_eq!(pending_view(None, false).outcome, None);
    }

    /// The outcome always replaces the stale hits when it lands: the same
    /// area, pending then settled, shows the old then the new, never both.
    #[test]
    fn the_outcome_replaces_the_stale_hits_when_it_lands() {
        let (old, new) = (hits("old"), SearchOutcome::Failed);
        assert_eq!(pending_view(Some(&old), true).outcome, Some(&old));
        let landed = pending_view(Some(&new), false);
        assert_eq!(landed.outcome, Some(&new));
        assert_ne!(landed.outcome, Some(&old));
        assert!(!landed.busy);
    }
}
