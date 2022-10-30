use crate::*;
#[near_bindgen]
impl Contract {
    /*
    #[init(ignore_state)]
    #[allow(dead_code)]
    #[private]
    pub fn migrate_state() -> Self {
        #[derive(BorshDeserialize)]
        struct OldContract {
            guardians: UnorderedSet<AccountId>,

            apps: UnorderedMap<AppId, VApp>,
            categories: UnorderedMap<CategoryId, VCategory>,

            app_id_by_slug: UnorderedMap<Slug, AppId>,
            apps_ids_by_category_id: UnorderedMap<CategoryId, UnorderedSet<AppId>>,
            app_id_by_dapp_account_id: UnorderedMap<AccountId, AppId>,

            next_app_id: AppId,
            next_category_id: CategoryId,
        }

        let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");

        Self {
            guardians: old_contract.guardians,
            apps: old_contract.apps,
            categories: old_contract.categories,

            app_id_by_slug: old_contract.app_id_by_slug,
            apps_ids_by_category_id: old_contract.apps_ids_by_category_id,
            app_id_by_dapp_account_id: old_contract.app_id_by_dapp_account_id,

            next_app_id: old_contract.next_app_id,
            next_category_id: old_contract.next_category_id,
            disabled_apps: 0
        }
    }
     */
}