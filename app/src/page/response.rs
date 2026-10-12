//! The response the page asks of the server: the status, `Retry-After` and
//! `Cache-Control` its outcome calls for. The decision is a pure function
//! ([`spec_for`]); applying it to the response needs the server's
//! `ResponseOptions`, so this module is `ssr` only.

use crate::search::SearchOutcome;

/// The `Retry-After` a cold engine gets when the API named none (the
/// server's own timeout is a cold answer too).
pub(super) const COLD_RETRY_SECONDS: u64 = 10;

/// What the response carries for one outcome of a non-empty search.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct ResponseSpec {
    /// The status, `None` for the usual 200.
    pub(super) status: Option<u16>,
    /// `Retry-After`, in seconds.
    pub(super) retry_after: Option<u64>,
    /// `Cache-Control: no-store`.
    pub(super) no_store: bool,
}

/// What the response carries for `outcome`: cold 503 with a `Retry-After`,
/// rate-limited 429 with one when the API gave it, failed 502, hits 200,
/// all `Cache-Control: no-store` — but only for a search of a non-empty
/// `?q=`: the home and a blank query (`query_is_blank`) carry nothing, so
/// the edge may cache them.
pub(super) fn spec_for(outcome: &SearchOutcome, query_is_blank: bool) -> ResponseSpec {
    if query_is_blank {
        return ResponseSpec {
            status: None,
            retry_after: None,
            no_store: false,
        };
    }
    let (status, retry_after) = match outcome {
        SearchOutcome::Hits(_) => (None, None),
        SearchOutcome::Cold { retry_after } => {
            (Some(503), Some(retry_after.unwrap_or(COLD_RETRY_SECONDS)))
        }
        SearchOutcome::RateLimited { retry_after } => (Some(429), *retry_after),
        SearchOutcome::Failed => (Some(502), None),
    };
    ResponseSpec {
        status,
        retry_after: retry_after.map(|seconds| seconds.clamp(1, 300)),
        no_store: true,
    }
}

