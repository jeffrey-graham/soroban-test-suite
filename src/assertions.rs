//! Assertion utilities for Soroban contract testing

use soroban_sdk::{testutils::Events as _, Address, Env, IntoVal, Val, Vec as SorobanVec};

/// Expected contract event matching contract ID, topics, and payload.
#[derive(Clone, Debug)]
pub struct ExpectedEvent {
    /// Emitting contract address.
    pub contract: Address,
    /// Event topic vector.
    pub topics: SorobanVec<Val>,
    /// Event payload value.
    pub data: Val,
}

impl ExpectedEvent {
    /// Build an expected event for `contract` with `topics` and `data`.
    pub fn new<T, D>(env: &Env, contract: &Address, topics: T, data: D) -> Self
    where
        T: IntoVal<Env, SorobanVec<Val>>,
        D: IntoVal<Env, Val>,
    {
        Self {
            contract: contract.clone(),
            topics: topics.into_val(env),
            data: data.into_val(env),
        }
    }
}

fn vals_equal(env: &Env, left: Val, right: Val) -> bool {
    soroban_sdk::vec![env, left] == soroban_sdk::vec![env, right]
}

fn event_matches(
    env: &Env,
    actual: &(Address, SorobanVec<Val>, Val),
    expected: &ExpectedEvent,
) -> bool {
    actual.0 == expected.contract
        && actual.1 == expected.topics
        && vals_equal(env, actual.2, expected.data)
}

fn describe_mismatch(
    env: &Env,
    actual: &(Address, SorobanVec<Val>, Val),
    expected: &ExpectedEvent,
) -> &'static str {
    if actual.0 != expected.contract {
        "contract ID mismatch"
    } else if actual.1 != expected.topics {
        "topics mismatch"
    } else if !vals_equal(env, actual.2, expected.data) {
        "payload mismatch"
    } else {
        "matched"
    }
}

/// Assert that at least one emitted event in `env` matches `contract`, `topics`, and `data`.
pub fn assert_event_emitted<T, D>(env: &Env, contract: &Address, topics: T, data: D)
where
    T: IntoVal<Env, SorobanVec<Val>>,
    D: IntoVal<Env, Val>,
{
    let expected = ExpectedEvent::new(env, contract, topics, data);
    let events = env.events().all();

    for event in events.iter() {
        if event_matches(env, &event, &expected) {
            return;
        }
    }

    let reasons: Vec<String> = events
        .iter()
        .enumerate()
        .map(|(idx, ev)| {
            format!(
                "#{idx} ({reason}): contract={:?}, topics={:?}, data={:?}",
                ev.0,
                ev.1,
                ev.2,
                reason = describe_mismatch(env, &ev, &expected)
            )
        })
        .collect();

    panic!(
        "Assertion failed: Expected contract event was not emitted.\n  Expected: contract={:?}, topics={:?}, data={:?}\n  Recorded events ({}): [{}]",
        expected.contract,
        expected.topics,
        expected.data,
        events.len(),
        reasons.join(", ")
    );
}

/// Assert that no emitted event in `env` matches `contract`, `topics`, and `data`.
pub fn assert_event_not_emitted<T, D>(env: &Env, contract: &Address, topics: T, data: D)
where
    T: IntoVal<Env, SorobanVec<Val>>,
    D: IntoVal<Env, Val>,
{
    let expected = ExpectedEvent::new(env, contract, topics, data);
    let events = env.events().all();

    for (idx, event) in events.iter().enumerate() {
        if event_matches(env, &event, &expected) {
            panic!(
                "Assertion failed: Expected contract event to be absent, but found match at index #{idx}: contract={:?}, topics={:?}, data={:?}",
                event.0, event.1, event.2
            );
        }
    }
}

