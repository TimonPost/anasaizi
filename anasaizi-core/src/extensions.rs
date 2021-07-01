use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::iter::FromIterator;

#[derive(Debug)]
pub struct Extensions {
    extensions: Vec<String>,
}

impl Extensions {
    pub fn new(extensions: Vec<String>) -> Extensions {
        Extensions { extensions }
    }

    pub fn extensions(&self) -> Vec<String> {
        self.extensions.clone()
    }

    pub fn has(&self, extensions: &Extensions) -> bool {
        let mut set1: HashSet<String> = HashSet::from_iter(extensions.extensions());
        let mut set2: HashSet<String> = HashSet::from_iter(self.extensions());

        let difference = set1.difference(&set2).count();

        difference == 0
    }

    pub fn extensions_ptr(&self) -> Vec<CString> {
        return self
            .extensions
            .iter()
            .map(|x| CString::new(x.as_str()).unwrap())
            .collect();
    }

    pub fn extensions_count(&self) -> u32 {
        self.extensions.len() as u32
    }
}

#[cfg(test)]
mod tests {
    use crate::ash_implementation::Extensions;
    use std::ffi::CString;

    #[test]
    fn has_extensions() {
        let required =
            Extensions::new(vec![String::from("1").unwrap(), String::from("2").unwrap()]);
        let available = Extensions::new(vec![String::from("1").unwrap()]);

        let difference = available.has(&required);

        assert_eq!(difference, false);
    }

    #[test]
    fn has_not_extensions() {
        let required = Extensions::new(vec![String::from("1").unwrap()]);
        let available =
            Extensions::new(vec![String::from("1").unwrap(), String::from("2").unwrap()]);

        let difference = available.has(&required);

        assert_eq!(difference, true);
    }
}