/// Writes the outcome's [`ResponseSpec`] (`query_is_blank`: the page is
/// the home) into the server's response options. The hydrate build has no options, nor does a render outside the
/// router: none in the context is no error, there is nothing to set.
pub(super) fn apply(outcome: &SearchOutcome, query_is_blank: bool) {
    use axum::http::{HeaderValue, StatusCode, header};
    use leptos::prelude::use_context;

    // Never `expect_context`: the hydrate build and a bare render have none.
    let Some(options) = use_context::<leptos_axum::ResponseOptions>() else {
        return;
    };
    let spec = spec_for(outcome, query_is_blank);
    if let Some(status) = spec.status.and_then(|code| StatusCode::from_u16(code).ok()) {
        options.set_status(status);
    }
    if let Some(seconds) = spec.retry_after {
        options.insert_header(header::RETRY_AFTER, HeaderValue::from(seconds));
    }
    if spec.no_store {
        options.insert_header(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::SearchResponse;
    use leptos::prelude::{Owner, provide_context};
    use leptos_axum::ResponseOptions;

    /// The `Retry-After` header [`apply`] writes for `outcome`, as text.
    fn retry_after_header(outcome: &SearchOutcome) -> Option<String> {
        let owner = Owner::new();
        owner.set();
        let options = ResponseOptions::default();
        provide_context(options.clone());
        apply(outcome, false);
        let parts = options.0.read().expect("the options lock").clone();
        parts
            .headers
            .get("retry-after")
            .map(|value| value.to_str().expect("a text header").to_owned())
    }

    fn spec(outcome: &SearchOutcome) -> (Option<u16>, Option<u64>, bool) {
        let spec = spec_for(outcome, false);
        (spec.status, spec.retry_after, spec.no_store)
    }

    #[test]
    fn hits_are_a_200_never_cached() {
        let hits = SearchOutcome::Hits(SearchResponse::empty("metta"));
        assert_eq!(spec(&hits), (None, None, true));
    }

    #[test]
    fn a_cold_engine_is_a_503_with_the_apis_retry_after_or_ten() {
        let cold = |retry_after| SearchOutcome::Cold { retry_after };
        assert_eq!(spec(&cold(Some(7))), (Some(503), Some(7), true));
        assert_eq!(spec(&cold(None)), (Some(503), Some(10), true));
    }

    /// A6: `Retry-After` is clamped to 1..=300 seconds.
    #[test]
    fn retry_after_is_clamped_to_one_through_three_hundred() {
        let cold = |retry_after| SearchOutcome::Cold { retry_after };
        assert_eq!(spec(&cold(Some(0))).1, Some(1));
        assert_eq!(spec(&cold(Some(9999))).1, Some(300));
    }

    #[test]
    fn rate_limited_is_a_429_with_a_retry_after_when_the_api_gave_one() {
        let limited = |retry_after| SearchOutcome::RateLimited { retry_after };
        assert_eq!(spec(&limited(Some(7))), (Some(429), Some(7), true));
        assert_eq!(spec(&limited(None)), (Some(429), None, true));
    }

    #[test]
    fn failed_is_a_502() {
        assert_eq!(spec(&SearchOutcome::Failed), (Some(502), None, true));
    }

    /// T1: the clamp is on the header itself, for both outcomes that carry
    /// one, and an absent `retry_after` is the cold default or no header.
    #[test]
    fn the_retry_after_header_is_clamped_defaulted_or_absent() {
        let cold = |retry_after| SearchOutcome::Cold { retry_after };
        let limited = |retry_after| SearchOutcome::RateLimited { retry_after };
        let header = |outcome| retry_after_header(&outcome);
        assert_eq!(header(cold(Some(0))).as_deref(), Some("1"));
        assert_eq!(header(cold(Some(99_999))).as_deref(), Some("300"));
        assert_eq!(header(cold(Some(7))).as_deref(), Some("7"));
        assert_eq!(header(cold(None)).as_deref(), Some("10"));
        assert_eq!(header(limited(Some(0))).as_deref(), Some("1"));
        assert_eq!(header(limited(Some(99_999))).as_deref(), Some("300"));
        assert_eq!(header(limited(None)), None);
        assert_eq!(header(SearchOutcome::Failed), None);
    }

    /// Δ2 (review 005, round 2): the rule lives in the spec — the home and a
    /// blank query (whatever the outcome) get no status, no `Retry-After`
    /// and no `no-store`; only a search of a non-empty query is never cached.
    #[test]
    fn the_home_and_a_blank_query_carry_nothing() {
        let blank = ResponseSpec {
            status: None,
            retry_after: None,
            no_store: false,
        };
        let outcomes = [
            SearchOutcome::Hits(SearchResponse::empty("")),
            SearchOutcome::Cold { retry_after: None },
            SearchOutcome::RateLimited {
                retry_after: Some(3),
            },
            SearchOutcome::Failed,
        ];
        for outcome in &outcomes {
            assert_eq!(spec_for(outcome, true), blank, "{outcome:?}");
            assert!(spec_for(outcome, false).no_store, "{outcome:?}");
        }
    }

    /// The blank spec writes nothing on the response options either.
    #[test]
    fn applying_the_blank_spec_sets_no_cache_control() {
        let owner = Owner::new();
        owner.set();
        let options = ResponseOptions::default();
        provide_context(options.clone());
        apply(&SearchOutcome::Hits(SearchResponse::empty("")), true);
        let parts = options.0.read().expect("the options lock").clone();
        assert_eq!(parts.status, None);
        assert_eq!(parts.headers.get("cache-control"), None);
    }

    /// T7 (review 005, round 3): the blank branch ignores every refusal:
    /// all unfavourable outcomes (Cold, `RateLimited`, Failed) set no status
    /// and no headers when the query is blank.
    #[test]
    fn the_blank_branch_ignores_every_refusal() {
        let owner = Owner::new();
        owner.set();
        let outcomes = [
            SearchOutcome::Cold {
                retry_after: Some(7),
            },
            SearchOutcome::RateLimited {
                retry_after: Some(5),
            },
            SearchOutcome::Failed,
        ];
        for outcome in &outcomes {
            let options = ResponseOptions::default();
            provide_context(options.clone());
            apply(outcome, true);
            let parts = options.0.read().expect("the options lock").clone();
            assert_eq!(parts.status, None, "{outcome:?}");
            assert_eq!(parts.headers.get("retry-after"), None, "{outcome:?}");
            assert_eq!(parts.headers.get("cache-control"), None, "{outcome:?}");
        }
    }
}
