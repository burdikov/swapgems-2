use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::num::ParseFloatError;
use crate::site::form::FormParseError::{De, Invalid, ParseFloat};

type FormData = HashMap<String, Vec<String>>;

#[derive(Debug)]
pub struct Form {
    buying: bool,

    selling_cur: String,
    buying_cur: String,

    eu_methods: Option<Vec<String>>,
    ru_methods: Option<Vec<String>>,
    eu_methods_str: Option<String>,
    ru_methods_str: Option<String>,

    in_parts: bool,
    location: Option<String>,    // todo there should be only one location

    sum: u32,
    // their_sum: u32,

    cb: bool,
    rate: Option<String>,

    comment: Option<String>,
}

impl TryFrom<FormData> for Form {
    type Error = FormParseError;

    fn try_from(mut data: FormData) -> Result<Self, Self::Error> {
        // let mut rate = None;
        // if let Some(r) = v.remove("rate").unwrap_or_default().pop().filter(|s| s != "") {
        //     rate = Some(r.parse::<f64>().map_err(|e| ParseFloat(e))?);
        // }
        let buy_or_sell = data.remove("buy-or-sell").unwrap().pop().unwrap();
        let selling_cur = data.remove("our-curr").unwrap().pop().unwrap();
        let buying_cur = data.remove("their-curr").unwrap().pop().unwrap();

        let cb = data.contains_key("cb");
        let rate = data.remove("rate").unwrap_or_default().pop();

        if !cb && rate.is_none() {
            return Err(Invalid("Не указан курс"))
        }

        Ok(Form {
            buying: buy_or_sell == "Купить",
            selling_cur,
            buying_cur,

            eu_methods: data.remove("eu-methods"),
            ru_methods: data.remove("ru-methods"),
            eu_methods_str: data.remove("eu-methods-str").unwrap_or_default().pop(),
            ru_methods_str: data.remove("ru-methods-str").unwrap_or_default().pop(),

            in_parts: data.contains_key("in-parts"),
            location: data.remove("location").unwrap_or_default().pop(),

            sum: data.remove("our-sum").unwrap_or_default().pop().unwrap().parse().unwrap(),

            cb,
            rate,
            comment: data.remove("comment").unwrap_or_default().pop(),
        })
    }
}

#[derive(Debug)]
pub enum FormParseError {
    Invalid(&'static str),
    De(serde::de::value::Error),
    ParseFloat(ParseFloatError),
}

impl TryFrom<&[u8]> for Form {
    type Error = FormParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut pairs: Vec<(String, String)> = serde_urlencoded::from_bytes(&bytes)
            .map_err(|e| De(e))?;
        let mut map: FormData = HashMap::with_capacity(20);  // we've got about 16 fields in our form

        loop {
            match pairs.pop() {
                None => { break }
                Some((key, value)) if value.len() > 0 => {
                    match map.get_mut(&key) {
                        Some(list) => { list.push(value); }
                        None => { map.insert(key, vec![value]); }
                    }
                }
                _ => {}
            }
        }

        map.try_into()
    }
}


fn methods(mut res: String, m: &Option<Vec<String>>, s: &Option<String>) -> String {
    let orig_len = res.len();
    if let Some(ref methods) = m {
        res.push_str(&methods.join(", "));
    }

    if let Some(ref met_str) = s {
        res.push_str(", ");
        res.push_str(met_str);
    }

    if res.len() != orig_len {
        res.push_str("\n");
        res
    } else {
        String::default()
    }
}

impl Display for Form {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let fixed_rate = format!("по курсу {}", self.rate.as_ref().unwrap_or(&String::default()));

        let eu_str = methods(String::from("eu: "), &self.eu_methods, &self.eu_methods_str);
        let ru_str = methods(String::from("ru: "), &self.ru_methods, &self.ru_methods_str);


        let res = format!(
            "<b>{summary}</b>\n\
            {rate_clause}\n\
            {parts_clause}\
            {eu_str}\
            {ru_str}\
            {comment}",
            summary = self.summary(),
            rate_clause = if self.cb { "по текущему курсу" } else { &fixed_rate },
            parts_clause = if self.in_parts { "Возможно частями\n" } else { "" },
            comment = self.comment.as_ref().map(|s| s.trim()).unwrap_or_default()
        );

        write!(f, "{res}")
    }
}

impl Form {
    fn summary(&self) -> String {
        format!(
            "{bos} {sum} {curr1} за {curr2}",
            bos = if self.buying { "Куплю" } else { "Продам" },
            sum = self.sum,
            curr1 = if self.buying { &self.buying_cur } else { &self.selling_cur },
            curr2 = if self.buying { &self.selling_cur } else { &self.buying_cur },
        )
    }
}