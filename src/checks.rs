use crate::common::{LineEntry, LintKind, Warning};

mod duplicated_key;
mod ending_blank_line;
mod extra_blank_line;
mod incorrect_delimiter;
mod key_without_value;
mod leading_character;
mod lowercase_key;
mod quote_character;
mod space_character;
mod substitution_key;
mod trailing_whitespace;
mod unordered_key;
mod value_without_quotes;

// This trait is used for checks which needs to know of only a single line
pub trait Check {
    fn run(&mut self, line: &LineEntry) -> Option<Warning>;
    fn name(&self) -> LintKind;
    fn skip_comments(&self) -> bool {
        true
    }
}

// Checklist for checks which needs to know of only a single line
fn checklist() -> Vec<Box<dyn Check>> {
    vec![
        Box::new(duplicated_key::DuplicatedKeyChecker::default()),
        Box::new(ending_blank_line::EndingBlankLineChecker::default()),
        Box::new(extra_blank_line::ExtraBlankLineChecker::default()),
        Box::new(incorrect_delimiter::IncorrectDelimiterChecker::default()),
        Box::new(key_without_value::KeyWithoutValueChecker::default()),
        Box::new(leading_character::LeadingCharacterChecker::default()),
        Box::new(lowercase_key::LowercaseKeyChecker::default()),
        Box::new(quote_character::QuoteCharacterChecker::default()),
        Box::new(space_character::SpaceCharacterChecker::default()),
        Box::new(substitution_key::SubstitutionKeyChecker::default()),
        Box::new(trailing_whitespace::TrailingWhitespaceChecker::default()),
        Box::new(unordered_key::UnorderedKeyChecker::default()),
        Box::new(value_without_quotes::ValueWithoutQuotesChecker::default()),
    ]
}

pub fn available_check_names() -> Vec<LintKind> {
    checklist().iter().map(|check| check.name()).collect()
}

