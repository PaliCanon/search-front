//! Text that echoes the reader's own words must never hold an email-shaped
//! run. Cloudflare's Email Address Obfuscation rewrites any such run of the
//! HTML body into an `<a class="__cf_email__">`, and a node the server
//! never rendered breaks hydration (tachys `hydration.rs`, "unreachable").
//! The `@` goes in its own element: no text run is an address, the
//! visible, copied and spoken text are unchanged.

use leptos::prelude::*;

/// `text`, split at each `@` so that the `@` is its own `<span>`:
/// `foo@bar.com` renders `foo<span>@</span>bar.com`. Text without an `@`
/// renders as the one text node it always was.
pub fn no_email_run(text: impl Into<String>) -> AnyView {
    let text: String = text.into();
    if !text.contains('@') {
        return text.into_any();
    }
    // An empty string renders as a placeholder space: emit none.
    let mut views = Vec::new();
    for (i, part) in text.split('@').enumerate() {
        if i > 0 {
            views.push(view! { <span>"@"</span> }.into_any());
        }
        if !part.is_empty() {
            views.push(part.to_owned().into_any());
        }
    }
    views.collect_view().into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn html(text: &str) -> String {
        Owner::new().with(|| no_email_run(text).to_html())
    }

    /// The markup without hydration markers.
    fn bare(text: &str) -> String {
        html(text).replace("<!>", "")
    }

    #[test]
    fn the_at_sign_is_its_own_element() {
        assert_eq!(bare("foo@bar.com"), "foo<span>@</span>bar.com");
    }

    #[test]
    fn text_without_an_at_sign_is_unchanged() {
        assert_eq!(html("anattā"), "anattā");
    }

    #[test]
    fn leading_trailing_and_double_at_signs_keep_every_character() {
        for text in ["@a", "a@", "@", "a@@b", "x@y@z.org"] {
            let bare = bare(text);
            assert_eq!(bare.replace("<span>@</span>", "@"), text, "{bare}");
        }
    }
}
