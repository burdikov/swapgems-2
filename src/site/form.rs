use std::fmt::{Display, Formatter};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Form {
    buy_or_sell: String,

    selling_curr: String,
    buying_curr: String,

    sum: String,
    in_parts: bool,
    cb: bool,
    rate: String,

    eu_methods: Vec<String>,
    ru_methods: Vec<String>,
    eu_methods_str: String,
    ru_methods_str: String,

    eu_more: bool,
    ru_more: bool,

    comment: String,
    cash: bool,
    cash_only: bool,
    location: String,
}

#[cfg(test)]
mod tests {
    use crate::site::form::methods;

    #[test]
    fn full_methods_work() {
        let mets = vec!["bizum".to_string(), "n25".to_string()];
        let additional = "a1, a2".to_string();
        let more = true;  // not necessarily true even when additional is not empty
        let res = methods(&mets, &additional, more, "eu: ");

        assert_eq!(res, "eu: bizum, n25, a1, a2\n".to_string())
    }

    #[test]
    fn empty_methods_work() {
        let mets: Vec<String> = vec![];
        assert_eq!(methods(&mets, "", false, "eu: "), "".to_string());
    }

    #[test]
    fn quick_only_methods_work() {
        let mets = ["n25", "bizum"].iter().map(|s|s.to_string()).collect();
        assert_eq!(methods(&mets, "", false, "eu: "), "eu: n25, bizum\n".to_string())
    }

    #[test]
    fn additional_only_methods_work() {
        let mets: Vec<String> = vec![];
        let additional = "n249, revolut";

        assert_eq!(methods(&mets, additional, true, "ru: "), "ru: n249, revolut\n".to_string())
    }

    #[test]
    fn no_more_methods_work() {
        let mets = ["a", "b"].iter().map(|s|s.to_string()).collect();
        let additional = "c, d";

        assert_eq!(methods(&mets, additional, false, ""), "a, b\n".to_string())
    }
}

fn methods(methods: &Vec<String>, additional: &str, more: bool, prefix: &str) -> String {
    // who let the overengineers out?

    // return early if everything is empty
    if methods.is_empty() && additional.is_empty() { return String::default(); }

    let mut res = if more { String::with_capacity(80 + prefix.len()) } else { String::with_capacity(40 + prefix.len()) };
    res.push_str(prefix);

    methods.iter().for_each(|method| {
        res.push_str(method);
        res.push_str(", ")
    });

    if more {
        res.push_str(additional);
    } else if !methods.is_empty() {
        // erase extra comma
        res.truncate(res.len() - 2);
    }

    res.push_str("\n");
    res
}

impl Display for Form {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let fixed_rate = format!("по курсу <b>{}</b>", &self.rate);

        let location = if self.cash {
            format!("Наличные: {}\n", self.location)
        } else { String::default() };

        let res = format!(
            "<b>{summary}</b>\n\
            {rate_clause}\n\
            {parts_clause}\
            {cash_only_clause}\
            {location}\
            {eu_str}\
            {ru_str}\
            {comment}",
            summary = self.summary(),
            eu_str = self.eu_methods(), ru_str = self.ru_methods(),
            rate_clause = if self.cb { "по текущему курсу" } else { &fixed_rate },
            parts_clause = if self.in_parts { "Возможно частями\n" } else { "" },
            cash_only_clause = if self.cash_only { "Только наличными\n" } else { "" },
            comment = self.comment.trim()
        );

        write!(f, "{res}")
    }
}

impl Form {
    fn is_buying(&self) -> bool {
        self.buy_or_sell == "Купить"
    }

    fn no_rubs(&self) -> bool {
        self.selling_curr != "RUB" && self.buying_curr != "RUB"
    }

    fn eu_methods(&self) -> String {
        if self.cash_only { String::default() } else {
            methods(&self.eu_methods, &self.eu_methods_str, self.eu_more, "eu: ")
        }
    }

    fn ru_methods(&self) -> String {
        if self.cash_only || self.no_rubs() { String::default() } else {
            methods(&self.ru_methods, &self.ru_methods_str, self.ru_more, "ru: ")
        }
    }

    fn summary(&self) -> String {
        format!(
            "#{}_{}\n{bos} {sum} {curr1} за {curr2}",
            self.selling_curr.to_lowercase(),
            self.buying_curr.to_lowercase(),
            bos = if self.is_buying() { "Куплю" } else { "Продам" },
            sum = self.sum,
            curr1 = if self.is_buying() { &self.buying_curr } else { &self.selling_curr },
            curr2 = if self.is_buying() { &self.selling_curr } else { &self.buying_curr },
        )
    }
}