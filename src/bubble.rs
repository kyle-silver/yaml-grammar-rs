use std::vec;

pub enum Bubble<T> {
    Single(T),
    Multi(Vec<T>),
}

impl<T> IntoIterator for Bubble<T> { 
    type Item = T;

    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Bubble::Single(val) => vec![val].into_iter(),
            Bubble::Multi(vec) => vec.into_iter(),
        }
    }
}

impl<T> Bubble<T> {
    pub fn get(self) -> Vec<T> {
        match self {
            Bubble::Single(t) => vec![t],
            Bubble::Multi(vec) => vec,
        }
    }

    pub fn all(&self, test: fn(&T) -> bool) -> bool {
        match self {
            Bubble::Single(t) => test(t),
            Bubble::Multi(v) => {
                v.iter().all(test)
            }
        }
    }
}