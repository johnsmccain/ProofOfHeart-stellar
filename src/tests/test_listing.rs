use super::helpers::*;
use crate::{Category};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_list_campaigns_exclusive_cursor_semantics() {
    let (env, _admin, creator, _c1, _c2, _token, _token_admin, client) = setup_env();

    for i in 0..3 {
        let id = client.create_campaign(&make_params(
            creator.clone(),
            String::from_str(&env, "Campaign"),
            String::from_str(&env, "Desc"),
            1000 + i as i128,
            30,
            Category::Learner,
            false,
            0,
            0i128,
        ));
        assert_eq!(id, (i + 1) as u32);
    }

    let page1 = client.list_campaigns(&0, &2);
    assert_eq!(page1.len(), 2);
    assert_eq!(page1.get(0).unwrap().id, 1);
    assert_eq!(page1.get(1).unwrap().id, 2);

    let page2 = client.list_campaigns(&2, &2);
    assert_eq!(page2.len(), 1);
    assert_eq!(page2.get(0).unwrap().id, 3);
}

#[test]
fn test_list_active_campaigns_exclusive_cursor_semantics() {
    let (env, _admin, creator, _c1, _c2, _token, _token_admin, client) = setup_env();

    for _ in 0..4 {
        let _ = client.create_campaign(&make_params(
            creator.clone(),
            String::from_str(&env, "Campaign"),
            String::from_str(&env, "Desc"),
            1000,
            30,
            Category::Learner,
            false,
            0,
            0i128,
        ));
    }

    client.cancel_campaign(&2);

    let active1 = client.list_active_campaigns(&0, &2);
    assert_eq!(active1.0.len(), 2);
    assert_eq!(active1.0.get(0).unwrap().id, 1);
    assert_eq!(active1.0.get(1).unwrap().id, 3);

    let active2 = client.list_active_campaigns(&3, &2);
    assert_eq!(active2.0.len(), 1);
    assert_eq!(active2.0.get(0).unwrap().id, 4);
}

#[test]
fn test_get_campaigns_by_category_with_pagination() {
    let (env, _admin, creator, _, _, _, _, client) = setup_env();

    let id1 = client.create_campaign(&make_params(
        creator.clone(),
        String::from_str(&env, "Learner 1"),
        String::from_str(&env, "a"),
        100,
        30,
        Category::Learner,
        false,
        0,
        0i128,
    ));
    let _id2 = client.create_campaign(&make_params(
        creator.clone(),
        String::from_str(&env, "Publisher 1"),
        String::from_str(&env, "b"),
        100,
        30,
        Category::Publisher,
        false,
        0,
        0i128,
    ));
    let id3 = client.create_campaign(&make_params(
        creator.clone(),
        String::from_str(&env, "Learner 2"),
        String::from_str(&env, "c"),
        100,
        30,
        Category::Learner,
        false,
        0,
        0i128,
    ));

    let learner_page_1 = client.get_campaigns_by_category(&Category::Learner, &0, &1);
    assert_eq!(learner_page_1.len(), 1);
    assert_eq!(learner_page_1.get(0).unwrap().id, id1);

    let learner_page_2 = client.get_campaigns_by_category(&Category::Learner, &1, &1);
    assert_eq!(learner_page_2.len(), 1);
    assert_eq!(learner_page_2.get(0).unwrap().id, id3);

    let publisher = client.get_campaigns_by_category(&Category::Publisher, &0, &10);
    assert_eq!(publisher.len(), 1);
    assert_eq!(publisher.get(0).unwrap().category, Category::Publisher);
}

#[test]
fn test_get_platform_stats_returns_aggregates() {
    let (env, _admin, creator, contributor1, contributor2, _token, token_admin, client) =
        setup_env();
    token_admin.mint(&contributor1, &2_000);
    token_admin.mint(&contributor2, &2_000);

    let c1 = client.create_campaign(&make_params(
        creator.clone(),
        String::from_str(&env, "Stats 1"),
        String::from_str(&env, "s1"),
        500,
        30,
        Category::Learner,
        false,
        0,
        0i128,
    ));
    let c2 = client.create_campaign(&make_params(
        creator.clone(),
        String::from_str(&env, "Stats 2"),
        String::from_str(&env, "s2"),
        500,
        30,
        Category::Learner,
        false,
        0,
        0i128,
    ));

    let _ = client.try_verify_campaign(&c1);
    let _ = client.try_verify_campaign(&c2);
    client.contribute(&c1, &contributor1, &400);
    client.contribute(&c2, &contributor2, &300);
    client.cancel_campaign(&c2);

    let stats = client.get_platform_stats();
    assert_eq!(stats.total_campaigns, 2);
    assert_eq!(stats.active_campaigns, 1);
    assert_eq!(stats.verified_campaigns, 2);
    assert_eq!(stats.cancelled_campaigns, 1);
    assert_eq!(stats.total_amount_raised, 700);
}

