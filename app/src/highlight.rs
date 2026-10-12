//! Diacritic-insensitive snippet highlighting: `satipatthana` lights up
//! `satipaṭṭhāna`. Ported from today's site
//! (`palisearch/web/src/highlight.tsx`): fold each character (NFD, strip
//! its combining mark, lowercase) one at a time, so the folded string
//! stays the same length as the original and a match's indices slice
//! back into it unchanged; terms under three folded characters are
//! ignored (too common to mean anything in a snippet).

use unicode_normalization::UnicodeNormalization;

/// One run of `text`: either every character under the query's terms
/// (`matched`), or none of them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    /// The run's original text (never folded: only the *search* is
    /// diacritic-insensitive, the page always shows the real spelling).
    pub text: String,
    /// Whether this run is under a matched term.
    pub matched: bool,
}

/// A combining mark `char::nfd` can produce for a Pali/Vietnamese letter
/// (macron, dot above/below, tilde, breve…): the Combining Diacritical
/// Marks block, U+0300–U+036F, covers every one this corpus uses.
fn is_combining_mark(c: char) -> bool {
    ('\u{0300}'..='\u{036F}').contains(&c)
}

/// `ch`, NFD-decomposed, its combining mark (if any) dropped, lowercased:
/// `ā` and `Ā` both fold to `a`, `ṭ` to `t`. Never empty: a character with
/// no base form left (should not happen for a letter) falls back to
/// itself unfolded.
fn fold_char(ch: char) -> char {
    ch.nfd()
        .find(|c| !is_combining_mark(*c))
        .and_then(|c| c.to_lowercase().next())
        .unwrap_or(ch)
}

/// `s`, one folded `char` per original `char` — same length, so a match
/// found here slices the original string at the same indices.
fn fold(s: &str) -> Vec<char> {
    s.chars().map(fold_char).collect()
}

/// Punctuation `query` splits on, besides whitespace (today's site's own
/// list, `highlight.tsx`): quotes and sentence punctuation, never a
/// diacritic or an apostrophe inside a word.
const SPLIT_PUNCTUATION: &str = ",.;:!?“”\"'()";

/// `query`'s own words, folded, at least three folded characters each —
/// what actually gets highlighted in a snippet.
fn terms(query: &str) -> Vec<Vec<char>> {
    query
        .split(|c: char| c.is_whitespace() || SPLIT_PUNCTUATION.contains(c))
        .map(fold)
        .filter(|t| t.len() >= 3)
        .collect()
}

/// `text`, split into runs by whether each character falls under one of
/// `query`'s terms (diacritic- and case-insensitive); no terms at all (a
/// query with nothing three folded characters long) is one unmatched run.
#[must_use]
pub fn highlight(text: &str, query: &str) -> Vec<Segment> {
    let terms = terms(query);
    if terms.is_empty() {
        return vec![Segment {
            text: text.to_owned(),
            matched: false,
        }];
    }
    let chars: Vec<char> = text.chars().collect();
    let folded = fold(text);
    let hit = mark_hits(&folded, &terms);
    group(&chars, &hit)
}

/// One `bool` per character of `folded`: `true` under any of `terms`.
fn mark_hits(folded: &[char], terms: &[Vec<char>]) -> Vec<bool> {
    let mut hit = vec![false; folded.len()];
    for term in terms {
        let n = term.len();
        if n == 0 || n > folded.len() {
            continue;
        }
        for start in 0..=folded.len() - n {
            if folded[start..start + n] == term[..] {
                hit[start..start + n].fill(true);
            }
        }
    }
    hit
}

/// `chars`, grouped into consecutive runs of the same `hit` value.
fn group(chars: &[char], hit: &[bool]) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut start = 0;
    for i in 1..=chars.len() {
        if i == chars.len() || hit[i] != hit[start] {
            segments.push(Segment {
                text: chars[start..i].iter().collect(),
                matched: hit[start],
            });
            start = i;
        }
    }
    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_match_is_one_matched_run() {
        let segments = highlight("the four noble truths", "noble");
        assert_eq!(
            segments,
            vec![
                Segment {
                    text: "the four ".to_owned(),
                    matched: false
                },
                Segment {
                    text: "noble".to_owned(),
                    matched: true
                },
                Segment {
                    text: " truths".to_owned(),
                    matched: false
                },
            ]
        );
    }

    #[test]
    fn folding_ignores_diacritics_both_ways() {
        // Plain query lights up the diacritic spelling…
        let segments = highlight("Satipaṭṭhānasutta", "satipatthana");
        assert_eq!(
            segments,
            vec![
                Segment {
                    text: "Satipaṭṭhāna".to_owned(),
                    matched: true
                },
                Segment {
                    text: "sutta".to_owned(),
                    matched: false
                },
            ]
        );
        // …and a diacritic query lights up the plain spelling.
        let segments = highlight("the satipatthana teaching", "paṭṭhāna");
        assert!(
            segments
                .iter()
                .any(|s| s.matched && s.text.contains("patthana"))
        );
    }

    #[test]
    fn terms_under_three_folded_characters_are_ignored() {
        let segments = highlight("to be or not to be", "to be or");
        assert!(segments.iter().all(|s| !s.matched), "{segments:?}");
    }

    #[test]
    fn several_terms_each_get_their_own_run() {
        let segments = highlight("Setting the Wheel of Dhamma in Motion", "wheel dhamma");
        let matched: Vec<&str> = segments
            .iter()
            .filter(|s| s.matched)
            .map(|s| s.text.as_str())
            .collect();
        assert_eq!(matched, vec!["Wheel", "Dhamma"]);
    }

    #[test]
    fn no_query_terms_is_one_unmatched_run() {
        let segments = highlight("plain text", "");
        assert_eq!(
            segments,
            vec![Segment {
                text: "plain text".to_owned(),
                matched: false
            }]
        );
    }
}
