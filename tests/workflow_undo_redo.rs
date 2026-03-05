use fcp_regex::domain::model::{FragmentRegistry, RegexEvent};
use fcp_regex::elements::{CharClass, Element, Quantifier};
use fcp_regex::fcpcore::event_log::EventLog;

fn reverse_event(event: &RegexEvent, registry: &mut FragmentRegistry) {
    match event {
        RegexEvent::Define { name, old, .. } => {
            if let Some(old_elements) = old {
                let _ = registry.define(name, old_elements.clone());
            } else {
                let _ = registry.drop(name);
            }
        }
        RegexEvent::Drop { name, elements } => {
            let _ = registry.define(name, elements.clone());
        }
        RegexEvent::Rename { old_name, new_name } => {
            let _ = registry.rename(new_name, old_name);
        }
    }
}

fn replay_event(event: &RegexEvent, registry: &mut FragmentRegistry) {
    match event {
        RegexEvent::Define { name, new, .. } => {
            let _ = registry.define(name, new.clone());
        }
        RegexEvent::Drop { name, .. } => {
            let _ = registry.drop(name);
        }
        RegexEvent::Rename { old_name, new_name } => {
            let _ = registry.rename(old_name, new_name);
        }
    }
}

#[test]
fn test_undo_define() {
    let mut reg = FragmentRegistry::new();
    let mut log = EventLog::new();

    // Define a fragment
    let event = reg
        .define(
            "digits",
            vec![Element::AnyClass(CharClass::Digit, Quantifier::OneOrMore)],
        )
        .unwrap();
    log.append(event);
    assert!(reg.contains("digits"));
    assert_eq!(reg.len(), 1);

    // Undo
    let undone = log.undo(1);
    assert_eq!(undone.len(), 1);
    for ev in &undone {
        reverse_event(ev, &mut reg);
    }
    assert!(!reg.contains("digits"));
    assert_eq!(reg.len(), 0);
}

#[test]
fn test_redo_define() {
    let mut reg = FragmentRegistry::new();
    let mut log = EventLog::new();

    let event = reg
        .define(
            "digits",
            vec![Element::AnyClass(CharClass::Digit, Quantifier::OneOrMore)],
        )
        .unwrap();
    log.append(event);

    // Undo
    let undone = log.undo(1);
    for ev in &undone {
        reverse_event(ev, &mut reg);
    }
    assert!(!reg.contains("digits"));

    // Redo
    let redone = log.redo(1);
    for ev in &redone {
        replay_event(ev, &mut reg);
    }
    assert!(reg.contains("digits"));
    assert_eq!(reg.len(), 1);
}

#[test]
fn test_undo_drop() {
    let mut reg = FragmentRegistry::new();
    let mut log = EventLog::new();

    let event = reg
        .define(
            "x",
            vec![Element::Literal("hello".to_string())],
        )
        .unwrap();
    log.append(event);

    let event = reg.drop("x").unwrap();
    log.append(event);
    assert!(!reg.contains("x"));

    // Undo the drop — should restore
    let undone = log.undo(1);
    for ev in &undone {
        reverse_event(ev, &mut reg);
    }
    assert!(reg.contains("x"));
    assert_eq!(reg.get("x").unwrap().elements, vec![Element::Literal("hello".to_string())]);
}

#[test]
fn test_undo_multiple() {
    let mut reg = FragmentRegistry::new();
    let mut log = EventLog::new();

    let e1 = reg
        .define("a", vec![Element::Literal("a".to_string())])
        .unwrap();
    log.append(e1);

    let e2 = reg
        .define("b", vec![Element::Literal("b".to_string())])
        .unwrap();
    log.append(e2);

    assert_eq!(reg.len(), 2);

    // Undo both
    let undone = log.undo(2);
    assert_eq!(undone.len(), 2);
    for ev in &undone {
        reverse_event(ev, &mut reg);
    }
    assert_eq!(reg.len(), 0);
}

#[test]
fn test_undo_overwrite() {
    let mut reg = FragmentRegistry::new();
    let mut log = EventLog::new();

    // Define v1
    let e1 = reg
        .define("x", vec![Element::Literal("v1".to_string())])
        .unwrap();
    log.append(e1);

    // Overwrite with v2
    let e2 = reg
        .define("x", vec![Element::Literal("v2".to_string())])
        .unwrap();
    log.append(e2);

    assert_eq!(
        reg.get("x").unwrap().elements,
        vec![Element::Literal("v2".to_string())]
    );

    // Undo should restore v1
    let undone = log.undo(1);
    for ev in &undone {
        reverse_event(ev, &mut reg);
    }
    assert_eq!(
        reg.get("x").unwrap().elements,
        vec![Element::Literal("v1".to_string())]
    );
}
