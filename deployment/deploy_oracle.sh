NEAR_ENV=$1 near deploy --accountId $2 --wasmFile ./res/oracle.wasm --initFunction new --initArgs '{ "config": { "gov": "flux-dev", "final_arbitrator": "flux-dev", "stake_token": "v2.wnear.flux-dev", "payment_token": "v2.wnear.flux-dev", "validity_bond": "1000000000000000000000000", "max_outcomes": 8, "default_challenge_window_duration": "120000000000", "min_initial_challenge_window_duration": "120000000000", "final_arbitrator_invoke_amount": "100000000000000000000000000", "resolution_fee_percentage": 5000, "fee": {"flux_market_cap": "10000000000000", "total_value_staked":"1000", "resolution_fee_percentage": 1000 } } }'