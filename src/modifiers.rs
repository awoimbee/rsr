use std::borrow::Cow;

pub type DynFnPtr = &'static (dyn (Fn(&str) -> Cow<str>) + Sync);

fn to_upper(s: &str) -> Cow<str> {
    Cow::from(s.to_ascii_uppercase())
}

fn to_lower(s: &str) -> Cow<str> {
    Cow::from(s.to_ascii_lowercase())
}

pub fn get_modifier(requested: &str) -> Option<DynFnPtr> {
    match requested {
        "U" => Some(&to_upper),
        "L" => Some(&to_lower),
        _ => None,
    }
}
