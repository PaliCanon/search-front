//! The service's shell around `SearchPage` and the footer pages (both
//! private): the HTML document ([`shell`], server-rendered) and the app
//! root ([`App`], what hydration mounts), `AppShell`'s rail with this
//! service's one Search entry, the footer (handed to `SiteFooter`) carrying the build badge, the theme and languages.

use leptos::prelude::*;
use leptos_i18n::td_string;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_location;
use leptos_router::path;
use palicanon_ui_lib::{
    AppShell, EnvBadgeInfo, HydrationRoot, IconName, NavEntry, NavList, NavSection,
    language_attribute, provide_language, theme_attribute,
};

use crate::catalogue::strings;
use crate::components::not_found::NotFoundPage;
use crate::components::site_footer::SiteFooter;
use crate::fragment::{SearchProvider, SearchRoutes};
use crate::pages::{FooterPage, PageKind};
use crate::search::site::SiteBackend;

/// The branch this build came from (`build.rs`,
/// `SEARCH_FRONT_GIT_BRANCH`, from git, or from the `BUILD_BRANCH` the
/// image build passes when it has no `.git`): `main`, `feat/…` — this service's own
/// `env` for [`EnvBadgeInfo`] (replaced by `dev` / `prod` once this
/// deploys, like every other service).
pub(crate) const BUILD_BRANCH: &str = env!("SEARCH_FRONT_GIT_BRANCH");
/// The short commit this build came from (`build.rs`,
/// `SEARCH_FRONT_GIT_SHA`, from git, or from `BUILD_SHA`), a trailing `*`
/// when the worktree had uncommitted changes at build time (git only).
pub(crate) const BUILD_SHA: &str = env!("SEARCH_FRONT_GIT_SHA");

/// This build, as the footer's badge shows it (at the footer's left —
/// `SiteFooter`; nothing on the shell's panel).
pub(crate) const BADGE: EnvBadgeInfo = EnvBadgeInfo {
    env: BUILD_BRANCH,
    sha: BUILD_SHA,
};

/// The HTML document around the app, rendered on the server. `data-theme`
/// and `lang` follow the request's cookies ([`theme_attribute`],
/// [`language_attribute`]), so the first paint never flashes the wrong
/// one. The app sits in ui-lib's [`HydrationRoot`] (`<div id="pc-app">`),
/// the one element of `<body>` the server renders, which the browser's
/// entry (`palicanon_ui_lib::hydrate`, in `lib.rs`) hydrates from: what
/// Cloudflare adds to `<body>` (a decoy link first, scripts after) stays
/// outside it (ui-lib 063).
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang=language_attribute() data-theme=theme_attribute()>
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <link rel="icon" href="/favicon.ico" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <HydrationRoot>
                    <App />
                </HydrationRoot>
            </body>
        </html>
    }
}

/// The app root: what hydration mounts, so it provides the page's one
/// language signal ([`provide_language`]; the server's shell already
/// set `<html lang>` from the very same resolution).
#[component]
pub fn App() -> impl IntoView {
    provide_language();
    provide_meta_context();
    view! {
        <Stylesheet id="leptos" href="/pkg/search-front.css" />
        <Title text="PaliCanon — Search" />
        <Router>
            <Layout />
        </Router>
    }
}

/// The rail's entries: the services, Search (this one, the current entry on
/// `/`), Learn, then Map. The same three, in the same order and with the same
/// hrefs, as learn-pali's and vizualisation's rails.
fn nav(current: Signal<String>) -> NavList {
    let t = strings();
    let sections = Signal::derive(move || {
        vec![NavSection {
            title: None,
            entries: vec![
                NavEntry {
                    label: td_string!(t.locale(), nav_search),
                    icon: IconName::Search,
                    href: "/",
                },
                // `IconName::Grid` and `IconName::Motion` stand in until ui-lib
                // has Learn and Map icons, as in learn-pali and vizualisation.
                NavEntry {
                    label: td_string!(t.locale(), nav_learn),
                    icon: IconName::Grid,
                    href: "https://learn.palicanon.app",
                },
                NavEntry {
                    label: td_string!(t.locale(), nav_map),
                    icon: IconName::Motion,
                    href: "https://map.palicanon.app",
                },
            ],
        }]
    });
    NavList::new(sections, current)
}

