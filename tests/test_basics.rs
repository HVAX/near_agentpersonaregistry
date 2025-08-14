use serde_json::json;

#[tokio::test]
async fn test_contract_is_operational() -> Result<(), Box<dyn std::error::Error>> {
    let contract_wasm = near_workspaces::compile_project("./").await?;

    test_basics_on(&contract_wasm).await?;
    Ok(())
}

async fn test_basics_on(contract_wasm: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract = sandbox.dev_deploy(contract_wasm).await?;

    let user_account = sandbox.dev_create_account().await?;

    // Initialize the contract
    let init_outcome = user_account
        .call(contract.id(), "new")
        .args_json(json!({}))
        .transact()
        .await?;
    assert!(init_outcome.is_success(), "Contract initialization failed: {:#?}", init_outcome.into_result().unwrap_err());

    // Test set_persona and get_persona for AgentRegistry
    let cid = "bafybeigdyrzt4persona";
    let outcome = user_account
        .call(contract.id(), "set_persona")
        .args_json(json!({"cid": cid}))
        .transact()
        .await?;
    assert!(outcome.is_success(), "{:#?}", outcome.into_result().unwrap_err());

    let get_outcome = contract
        .view("get_persona")
        .args_json(json!({"account_id": user_account.id()}))
        .await?;
    assert_eq!(get_outcome.json::<Option<String>>()?, Some(cid.to_string()));

    // Create a second account and set its persona
    let second_account = sandbox.dev_create_account().await?;
    let cid2 = "bafybeigdyrzt4second";
    let outcome2 = second_account
        .call(contract.id(), "set_persona")
        .args_json(json!({"cid": cid2}))
        .transact()
        .await?;
    assert!(outcome2.is_success(), "{:#?}", outcome2.into_result().unwrap_err());

    // Ensure both accounts have their own CID
    let get_outcome1 = contract
        .view("get_persona")
        .args_json(json!({"account_id": user_account.id()}))
        .await?;
    assert_eq!(get_outcome1.json::<Option<String>>()?, Some(cid.to_string()));

    let get_outcome2 = contract
        .view("get_persona")
        .args_json(json!({"account_id": second_account.id()}))
        .await?;
    assert_eq!(get_outcome2.json::<Option<String>>()?, Some(cid2.to_string()));

    // Overwrite CID for user_account
    let new_cid = "bafybeigdyrzt4newpersona";
    let overwrite_outcome = user_account
        .call(contract.id(), "set_persona")
        .args_json(json!({"cid": new_cid}))
        .transact()
        .await?;
    assert!(overwrite_outcome.is_success(), "{:#?}", overwrite_outcome.into_result().unwrap_err());
    let get_overwritten = contract
        .view("get_persona")
        .args_json(json!({"account_id": user_account.id()}))
        .await?;
    assert_eq!(get_overwritten.json::<Option<String>>()?, Some(new_cid.to_string()));

    // Reject empty CID
    let empty_cid_outcome = user_account
        .call(contract.id(), "set_persona")
        .args_json(json!({"cid": ""}))
        .transact()
        .await?;
    assert!(!empty_cid_outcome.is_success(), "Empty CID should fail");

    // Query for unset account
    let third_account = sandbox.dev_create_account().await?;
    let get_unset = contract
        .view("get_persona")
        .args_json(json!({"account_id": third_account.id()}))
        .await?;
    assert_eq!(get_unset.json::<Option<String>>()?, None);

    Ok(())
}


