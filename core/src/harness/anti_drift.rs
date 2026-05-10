// Anti-drift: detects when the model has forgotten its role
// and generates a reminder injection

pub const GUARD: &str = "Remember: You are MICRODRAGON. Execute, don't just advise. Be specific, not generic. Use tools when available. End with a clear next step.";

/// Check if a response shows signs of model drift
/// Returns 0.0 (no drift) to 1.0 (full drift)
pub fn check_drift(response: &str) -> f32 {
    let r = response.to_lowercase();
    let mut drift: f32 = 0.0;

    // Signs of "assistant brain" overriding Microdragon identity
    let drift_phrases = [
        ("i'm an ai language model", 0.9),
        ("as an ai assistant", 0.8),
        ("i cannot help with", 0.7),
        ("i don't have the ability to", 0.5),
        ("i'm not able to", 0.5),
        ("that's a great question", 0.3),
        ("certainly! i'd be happy to", 0.4),
        ("of course! here's", 0.2),
    ];

    for (phrase, weight) in &drift_phrases {
        if r.contains(phrase) {
            drift = drift.max(*weight);
        }
    }

    drift
}
