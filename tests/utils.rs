use serde::de::DeserializeOwned;

#[macro_export]
macro_rules! valpath {
    ($($x:expr,)*) => (vec![$(&valstr!($x)),*]);
    ($($x:expr),*) => (vec![$(&valstr!($x)),*]);
}

#[macro_export]
macro_rules! valnum {
    ($n:expr) => {
        Value::Number(Number::from($n))
    }
}

pub fn spec<T: DeserializeOwned>(case: &str) -> T {
    let path = format!("tests/res/{}/spec.yamlfmt", case);
    let raw = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&raw).unwrap()
}

pub fn input<T: DeserializeOwned>(case: &str, file: &str) -> T {
    let path = format!("tests/res/{}/{}", case, file);
    let raw = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&raw).unwrap()
}