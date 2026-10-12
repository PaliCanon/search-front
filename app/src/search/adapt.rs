//! The one seam from the API's wire types to the page's own DTOs (slice 2 of task 005).
//!
//! `palicanon_api_client`'s `SearchResponse` and `HitDto` carry more than
//! the page shows (the engine's document text, the segment key, timings,
//! the cache and degraded flags); [`response`] keeps what the page shows
//! and drops the rest, so nothing past this file sees a wire type.

use palicanon_api_client as wire;

use super::{Collection, HitDto, Mode, SearchResponse, SnippetKind, Sources, Titles};

/// The page's response for a wire one. `fell_back` is not on the wire: the
/// caller knows whether it asked again in English (slice 3), and says so.
/// The wire `mode` (`hybrid`…) is read back through [`mode_from_wire`];
/// `timings`, `cached`, `degraded` and the rest are dropped.
pub(crate) fn response(wire: wire::SearchResponse, fell_back: bool) -> SearchResponse {
    SearchResponse {
        query: wire.query,
        mode: mode_from_wire(wire.mode.code()),
        hits: wire.hits.into_iter().map(hit).collect(),
        fell_back,
    }
}

/// The page's hit for a wire one: the fields the card shows, `score` and
/// `sources` copied as they are (`sources.reference` is dropped), and the
/// link from [`read_url`].
pub(crate) fn hit(wire: wire::HitDto) -> HitDto {
    let url = read_url(&wire);
    HitDto {
        sutta_uid: wire.sutta_uid,
        display_id: wire.display_id,
        title: wire.title,
        pali_title: wire.pali_title,
        titles: Titles {
            en: wire.titles.en,
            fr: wire.titles.fr,
            vi: wire.titles.vi,
        },
        snippet: wire.snippet,
        snippet_kind: match wire.snippet_kind {
            wire::SnippetKind::Passage => SnippetKind::Passage,
            wire::SnippetKind::Blurb => SnippetKind::Blurb,
        },
        url,
        sources: Sources {
            dense: wire.sources.dense,
            sparse: wire.sources.sparse,
        },
        score: wire.score,
    }
}

/// The API's `mode` code: the same as [`Mode::code`] except that
/// [`Mode::Both`] is `hybrid` there (the UI's word for it is "both"). The
/// one place the two vocabularies meet, with [`mode_from_wire`].
pub(crate) fn mode_wire_code(mode: Mode) -> &'static str {
    match mode {
        Mode::Both => "hybrid",
        Mode::Words => "words",
        Mode::Meaning => "meaning",
    }
}

/// Reads an API `mode` code back: `words` and `meaning` are their own
/// mode, `hybrid` and anything unknown are [`Mode::Both`] (never a
/// refusal, like [`Mode::parse`]).
pub(crate) fn mode_from_wire(raw: &str) -> Mode {
    match raw {
        "words" => Mode::Words,
        "meaning" => Mode::Meaning,
        _ => Mode::Both,
    }
}

/// The API's `collections` code: the nikāya's own code, and none at all
/// for [`Collection::All`] (`all` is not an API code: no narrowing is
/// asked by sending nothing).
pub(crate) fn collection_wire_code(collection: Collection) -> Option<&'static str> {
    match collection {
        Collection::All => None,
        other => Some(other.code()),
    }
}

/// The "Read" link, as today's site builds it (`web/src/components/ResultList.tsx`,
/// `readUrl`): a passage whose `snippet_key` is a segment id (`mn10:3.2`)
/// links to that segment, `url` stripped of any fragment it already carried
/// (`Segment::suttacentral_url` can put one there), then `#key`. Every
/// other hit — a blurb, no key, an empty key, a translation paragraph
/// (`p4`) — gets `url` exactly as the wire sent it, fragment included.
fn read_url(wire: &wire::HitDto) -> String {
    match (&wire.snippet_kind, wire.snippet_key.as_deref()) {
        (wire::SnippetKind::Passage, Some(key)) if is_segment_id(key) => {
            let base = wire.url.split('#').next().unwrap_or_default();
            format!("{base}#{key}")
        }
        _ => wire.url.clone(),
    }
}