/// The shell around [`SearchPage`]: `AppShell`'s rail, theme and
/// languages, its routes (the search page at `/`, the four footer pages), the footer under each,
/// carrying the build badge.
#[component]
fn Layout() -> impl IntoView {
    let location = use_location();
    view! {
        <SearchProvider backend=SiteBackend::new()>
            <AppShell nav=nav(location.pathname.into()) footer=|| view! { <SiteFooter /> }>
                <Routes fallback=|| view! { <NotFoundPage /> }>
                    <SearchRoutes />
                    <Route
                        path=path!("/acknowledgements")
                        view=|| view! { <FooterPage kind=PageKind::Acknowledgements /> }
                    />
                    <Route
                        path=path!("/terms")
                        view=|| view! { <FooterPage kind=PageKind::Terms /> }
                    />
                    <Route
                        path=path!("/privacy")
                        view=|| view! { <FooterPage kind=PageKind::Privacy /> }
                    />
                    <Route
                        path=path!("/support")
                        view=|| view! { <FooterPage kind=PageKind::Support /> }
                    />
                </Routes>
            </AppShell>
        </SearchProvider>
    }
}

#[cfg(test)]
mod tests {
    use leptos_router::location::RequestUrl;

    use super::*;

    /// [`Layout`] as the server renders it for `path`, the language set.
    fn render_layout(language: palicanon_ui_lib::Language, path: &str) -> String {
        Owner::new().with(|| {
            palicanon_ui_lib::provide_language().set(language);
            provide_meta_context();
            provide_context(RequestUrl::new(path));
            view! {
                <Router>
                    <Layout />
                </Router>
            }
            .to_html()
        })
    }

    /// Round 4 (user check): every footer page has its route, renders its
    /// title in each language, inside the shell, the same footer under it.
    #[test]
    fn every_footer_page_route_renders_its_title_in_every_language() {
        use palicanon_ui_lib::Language;
        for (path, titles) in [
            (
                "/acknowledgements",
                ["Acknowledgements", "Remerciements", "Tri ân"],
            ),
            (
                "/terms",
                [
                    "Terms of use",
                    "Conditions d’utilisation",
                    "Điều khoản sử dụng",
                ],
            ),
            ("/privacy", ["Privacy", "Confidentialité", "Quyền riêng tư"]),
            ("/support", ["Support", "Soutenir", "Ủng hộ"]),
        ] {
            for (language, title) in Language::ALL.into_iter().zip(titles) {
                let html = render_layout(language, path);
                let h1 = html.find(&format!(">{title}</h1>")).expect(&html);
                let footer = html.find("<footer").expect(&html);
                assert!(h1 < footer, "{path} {language:?}: {html}");
                assert!(!html.contains("Not found."), "{html}");
            }
        }
    }

