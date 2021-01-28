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

pub fn from_file<T: DeserializeOwned>(file: &str) -> T {
    let raw = std::fs::read_to_string(file).unwrap();
    serde_yaml::from_str(&raw).unwrap()
}

pub fn spec<T: DeserializeOwned>(case: &str) -> T {
    let path = format!("tests/res/{}/spec.yamlfmt", case);
    let raw = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&raw).unwrap()
}

pub fn input<T: DeserializeOwned>(case: &str) -> T {
    let path = format!("tests/res/{}/input.yaml", case);
    let raw = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&raw).unwrap()
}