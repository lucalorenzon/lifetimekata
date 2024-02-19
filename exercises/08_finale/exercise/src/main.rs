use require_lifetimes::require_lifetimes;

#[derive(Debug, PartialEq, Eq)]
enum MatcherToken<'a> {
    /// This is just text without anything special.
    RawText(&'a str),
    /// This is when text could be any one of multiple
    /// strings. It looks like `(one|two|three)`, where
    /// `one`, `two` or `three` are the allowed strings.
    OneOfText(Vec<&'a str>),
    /// This is when you're happy to accept any single character.
    /// It looks like `.`
    WildCard,
}

#[derive(Debug, PartialEq, Eq)]
struct Matcher<'a> {
    /// This is the actual text of the matcher
    text: &'a str,
    /// This is a vector of the tokens inside the expression.
    tokens: Vec<MatcherToken<'a>>,
    /// This keeps track of the most tokens that this matcher has matched.
    most_tokens_matched: usize,
}

/* STATE MACHINE
 *  state1: curr: <char> && prev: None  => [push curr; next]
 *      state11: curr: <char> && prev: <char> => [push curr; next]
 *      state12: curr: '.' && prev: <char> => [store RawText, goto state2]
 *      state13: curr: '(' && prev: <char> => [store RawText, goto state3]
 *      state14: curr: ')' && prev: <char> => [store OneOfText, openGroup=false, next]
 *      state15: curr: '|' && prev: <char> => [push in OneOfText, next]
 *  state2: curr: '.' && prev: None => [push WildCard]
 *      state21: curr: <char> && prev: '.' => [push curr; next]
 *      state22: curr: '.' && prev: '.' => [goto state2]
 *      state23: curr: '(' && prev: '.' => [goto state3]
 *      state24: curr: ')' && prev: '.' => [goto state4]
 *      state25: curr: '|' && prev: '.' => [goto state5]
 *  state3: curr: '(' && prev: None => [openGroup = true, next]
 *      state31: curr: <char> && prev: '(' => [push curr; next]
 *      state32: curr: '.' && prev: '(' => [err]
 *      state33: curr: '(' && prev: '(' => [err]
 *      state34: curr: ')' && prev: '(' => [err]
 *      state35: curr: '|' && prev: '(' => [err]
 *  state4: curr: ')' && prev: None => [err]
 *  state5: curr: '|' && prev: None => [err]
 */

impl<'a> Matcher<'a> {
    /// This should take a string reference, and return
    /// an `Matcher` which has parsed that reference.
    #[require_lifetimes]
    fn new(text: &'a str) -> Option<Matcher<'a>> {
        let mut text_under_analysis = text;
        let mut tokens: Vec<MatcherToken> = vec![];

        while !text_under_analysis.is_empty() {
            match text_under_analysis {
                value if value.is_empty() => {
                    break;
                }
                value if value.starts_with('.') => {
                    tokens.push(MatcherToken::WildCard);
                    text_under_analysis = &text_under_analysis[1..];
                }
                value if value.starts_with('(') => {
                    if let Some(close_token) = text_under_analysis.find(')') {
                        let (alternative_text, remaining_string) =
                            text_under_analysis.split_at(close_token);
                        tokens.push(MatcherToken::OneOfText(
                            alternative_text.split('|').collect(),
                        ));
                        text_under_analysis = &remaining_string[1..];
                    } else {
                        return None;
                    }
                }
                value if value.starts_with(')') => {
                    return None;
                }
                _ => {
                    if let Some(next_token) = text_under_analysis.find(r"[.(]") {
                        let (raw_text, remaing_string) = text_under_analysis.split_at(next_token);
                        tokens.push(MatcherToken::RawText(raw_text));
                        text_under_analysis = remaing_string;
                    }
                }
            }
        }
        Some(Matcher {
            text,
            tokens,
            most_tokens_matched: 0,
        })
    }

    /// This should take a string, and return a vector of tokens, and the corresponding part
    /// of the given string. For examples, see the test cases below.
    #[require_lifetimes]
    fn match_string<'b, 'c>(&'b mut self, string: &'c str) -> Vec<(&'b MatcherToken<'a>, &'c str)> {
        let mut matched_tokens = vec![];
        let mut str_under_analysis = string;

        for token in self.tokens.iter() {
            match token {
                MatcherToken::WildCard => {
                    let byte_offset = str_under_analysis.chars().next().unwrap().len_utf8();
                    let matched_char = &str_under_analysis[..byte_offset];
                    matched_tokens.push((token, matched_char));
                    str_under_analysis = &str_under_analysis[byte_offset..];
                }
                MatcherToken::OneOfText(list_value) => {
                    if let Some(matched_str) = list_value
                        .iter()
                        .find(|&value| str_under_analysis.starts_with(value))
                    {
                        let byte_offset = matched_str.len();
                        matched_tokens.push((token, &str_under_analysis[..byte_offset]));
                        str_under_analysis = &str_under_analysis[matched_str.chars().count()..];
                        continue;
                    }
                    break;
                }
                MatcherToken::RawText(value) => {
                    if str_under_analysis.starts_with(value) {
                        let byte_offset = value.len();
                        matched_tokens.push((token, &str_under_analysis[..byte_offset]));
                        str_under_analysis = &str_under_analysis[byte_offset..];
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
        if matched_tokens.len() > self.most_tokens_matched {
            self.most_tokens_matched = matched_tokens.len();
        }
        matched_tokens
    }
}

fn main() {
    unimplemented!()
}

#[cfg(test)]
mod test {
    use super::{Matcher, MatcherToken};

    #[test]
    fn simple_test() {
        let match_string = "abc(d|e|f).".to_string();
        let mut matcher = Matcher::new(&match_string).unwrap();

        assert_eq!(matcher.most_tokens_matched, 0);

        {
            let candidate1 = "abcge".to_string();
            let result = matcher.match_string(&candidate1);
            assert_eq!(result, vec![(&MatcherToken::RawText("abc"), "abc"),]);
            assert_eq!(matcher.most_tokens_matched, 1);
        }

        {
            // Change 'e' to '💪' if you want to test unicode.
            let candidate1 = "abcde".to_string();
            let result = matcher.match_string(&candidate1);
            assert_eq!(
                result,
                vec![
                    (&MatcherToken::RawText("abc"), "abc"),
                    (&MatcherToken::OneOfText(vec!["d", "e", "f"]), "d"),
                    (&MatcherToken::WildCard, "e") // or '💪'
                ]
            );
            assert_eq!(matcher.most_tokens_matched, 3);
        }
    }

    #[test]
    fn broken_matcher() {
        let match_string = "abc(d|e|f.".to_string();
        let matcher = Matcher::new(&match_string);
        assert_eq!(matcher, None);
    }
}