#[test]
fn test_total_raised_global_tracking() {
    let (env, _admin, creator, contributor1, contributor2, _token, token_admin, client) =
        setup_env();

    token_admin.mint(&contributor1, &5000);
    token_admin.mint(&contributor2, &5000);

    let c1 = client.create_campaign(&make_params(
        creator.clone(),
        String::from_str(&env, "Campaign 1"),
        String::from_str(&env, "First"),
        1000,
        30,
        Category::Educator,
        false,
        0,
        0i128,
    ));
    client.verify_campaign(&c1);

    let c2 = client.create_campaign(&make_params(
        creator.clone(),
        String::from_str(&env, "Campaign 2"),
        String::from_str(&env, "Second"),
        2000,
        30,
        Category::Learner,
        false,
        0,
        0i128,
    ));
    client.verify_campaign(&c2);

    assert_eq!(client.get_total_raised_global(), 0);

    client.contribute(&c1, &contributor1, &500);
    assert_eq!(client.get_total_raised_global(), 500);

    client.contribute(&c2, &contributor2, &1000);
    assert_eq!(client.get_total_raised_global(), 1500);

    client.cancel_campaign(&c2);
    client.claim_refund(&c2, &contributor2);
    assert_eq!(client.get_total_raised_global(), 500);

    client.contribute(&c1, &contributor2, &500);
    assert_eq!(client.get_total_raised_global(), 1000);

    client.withdraw_funds(&c1);
    assert_eq!(client.get_total_raised_global(), 0);
}

#[test]
fn test_creator_campaigns_listing_and_transfer() {
    let (env, _admin, creator1, _c1, _c2, _token, _token_admin, client) = setup_env();
    let creator2 = Address::generate(&env);

    let id1 = client.create_campaign(&make_params(
        creator1.clone(),
        String::from_str(&env, "Campaign 1"),
        String::from_str(&env, "First"),
        1000,
        30,
        Category::Educator,
        false,
        0,
        0i128,
    ));

    let id2 = client.create_campaign(&make_params(
        creator1.clone(),
        String::from_str(&env, "Campaign 2"),
        String::from_str(&env, "Second"),
        2000,
        30,
        Category::Learner,
        false,
        0,
        0i128,
    ));

    let list1 = client.get_creator_campaigns(&creator1, &0, &10);
    assert_eq!(list1.len(), 2);
    assert_eq!(list1.get(0).unwrap().id, id1);
    assert_eq!(list1.get(1).unwrap().id, id2);

    let paginated1 = client.get_creator_campaigns(&creator1, &0, &1);
    assert_eq!(paginated1.len(), 1);
    assert_eq!(paginated1.get(0).unwrap().id, id1);

    let paginated2 = client.get_creator_campaigns(&creator1, &1, &1);
    assert_eq!(paginated2.len(), 1);
    assert_eq!(paginated2.get(0).unwrap().id, id2);

    let list2 = client.get_creator_campaigns(&creator2, &0, &10);
    assert_eq!(list2.len(), 0);

    client.initiate_campaign_transfer(&id1, &creator2);
    client.accept_campaign_transfer(&id1);

    let list1_after = client.get_creator_campaigns(&creator1, &0, &10);
    assert_eq!(list1_after.len(), 1);
    assert_eq!(list1_after.get(0).unwrap().id, id2);

    let list2_after = client.get_creator_campaigns(&creator2, &0, &10);
    assert_eq!(list2_after.len(), 1);
    assert_eq!(list2_after.get(0).unwrap().id, id1);
}

/// Verifies that list_active_campaigns correctly advances the cursor and returns
/// all active campaigns when the majority of campaigns are cancelled (sparse distribution).
///
/// Scenario: 12 campaigns created, campaigns 1–10 cancelled, only 11 and 12 active.
/// Paginating with page size 1 must return campaign 11 then 12 with no omissions,
/// no duplicates, and a zero cursor after the final page.
#[test]
fn test_list_active_campaigns_sparse_active_distribution() {
    let (env, _admin, creator, _c1, _c2, _token, _token_admin, client) = setup_env();

    let total = 12u32;
    let active_from = 11u32; // campaigns 11 and 12 are active; 1–10 are cancelled

    for _ in 0..total {
        client.create_campaign(&make_params(
            creator.clone(),
            String::from_str(&env, "Campaign"),
            String::from_str(&env, "Desc"),
            1000,
            30,
            Category::Learner,
            false,
            0,
            0i128,
        ));
    }

    for id in 1..active_from {
        client.cancel_campaign(&id);
    }

    // Page 1: expect campaign 11, cursor should advance past it
    let (page1, cursor1) = client.list_active_campaigns(&0, &1);
    assert_eq!(page1.len(), 1);
    assert_eq!(page1.get(0).unwrap().id, 11);
    assert!(cursor1 > 0, "cursor must be non-zero when more results remain");

    // Page 2: continue from cursor1, expect campaign 12
    let (page2, cursor2) = client.list_active_campaigns(&cursor1.saturating_sub(1), &1);
    assert_eq!(page2.len(), 1);
    assert_eq!(page2.get(0).unwrap().id, 12);

    // No duplicates across pages
    assert_ne!(
        page1.get(0).unwrap().id,
        page2.get(0).unwrap().id,
        "pages must not overlap"
    );

    // Final page: cursor must be zero (no more results)
    let (page3, cursor3) = client.list_active_campaigns(&12, &1);
    assert_eq!(page3.len(), 0);
    assert_eq!(cursor3, 0);

    // Full sweep with large limit returns exactly the two active campaigns
    let (all, _) = client.list_active_campaigns(&0, &50);
    assert_eq!(all.len(), 2);
    assert_eq!(all.get(0).unwrap().id, 11);
    assert_eq!(all.get(1).unwrap().id, 12);
}