/// Whether `key` is a segment id: the site's `^[a-z]+[\d.-]*:[\d.]+$`
/// (`mn10:3.2`, `dhp221-234:1.1`), by hand — the pattern is three runs, a
/// regex crate for it would be a dependency for nothing. A translation
/// paragraph (`p4`) has no colon and is not one.
fn is_segment_id(key: &str) -> bool {
    let Some((head, tail)) = key.split_once(':') else {
        return false;
    };
    let letters = head.chars().take_while(char::is_ascii_lowercase).count();
    let rest = &head[letters..];
    letters > 0
        && rest
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
        && !tail.is_empty()
        && tail.chars().all(|c| c.is_ascii_digit() || c == '.')
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn mode_wire_code_round_trips() {
        for mode in Mode::all() {
            assert_eq!(mode_from_wire(mode_wire_code(mode)), mode);
        }
        assert_eq!(mode_wire_code(Mode::Both), "hybrid");
        assert_eq!(mode_wire_code(Mode::Words), "words");
        assert_eq!(mode_wire_code(Mode::Meaning), "meaning");
        assert_eq!(mode_from_wire("hybrid"), Mode::Both);
    }

    #[test]
    fn an_unknown_wire_mode_is_both() {
        assert_eq!(mode_from_wire(""), Mode::Both);
        assert_eq!(mode_from_wire("bogus"), Mode::Both);
        // the UI's own word is no wire code, but it must not refuse either
        assert_eq!(mode_from_wire("both"), Mode::Both);
    }

    #[test]
    fn collection_all_has_no_wire_code() {
        assert_eq!(collection_wire_code(Collection::All), None);
    }

    #[test]
    fn a_nikaya_wire_code_is_its_own_code() {
        for collection in Collection::all() {
            if collection != Collection::All {
                assert_eq!(collection_wire_code(collection), Some(collection.code()));
            }
        }
        assert_eq!(collection_wire_code(Collection::Mn), Some("mn"));
    }

    /// A wire hit as the API sends it, built from JSON (the `Sources` type
    /// is `search_core`'s, which this crate does not depend on).
    fn wire_hit(patch: Value) -> wire::HitDto {
        let mut hit = json!({
            "sutta_uid": "mn10",
            "segment_key": "mn10:1.1",
            "display_id": "MN 10",
            "title": "Satipaṭṭhānasutta",
            "pali_title": "Satipaṭṭhānasutta",
            "titles": {"en": "The Foundations of Mindfulness", "fr": "Les fondements", "vi": "Tứ niệm xứ"},
            "text": "engine document text",
            "snippet": "the quoted passage",
            "snippet_key": "mn10:3.2",
            "snippet_kind": "passage",
            "pali": "Evaṃ me sutaṃ",
            "url": "https://suttacentral.net/mn10/en/sujato",
            "score": 0.8125,
            "sources": {"dense": true, "sparse": false, "reference": true}
        });
        for (key, value) in patch.as_object().unwrap() {
            hit[key] = value.clone();
        }
        serde_json::from_value(hit).unwrap()
    }

    fn url_of(patch: Value) -> String {
        hit(wire_hit(patch)).url
    }

    #[test]
    fn a_passage_with_a_segment_key_links_to_the_segment() {
        assert_eq!(
            url_of(json!({})),
            "https://suttacentral.net/mn10/en/sujato#mn10:3.2"
        );
    }

    #[test]
    fn a_fragment_already_on_the_wire_url_is_stripped_first() {
        assert_eq!(
            url_of(json!({"url": "https://suttacentral.net/mn10/en/sujato#mn10:1.1"})),
            "https://suttacentral.net/mn10/en/sujato#mn10:3.2"
        );
    }

    #[test]
    fn a_segment_key_with_a_dashed_book_prefix_is_one() {
        assert_eq!(
            url_of(json!({"snippet_key": "dhp221-234:1.1", "url": "https://x/dhp"})),
            "https://x/dhp#dhp221-234:1.1"
        );
    }

    #[test]
    fn a_blurb_keeps_the_bare_url_even_with_a_key() {
        assert_eq!(
            url_of(json!({"snippet_kind": "blurb"})),
            "https://suttacentral.net/mn10/en/sujato"
        );
    }

    #[test]
    fn an_empty_key_keeps_the_bare_url() {
        assert_eq!(
            url_of(json!({"snippet_key": ""})),
            "https://suttacentral.net/mn10/en/sujato"
        );
    }

    #[test]
    fn a_translation_paragraph_key_has_no_anchor() {
        assert_eq!(
            url_of(json!({"snippet_key": "p4"})),
            "https://suttacentral.net/mn10/en/sujato"
        );
    }

    #[test]
    fn no_key_keeps_the_bare_url() {
        assert_eq!(
            url_of(json!({"snippet_key": null})),
            "https://suttacentral.net/mn10/en/sujato"
        );
    }

    /// As the site: only the anchored case strips a fragment; any other hit
    /// keeps the wire `url` as it came, fragment and all.
    #[test]
    fn a_bare_url_keeps_a_fragment_it_already_had() {
        assert_eq!(
            url_of(json!({"snippet_kind": "blurb", "url": "https://x/mn10#mn10:1.1"})),
            "https://x/mn10#mn10:1.1"
        );
    }

    #[test]
    fn the_segment_id_matcher_follows_the_sites_pattern() {
        for ok in [
            "mn10:3.2",
            "sn56.11:1.1",
            "dhp221-234:1.1",
            "an:1",
            "mn10:3",
        ] {
            assert!(is_segment_id(ok), "{ok}");
        }
        for bad in [
            "",
            "p4",
            ":1.1",
            "mn10:",
            "mn10:3.x",
            "MN10:3.2",
            "10mn:3.2",
            "mn10:3.2:1",
            "mn10 :3.2",
            "mn10:3.2 ",
            "mn_10:3.2",
        ] {
            assert!(!is_segment_id(bad), "{bad:?}");
        }
    }

    #[test]
    fn the_card_fields_are_copied() {
        let h = hit(wire_hit(json!({})));
        assert_eq!(h.sutta_uid, "mn10");
        assert_eq!(h.display_id, "MN 10");
        assert_eq!(h.title, "Satipaṭṭhānasutta");
        assert_eq!(h.pali_title.as_deref(), Some("Satipaṭṭhānasutta"));
        assert_eq!(
            h.titles.en.as_deref(),
            Some("The Foundations of Mindfulness")
        );
        assert_eq!(h.titles.fr.as_deref(), Some("Les fondements"));
        assert_eq!(h.titles.vi.as_deref(), Some("Tứ niệm xứ"));
        assert_eq!(h.snippet, "the quoted passage");
        assert_eq!(h.snippet_kind, SnippetKind::Passage);
    }

    #[test]
    fn every_absent_optional_stays_none() {
        let h = hit(wire_hit(json!({
            "pali_title": null,
            "titles": {},
            "snippet_key": null
        })));
        assert_eq!(h.pali_title, None);
        assert_eq!(h.titles, Titles::default());
    }

    #[test]
    fn a_title_missing_in_one_language_is_none_only_there() {
        let h = hit(wire_hit(json!({"titles": {"fr": "Les fondements"}})));
        assert_eq!(h.titles.en, None);
        assert_eq!(h.titles.fr.as_deref(), Some("Les fondements"));
        assert_eq!(h.titles.vi, None);
    }

    #[test]
    fn a_blurb_hit_is_a_blurb() {
        let h = hit(wire_hit(json!({"snippet_kind": "blurb"})));
        assert_eq!(h.snippet_kind, SnippetKind::Blurb);
    }

    #[test]
    fn the_score_is_copied() {
        assert_eq!(
            hit(wire_hit(json!({}))).score.to_bits(),
            0.8125_f32.to_bits()
        );
    }

    #[test]
    fn each_branch_of_sources_is_copied() {
        for (dense, sparse) in [(true, false), (false, true), (true, true), (false, false)] {
            let h = hit(wire_hit(json!({
                "sources": {"dense": dense, "sparse": sparse, "reference": true}
            })));
            assert_eq!(h.sources, Sources { dense, sparse });
        }
    }

    fn wire_response(mode: &str) -> wire::SearchResponse {
        serde_json::from_value(json!({
            "query": "satipaṭṭhāna",
            "lang": "fr",
            "detected_lang": null,
            "k": 20,
            "mode": mode,
            "collections": [],
            "corrected_query": null,
            "took_ms": 12,
            "timings": {"embed_ms": null, "search_ms": 10, "snippet_ms": 2},
            "degraded": false,
            "cached": true,
            "hits": [json!({
                "sutta_uid": "mn10", "segment_key": "mn10:1.1", "display_id": "MN 10",
                "title": "t", "pali_title": null, "text": "", "snippet": "s",
                "pali": null, "url": "https://x/mn10", "score": 1.0,
                "sources": {"dense": true, "sparse": true}
            })]
        }))
        .unwrap()
    }

    #[test]
    fn the_wire_hybrid_mode_is_both_and_back() {
        let page = response(wire_response("hybrid"), false);
        assert_eq!(page.mode, Mode::Both);
        assert_eq!(mode_wire_code(page.mode), "hybrid");
    }

    #[test]
    fn words_and_meaning_keep_their_mode() {
        assert_eq!(response(wire_response("words"), false).mode, Mode::Words);
        assert_eq!(
            response(wire_response("meaning"), false).mode,
            Mode::Meaning
        );
    }

    #[test]
    fn fell_back_is_the_callers_not_the_wires() {
        assert!(!response(wire_response("hybrid"), false).fell_back);
        assert!(response(wire_response("hybrid"), true).fell_back);
    }

    #[test]
    fn the_query_and_every_hit_are_carried() {
        let page = response(wire_response("hybrid"), false);
        assert_eq!(page.query, "satipaṭṭhāna");
        assert_eq!(page.hits.len(), 1);
        assert_eq!(page.hits[0].sutta_uid, "mn10");
        assert_eq!(page.hits[0].url, "https://x/mn10");
    }

    #[test]
    fn no_hits_is_an_empty_page_not_an_error() {
        let mut wire = wire_response("hybrid");
        wire.hits.clear();
        assert_eq!(response(wire, false).hits.len(), 0);
    }
}
