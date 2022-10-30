use near_sdk::{
    near_bindgen, AccountId, BorshStorageKey, PanicOnDefault,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{UnorderedMap, UnorderedSet},
    serde::{Deserialize, Serialize},
    Gas, log, PromiseError, Balance, Promise,
    env, require, ext_contract,
    json_types::U128
};
use std::collections::HashMap;

mod utils;
mod app;
mod category;
mod web4;
mod guardians;
mod migration;

type AppId = u64;
type CategoryId = u64;
type Slug = String;

use crate::app::*;
use crate::category::*;
use crate::web4::*;
use crate::utils::*;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Apps,
    Categories,
    AppCategories { app_id: AppId },
    AppContracts { app_id: AppId },

    AppIdBySlug,
    AppIdsByCategoryId,
    AppIdsSetInCategoryId { category_id: CategoryId },
    AppIdsByAccountId,

    Guardians,
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    guardians: UnorderedSet<AccountId>,

    apps: UnorderedMap<AppId, VApp>,
    categories: UnorderedMap<CategoryId, VCategory>,

    app_id_by_slug: UnorderedMap<Slug, AppId>,
    apps_ids_by_category_id: UnorderedMap<CategoryId, UnorderedSet<AppId>>,
    app_id_by_dapp_account_id: UnorderedMap<AccountId, AppId>,

    next_app_id: AppId,
    next_category_id: CategoryId,
    disabled_apps: u64
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(guardian_id: AccountId) -> Self {
        let mut guardians = UnorderedSet::new(StorageKey::Guardians);
        guardians.insert(&guardian_id);

        Self {
            guardians,
            apps: UnorderedMap::new(StorageKey::Apps),
            categories: UnorderedMap::new(StorageKey::Categories),

            app_id_by_slug: UnorderedMap::new(StorageKey::AppIdBySlug),
            apps_ids_by_category_id: UnorderedMap::new(StorageKey::AppIdsByCategoryId),
            app_id_by_dapp_account_id: UnorderedMap::new(StorageKey::AppIdsByAccountId),

            next_app_id: 0,
            next_category_id: 0,
            disabled_apps: 0
        }
    }

    pub fn get_app(&self, app_id: AppId) -> AppJSON {
        self.apps.get(&app_id).expect("ERR_NO_APP").into()
    }

    pub fn get_apps(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<(AppId, AppJSON)> {
        unordered_map_pagination(&self.apps, from_index, limit)
    }

    pub fn get_categories(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<(CategoryId, CategoryJSON)> {
        unordered_map_pagination(&self.categories, from_index, limit)
    }

    pub fn get_category_apps_count(&self, category_id: CategoryId) -> u64 {
        self.apps_ids_by_category_id.get(&category_id).expect("ERR_NO_CATEGORY").len()
    }

    pub fn get_app_by_slug(&self, slug: String) -> AppJSON {
       self.internal_get_app_by_slug(&slug).1.into()
    }

    pub fn get_app_by_account_id(&self, account_id: AccountId) -> AppJSON {
        let app_id = self.app_id_by_dapp_account_id.get(&account_id).expect("ERR_ACCOUNT_ID");
        self.apps.get(&app_id).expect("ERR_NO_APP").into()
    }
}
