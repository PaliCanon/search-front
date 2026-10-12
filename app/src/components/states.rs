//! The states besides "results" and "no result"
//! ([`crate::components::result_list`]): the engine waking up, the API
//! rate-limiting, any other failure (the idle page is the home,
//! `crate::page`). Each is an alert; the first two carry a plain link that
//! asks the same GET again.

use leptos::prelude::*;

use crate::catalogue::strings;

/// The backend could not answer: any failure but a cold engine or a rate
/// limit.
#[component]
pub fn ErrorState() -> impl IntoView {
    let t = strings();
    view! {
        <div role="alert" class=ALERT>
            <p class="font-serif text-xl text-heading">
                {move || leptos_i18n::td_string!(t.locale(), error_title)}
            </p>
            <p class="text-muted">{move || leptos_i18n::td_string!(t.locale(), error_hint)}</p>
        </div>
    }
}

/// The engine is waking up (the API's `engine_starting`, or a search that
/// outlasted the server's patience): no number to count down, only a link
/// that asks again — the same GET, `href`.
#[component]
pub fn ColdState(
    /// The page's own URL for this search: the link re-submits it.
    #[prop(into)]
    href: String,
) -> impl IntoView {
    let t = strings();
    view! {
        <div role="alert" class=ALERT>
            <p class="font-serif text-xl text-heading">
                {move || leptos_i18n::td_string!(t.locale(), cold_text)}
            </p>
            <RetryLink href=href />
        </div>
    }
}

/// The API is rate-limiting this reader: wait a moment, then the link asks
/// again — the same GET, `href`.
#[component]
pub fn RateLimitedState(
    /// The page's own URL for this search: the link re-submits it.
    #[prop(into)]
    href: String,
) -> impl IntoView {
    let t = strings();
    view! {
        <div role="alert" class=ALERT>
            <p class="font-serif text-xl text-heading">
                {move || leptos_i18n::td_string!(t.locale(), rate_limited_text)}
            </p>
            <RetryLink href=href />
        </div>
    }
}

/// The alerts' box: the same as [`ErrorState`]'s.
const ALERT: &str = "mt-10 max-w-[60ch] space-y-2";

/// The plain link that asks the same search again: a real `<a href>`, no
/// script (the page it points at answers with whatever the engine says
/// then).
#[component]
fn RetryLink(
    /// The search's own GET.
    href: String,
) -> impl IntoView {
    let t = strings();
    view! {
        <a href=href class="inline-block text-gold underline underline-offset-4 hover:text-heading">
            {move || leptos_i18n::td_string!(t.locale(), retry_link)}
        </a>
    }
}

#[cfg(test)]
mod tests {
    use palicanon_ui_lib::Language;

    use super::*;

    #[test]
    fn the_error_state_is_announced_as_an_alert() {
        let html = Owner::new().with(|| view! { <ErrorState /> }.to_html());
        assert!(html.contains(r#"role="alert""#), "{html}");
    }

    /// Renders `view` as the server would with the app's language set.
    fn in_language(language: Language, view: impl Fn() -> AnyView) -> String {
        Owner::new().with(|| {
            palicanon_ui_lib::provide_language().set(language);
            view().to_html()
        })
    }

    /// The cold state: the site's own `error_starting` wording in every
    /// language, an alert, no number, and a plain link back to the same GET.
    #[test]
    fn the_cold_state_is_an_alert_with_the_sites_wording_and_a_retry_link() {
        for (language, text, retry) in [
            (
                Language::En,
                "The search engine is still starting. Search again in a few seconds.",
                "Search again",
            ),
            (
                Language::Fr,
                "Le moteur de recherche démarre encore. Relancez la recherche dans quelques secondes.",
                "Relancer la recherche",
            ),
            (
                Language::Vi,
                "Công cụ tìm kiếm vẫn đang khởi động. Hãy tìm lại sau vài giây.",
                "Tìm lại",
            ),
        ] {
            let html = in_language(language, || {
                view! { <ColdState href="/?q=metta&c=all&mode=both" /> }.into_any()
            });
            assert!(html.contains(r#"role="alert""#), "{language:?}: {html}");
            assert!(html.contains(text), "{language:?}: {html}");
            assert!(
                html.contains(r#"<a href="/?q=metta&amp;c=all&amp;mode=both""#),
                "{language:?}: {html}"
            );
            assert!(html.contains(retry), "{language:?}: {html}");
            assert!(!html.contains("seconds</"), "no countdown: {html}");
        }
    }

    /// The rate-limited state: its own wording in every language, an alert,
    /// the same plain retry link.
    #[test]
    fn the_rate_limited_state_is_an_alert_with_its_wording_and_a_retry_link() {
        for (language, text, retry) in [
            (
                Language::En,
                "Too many searches at once. Wait a moment and try again.",
                "Search again",
            ),
            (
                Language::Fr,
                "Trop de recherches à la fois. Patientez un instant, puis réessayez.",
                "Relancer la recherche",
            ),
            (
                Language::Vi,
                "Có quá nhiều lượt tìm cùng lúc. Vui lòng chờ một chút rồi thử lại.",
                "Tìm lại",
            ),
        ] {
            let html = in_language(language, || {
                view! { <RateLimitedState href="/?q=metta&c=mn&mode=words" /> }.into_any()
            });
            assert!(html.contains(r#"role="alert""#), "{language:?}: {html}");
            assert!(html.contains(text), "{language:?}: {html}");
            assert!(
                html.contains(r#"<a href="/?q=metta&amp;c=mn&amp;mode=words""#),
                "{language:?}: {html}"
            );
            assert!(html.contains(retry), "{language:?}: {html}");
        }
    }
}