/// Assert that the emitted events in `env` match `expected` in exact order,
/// reporting the first missing or unexpected event clearly.
pub fn assert_events_ordered(env: &Env, expected: &[ExpectedEvent]) {
    let events = env.events().all();
    let actual_len = events.len() as usize;

    for (idx, exp) in expected.iter().enumerate() {
        let Some(actual) = events.get(idx as u32) else {
            panic!(
                "Assertion failed: Missing expected event at index #{idx}: expected contract={:?}, topics={:?}, data={:?}, but only {actual_len} event(s) were emitted",
                exp.contract, exp.topics, exp.data
            );
        };

        if !event_matches(env, &actual, exp) {
            let reason = describe_mismatch(env, &actual, exp);
            panic!(
                "Assertion failed: Event mismatch at index #{idx} ({reason}).\n  Expected: contract={:?}, topics={:?}, data={:?}\n  Actual:   contract={:?}, topics={:?}, data={:?}",
                exp.contract, exp.topics, exp.data, actual.0, actual.1, actual.2
            );
        }
    }

    if actual_len > expected.len() {
        let unexpected_idx = expected.len();
        let unexpected = events.get(unexpected_idx as u32).unwrap();
        panic!(
            "Assertion failed: Unexpected extra event at index #{unexpected_idx}: contract={:?}, topics={:?}, data={:?} (expected {} event(s), got {actual_len})",
            unexpected.0,
            unexpected.1,
            unexpected.2,
            expected.len()
        );
    }
}

/// Assert that two values are equal with a custom message
#[macro_export]
macro_rules! assert_contract_eq {
    ($left:expr, $right:expr, $msg:expr) => {
        if $left != $right {
            panic!(
                "Assertion failed: {} - Expected: {:?}, Got: {:?}",
                $msg, $right, $left
            );
        }
    };
    ($left:expr, $right:expr) => {
        assert_contract_eq!($left, $right, "Values should be equal")
    };
}

/// Assert that a value is greater than another
#[macro_export]
macro_rules! assert_contract_gt {
    ($left:expr, $right:expr, $msg:expr) => {
        if $left <= $right {
            panic!(
                "Assertion failed: {} - Expected {} > {}, but {} <= {}",
                $msg,
                stringify!($left),
                stringify!($right),
                $left,
                $right
            );
        }
    };
    ($left:expr, $right:expr) => {
        assert_contract_gt!($left, $right, "Value should be greater")
    };
}

