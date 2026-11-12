//! OLY-B-SET-02: the roster holds Opus, Sonnet and Haiku, plus Jev through the Vercel AI Gateway.

use athena_core::jev::{ENDPOINT, MODEL, ROSTER, roster_entry};

#[test]
fn roster_holds_opus_sonnet_haiku_and_jev() {
    for family in ["opus", "sonnet", "haiku"] {
        let entry = roster_entry(family).unwrap_or_else(|| panic!("the roster has no {family}"));
        assert!(
            entry.model.starts_with("claude-") && entry.model.contains(family),
            "{family} has model {}",
            entry.model
        );
        assert_eq!(entry.endpoint, None, "{family} is run by the agent");
    }
    let jev = roster_entry("jev").expect("the roster has no jev");
    assert_eq!(jev.model, MODEL);
    assert_eq!(jev.endpoint, Some(ENDPOINT));
    assert!(ENDPOINT.contains("ai-gateway.vercel.sh"));
    assert_eq!(ROSTER.len(), 4, "the roster holds exactly four models");
}
