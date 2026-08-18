use std::borrow::Cow;

pub struct Flags {
    // TODO make private and accept flags
    pub ignore_case: bool,
}

impl Flags {
    // TODO move to a builder
    pub fn new(ignore_case: bool) -> Flags {
        Flags { ignore_case }
    }
}

pub fn search(query: &str, contents: impl Iterator<Item = String>, case_sensitive: Option<bool>) -> impl Iterator<Item = String> {
    let case_sensitive = case_sensitive.unwrap_or(true);
    let transformed_query = if case_sensitive {
        Cow::Borrowed(query)
    } else {
        Cow::Owned(query.to_lowercase())
    };
    contents
        .filter(move |line| -> bool {
            let processed: &str = match case_sensitive {
                false => &line.to_lowercase(),
                true => line,
            };
            processed.contains(transformed_query.as_ref())
        })
}

/* 
1. Accept piped  DONE
2. Glob multiple files
3. Accept regex
4. -i case insensitive
5. -B / -A before, after flags
6. Unicode robustness
7. -w flag
8. -c flag count
9. -v flag not
10. -x match line
*/

#[cfg(test)]
mod tests {
    use super::*;

    const CONTENTS: &str = "\
Rust:
safe, fast, productive.
Pick three (3).";

    fn to_iter() -> impl Iterator<Item = String> {
        CONTENTS.lines().map(|line| line.to_string())
    }

    #[test]
    fn one_result() {
        let query = "duct";

        assert_eq!(vec!["safe, fast, productive."], search(query, to_iter(), None).collect::<Vec<String>>());
    }

    #[test]
    fn no_results() {
        let query = "deduce";
        assert_eq!(search(query, to_iter(), None).count(), 0);
    }

    #[test]
    fn empty_query() {
        let query = "";
        assert_eq!(
            vec!["Rust:", "safe, fast, productive.", "Pick three (3)."],
            search(query, to_iter(), None).collect::<Vec<String>>(),
        );
    }

    #[test]
    fn empty_contents() {
        let query = "Apple";
        assert_eq!(search(query, "".lines().map(|line| line.to_string()), None).count(), 0);
    }

    #[test]
    fn test_case_sensitive_no_match() {
        let query = "rust";
        assert_eq!(search(query, to_iter(), Some(true)).count(), 0);
    }

    #[test]
    fn test_case_insensitive_match() {
        let query = "rUst";
        assert_eq!(
            search(query, to_iter(), Some(false)).collect::<Vec<String>>(),
            vec!["Rust:"],
        );
    }

    #[test]
    fn match_regex() {
        unimplemented!();
    }

    #[test]
    fn return_line_numbers() {
        unimplemented!();
    }

    #[test]
    fn return_non_matches() {
        unimplemented!();
    }

    #[test]
    fn match_full_words_flag() {
        unimplemented!();
    }

    #[test]
    fn match_full_line_flag() {
        unimplemented!();
    }

}
