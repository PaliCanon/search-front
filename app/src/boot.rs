//! The service's boot configuration: which source to build from the environment (slice 4 of task 005).
//!
//! `SEARCH_API_URL` and `SEARCH_API_TOKEN` go together: both set and
//! non-blank → the real source; neither set → the mock, with one log line;
//! anything else (one alone, or a set but blank value) refuses to start,
//! naming the variable and never its value. A deployed build (Cloud Run's
//! `K_SERVICE`, or `LEPTOS_ENV` prod) never runs on the mock, and requires
//! `ORIGIN_SECRET` (task 007: the origin answers Cloudflare's Worker only).
//! The token is read from the environment once here, wrapped in `ApiToken`
//! (whose `Debug` is redacted), and handed to the client: it is in no file
//! and in no message.

use std::fmt;

use crate::origin::{Origin, OriginSecret};
use crate::search::mock::MockSource;
use crate::search::real::{API_TIMEOUT, ApiToken, RealSource};
use crate::search::source::Source;

/// The variable holding the API's base URL.
pub const URL_VAR: &str = "SEARCH_API_URL";
/// The variable holding the API's token.
pub const TOKEN_VAR: &str = "SEARCH_API_TOKEN";

/// The variable holding the secret the Worker sends in `x-origin-auth`.
pub const ORIGIN_VAR: &str = "ORIGIN_SECRET";

/// Which source the service runs with.
#[derive(Debug)]
pub enum Config {
    /// The in-repo mock: neither variable is set.
    Mock,
    /// The PaliCanon API.
    Real {
        /// The API's base URL.
        url: String,
        /// The API's token; its `Debug` is redacted.
        token: ApiToken,
    },
}

/// Why the service refuses to start. Names a variable, never a value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootError {
    /// The other variable is set but this one is not.
    Missing(&'static str),
    /// The variable is set to nothing, or to blanks.
    Empty(&'static str),
    /// A deployed build (Cloud Run, or `LEPTOS_ENV` prod) has no origin secret.
    OriginRequired,
    /// A deployed build has neither search variable: it would serve the mock.
    MockInProduction,
}

impl BootError {
    /// The variable the error is about (a name, never a value).
    #[must_use]
    pub fn variable(self) -> &'static str {
        match self {
            BootError::Missing(var) | BootError::Empty(var) => var,
            BootError::OriginRequired => ORIGIN_VAR,
            BootError::MockInProduction => URL_VAR,
        }
    }
}

impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BootError::Missing(var) => write!(
                f,
                "{var} is not set, but the other of {URL_VAR} and {TOKEN_VAR} is: set both, or neither for the mock"
            ),
            BootError::Empty(ORIGIN_VAR) => write!(
                f,
                "{ORIGIN_VAR} is set but empty: give it a value, or unset it for a local run"
            ),
            BootError::Empty(var) => write!(
                f,
                "{var} is set but empty: give it a value, or unset both {URL_VAR} and {TOKEN_VAR} for the mock"
            ),
            BootError::OriginRequired => write!(
                f,
                "{ORIGIN_VAR} is not set: a deployed build answers only the requests that carry it"
            ),
            BootError::MockInProduction => write!(
                f,
                "{URL_VAR} and {TOKEN_VAR} are not set: a deployed build never serves the mock"
            ),
        }
    }
}

impl std::error::Error for BootError {}

/// Whether this is a deployed build: Cloud Run sets `K_SERVICE` in every
/// container, and the image sets `LEPTOS_ENV` to `PROD`. A local
/// `cargo leptos watch` (`LEPTOS_ENV=DEV`) is neither.
fn deployed(lookup: &impl Fn(&str) -> Option<String>) -> bool {
    let production = lookup("LEPTOS_ENV").is_some_and(|env| {
        let env = env.trim();
        env.eq_ignore_ascii_case("prod") || env.eq_ignore_ascii_case("production")
    });
    production || lookup("K_SERVICE").is_some()
}

impl Origin {
    /// Reads `ORIGIN_SECRET` through `lookup`.
    ///
    /// # Errors
    /// [`BootError`] when the variable is blank, or unset in a deployed build.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Origin, BootError> {
        match read(&lookup, ORIGIN_VAR)? {
            Some(secret) => Ok(Origin::Checked(OriginSecret::new(secret))),
            None if deployed(&lookup) => Err(BootError::OriginRequired),
            None => Ok(Origin::Open),
        }
    }
}

