//! The scanner's copy of the confidence range, held to the graph's.
//!
//! `rule_confidence_range` is decided by `validate_confidence_score` in
//! `provenance-core/src/model/validation.rs`. The scanner reads confidences out
//! of source comments before any record exists, so it carries its own copy of
//! the range in `parser.rs`. The two channels do different things about a bad
//! score, and are meant to: the graph rejects the record, the scanner warns on
//! the line and falls back to the default 1.0, because one bad comment must not
//! stop a scan of the repository. What they must never do is disagree about
//! which scores are in range, which is what this checks.

use provenance_core::validate_confidence_score;
use provenance_macros::verifies;
use provenance_scanner::parse_annotations;

struct Rng(u64);

impl Rng {
    const fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut word = self.0;
        word = (word ^ (word >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        word = (word ^ (word >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        word ^ (word >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next_u64() % bound as u64).unwrap()
    }
}

/// Scores from three sources: the values a fence post gets wrong, a spread that
/// straddles both ends, and whole random bit patterns, which is where the NaNs
/// and the enormous numbers come from.
fn gen_score(rng: &mut Rng) -> f64 {
    const FIXED: [f64; 14] = [
        0.0,
        -0.0,
        1.0,
        0.5,
        f64::MIN_POSITIVE,
        -f64::MIN_POSITIVE,
        f64::EPSILON,
        1.0 - f64::EPSILON,
        1.0 + f64::EPSILON,
        -1.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX,
    ];
    const SPREAD: usize = 1 << 20;

    match rng.below(3) {
        0 => FIXED[rng.below(FIXED.len())],
        1 => {
            let step = u32::try_from(rng.below(SPREAD)).unwrap();
            let width = u32::try_from(SPREAD).unwrap();
            (f64::from(step) / f64::from(width)).mul_add(1.5, -0.25)
        }
        _ => f64::from_bits(rng.next_u64()),
    }
}

#[test]
#[verifies("rule_confidence_range", conformance)]
fn scanner_and_core_agree_on_which_confidences_are_in_range() {
    let mut rng = Rng(0x5eed_0003);
    let mut kept = 0;
    let mut refused = 0;
    for _ in 0..4096 {
        let score = gen_score(&mut rng);
        // `{:?}` on an f64 round-trips, so the number the scanner reads out of
        // the comment is the number core was handed.
        let written = format!("{score:?}");
        let read_back: f64 = written.parse().unwrap();
        let graph_accepts = validate_confidence_score(read_back).is_ok();

        let parsed = parse_annotations(&format!(
            "@provenance confidence: {written}\n@provenance rule: RULE-001"
        ));
        let scanner_accepts = parsed.warnings.is_empty();

        assert_eq!(
            scanner_accepts, graph_accepts,
            "scanner and graph disagree on {written}"
        );
        let confidence = parsed.annotations[0].confidence;
        if graph_accepts {
            assert_eq!(
                confidence.to_bits(),
                read_back.to_bits(),
                "scanner altered an in-range confidence {written}"
            );
            kept += 1;
        } else {
            // The remedy may differ; the verdict may not. The scanner warns and
            // falls back rather than keeping a rounded-off score.
            assert_eq!(
                parsed.warnings.len(),
                1,
                "scanner must emit one warning before falling back for {written}"
            );
            assert_eq!(parsed.warnings[0].line, 1);
            let expected_warning = if read_back.is_finite() {
                format!("confidence `{written}` is outside 0.0-1.0, using default")
            } else {
                format!("invalid confidence `{written}`, using default")
            };
            assert_eq!(parsed.warnings[0].message, expected_warning);
            assert!(
                (confidence - 1.0).abs() < f64::EPSILON,
                "scanner kept {confidence:?} for an out-of-range {written}"
            );
            refused += 1;
        }
    }
    assert!(
        kept >= 256 && refused >= 256,
        "{kept} kept and {refused} refused; the conformance check proves too little"
    );
}
