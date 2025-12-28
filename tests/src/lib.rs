#[cfg(test)]
mod tests {
    use super::*;
    use linera_sdk::test::{ContractRuntime, test_contract_runtime};

    #[test]
    fn test_market_creation() {
        let runtime = ContractRuntime::new();
        let mut conwaybets = ConwayBets::new(runtime);

        let market_id = conwaybets.create_market(
            Owner::from([0u8; 32]),
            "Test Market".to_string(),
            "Description".to_string(),
            1_000_000_000,
            vec!["Yes".to_string(), "No".to_string()],
        ).now_or_never().unwrap();

        assert!(conwaybets.markets.contains_key(&market_id));
    }
}