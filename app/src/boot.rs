//! The service's boot configuration: which source to build from the environment (slice 4 of task 005).
//!
//! `SEARCH_API_URL` and `SEARCH_API_TOKEN` go together: both set and
//! non-blank → the real source; neither set → the mock, with one log line;
//! anything else (one alone, or a set but blank value) refuses to start,
//! naming the variable and never its value. The token is read from the
//! environment once here, wrapped in `ApiToken` (whose `Debug` is
//! redacted), and handed to the client: it is in no file and in no message.

use std::fmt;

use crate::search::mock::MockSource;
use crate::search::real::{API_TIMEOUT, ApiToken, RealSource};
use crate::search::source::Source;

/// The variable holding the API's base URL.
pub const URL_VAR: &str = "SEARCH_API_URL";
/// The variable holding the API's token.
pub const TOKEN_VAR: &str = "SEARCH_API_TOKEN";

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
}

impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BootError::Missing(var) => write!(
                f,
                "{var} is not set, but the other of {URL_VAR} and {TOKEN_VAR} is: set both, or neither for the mock"
            ),
            BootError::Empty(var) => write!(
                f,
                "{var} is set but empty: give it a value, or unset both {URL_VAR} and {TOKEN_VAR} for the mock"
            ),
        }
    }
}

impl std::error::Error for BootError {}

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
    /// [`BootError`] when only one of the two is set, or one is blank.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Config, BootError> {
        let url = read(&lookup, URL_VAR)?;
        let token = read(&lookup, TOKEN_VAR)?;
        match (url, token) {
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
            let (BootError::Missing(var) | BootError::Empty(var)) = want;
            for shown in [err.to_string(), format!("{err:?}")] {
                assert!(shown.contains(var), "{name}: {shown} names {var}");
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
}
