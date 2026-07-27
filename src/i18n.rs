//! Locale-file parity test — ensures `en-US.ftl` and `fr-FR.ftl` (the
//! dioxus-i18n bundles loaded in `main.rs`) define the same key set, so a
//! new key added to one locale can't silently be missing from the other.

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    fn keys(ftl: &str) -> HashSet<String> {
        ftl.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .filter_map(|line| line.split_once('='))
            .map(|(key, _)| key.trim().to_owned())
            .collect()
    }

    #[test]
    fn unit_locale_files_have_matching_keys() {
        let en = keys(include_str!("./i18n/en-US.ftl"));
        let fr = keys(include_str!("./i18n/fr-FR.ftl"));
        let only_en: Vec<_> = en.difference(&fr).collect();
        let only_fr: Vec<_> = fr.difference(&en).collect();
        assert!(
            only_en.is_empty() && only_fr.is_empty(),
            "locale key mismatch — only in en-US.ftl: {:?}, only in fr-FR.ftl: {:?}",
            only_en,
            only_fr
        );
    }
}
