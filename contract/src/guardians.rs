use crate::*;

#[near_bindgen]
impl Contract {
    pub fn set_disabled_apps_number(&mut self, disabled_apps: u64) {
        self.assert_guardian();
        self.disabled_apps = disabled_apps;
    }

    pub fn get_guardians(&self) -> Vec<AccountId> {
        self.guardians.to_vec()
    }

    pub fn add_guardian(&mut self, account_id: AccountId) {
        self.assert_guardian();
        self.guardians.insert(&account_id);
    }

    pub fn remove_guardian(&mut self, account_id: AccountId) {
        self.assert_guardian();
        require!(self.guardians.len() > 1, "ERR_LAST_GUARDIAN");
        self.guardians.remove(&account_id);
    }

    pub fn add_category(&mut self, title: String, slug: String) {
        self.assert_guardian();

        for category_id in 0..self.next_category_id {
            let category: Category = self.categories.get(&category_id).expect("ERR_NO_CATEGORY").into();
            require!(category.slug != slug.clone(), "ERR_SLUG_ALREADY_EXISTS");
            require!(category.title != title.clone(), "ERR_TITLE_ALREADY_EXISTS");
        }

        let category = Category {
            slug,
            title,
        };

        self.categories.insert(&self.next_category_id, &VCategory::Current(category));

        self.apps_ids_by_category_id.insert(&self.next_category_id, &UnorderedSet::new(StorageKey::AppIdsSetInCategoryId { category_id: self.next_category_id }));

        self.next_category_id += 1;
    }

    pub fn disable_app(&mut self, app_id: AppId) {
        self.assert_guardian();
        let mut app: App = self.apps.get(&app_id).expect("ERR_NO_APP").into();

        if app.active == Some(true) {
            self.disabled_apps += 1;
        }

        app.active = Some(false);

        // clear categories for disabled app to keep proper counters value
        for category_id in app.categories.to_vec() {
            if self.categories.get(&category_id).is_some() {
                let mut apps_ids_by_category_id = self.apps_ids_by_category_id.get(&category_id).expect("ERR_NO_DATA");
                apps_ids_by_category_id.remove(&app_id);
                self.apps_ids_by_category_id.insert(&category_id, &apps_ids_by_category_id);
            }
        }

        self.apps.insert(&app_id, &VApp::Current(app));
    }

    #[payable]
    pub fn update_app(&mut self, app_id: AppId, mut app: AppJSON) {
        let edit_by_guardian = self.guardians.contains(&env::predecessor_account_id());
        let edit_by_author = app.added_by_account_id == Some(env::predecessor_account_id());
        if !(edit_by_guardian || edit_by_author) {
            env::panic_str("ERR_NO_ACCESS")
        }
        let old_app: App = self.apps.get(&app_id).expect("ERR_NO_APP").into();

        if !edit_by_guardian {
            // EDIT BY AUTHOR, NOT GUARDIAN
            require!(env::attached_deposit() >= LISTING_FEE, "ERR_LISTING_FEE_REQUIRED");
            app.active = old_app.active;
        }

        require!(old_app.dapp_account_id == app.dapp_account_id, "ERR_CANT_UPDATE_ACCOUNT_ID");

        if old_app.active == Some(false) && self.disabled_apps > 0 {
            self.disabled_apps -= 1;
        }

        /* REMOVE OLD DATA */
        self.app_id_by_slug.remove(&old_app.slug);
        self.app_id_by_dapp_account_id.remove(&old_app.dapp_account_id);

        for category_id in old_app.categories.to_vec() {
            if self.categories.get(&category_id).is_some() {
                let mut apps_ids_by_category_id = self.apps_ids_by_category_id.get(&category_id).expect("ERR_NO_DATA");
                apps_ids_by_category_id.remove(&app_id);
                self.apps_ids_by_category_id.insert(&category_id, &apps_ids_by_category_id);
            }
        }

        /* CREATE NEW DATA */
        let mut categories = old_app.categories;
        categories.clear();
        for category_string in app.categories.clone() {
            let category_id: CategoryId = category_string.parse().expect("ERR_WRONG_CATEGORY");
            if self.categories.get(&category_id).is_some() {
                let mut apps_ids_by_category_id = self.apps_ids_by_category_id.get(&category_id).expect("ERR_NO_DATA");
                apps_ids_by_category_id.insert(&app_id);
                self.apps_ids_by_category_id.insert(&category_id, &apps_ids_by_category_id);
                categories.insert(&category_id);
            }
        }
        let mut contracts = old_app.contracts;
        contracts.clear();
        let contracts_vec = if let Some(contracts_vec) = app.contracts.clone() { contracts_vec } else { vec![] };
        for contract in contracts_vec {
            contracts.insert(&contract);
        }

        let added_by_account_id = if app.added_by_account_id.is_some() {
            app.added_by_account_id.clone()
        } else {
            Some(old_app.added_by_account_id)
        };

        self.internal_insert_app(app_id, app, added_by_account_id, categories, contracts);
    }
}