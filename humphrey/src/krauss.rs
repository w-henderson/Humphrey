use std::iter::Peekable;
use std::str::Chars;

/// Checks whether a wild string (a string possibly containing wildcards) matches a tame string (a string with no wildcards).
///
/// Uses Krauss' algorithm, which you can read more about
/// [here](https://www.drdobbs.com/architecture-and-design/matching-wildcards-an-algorithm/210200888).
pub fn wildcard_match(wild: &str, tame: &str) -> bool {
    let mut wild_iter: Peekable<Chars> = wild.chars().peekable();
    let mut tame_iter: Peekable<Chars> = tame.chars().peekable();
    let mut after_last_wild: Option<Peekable<Chars>> = None;

    loop {
        let tame_char = tame_iter.peek().copied();
        let wild_char = wild_iter.peek().copied();

        if tame_char.is_none() {
            // If the tame string is finished and so far matches

            if wild_char.is_none() {
                // If there are no more characters to match in the wild string, they are identical
                // For example "abc" matches "abc"
                return true;
            } else if wild_char == Some('*') {
                // If the wild string still has a wildcard character, this could match zero characters
                // Move on to the next wildcard character and run this section again since `tame_char` will still be `None`
                // For example, "abc" matches "abc*"
                wild_iter.next();
                continue;
            }

            // If the tame string is finished but the wild string continues with non-wildcard characters, they do not match
            // For example, "abc" does not match "abcdef"
            return false;
        } else {
            // If the tame string has more characters

            if tame_char != wild_char {
                // If the tame character and the wild character do not match, the only way they can be identical is if there
                //   was previously or is currently a wildcard character
                // For example, "abcd" matches "abc*" and "a*"
                if wild_char == Some('*') {
                    // If the wild character is a wildcard character, store the position after it
                    // This is needed in cases such as "abcd" matching "a*d"
                    wild_iter.next();
                    after_last_wild = Some(wild_iter.clone());
                    continue;
                } else if let Some(after_last_wild_iter) = &after_last_wild {
                    // If there is not a new wildcard character, but there has previously been one, move the iterator to
                    //   immediately after the last wildcard character, and store the next character.
                    wild_iter = after_last_wild_iter.clone();
                    let wild_char = wild_iter.peek().copied();

                    if wild_char.is_none() {
                        // If there are no more wild characters, this means that the last character of the wild string was a
                        //   wildcard character and the strings matched up to that point. Therefore, the strings match.
                        // For example, "abcd" matches "a*"
                        return true;
                    } else if tame_char == wild_char {
                        // If the characters do match, the end of the wildcard segment must have been reached, so increment the
                        //   iterator.
                        wild_iter.next();
                    }

                    tame_iter.next();
                    continue;
                } else {
                    // If the characters do not match, are not wildcard, do not follow a wildcard, and do not complete a wildcard
                    //   segment, then the strings do not match.
                    return false;
                }
            }
        }

        tame_iter.next();
        wild_iter.next();
    }
}
