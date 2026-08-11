use super::support::{binding, Fixture, DUPLICATES};

/// Two identical sites, one deleted. The survivor pins to its exact baseline
/// line, so the loss is attributable and the missing instance is gone.
#[test]
fn duplicate_loss_surfaces_the_missing_instance_as_gone() {
    let fixture = Fixture::new(DUPLICATES);
    std::fs::write(
        &fixture.source,
        "mod second {\n#[verifies(\"rule_anchor\", examples)]\nfn duplicate() {}\n}\n",
    )
    .unwrap();

    let report = fixture.rescan();

    let unchanged = binding(&report, "duplicate", "unchanged");
    assert_eq!(unchanged["line"], 2);
    let gone = binding(&report, "duplicate", "gone");
    assert_eq!(gone["line"], 7);
    assert!(report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["message"].as_str().unwrap().contains("gone")));
}

/// Two identical sites, one deleted and the survivor shifted off its baseline
/// line. No pairing is attributable, so the scan says the group shrank
/// instead of quietly absorbing the loss or guessing which instance died.
#[test]
fn unattributable_duplicate_loss_warns_that_the_group_shrank() {
    let fixture = Fixture::new(DUPLICATES);
    std::fs::write(
        &fixture.source,
        "\nmod second {\n#[verifies(\"rule_anchor\", examples)]\nfn duplicate() {}\n}\n",
    )
    .unwrap();

    let report = fixture.rescan();

    let duplicate_sites = report["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|site| site["item_name"] == "duplicate")
        .collect::<Vec<_>>();
    assert_eq!(duplicate_sites.len(), 1);
    assert_eq!(duplicate_sites[0]["anchor_state"], "unchanged");
    assert!(report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| {
            let message = warning["message"].as_str().unwrap();
            message.contains("lost 1 instance") && message.contains("cannot pair")
        }));
    assert!(!report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["message"].as_str().unwrap().contains("gone")));
}

/// Identical duplicates shuffled onto new lines with none lost stay quiet:
/// the sites are interchangeable and nothing worth reporting happened.
#[test]
fn duplicate_shuffle_without_loss_stays_silent() {
    let fixture = Fixture::new(DUPLICATES);
    std::fs::write(
        &fixture.source,
        "\nmod first {\n#[verifies(\"rule_anchor\", examples)]\nfn duplicate() {}\n}\n\n\
         mod second {\n#[verifies(\"rule_anchor\", examples)]\nfn duplicate() {}\n}\n",
    )
    .unwrap();

    let report = fixture.rescan();

    let duplicate_sites = report["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|site| site["item_name"] == "duplicate")
        .collect::<Vec<_>>();
    assert_eq!(duplicate_sites.len(), 2);
    assert!(duplicate_sites
        .iter()
        .all(|site| site["anchor_state"] == "unchanged"));
    assert!(!report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| {
            let message = warning["message"].as_str().unwrap();
            message.contains("gone") || message.contains("cannot pair")
        }));
}