pub fn run(lines: &[LineEntry], skip_checks: &[LintKind]) -> Vec<Warning> {
    let mut checks = checklist();

    // Skip checks with the --skip argument (globally)
    checks.retain(|c| !skip_checks.contains(&c.name()));

    // Skip checks with comments (dotenv-linter:on/off)
    let mut disabled_checks: Vec<LintKind> = Vec::new();

    let mut warnings: Vec<Warning> = Vec::new();

    for line in lines {
        if let Some(comment) = line.get_control_comment() {
            if comment.is_disabled() {
                // Disable checks from a comment using the dotenv-linter:off flag
                disabled_checks.extend(comment.checks);
            } else {
                // Enable checks if the comment has the dotenv-linter:on flag
                disabled_checks.retain(|&s| !comment.checks.contains(&s));
            }
        }

        for ch in &mut checks {
            if line.is_comment() && ch.skip_comments() {
                continue;
            }

            if disabled_checks.contains(&ch.name()) {
                continue;
            }

            if let Some(warning) = ch.run(line) {
                warnings.push(warning);
            }
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::tests::*;

    #[test]
    fn run_with_empty_vec_test() {
        let empty: Vec<LineEntry> = Vec::new();
        let expected: Vec<Warning> = Vec::new();
        let skip_checks: Vec<LintKind> = Vec::new();

        assert_eq!(expected, run(&empty, &skip_checks));
    }

    #[test]
    fn run_with_empty_line_test() {
        let lines: Vec<LineEntry> = vec![blank_line_entry(1, 1)];
        let expected: Vec<Warning> = Vec::new();
        let skip_checks: Vec<LintKind> = Vec::new();

        assert_eq!(expected, run(&lines, &skip_checks));
    }

    #[test]
    fn run_with_comment_line_test() {
        let lines: Vec<LineEntry> = vec![
            line_entry(1, 2, "# Comment = 'Value'"),
            blank_line_entry(2, 2),
        ];
        let expected: Vec<Warning> = Vec::new();
        let skip_checks: Vec<LintKind> = Vec::new();

        assert_eq!(expected, run(&lines, &skip_checks));
    }

    #[test]
    fn run_with_valid_line_test() {
        let lines: Vec<LineEntry> = vec![line_entry(1, 2, "FOO=BAR"), blank_line_entry(2, 2)];
        let expected: Vec<Warning> = Vec::new();
        let skip_checks: Vec<LintKind> = Vec::new();

        assert_eq!(expected, run(&lines, &skip_checks));
    }

    #[test]
    fn run_with_invalid_line_test() {
        let line = line_entry(1, 2, "FOO");
        let warning = Warning::new(
            line.number,
            LintKind::KeyWithoutValue,
            "The FOO key should be with a value or have an equal sign",
        );
        let lines: Vec<LineEntry> = vec![line, blank_line_entry(2, 2)];
        let expected: Vec<Warning> = vec![warning];
        let skip_checks: Vec<LintKind> = Vec::new();

        assert_eq!(expected, run(&lines, &skip_checks));
    }

    #[test]
    fn run_without_blank_line_test() {
        let line = line_entry(1, 1, "FOO=BAR");
        let warning = Warning::new(
            line.number,
            LintKind::EndingBlankLine,
            "No blank line at the end of the file",
        );
        let lines: Vec<LineEntry> = vec![line];
        let expected: Vec<Warning> = vec![warning];
        let skip_checks: Vec<LintKind> = Vec::new();

        assert_eq!(expected, run(&lines, &skip_checks));
    }

    #[test]
    fn skip_one_check() {
        let line1 = line_entry(1, 3, "FOO\n");
        let line2 = line_entry(2, 3, "1FOO\n");
        let warning = Warning::new(
            line2.number,
            LintKind::LeadingCharacter,
            "Invalid leading character detected",
        );
        let lines: Vec<LineEntry> = vec![line1, line2, blank_line_entry(3, 3)];
        let expected: Vec<Warning> = vec![warning];
        let skip_checks: Vec<LintKind> = vec![LintKind::KeyWithoutValue, LintKind::UnorderedKey];

        assert_eq!(expected, run(&lines, &skip_checks));
    }

    #[test]
    fn skip_all_checks() {
        let line = line_entry(1, 1, "FOO");
        let lines: Vec<LineEntry> = vec![line];
        let expected: Vec<Warning> = Vec::new();
        let skip_checks: Vec<LintKind> = vec![LintKind::KeyWithoutValue, LintKind::EndingBlankLine];

        assert_eq!(expected, run(&lines, &skip_checks));
    }

    #[test]
    fn skip_one_check_via_comment() {
        let line1 = line_entry(1, 4, "# dotenv-linter:off KeyWithoutValue\n");
        let line2 = line_entry(2, 4, "FOO\n");
        let line3 = line_entry(3, 4, "1FOO\n");
        let warning = Warning::new(
            line3.number,
            LintKind::LeadingCharacter,
            "Invalid leading character detected",
        );
        let lines: Vec<LineEntry> = vec![line1, line2, line3, blank_line_entry(4, 4)];
        let expected: Vec<Warning> = vec![warning];
        let skip_checks: Vec<LintKind> = vec![LintKind::UnorderedKey];

        assert_eq!(expected, run(&lines, &skip_checks));
    }

    #[test]
    fn skip_collision() {
        let line1 = line_entry(1, 4, "# dotenv-linter:on KeyWithoutValue\n");
        let line2 = line_entry(2, 4, "FOO\n");
        let line3 = line_entry(3, 4, "1FOO\n");
        let warning = Warning::new(
            line3.number,
            LintKind::LeadingCharacter,
            "Invalid leading character detected",
        );
        let lines: Vec<LineEntry> = vec![line1, line2, line3, blank_line_entry(4, 4)];
        let expected: Vec<Warning> = vec![warning];
        let skip_checks: Vec<LintKind> = vec![LintKind::KeyWithoutValue, LintKind::UnorderedKey];

        assert_eq!(expected, run(&lines, &skip_checks));
    }

    #[test]
    fn on_and_off_same_checks() {
        let line1 = line_entry(
            1,
            5,
            "# dotenv-linter:off KeyWithoutValue, LeadingCharacter\n",
        );
        let line2 = line_entry(2, 5, "FOO\n");
        let line3 = line_entry(3, 5, "# dotenv-linter:on LeadingCharacter\n");
        let line4 = line_entry(4, 5, "1FOO\n");
        let warning = Warning::new(
            line4.number,
            LintKind::LeadingCharacter,
            "Invalid leading character detected",
        );
        let lines: Vec<LineEntry> = vec![line1, line2, line3, line4, blank_line_entry(5, 5)];
        let expected: Vec<Warning> = vec![warning];
        let skip_checks: Vec<LintKind> = Vec::new();

        assert_eq!(expected, run(&lines, &skip_checks));
    }

    #[test]
    fn only_simple_comment() {
        let line = line_entry(1, 1, "# Simple comment");
        let warning = Warning::new(
            line.number,
            LintKind::EndingBlankLine,
            "No blank line at the end of the file",
        );
        let lines: Vec<LineEntry> = vec![line];
        let expected: Vec<Warning> = vec![warning];
        let skip_checks: Vec<LintKind> = Vec::new();

        assert_eq!(expected, run(&lines, &skip_checks));
    }

    #[test]
    fn check_name_list() {
        let available_check_names = available_check_names();
        for check in checklist() {
            let check_name = check.name();
            assert!(available_check_names.iter().any(|name| name == &check_name));
        }
    }

    #[test]
    fn unordered_key_with_control_comment_test() {
        let line_entries = vec![
            line_entry(1, 7, "FOO=BAR"),
            line_entry(2, 7, "# dotenv-linter:off LowercaseKey"),
            line_entry(3, 7, "Bar=FOO"),
            line_entry(4, 7, "bar=FOO"),
            line_entry(5, 7, "# dotenv-linter:on LowercaseKey"),
            line_entry(6, 7, "X=X"),
            blank_line_entry(7, 7),
        ];

        let expected: Vec<Warning> = Vec::new();
        let skip_checks: Vec<LintKind> = Vec::new();

        assert_eq!(expected, run(&line_entries, &skip_checks));
    }
}
