use cbc_node::fork_detection::ForkReport;

#[tokio::test]
async fn test_fork_report_creation() {
    let report = ForkReport {
        peer_id: "test-peer".to_string(),
        local_best: 100,
        peer_best: 95,
        divergence: 5,
    };
    
    assert_eq!(report.peer_id, "test-peer");
    assert_eq!(report.local_best, 100);
    assert_eq!(report.peer_best, 95);
    assert_eq!(report.divergence, 5);
}

#[test]
fn test_fork_report_serialization() {
    let report = ForkReport {
        peer_id: "test-peer".to_string(),
        local_best: 100,
        peer_best: 95,
        divergence: 5,
    };
    
    let json = serde_json::to_string(&report).unwrap();
    let deserialized: ForkReport = serde_json::from_str(&json).unwrap();
    
    assert_eq!(report.peer_id, deserialized.peer_id);
    assert_eq!(report.local_best, deserialized.local_best);
    assert_eq!(report.peer_best, deserialized.peer_best);
    assert_eq!(report.divergence, deserialized.divergence);
}

#[test]
fn test_fork_detection_threshold_logic() {
    // Test divergence calculation using production ForkReport implementation
    let threshold = 10u32;
    let report_normal = ForkReport::new("peer-1", 100, 95);

    assert_eq!(report_normal.divergence, 5);
    assert!(!report_normal.is_divergent(threshold), "Divergence of 5 should be within threshold of 10");

    // Test case where divergence exceeds threshold using production logic
    let report_far = ForkReport::new("peer-2", 100, 80);

    assert_eq!(report_far.divergence, 20);
    assert!(report_far.is_divergent(threshold), "Divergence of 20 should exceed threshold of 10");
}