pub mod preview {
    pub fn process_wikilinks(content: &str) -> String {
        use regex::Regex;
        let re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
        re.replace_all(content, |caps: &regex::Captures| {
            let name = &caps[1];
            format!("<wikilink>{}</wikilink>", name)
        })
        .to_string()
    }
}
