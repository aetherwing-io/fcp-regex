use crate::elements::{validate_fragment_name, Element};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Fragment {
    pub name: String,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone)]
pub enum RegexEvent {
    Define {
        name: String,
        old: Option<Vec<Element>>,
        new: Vec<Element>,
    },
    Drop {
        name: String,
        elements: Vec<Element>,
    },
    Rename {
        old_name: String,
        new_name: String,
    },
}

pub struct FragmentRegistry {
    fragments: HashMap<String, Fragment>,
}

impl FragmentRegistry {
    pub fn new() -> Self {
        FragmentRegistry {
            fragments: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, elements: Vec<Element>) -> Result<RegexEvent, String> {
        validate_fragment_name(name)?;
        let old = self.fragments.get(name).map(|f| f.elements.clone());
        let event = RegexEvent::Define {
            name: name.to_string(),
            old,
            new: elements.clone(),
        };
        self.fragments.insert(
            name.to_string(),
            Fragment {
                name: name.to_string(),
                elements,
            },
        );
        Ok(event)
    }

    pub fn drop(&mut self, name: &str) -> Result<RegexEvent, String> {
        let fragment = self
            .fragments
            .remove(name)
            .ok_or_else(|| format!("fragment {name:?} not found"))?;
        Ok(RegexEvent::Drop {
            name: name.to_string(),
            elements: fragment.elements,
        })
    }

    pub fn rename(&mut self, old: &str, new: &str) -> Result<RegexEvent, String> {
        validate_fragment_name(new)?;
        if !self.fragments.contains_key(old) {
            return Err(format!("fragment {old:?} not found"));
        }
        if self.fragments.contains_key(new) {
            return Err(format!("fragment {new:?} already exists"));
        }

        let mut fragment = self.fragments.remove(old).unwrap();
        fragment.name = new.to_string();
        self.fragments.insert(new.to_string(), fragment);

        // Update all references in all fragments
        let old_str = old.to_string();
        let new_str = new.to_string();
        for fragment in self.fragments.values_mut() {
            for elem in &mut fragment.elements {
                update_ref(elem, &old_str, &new_str);
            }
        }

        Ok(RegexEvent::Rename {
            old_name: old.to_string(),
            new_name: new.to_string(),
        })
    }

    pub fn get(&self, name: &str) -> Option<&Fragment> {
        self.fragments.get(name)
    }

    pub fn list(&self) -> Vec<&Fragment> {
        let mut frags: Vec<&Fragment> = self.fragments.values().collect();
        frags.sort_by_key(|f| &f.name);
        frags
    }

    pub fn contains(&self, name: &str) -> bool {
        self.fragments.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.fragments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fragments.is_empty()
    }
}

impl Default for FragmentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn update_ref(elem: &mut Element, old: &str, new: &str) {
    match elem {
        Element::Ref(name) if name == old => *name = new.to_string(),
        Element::Optional(name) if name == old => *name = new.to_string(),
        Element::Capture(name) if name == old => *name = new.to_string(),
        Element::NamedCapture(_, name) if name == old => *name = new.to_string(),
        Element::Alternation(names) => {
            for name in names {
                if name == old {
                    *name = new.to_string();
                }
            }
        }
        Element::SepBy(name, _) if name == old => *name = new.to_string(),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{CharClass, Quantifier};

    #[test]
    fn test_new_registry_is_empty() {
        let reg = FragmentRegistry::new();
        assert_eq!(reg.len(), 0);
        assert!(reg.is_empty());
    }

    #[test]
    fn test_define_and_get() {
        let mut reg = FragmentRegistry::new();
        let elems = vec![Element::AnyClass(CharClass::Digit, Quantifier::OneOrMore)];
        reg.define("digits", elems.clone()).unwrap();
        let frag = reg.get("digits").unwrap();
        assert_eq!(frag.name, "digits");
        assert_eq!(frag.elements, elems);
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_define_overwrite() {
        let mut reg = FragmentRegistry::new();
        let elems1 = vec![Element::Literal("v1".to_string())];
        let elems2 = vec![Element::Literal("v2".to_string())];
        reg.define("x", elems1.clone()).unwrap();
        let event = reg.define("x", elems2.clone()).unwrap();
        match event {
            RegexEvent::Define { old, new, .. } => {
                assert_eq!(old, Some(elems1));
                assert_eq!(new, elems2);
            }
            _ => panic!("expected Define event"),
        }
        assert_eq!(reg.get("x").unwrap().elements, elems2);
    }

    #[test]
    fn test_define_invalid_name() {
        let mut reg = FragmentRegistry::new();
        assert!(reg.define("bad name", vec![]).is_err());
    }

    #[test]
    fn test_drop() {
        let mut reg = FragmentRegistry::new();
        reg.define("x", vec![Element::Literal("a".to_string())])
            .unwrap();
        let event = reg.drop("x").unwrap();
        match event {
            RegexEvent::Drop { name, elements } => {
                assert_eq!(name, "x");
                assert_eq!(elements, vec![Element::Literal("a".to_string())]);
            }
            _ => panic!("expected Drop event"),
        }
        assert!(!reg.contains("x"));
    }

    #[test]
    fn test_drop_not_found() {
        let mut reg = FragmentRegistry::new();
        assert!(reg.drop("nonexistent").is_err());
    }

    #[test]
    fn test_rename() {
        let mut reg = FragmentRegistry::new();
        reg.define("old", vec![Element::Literal("a".to_string())])
            .unwrap();
        let event = reg.rename("old", "new").unwrap();
        match event {
            RegexEvent::Rename {
                old_name,
                new_name,
            } => {
                assert_eq!(old_name, "old");
                assert_eq!(new_name, "new");
            }
            _ => panic!("expected Rename event"),
        }
        assert!(!reg.contains("old"));
        assert!(reg.contains("new"));
    }

    #[test]
    fn test_rename_updates_refs() {
        let mut reg = FragmentRegistry::new();
        reg.define("digits", vec![Element::AnyClass(CharClass::Digit, Quantifier::OneOrMore)])
            .unwrap();
        reg.define(
            "version",
            vec![
                Element::Ref("digits".to_string()),
                Element::Literal(".".to_string()),
                Element::Ref("digits".to_string()),
            ],
        )
        .unwrap();
        reg.define(
            "wrapped",
            vec![
                Element::Capture("digits".to_string()),
                Element::Optional("digits".to_string()),
                Element::NamedCapture("major".to_string(), "digits".to_string()),
                Element::Alternation(vec!["digits".to_string(), "other".to_string()]),
                Element::SepBy("digits".to_string(), "lit:.".to_string()),
            ],
        )
        .unwrap();

        reg.rename("digits", "nums").unwrap();

        // Check version fragment refs updated
        let version = reg.get("version").unwrap();
        assert_eq!(version.elements[0], Element::Ref("nums".to_string()));
        assert_eq!(version.elements[2], Element::Ref("nums".to_string()));

        // Check all reference types updated in wrapped
        let wrapped = reg.get("wrapped").unwrap();
        assert_eq!(wrapped.elements[0], Element::Capture("nums".to_string()));
        assert_eq!(wrapped.elements[1], Element::Optional("nums".to_string()));
        assert_eq!(
            wrapped.elements[2],
            Element::NamedCapture("major".to_string(), "nums".to_string())
        );
        assert_eq!(
            wrapped.elements[3],
            Element::Alternation(vec!["nums".to_string(), "other".to_string()])
        );
        assert_eq!(
            wrapped.elements[4],
            Element::SepBy("nums".to_string(), "lit:.".to_string())
        );
    }

    #[test]
    fn test_rename_not_found() {
        let mut reg = FragmentRegistry::new();
        assert!(reg.rename("nope", "new").is_err());
    }

    #[test]
    fn test_rename_target_exists() {
        let mut reg = FragmentRegistry::new();
        reg.define("a", vec![]).unwrap();
        reg.define("b", vec![]).unwrap();
        assert!(reg.rename("a", "b").is_err());
    }

    #[test]
    fn test_rename_invalid_new_name() {
        let mut reg = FragmentRegistry::new();
        reg.define("a", vec![]).unwrap();
        assert!(reg.rename("a", "bad name").is_err());
    }

    #[test]
    fn test_list() {
        let mut reg = FragmentRegistry::new();
        reg.define("b", vec![]).unwrap();
        reg.define("a", vec![]).unwrap();
        let listed = reg.list();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].name, "a");
        assert_eq!(listed[1].name, "b");
    }

    #[test]
    fn test_contains() {
        let mut reg = FragmentRegistry::new();
        reg.define("x", vec![]).unwrap();
        assert!(reg.contains("x"));
        assert!(!reg.contains("y"));
    }

    #[test]
    fn test_define_event_no_old() {
        let mut reg = FragmentRegistry::new();
        let event = reg
            .define("x", vec![Element::Literal("a".to_string())])
            .unwrap();
        match event {
            RegexEvent::Define { old, .. } => assert!(old.is_none()),
            _ => panic!("expected Define event"),
        }
    }
}