/// The value of `var` through `lookup`, trimmed; blank is an error, not "unset".
fn read(
    lookup: &impl Fn(&str) -> Option<String>,
    var: &'static str,
) -> Result<Option<String>, BootError> {
    match lookup(var) {
        None => Ok(None),
        Some(value) if value.trim().is_empty() => Err(BootError::Empty(var)),
        Some(value) => Ok(Some(value.trim().to_owned())),
    }
}

impl Config {
    /// Reads the two variables through `lookup` (`std::env::var(..).ok()` in
    /// production, a table in tests).
    ///
    /// # Errors
    /// [`BootError`] when only one of the two is set, or one is blank, or
    /// neither is set in a deployed build (it never serves the mock).
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Config, BootError> {
        let url = read(&lookup, URL_VAR)?;
        let token = read(&lookup, TOKEN_VAR)?;
        match (url, token) {
            (None, None) if deployed(&lookup) => Err(BootError::MockInProduction),
            (None, None) => Ok(Config::Mock),
            (Some(url), Some(token)) => Ok(Config::Real {
                url,
                token: ApiToken::new(token),
            }),
            (Some(_), None) => Err(BootError::Missing(TOKEN_VAR)),
            (None, Some(_)) => Err(BootError::Missing(URL_VAR)),
        }
    }