    /// The rail lists the three services, Search, Learn, Map, in that order,
    /// labelled in each language; only Search (`/`) is current on the home,
    /// the other two are plain external links.
    #[tokio::test]
    async fn the_rail_lists_search_then_learn_then_map() {
        use palicanon_ui_lib::Language;
        // The home spawns the search's resources: an executor, whichever
        // test runs first (harness::render_view does the same).
        let _ = any_spawner::Executor::init_tokio();
        for (language, labels) in [
            (Language::En, ["Search", "Learn", "Map"]),
            (Language::Fr, ["Recherche", "Apprendre", "Carte"]),
            (Language::Vi, ["Tìm kiếm", "Học", "Bản đồ"]),
        ] {
            let html = render_layout(language, "/");
            let at = |href: &str| {
                html.find(&format!(r#"href="{href}""#))
                    .unwrap_or_else(|| panic!("{href}: {html}"))
            };
            let search = at("/");
            let learn = at("https://learn.palicanon.app");
            let map = at("https://map.palicanon.app");
            assert!(search < learn && learn < map, "{language:?}: {html}");
            for label in labels {
                assert!(
                    html.contains(&format!(">{label}</span>")),
                    "{label}: {html}"
                );
            }
            for href in ["https://learn.palicanon.app", "https://map.palicanon.app"] {
                let open = html[..at(href)].rfind("<a").expect(&html);
                let close = open + html[open..].find('>').expect(&html);
                assert!(
                    !html[open..close].contains("aria-current"),
                    "{href}: {html}"
                );
            }
        }
    }

    /// No Legal anywhere: its old route is not a page.
    #[test]
    fn legal_is_not_a_page() {
        let html = render_layout(palicanon_ui_lib::Language::En, "/legal");
        assert!(!html.contains("Not found."), "{html}");
        // Should render the not-found page instead
        assert!(html.contains("Anattā"), "{html}");
    }

    /// The not-found page renders for unknown paths in all languages.
    #[test]
    fn unknown_path_renders_not_found_page() {
        use palicanon_ui_lib::Language;
        for language in Language::ALL {
            let html = render_layout(language, "/nimportequoi");
            // Should contain the title "Anattā"
            assert!(
                html.contains("Anattā"),
                "title missing in {language:?}: {html}"
            );
            // Should contain "404" eyebrow
            assert!(
                html.contains("404"),
                "404 eyebrow missing in {language:?}: {html}"
            );
            // Should contain a link to home
            assert!(
                html.contains(r#"href="/"#),
                "home link missing in {language:?}: {html}"
            );
            // Should NOT contain the old fallback text
            assert!(
                !html.contains("Not found."),
                "old fallback text present in {language:?}: {html}"
            );
        }
    }

    /// The not-found page renders the correct content in all languages.
    #[test]
    fn not_found_page_renders_in_all_languages() {
        use palicanon_ui_lib::Language;
        for language in Language::ALL {
            let html = render_layout(language, "/nimportequoi");
            // Should contain the title "Anattā" (as page title and in the h1)
            assert!(
                html.contains("Anattā"),
                "title missing in {language:?}: {html}"
            );
            // Should contain a link to home
            assert!(
                html.contains(r#"href="/"#),
                "home link missing in {language:?}: {html}"
            );
        }
    }

    /// Where the `<div …>` opened at `start` closes: the index of its
    /// matching `</div>` (the app's markup nests divs, never self-closes one).
    fn matching_div_close(html: &str, start: usize) -> usize {
        let mut depth = 0usize;
        let mut at = start;
        loop {
            let open = html[at..].find("<div").map(|i| at + i);
            let close = html[at..].find("</div>").map(|i| at + i).expect(html);
            match open {
                Some(open) if open < close => {
                    depth += 1;
                    at = open + 4;
                }
                _ => {
                    depth -= 1;
                    if depth == 0 {
                        return close;
                    }
                    at = close + 6;
                }
            }
        }
    }

    /// 017 (ui-lib 063): the server's document wraps the app in ui-lib's
    /// hydration root, `<div id="pc-app">`, the one element of `<body>` it
    /// renders, which the browser's `palicanon_ui_lib::hydrate` starts
    /// from: the nodes Cloudflare adds to `<body>` (its decoy link first)
    /// then sit outside the app.
    #[test]
    fn the_shell_body_is_the_hydration_root_around_the_app() {
        let options = LeptosOptions::builder().output_name("search-front").build();
        let html = Owner::new().with(|| {
            provide_context(RequestUrl::new("/privacy"));
            shell(options).to_html()
        });
        let body = html.find("<body>").expect(&html) + "<body>".len();
        let end = html.find("</body>").expect(&html);
        assert!(
            html[body..].starts_with(r#"<div id="pc-app">"#),
            "the root opens the body: {html}"
        );
        assert_eq!(
            matching_div_close(&html, body) + "</div>".len(),
            end,
            "the root closes the body: {html}"
        );
        let page = html.find(">Privacy</h1>").expect(&html);
        let footer = html.find("<footer").expect(&html);
        assert!(body < page && footer < end, "the app is inside: {html}");
    }

    /// The app layout has no screen-tall column wrapper; the footer is in
    /// the shell's footer slot. A short page fits the phone's screen.
    #[tokio::test]
    async fn the_layout_has_no_screen_tall_column_of_its_own() {
        let html = crate::page::harness::render_view(|| {
            provide_context(RequestUrl::new("/"));
            view! {
                <Router>
                    <Layout />
                </Router>
            }
        })
        .await;
        // The app's wrapper does not have min-h-dvh.
        assert!(
            !html.contains(r#"<div class="flex min-h-dvh flex-col">"#),
            "app layout must not have min-h-dvh wrapper: {html}"
        );
        // The footer is rendered inside the shell's footer slot, marked by mt-auto.
        let footer = html.find("<footer").expect(&html);
        let mt_auto = html.find(r#"<div class="mt-auto">"#).expect(&html);
        assert!(
            mt_auto < footer,
            "footer must be inside shell's footer slot (mt-auto): {html}"
        );
    }

    /// 044 (user, 2026-10-05): the build badge sits in the page footer
    /// only — one copy, after the page, inside `<footer>`; none on the
    /// shell's panel (the rail, the phone's top bar).
    #[tokio::test]
    async fn the_badge_sits_in_the_footer_after_the_page() {
        // The page holds a `Resource`: it renders through the page's async
        // harness, like the server does.
        let html = crate::page::harness::render_view(|| {
            provide_context(RequestUrl::new("/"));
            view! {
                <Router>
                    <Layout />
                </Router>
            }
        })
        .await;
        let badge = format!(r#"title="{} · {}""#, BADGE.env, BADGE.sha);
        let page = html.find(r#"role="search""#).expect(&html);
        let footer = html.find("<footer").expect(&html);
        let copies: Vec<usize> = html.match_indices(&badge).map(|(i, _)| i).collect();
        assert_eq!(copies.len(), 1, "one badge, in the footer: {html}");
        assert!(copies[0] > page && copies[0] > footer, "{html}");
    }
}
