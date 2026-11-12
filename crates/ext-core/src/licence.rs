//! Store listing licence rule: free and open source only.
//!
//! An SPDX expression is accepted when every licence it names is OSI-approved and it uses only `AND`,
//! `OR`, `WITH` (OSI-compatible exceptions) and parentheses. Local installs may use any licence.

/// OSI-approved SPDX ids accepted by the store (a practical subset; extend by PR in craft-extensions).
pub const OSI_APPROVED: &[&str] = &[
    "0BSD", "AFL-3.0", "AGPL-3.0-only", "AGPL-3.0-or-later", "Apache-2.0", "Artistic-2.0", "BSD-1-Clause", "BSD-2-Clause",
    "BSD-3-Clause", "BSL-1.0", "CDDL-1.0", "ECL-2.0", "EPL-1.0", "EPL-2.0", "EUPL-1.2", "GPL-2.0-only", "GPL-2.0-or-later",
    "GPL-3.0-only", "GPL-3.0-or-later", "ISC", "LGPL-2.1-only", "LGPL-2.1-or-later", "LGPL-3.0-only", "LGPL-3.0-or-later",
    "MIT", "MIT-0", "MPL-2.0", "MS-PL", "MS-RL", "MulanPSL-2.0", "NCSA", "OFL-1.1", "OSL-3.0", "PostgreSQL", "UPL-1.0",
    "Unicode-3.0", "Unlicense", "Zlib",
];

const EXCEPTIONS: &[&str] = &["LLVM-exception", "Classpath-exception-2.0", "GCC-exception-3.1"];

/// Returns `Err(term)` naming the first term that is not acceptable.
pub fn check_osi(expr: &str) -> Result<(), String> {
    let spaced = expr.replace('(', " ( ").replace(')', " ) ");
    let mut depth = 0i32;
    let mut expect_licence = true;
    let mut after_with = false;
    let mut any = false;
    for tok in spaced.split_whitespace() {
        match tok {
            "(" if expect_licence => depth += 1,
            ")" if !expect_licence => {
                depth -= 1;
                if depth < 0 {
                    return Err(")".into());
                }
            }
            "AND" | "OR" if !expect_licence => expect_licence = true,
            "WITH" if !expect_licence => {
                expect_licence = true;
                after_with = true;
            }
            t if expect_licence && after_with => {
                if !EXCEPTIONS.contains(&t) {
                    return Err(t.into());
                }
                after_with = false;
                expect_licence = false;
            }
            t if expect_licence => {
                if !OSI_APPROVED.contains(&t) {
                    return Err(t.into());
                }
                any = true;
                expect_licence = false;
            }
            t => return Err(t.into()),
        }
    }
    if !any || expect_licence || depth != 0 { Err(expr.into()) } else { Ok(()) }
}

/// Words in listing text that suggest a paid or monetised plugin. Flagged for the maintainer; a
/// listing with payment fields is refused outright by the catalogue schema (it has none).
pub const MONETISATION_WORDS: &[&str] =
    &["licence key", "license key", "purchase", "buy now", "subscription", "premium", "paid version", "trial", "activation code", "pro version", "paywall", "gumroad", "patreon-only"];

pub fn monetisation_flags(text: &str) -> Vec<&'static str> {
    let lower = text.to_lowercase();
    MONETISATION_WORDS.iter().copied().filter(|w| lower.contains(w)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osi_expressions() {
        for ok in ["MIT", "MIT OR Apache-2.0", "(MIT OR Apache-2.0) AND OFL-1.1", "Apache-2.0 WITH LLVM-exception", "GPL-3.0-or-later"] {
            assert!(check_osi(ok).is_ok(), "{ok}");
        }
        for bad in ["", "Proprietary", "MIT OR", "CC-BY-NC-4.0", "MIT AND (Apache-2.0", "MIT WITH Foo", "SSPL-1.0", "BUSL-1.1"] {
            assert!(check_osi(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn flags_paid_wording() {
        assert_eq!(monetisation_flags("Get the Premium version"), vec!["premium"]);
        assert!(monetisation_flags("Free halftone").is_empty());
    }
}