    /// The one line logged at boot: which source, never its URL or token.
    #[must_use]
    pub fn log_line(&self) -> &'static str {
        match self {
            Config::Mock => "search: mock",
            Config::Real { .. } => "search: real",
        }
    }

    /// Builds the source, once.
    #[must_use]
    pub fn into_source(self) -> Source {
        match self {
            Config::Mock => Source::Mock(MockSource),
            Config::Real { url, token } => Source::Real(RealSource::new(&url, token, API_TIMEOUT)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A token no real one looks like: it must be in no message.
    const CANARY: &str = "pc_canary_boot_91d2";
    const URL: &str = "https://api.example.test";

    fn lookup(env: &[(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        let env = env.to_vec();
        move |name| {
            env.iter()
                .find(|(var, _)| *var == name)
                .map(|(_, value)| (*value).to_owned())
        }
    }

    /// What a row of the table expects.
    enum Expect {
        Mock,
        Real,
        Refused(BootError),
    }

    /// A named environment and what booting it must give.
    type Row = (&'static str, Vec<(&'static str, &'static str)>, Expect);

    fn table() -> Vec<Row> {
        let mut rows = deployed_rows();
        rows.extend(local_rows());
        rows
    }

    /// The rows about a deployed build (task 007: it never serves the mock).
    fn deployed_rows() -> Vec<Row> {
        vec![
            (
                "deployed (LEPTOS_ENV=PROD), neither: no silent mock",
                vec![("LEPTOS_ENV", "PROD")],
                Expect::Refused(BootError::MockInProduction),
            ),
            (
                "deployed (K_SERVICE), neither: no silent mock",
                vec![("K_SERVICE", "search-front-dev")],
                Expect::Refused(BootError::MockInProduction),
            ),
            (
                "deployed with both",
                vec![("LEPTOS_ENV", "PROD"), (URL_VAR, URL), (TOKEN_VAR, CANARY)],
                Expect::Real,
            ),
            (
                "deployed with a url only still names the token",
                vec![("K_SERVICE", "x"), (URL_VAR, URL)],
                Expect::Refused(BootError::Missing(TOKEN_VAR)),
            ),
            (
                "LEPTOS_ENV=DEV is not deployed",
                vec![("LEPTOS_ENV", "DEV")],
                Expect::Mock,
            ),
        ]
    }

    fn local_rows() -> Vec<Row> {
        vec![
            ("neither", vec![], Expect::Mock),
            (
                "both",
                vec![(URL_VAR, URL), (TOKEN_VAR, CANARY)],
                Expect::Real,
            ),
            (
                "url only",
                vec![(URL_VAR, URL)],
                Expect::Refused(BootError::Missing(TOKEN_VAR)),
            ),
            (
                "token only",
                vec![(TOKEN_VAR, CANARY)],
                Expect::Refused(BootError::Missing(URL_VAR)),
            ),
            (
                "empty token, no url (not 'unset': it refuses)",
                vec![(TOKEN_VAR, "")],
                Expect::Refused(BootError::Empty(TOKEN_VAR)),
            ),
            (
                "empty token with a url",
                vec![(URL_VAR, URL), (TOKEN_VAR, "")],
                Expect::Refused(BootError::Empty(TOKEN_VAR)),
            ),
            (
                "blank token with a url",
                vec![(URL_VAR, URL), (TOKEN_VAR, "  \t")],
                Expect::Refused(BootError::Empty(TOKEN_VAR)),
            ),
            (
                "empty url, no token",
                vec![(URL_VAR, "")],
                Expect::Refused(BootError::Empty(URL_VAR)),
            ),
            (
                "empty url with a token",
                vec![(URL_VAR, ""), (TOKEN_VAR, CANARY)],
                Expect::Refused(BootError::Empty(URL_VAR)),
            ),
        ]
    }

    #[test]
    fn the_boot_cases() {
        for (name, env, expect) in table() {
            let got = Config::from_lookup(lookup(&env));
            match (expect, got) {
                (Expect::Mock, Ok(Config::Mock)) => {}
                (Expect::Real, Ok(Config::Real { url, .. })) => assert_eq!(url, URL, "{name}"),
                (Expect::Refused(want), Err(err)) => assert_eq!(err, want, "{name}"),
                (_, got) => panic!("{name}: unexpected {got:?}"),
            }
        }
    }

    #[test]
    fn values_are_trimmed() {
        let env = [
            (URL_VAR, " https://api.example.test\n"),
            (TOKEN_VAR, CANARY),
        ];
        let Ok(Config::Real { url, .. }) = Config::from_lookup(lookup(&env)) else {
            panic!("expected Real");
        };
        assert_eq!(url, URL);
    }

    #[test]
    fn the_error_names_the_variable_and_never_the_value() {
        for (name, env, expect) in table() {
            let Expect::Refused(want) = expect else {
                continue;
            };
            let err = Config::from_lookup(lookup(&env)).unwrap_err();
            let var = want.variable();
            let shown = err.to_string();
            assert!(shown.contains(var), "{name}: {shown} names {var}");
            for shown in [shown, format!("{err:?}")] {
                assert!(!shown.contains(CANARY), "{name}: token leaked: {shown}");
            }
        }
    }

    #[test]
    fn an_empty_token_error_says_it_is_empty() {
        let err = Config::from_lookup(lookup(&[(TOKEN_VAR, "")])).unwrap_err();
        let shown = err.to_string();
        assert!(shown.contains(TOKEN_VAR), "{shown}");
        assert!(shown.contains("empty"), "{shown}");
    }

    #[test]
    fn the_debug_of_the_config_has_no_token() {
        let env = [(URL_VAR, URL), (TOKEN_VAR, CANARY)];
        let config = Config::from_lookup(lookup(&env)).unwrap();
        let shown = format!("{config:?}");
        assert!(!shown.contains(CANARY), "token leaked: {shown}");
        assert!(shown.contains(URL), "the url is no secret: {shown}");
    }

    #[test]
    fn the_log_line_names_the_source_and_no_secret() {
        assert_eq!(Config::Mock.log_line(), "search: mock");
        let env = [(URL_VAR, URL), (TOKEN_VAR, CANARY)];
        let real = Config::from_lookup(lookup(&env)).unwrap();
        assert_eq!(real.log_line(), "search: real");
    }

    #[test]
    fn each_config_builds_its_source() {
        assert!(matches!(Config::Mock.into_source(), Source::Mock(_)));
        let env = [(URL_VAR, URL), (TOKEN_VAR, CANARY)];
        let real = Config::from_lookup(lookup(&env)).unwrap();
        assert!(matches!(real.into_source(), Source::Real(_)));
    }

    /// The origin secret: a value no real one looks like.
    const SECRET: &str = "origin_canary_boot_6b0d";

    /// What a row of the origin table expects.
    enum Check {
        Open,
        Checked,
        Refused(BootError),
    }

    /// A named environment and what the origin must make of it.
    type OriginRow = (&'static str, Vec<(&'static str, &'static str)>, Check);

    fn origin_table() -> Vec<OriginRow> {
        vec![
            ("local, unset: no check", vec![], Check::Open),
            (
                "LEPTOS_ENV=DEV, unset: no check",
                vec![("LEPTOS_ENV", "DEV")],
                Check::Open,
            ),
            (
                "PROD, unset",
                vec![("LEPTOS_ENV", "PROD")],
                Check::Refused(BootError::OriginRequired),
            ),
            (
                "prod in lower case, unset",
                vec![("LEPTOS_ENV", "prod")],
                Check::Refused(BootError::OriginRequired),
            ),
            (
                "production, unset",
                vec![("LEPTOS_ENV", "Production")],
                Check::Refused(BootError::OriginRequired),
            ),
            (
                "Cloud Run (K_SERVICE), unset",
                vec![("K_SERVICE", "search-front-dev")],
                Check::Refused(BootError::OriginRequired),
            ),
            (
                "blank locally is refused too",
                vec![(ORIGIN_VAR, "  ")],
                Check::Refused(BootError::Empty(ORIGIN_VAR)),
            ),
            (
                "empty when deployed",
                vec![("LEPTOS_ENV", "PROD"), (ORIGIN_VAR, "")],
                Check::Refused(BootError::Empty(ORIGIN_VAR)),
            ),
            (
                "set locally: checked",
                vec![(ORIGIN_VAR, SECRET)],
                Check::Checked,
            ),
            (
                "set when deployed: checked",
                vec![("K_SERVICE", "x"), (ORIGIN_VAR, SECRET)],
                Check::Checked,
            ),
        ]
    }

    #[test]
    fn the_origin_cases() {
        for (name, env, expect) in origin_table() {
            let got = Origin::from_lookup(lookup(&env));
            match (expect, got) {
                (Check::Open, Ok(Origin::Open)) => {}
                (Check::Checked, Ok(Origin::Checked(secret))) => {
                    assert!(secret.matches(SECRET.as_bytes()), "{name}");
                }
                (Check::Refused(want), Err(err)) => assert_eq!(err, want, "{name}"),
                (_, got) => panic!("{name}: unexpected {got:?}"),
            }
        }
    }

    #[test]
    fn the_origin_secret_is_trimmed() {
        let env = [(ORIGIN_VAR, " origin_canary_boot_6b0d\n")];
        let Ok(Origin::Checked(secret)) = Origin::from_lookup(lookup(&env)) else {
            panic!("expected Checked");
        };
        assert!(secret.matches(SECRET.as_bytes()));
    }

    #[test]
    fn the_origin_errors_name_the_variable_and_never_the_value() {
        for (name, env, expect) in origin_table() {
            let Check::Refused(want) = expect else {
                continue;
            };
            let err = Origin::from_lookup(lookup(&env)).unwrap_err();
            let shown = err.to_string();
            assert!(shown.contains(ORIGIN_VAR), "{name}: {shown}");
            for shown in [shown, format!("{err:?}")] {
                assert!(!shown.contains(SECRET), "{name}: secret leaked: {shown}");
            }
            assert_eq!(want.variable(), ORIGIN_VAR, "{name}");
        }
    }

    #[test]
    fn the_debug_of_the_origin_has_no_secret() {
        let env = [(ORIGIN_VAR, SECRET)];
        let origin = Origin::from_lookup(lookup(&env)).unwrap();
        let shown = format!("{origin:?}");
        assert!(!shown.contains(SECRET), "secret leaked: {shown}");
        assert!(!origin.log_line().contains(SECRET));
        assert_eq!(origin.log_line(), "origin: checked");
        assert_eq!(Origin::Open.log_line(), "origin: open (local)");
    }

    #[test]
    fn the_mock_refusal_names_the_search_variables_never_a_value() {
        let err = Config::from_lookup(lookup(&[("LEPTOS_ENV", "PROD")])).unwrap_err();
        assert_eq!(err, BootError::MockInProduction);
        let shown = err.to_string();
        assert!(
            shown.contains(URL_VAR) && shown.contains(TOKEN_VAR),
            "{shown}"
        );
    }
}
