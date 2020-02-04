use std::borrow::Cow;

pub type DynFnPtr = &'static (dyn (Fn(&str) -> Cow<str>) + Sync);

struct Modifier {
    s: &'static str,
    fn_ptr: DynFnPtr,
}

const MODIFIERS: [Modifier; 2] = [
    Modifier {
        s: "U",
        fn_ptr: &to_upper,
    },
    Modifier {
        s: "L",
        fn_ptr: &to_lower,
    },
];

fn to_upper(s: &str) -> Cow<str> {
    Cow::from(s.to_ascii_uppercase())
}

fn to_lower(s: &str) -> Cow<str> {
    Cow::from(s.to_ascii_lowercase())
}

pub fn get_modifier(requested: &str) -> Option<DynFnPtr> {
    for m in MODIFIERS.iter() {
        if requested == m.s {
            return Some(m.fn_ptr);
        };
    }
    None
}
