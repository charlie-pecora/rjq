use std::usize;

use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;

struct GetKey {
    k: String,
}

impl GetKey {
    fn apply(&self, input: Value) -> Vec<Value> {
        match input {
            Value::Object(_) => {
                if self.k == "." {
                    vec![input]
                } else {
                    match input.get(&self.k) {
                        Some(v) => vec![v.to_owned()],
                        None => vec![Value::Null],
                    }
                }
            },
            _ => {
                eprintln!("error, not an object");
                Vec::new()
            }
        }
    }
}

struct GetArrayElements {
    pub indices: String,
}

impl GetArrayElements {
    fn apply(&self, input: Value) -> Vec<Value> {
        match input {
            Value::Array(vec_value) => {
                if (self.indices == "") | (self.indices == ":") {
                    vec_value.to_vec()
                } else {
                    self.indices
                        .split(',')
                        .map(|v| match v.parse::<usize>() {
                            Ok(i) => match vec_value.get(i) {
                                Some(result) => result.to_owned(),
                                None => Value::Null,
                            },
                            Err(_) => Value::Null,
                        })
                        .collect()
                }
            }
            _ => vec![Value::Null],
        }
    }
}


struct ListKeys {
}

impl ListKeys {
    fn apply(&self, input: Value) -> Vec<Value> {
        match input {
            Value::Object(v) => {
                v.keys()
                    .map(|x| match serde_json::to_value(x) {
                        Ok(result) => result,
                        Err(_) => Value::Null,
                    }
                         ).collect()
                
            },
            _ => {
                eprintln!("error, not an object");
                Vec::new()
            }
        }
    }
}

enum Command {
    GetArrayElements(GetArrayElements),
    GetKey(GetKey),
    ListKeys(ListKeys),
}

impl Command {
    fn apply(&self, input: Value) -> Vec<Value> {
        match &self {
            Command::GetArrayElements(getter) => getter.apply(input),
            Command::GetKey(getter) => getter.apply(input),
            Command::ListKeys(getter) => getter.apply(input),
        }
    }
}

pub fn query_json(data: Value, query: &str) -> Vec<Value> {
    let mut inputs = vec![data];
    let mut outputs: Vec<Value> = Vec::new();
    let mut getters = Vec::<Command>::new();
    lazy_static! {
        static ref PATTERN: Regex = Regex::new(r#"(\.|\[[0-9,]*\]|[[:alnum:]_-]+)"#).unwrap();
    }

    for _command in PATTERN.find_iter(&query) {
        let command = _command.as_str();
        if command == "listkeys" {
            getters.push(Command::ListKeys(ListKeys{}))
        } else if command.starts_with("[") && command.ends_with("]") {
            getters.push(Command::GetArrayElements(GetArrayElements {
                indices: command[1..(command.len() - 1)].to_string(),
            }));
        } else if command != "." {
            getters.push(Command::GetKey(GetKey {
                k: command.to_string(),
            }));
        }
    }
    for getter in getters {
        for input in inputs {
            outputs.extend(getter.apply(input));
        }
        inputs = outputs.drain(..).collect();
    }
    inputs
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    #[test]
    fn test_query_json() {
        let result = query_json(json!({}), ".");
        assert_eq!(result, vec![json!({})]);
    }

    #[test]
    fn test_get_key() {
        let result = query_json(json!({"a": 1}), "a");
        assert_eq!(result, vec![1]);
    }

    #[rstest]
    #[case(json!({}), "key".to_string(), vec![Value::Null])]
    #[case(json!({"key": 1}), "key".to_string(), vec![json!(1)])]
    #[case(json!({"key": 1}), "_key".to_string(), vec![Value::Null])]
    fn test_query_key(#[case] input: Value, #[case] k: String, #[case] expected: Vec<Value>) {
        let command = GetKey { k };
        let result = command.apply(input);
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(json!([]), "0".to_string(), vec![Value::Null])]
    #[case(json!([1, 2, 3]), "0".to_string(), vec![json!(1)])]
    #[case(json!([1, 2, 3]), "0,2".to_string(), vec![json!(1), json!(3)])]
    #[case(json!([1, 2, 3]), "".to_string(), vec![json!(1), json!(2), json!(3)])]
    #[case(json!([1, 2, 3]), ":".to_string(), vec![json!(1), json!(2), json!(3)])]
    #[case(json!([1, 2, 3]), "0,4".to_string(), vec![json!(1), Value::Null])]
    fn test_query_array_elements(
        #[case] input: Value,
        #[case] indices: String,
        #[case] expected: Vec<Value>,
    ) {
        let command = GetArrayElements { indices };
        let result = command.apply(input);
        assert_eq!(result, expected.to_vec());
    }

    #[test]
    fn test_query_keys() {
        let result = query_json(json!({"a": 1, "c": 3}), "listkeys");
        assert_eq!(result, vec!["a", "c"]);
    }
}
