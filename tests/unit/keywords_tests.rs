//! Unit tests for keywords module.

use cardgame::keywords::Keywords;

#[test]
fn test_keywords_size() {
    // Keywords is 2 bytes (u16) to support up to 16 keywords
    assert_eq!(std::mem::size_of::<Keywords>(), 2);
}

#[test]
fn test_keyword_checks() {
    let kw = Keywords::none().with_rush().with_guard();
    assert!(kw.has_rush());
    assert!(kw.has_guard());
    assert!(!kw.has_ranged());
    assert!(!kw.has_lethal());
}

#[test]
fn test_keyword_mutators() {
    let mut kw = Keywords::none();
    assert!(!kw.has_shield());

    kw.add(Keywords::SHIELD);
    assert!(kw.has_shield());

    kw.remove(Keywords::SHIELD);
    assert!(!kw.has_shield());
}

#[test]
fn test_builder_pattern() {
    let kw = Keywords::none()
        .with_rush()
        .with_lethal()
        .with_quick();

    assert!(kw.has_rush());
    assert!(kw.has_lethal());
    assert!(kw.has_quick());
    assert!(!kw.has_guard());
}

#[test]
fn test_from_names() {
    let kw = Keywords::from_names(&["Rush", "GUARD", "lethal"]);
    assert!(kw.has_rush());
    assert!(kw.has_guard());
    assert!(kw.has_lethal());
    assert!(!kw.has_ranged());
}

#[test]
fn test_to_names() {
    let kw = Keywords::none().with_rush().with_piercing();
    let names = kw.to_names();
    assert!(names.contains(&"Rush"));
    assert!(names.contains(&"Piercing"));
    assert_eq!(names.len(), 2);
}

#[test]
fn test_union() {
    let kw1 = Keywords::none().with_rush();
    let kw2 = Keywords::none().with_guard();
    let combined = kw1.union(kw2);

    assert!(combined.has_rush());
    assert!(combined.has_guard());
}

#[test]
fn test_generic_has() {
    let kw = Keywords::none().with_lifesteal();
    assert!(kw.has(Keywords::LIFESTEAL));
    assert!(!kw.has(Keywords::RUSH));
}
