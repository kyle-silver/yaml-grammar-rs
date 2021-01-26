use serde::de::DeserializeOwned;

pub fn from_file<T: DeserializeOwned>(file: &str) -> T {
    let raw = std::fs::read_to_string(file).unwrap();
    serde_yaml::from_str(&raw).unwrap()
}

pub fn fmt<T: DeserializeOwned>(fmt: &str) -> T {
    let path = format!("tests/res/fmt/{}", fmt);
    let raw = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&raw).unwrap()
}

pub fn input<T: DeserializeOwned>(input: &str) -> T {
    let path = format!("tests/res/inputs/{}", input);
    let raw = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&raw).unwrap()
}