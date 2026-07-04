use linkme::distributed_slice;

use crate::api::Api;

/// Every `#[papermc::test]` in the final plugin cdylib, including those from all linked crates.
#[distributed_slice]
pub static TESTS: [TestCase];

/// The function-pointer shape stored in [TestCase::run].
pub type RunFn = for<'a, 'l, 'c> fn(&'c mut TestCtx<'a, 'l>) -> TestOutcome;

/// A registered test.
///
/// Constructed by `#[papermc::test]`
pub struct TestCase {
    pub name: &'static str,
    pub ignored: bool,
    pub ignore_reason: Option<&'static str>,
    pub run: RunFn,
}

/// Per-test execution context handed to [TestCase::run].
pub struct TestCtx<'a, 'l> {
    pub api: Api<'a, 'l>,
}

pub enum TestOutcome {
    Passed,
    Failed(String),
    Skipped(&'static str),
}

pub enum Fixture<T> {
    Present(T),
    Skip(&'static str),
}

/// Extraction of a test-function fixture parameter from the execution context.
///
/// Not all fixtures will be present in every test context. For example, a `Player` fixture would
/// only be present if a player ran the test command, as opposed to the server console.
pub trait TestFixture<'a, 'l>: Sized {
    fn extract(ctx: &mut TestCtx<'a, 'l>) -> eyre::Result<Fixture<Self>>;
}

/// Conversion from a test function's return type to [TestOutcome].
pub trait IntoOutcome {
    fn into_outcome(self) -> TestOutcome;
}

impl IntoOutcome for () {
    fn into_outcome(self) -> TestOutcome {
        TestOutcome::Passed
    }
}

impl IntoOutcome for eyre::Result<()> {
    fn into_outcome(self) -> TestOutcome {
        match self {
            Ok(()) => TestOutcome::Passed,
            Err(e) => TestOutcome::Failed(format!("{e:?}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[papermc::test]
    fn sample(api: &mut Api) {
        let _ = api;
    }

    #[test]
    fn attribute_registers_test_case() {
        let case = TESTS
            .iter()
            .find(|c| c.name == "papermc::testing::tests::sample")
            .expect("sample test registered in TESTS");
        assert!(!case.ignored);
        assert_eq!(case.ignore_reason, None);
    }
}
