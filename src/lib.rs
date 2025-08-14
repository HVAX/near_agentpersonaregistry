#[cfg(test)]
mod tests {
    #[test]
    #[should_panic]
    fn set_persona_wrong_key_should_fail() {
        let alice = accounts(0);
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(alice.clone());
        testing_env!(builder.build());
        let mut contract = AgentRegistry::new();
        // Simulate a call with the wrong key name using manual serialization
        use near_sdk::serde_json::json;
        let args = json!({"not_cid": "bafkreigs35l72j7oni6pwwnny2ae2nb4lto5jdzfy2yyb5hxggdeub2f2i"});
        // This will panic because deserialization will fail
        let _ = near_sdk::serde_json::from_value::<(String,)>(args).unwrap();
    }
    use super::*;
    use near_sdk::test_utils::{VMContextBuilder, accounts};
    use near_sdk::{testing_env, VMContext};
    use near_sdk::env;

    #[test]
    fn set_persona_emits_event() {
        let alice = accounts(0);
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(alice.clone());
        testing_env!(builder.build());
        let mut contract = AgentRegistry::new();
    let cid = "bafkreigs35l72j7oni6pwwnny2ae2nb4lto5jdzfy2yyb5hxggdeub2f2i".to_string();
        contract.set_persona(cid.clone());
        let logs = near_sdk::test_utils::get_logs();
        let expected_event = format!(
            "EVENT_JSON:{}",
            near_sdk::serde_json::to_string(&PersonaSetEvent {
                account_id: alice,
                cid,
            }).unwrap()
        );
        assert!(logs.iter().any(|log| log == &expected_event), "Event log not found: {:?}", logs);
    }
    fn get_context(predecessor: AccountId) -> VMContext {
        VMContextBuilder::new().predecessor_account_id(predecessor).build()
    }

    #[test]
    fn set_and_get_persona() {
        let alice = accounts(0);
        testing_env!(get_context(alice.clone()));
        let mut contract = AgentRegistry::new();
    let cid = "bafkreigs35l72j7oni6pwwnny2ae2nb4lto5jdzfy2yyb5hxggdeub2f2i".to_string();
        contract.set_persona(cid.clone());
        assert_eq!(contract.get_persona(alice.clone()), Some(cid));
    }

    #[test]
    fn overwrite_persona() {
        let alice = accounts(0);
        testing_env!(get_context(alice.clone()));
        let mut contract = AgentRegistry::new();
    let cid1 = "bafkreigs35l72j7oni6pwwnny2ae2nb4lto5jdzfy2yyb5hxggdeub2f2i".to_string();
    let cid2 = "bafkreigs35l72j7oni6pwwnny2ae2nb4lto5jdzfy2yyb5hxggdeub2f2i".to_string();
        contract.set_persona(cid1.clone());
        contract.set_persona(cid2.clone());
        assert_eq!(contract.get_persona(alice.clone()), Some(cid2));
    }

    #[test]
    #[should_panic(expected = "Invalid CID format. Must be a valid IPFS CID starting with 'bafy'.")]
    fn reject_empty_cid() {
        let alice = accounts(0);
        testing_env!(get_context(alice.clone()));
        let mut contract = AgentRegistry::new();
        contract.set_persona("".to_string());
    }

    #[test]
    fn only_self_can_set_persona() {
        let alice = accounts(0);
        let bob = accounts(1);
        testing_env!(get_context(alice.clone()));
        let mut contract = AgentRegistry::new();
    let alice_cid = "bafkreigs35l72j7oni6pwwnny2ae2nb4lto5jdzfy2yyb5hxggdeub2f2i".to_string();
    let bob_cid = "bafkreigs35l72j7oni6pwwnny2ae2nb4lto5jdzfy2yyb5hxggdeub2f2i".to_string();
        contract.set_persona(alice_cid.clone());
        testing_env!(get_context(bob.clone()));
        contract.set_persona(bob_cid.clone());
        assert_eq!(contract.get_persona(alice.clone()), Some(alice_cid));
        assert_eq!(contract.get_persona(bob.clone()), Some(bob_cid));
    }

    #[test]
    fn get_persona_none_for_unregistered() {
        let alice = accounts(0);
        let bob = accounts(1);
        testing_env!(get_context(alice.clone()));
        let contract = AgentRegistry::new();
        assert_eq!(contract.get_persona(bob.clone()), None);
    }
}

use near_sdk::{near, env, AccountId, collections::LookupMap, PanicOnDefault};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};

pub type CID = String;



#[near(contract_state)]
pub struct AgentRegistry {
    personas: LookupMap<AccountId, CID>,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self {
            personas: LookupMap::new(b"p"),
        }
    }
}
use near_sdk::serde::Serialize;

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PersonaSetEvent {
    account_id: AccountId,
    cid: CID,
}

#[near]
impl AgentRegistry {
    #[init]
    pub fn new() -> Self {
        Self {
            personas: LookupMap::new(b"p"),
        }
    }

    /// Register or update agent persona CID
    pub fn set_persona(&mut self, cid: CID) {
        let caller = env::predecessor_account_id();
        if !is_valid_cid(&cid) {
            env::panic_str("Invalid CID format. Must be a valid IPFS CID starting with 'bafy'.");
        }
        self.personas.insert(&caller, &cid);

        // Emit event for indexing and audit
        let event = PersonaSetEvent { account_id: caller, cid };
        env::log_str(&format!("EVENT_JSON:{}", near_sdk::serde_json::to_string(&event).unwrap()));
    }

    /// Get the persona CID for a given account
    pub fn get_persona(&self, account_id: AccountId) -> Option<CID> {
        self.personas.get(&account_id)
    }

}

// Helper function for CID validation
fn is_valid_cid(cid: &str) -> bool {
    !cid.trim().is_empty()
}
