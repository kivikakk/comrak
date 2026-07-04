use crate::nodes::Attributes;

pub fn parse_attributes(input: &str) -> Option<(Attributes, usize)> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trivial() {
        assert!(parse_attributes("").is_none());
    }
}