/// Assert that a value is greater than or equal to another
#[macro_export]
macro_rules! assert_contract_gte {
    ($left:expr, $right:expr, $msg:expr) => {
        if $left < $right {
            panic!(
                "Assertion failed: {} - Expected {} >= {}, but {} < {}",
                $msg,
                stringify!($left),
                stringify!($right),
                $left,
                $right
            );
        }
    };
    ($left:expr, $right:expr) => {
        assert_contract_gte!($left, $right, "Value should be greater or equal")
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_assert_contract_eq() {
        assert_contract_eq!(5, 5);
        assert_contract_eq!(5, 5, "Numbers should match");
    }

    #[test]
    fn test_assert_contract_gt() {
        assert_contract_gt!(10, 5);
        assert_contract_gt!(10, 5, "10 should be greater than 5");
    }

    #[test]
    fn test_assert_contract_gte() {
        assert_contract_gte!(10, 5);
        assert_contract_gte!(5, 5);
        assert_contract_gte!(10, 5, "10 should be >= 5");
    }

    #[test]
    #[should_panic]
    fn test_assert_contract_eq_fails() {
        assert_contract_eq!(5, 10);
    }

    #[test]
    #[should_panic]
    fn test_assert_contract_gt_fails() {
        assert_contract_gt!(5, 10);
    }

    #[test]
    #[should_panic]
    fn test_assert_contract_gte_fails() {
        assert_contract_gte!(5, 10);
    }

    #[soroban_sdk::contract]
    struct EventTestContract;

    #[soroban_sdk::contractimpl]
    impl EventTestContract {
        pub fn emit_transfer(env: Env, from: Address, to: Address, amount: i128) {
            env.events()
                .publish((soroban_sdk::symbol_short!("transfer"), from, to), amount);
        }

        pub fn emit_status(env: Env, code: u32) {
            env.events()
                .publish((soroban_sdk::symbol_short!("status"),), code);
        }
    }

    #[test]
    fn test_assert_event_emitted_and_absent() {
        let env = Env::default();
        let contract_id = env.register_contract(None, EventTestContract);
        let other_contract = Address::generate(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        let client = EventTestContractClient::new(&env, &contract_id);
        client.emit_transfer(&alice, &bob, &250i128);

        assert_event_emitted(
            &env,
            &contract_id,
            (
                soroban_sdk::symbol_short!("transfer"),
                alice.clone(),
                bob.clone(),
            ),
            250i128,
        );
        assert_event_not_emitted(
            &env,
            &other_contract,
            (
                soroban_sdk::symbol_short!("transfer"),
                alice.clone(),
                bob.clone(),
            ),
            250i128,
        );
        assert_event_not_emitted(
            &env,
            &contract_id,
            (
                soroban_sdk::symbol_short!("approve"),
                alice.clone(),
                bob.clone(),
            ),
            250i128,
        );
        assert_event_not_emitted(
            &env,
            &contract_id,
            (soroban_sdk::symbol_short!("transfer"), alice, bob),
            999i128,
        );
    }

    #[test]
    #[should_panic(expected = "contract ID mismatch")]
    fn test_assert_event_emitted_wrong_contract_panics() {
        let env = Env::default();
        let contract_id = env.register_contract(None, EventTestContract);
        let wrong_contract = Address::generate(&env);

        let client = EventTestContractClient::new(&env, &contract_id);
        client.emit_status(&1u32);

        assert_event_emitted(
            &env,
            &wrong_contract,
            (soroban_sdk::symbol_short!("status"),),
            1u32,
        );
    }

    #[test]
    #[should_panic(expected = "topics mismatch")]
    fn test_assert_event_emitted_wrong_topic_panics() {
        let env = Env::default();
        let contract_id = env.register_contract(None, EventTestContract);

        let client = EventTestContractClient::new(&env, &contract_id);
        client.emit_status(&1u32);

        assert_event_emitted(
            &env,
            &contract_id,
            (soroban_sdk::symbol_short!("other"),),
            1u32,
        );
    }

    #[test]
    #[should_panic(expected = "payload mismatch")]
    fn test_assert_event_emitted_wrong_payload_panics() {
        let env = Env::default();
        let contract_id = env.register_contract(None, EventTestContract);

        let client = EventTestContractClient::new(&env, &contract_id);
        client.emit_status(&1u32);

        assert_event_emitted(
            &env,
            &contract_id,
            (soroban_sdk::symbol_short!("status"),),
            2u32,
        );
    }

    #[test]
    #[should_panic(expected = "Expected contract event to be absent")]
    fn test_assert_event_not_emitted_panics_when_present() {
        let env = Env::default();
        let contract_id = env.register_contract(None, EventTestContract);

        let client = EventTestContractClient::new(&env, &contract_id);
        client.emit_status(&7u32);

        assert_event_not_emitted(
            &env,
            &contract_id,
            (soroban_sdk::symbol_short!("status"),),
            7u32,
        );
    }

    #[test]
    fn test_assert_events_ordered_multiple_events() {
        let env = Env::default();
        let contract_a = env.register_contract(None, EventTestContract);
        let contract_b = env.register_contract(None, EventTestContract);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        let client_a = EventTestContractClient::new(&env, &contract_a);
        let client_b = EventTestContractClient::new(&env, &contract_b);

        client_a.emit_status(&10u32);
        client_b.emit_transfer(&alice, &bob, &500i128);

        let expected = [
            ExpectedEvent::new(
                &env,
                &contract_a,
                (soroban_sdk::symbol_short!("status"),),
                10u32,
            ),
            ExpectedEvent::new(
                &env,
                &contract_b,
                (soroban_sdk::symbol_short!("transfer"), alice, bob),
                500i128,
            ),
        ];

        assert_events_ordered(&env, &expected);
    }

    #[test]
    #[should_panic(expected = "Missing expected event at index #1")]
    fn test_assert_events_ordered_reports_missing_event() {
        let env = Env::default();
        let contract_id = env.register_contract(None, EventTestContract);

        let client = EventTestContractClient::new(&env, &contract_id);
        client.emit_status(&1u32);

        let expected = [
            ExpectedEvent::new(
                &env,
                &contract_id,
                (soroban_sdk::symbol_short!("status"),),
                1u32,
            ),
            ExpectedEvent::new(
                &env,
                &contract_id,
                (soroban_sdk::symbol_short!("status"),),
                2u32,
            ),
        ];

        assert_events_ordered(&env, &expected);
    }

    #[test]
    #[should_panic(expected = "Unexpected extra event at index #1")]
    fn test_assert_events_ordered_reports_unexpected_extra_event() {
        let env = Env::default();
        let contract_id = env.register_contract(None, EventTestContract);

        let client = EventTestContractClient::new(&env, &contract_id);
        client.emit_status(&1u32);
        client.emit_status(&2u32);

        let expected = [ExpectedEvent::new(
            &env,
            &contract_id,
            (soroban_sdk::symbol_short!("status"),),
            1u32,
        )];

        assert_events_ordered(&env, &expected);
    }
}
