pub(crate) const USAGE: &str =
    "usage: /test [FILTER...] [--exact] [--list] [--ignored] [--include-ignored]";

#[derive(Debug)]
pub(crate) struct RunSpec {
    /// Positional args. A test runs if it matches any filter; no filters matches everything.
    pub filters: Vec<String>,
    /// Filters match by whole-name equality instead of substring.
    pub exact: bool,
    /// Print matched test names without running anything.
    pub list: bool,
    /// Run only tests marked `ignore`.
    pub ignored: bool,
    /// Run everything, including tests marked `ignore`.
    pub include_ignored: bool,
}

/// What the runner should do with a matched test.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Disposition {
    Run,
    /// Report as `ignored` without running (the default treatment of `ignore` tests).
    ReportIgnored,
    /// Drop from the run entirely (non-ignored tests under `--ignored`).
    Exclude,
}

/// Parse command arguments. Flags may appear anywhere; every non-flag argument is a filter.
pub(crate) fn parse(args: &[String]) -> Result<RunSpec, String> {
    let mut spec = RunSpec {
        filters: Vec::new(),
        exact: false,
        list: false,
        ignored: false,
        include_ignored: false,
    };
    for arg in args {
        match arg.as_str() {
            "--exact" => spec.exact = true,
            "--list" => spec.list = true,
            "--ignored" => spec.ignored = true,
            "--include-ignored" => spec.include_ignored = true,
            flag if flag.starts_with('-') => {
                return Err(format!("unrecognized option '{flag}'\n{USAGE}"));
            }
            filter => spec.filters.push(filter.to_string()),
        }
    }
    if spec.ignored && spec.include_ignored {
        return Err(format!(
            "the options --ignored and --include-ignored are mutually exclusive\n{USAGE}"
        ));
    }
    Ok(spec)
}

pub(crate) fn disposition(spec: &RunSpec, ignored: bool) -> Disposition {
    if spec.include_ignored {
        Disposition::Run
    } else if spec.ignored {
        if ignored {
            Disposition::Run
        } else {
            Disposition::Exclude
        }
    } else if ignored {
        Disposition::ReportIgnored
    } else {
        Disposition::Run
    }
}

/// Whether the test named `name` passes the spec's filters.
pub(crate) fn matches(spec: &RunSpec, name: &str) -> bool {
    if spec.filters.is_empty() {
        return true;
    }
    if spec.exact {
        spec.filters.iter().any(|f| f == name)
    } else {
        spec.filters.iter().any(|f| name.contains(f.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_no_args() {
        let spec = parse(&[]).unwrap();
        assert!(spec.filters.is_empty());
        assert!(!spec.exact);
        assert!(!spec.list);
    }

    #[test]
    fn parse_filters_and_flags_intermixed() {
        let spec = parse(&strings(&["foo", "--exact", "bar", "--list"])).unwrap();
        assert_eq!(spec.filters, vec!["foo", "bar"]);
        assert!(spec.exact);
        assert!(spec.list);
    }

    #[test]
    fn parse_rejects_unknown_flag() {
        let err = parse(&strings(&["--bogus"])).unwrap_err();
        assert!(err.contains("unrecognized option '--bogus'"), "{err}");
        assert!(err.contains(USAGE), "{err}");
    }

    #[test]
    fn parse_rejects_ignored_with_include_ignored() {
        let err = parse(&strings(&["--ignored", "--include-ignored"])).unwrap_err();
        assert!(err.contains("mutually exclusive"), "{err}");
    }

    #[test]
    fn disposition_default_reports_ignored() {
        let spec = parse(&[]).unwrap();
        assert_eq!(disposition(&spec, false), Disposition::Run);
        assert_eq!(disposition(&spec, true), Disposition::ReportIgnored);
    }

    #[test]
    fn disposition_ignored_runs_only_ignored() {
        let spec = parse(&strings(&["--ignored"])).unwrap();
        assert_eq!(disposition(&spec, true), Disposition::Run);
        assert_eq!(disposition(&spec, false), Disposition::Exclude);
    }

    #[test]
    fn disposition_include_ignored_runs_everything() {
        let spec = parse(&strings(&["--include-ignored"])).unwrap();
        assert_eq!(disposition(&spec, true), Disposition::Run);
        assert_eq!(disposition(&spec, false), Disposition::Run);
    }

    #[test]
    fn matches_everything_without_filters() {
        let spec = parse(&[]).unwrap();
        assert!(matches(&spec, "any::name::at_all"));
    }

    #[test]
    fn matches_any_substring_filter() {
        let spec = parse(&strings(&["village", "spawn"])).unwrap();
        assert!(matches(&spec, "nitwittery_plugin::spawn::choose_point"));
        assert!(matches(&spec, "nitwittery_plugin::locate::village_lookup"));
        assert!(!matches(&spec, "papermc::testing::selftest::class_lookup"));
    }

    #[test]
    fn matches_exact_requires_equality() {
        let spec = parse(&strings(&[
            "papermc::testing::selftest::class_lookup",
            "--exact",
        ]))
        .unwrap();
        assert!(matches(&spec, "papermc::testing::selftest::class_lookup"));
        assert!(!matches(
            &spec,
            "papermc::testing::selftest::class_lookup_more"
        ));
    }
}
