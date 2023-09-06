use serde_cw_value::Value;

pub fn json_pointer<'a>(value: &'a mut Value, pointer: &str) -> Option<&'a mut Value> {
    if pointer.is_empty() {
        return Some(value);
    }

    if !pointer.starts_with('/') {
        return None;
    }

    pointer
        .split('/')
        .skip(1)
        .map(|x| x.replace("~1", "/").replace("~0", "~"))
        .try_fold(value, |target, token| match target {
            Value::Map(map) => map.get_mut(&Value::String(token)),
            Value::Seq(list) => parse_index(&token).and_then(move |x| list.get_mut(x)),
            _ => None,
        })
}

fn parse_index(s: &str) -> Option<usize> {
    if s.starts_with('+') || (s.starts_with('0') && s.len() != 1) {
        return None;
    }

    s.parse().ok()
}
