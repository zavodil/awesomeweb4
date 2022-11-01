use near_sdk::is_promise_success;
use crate::*;

pub const GAS_FOR_WEB4_GET_CHECK_PROMISE_RESULT: Gas = Gas(Gas::ONE_TERA.0 * 40);
pub const GAS_FOR_ON_WEB4_GET_CHECK_PROMISE_RESULT: Gas = Gas(Gas::ONE_TERA.0 * 30);
pub const GAS_FOR_WEB4_GET_IGNORE_PROMISE_RESULT: Gas = Gas(Gas::ONE_TERA.0 * 10);
pub const GAS_FOR_ON_WEB4_GET_IGNORE_PROMISE_RESULT: Gas = Gas(Gas::ONE_TERA.0 * 5);

pub const LISTING_FEE: Balance = 100_000_000_000_000_000_000_000; // 0.1 NEAR

pub type WrappedBalance = U128;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn add_app(&mut self, app: AppJSON) {
        require!(self.app_id_by_slug.get(&app.slug).is_none(), "ERR_SLUG_ALREADY_EXISTS");
        require!(self.app_id_by_dapp_account_id.get(&app.dapp_account_id).is_none(), "ERR_ACCOUNT_ID_ALREADY_EXISTS");

        require!(app.title.len() <= 50, "ERR_TITLE_IS_TOO_LONG");
        require!(app.slug.len() <= 50, "ERR_SLUG_IS_TOO_LONG");
        require!(app.oneliner.clone().unwrap_or_default().len() <= 200, "ERR_ONELINER_IS_TOO_LONG");
        require!(app.description.clone().unwrap_or_default().len() <= 5000, "ERR_DESCRIPTION_IS_TOO_LONG");

        if !self.guardians.contains(&env::predecessor_account_id()) {
            require!(env::attached_deposit() >= LISTING_FEE, "ERR_LISTING_FEE_REQUIRED");
        }

        self.assert_web4(app.dapp_account_id.clone(), app, WrappedBalance::from(env::attached_deposit()), env::predecessor_account_id(), false);
    }

    #[private]
    pub fn after_web4_get(
        &mut self,
        #[callback_result] response: Result<Web4Response, PromiseError>,
        mut app: AppJSON,
        deposit: WrappedBalance,
        added_by_account_id: AccountId,
        ignore_promise_success: bool
    ) {
        if ignore_promise_success || is_promise_success() {
            match response {
                Ok(_) => {
                    let mut categories = UnorderedSet::new(StorageKey::AppCategories { app_id: self.next_app_id });
                    for category_string in app.categories.clone() {
                        let category_id: CategoryId = category_string.parse().expect("ERR_WRONG_CATEGORY");
                        if self.categories.get(&category_id).is_some() {
                            let mut apps_ids_by_category_id = self.apps_ids_by_category_id.get(&category_id).expect("ERR_NO_DATA");
                            apps_ids_by_category_id.insert(&self.next_app_id);
                            self.apps_ids_by_category_id.insert(&category_id, &apps_ids_by_category_id);
                            categories.insert(&category_id);
                        }
                    }
                    let mut contracts = UnorderedSet::new(StorageKey::AppContracts { app_id: self.next_app_id });
                    let contracts_vec = if let Some(contracts_vec) = app.contracts.clone() { contracts_vec } else { vec![] };
                    for contract in contracts_vec {
                        contracts.insert(&contract);
                    }
                    app.active = Some(true);
                    self.internal_insert_app(self.next_app_id, app, Some(added_by_account_id), categories, contracts);

                    self.next_app_id += 1;
                }
                Err(_) => {
                    if deposit.0 > 0 {
                        Promise::new(added_by_account_id).transfer(deposit.0);
                        log!("Deposit reverted");
                    }

                    log!("ERR_NOT_WEB4_APP");
                }
            }
        } else {
            log!("Promise failed. Sending request with data");
            self.assert_web4(app.dapp_account_id.clone(), app, deposit, added_by_account_id, true);
        }
    }
}

impl Contract {
    pub fn assert_guardian(&self) {
        require!(self.guardians.contains(&env::predecessor_account_id()), "ERR_NO_ACCESS");
    }

    pub fn assert_web4(&self, contract_id: AccountId, app: AppJSON, deposit: WrappedBalance, added_by_account_id: AccountId, send_request_with_data: bool) {
        let (request, ignore_promise_success) = if send_request_with_data {
            (
                Web4Request {
                    account_id: Some(contract_id.clone()),
                    path: "".to_string(),
                    params: Some(HashMap::new()),
                    query: Some(HashMap::new()),
                    preloads: Some(HashMap::new()),
                },
                true
            )
        } else {
            (
                Web4Request {
                    account_id: Some(contract_id.clone()),
                    path: "".to_string(),
                    params: None,
                    query: None,
                    preloads: None,
                },
                false
            )
        };

        let (get_gas, callback_gas) =
            if ignore_promise_success {
                (GAS_FOR_WEB4_GET_IGNORE_PROMISE_RESULT, GAS_FOR_ON_WEB4_GET_IGNORE_PROMISE_RESULT)
            } else {
                (GAS_FOR_WEB4_GET_CHECK_PROMISE_RESULT, GAS_FOR_ON_WEB4_GET_CHECK_PROMISE_RESULT)
            };

        ext_web4::ext(contract_id)
            .with_static_gas(get_gas)
            .web4_get(
                request
            )
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(callback_gas)
                    .after_web4_get(
                        app,
                        deposit,
                        added_by_account_id,
                        ignore_promise_success
                    )
            );
    }

    pub fn internal_get_apps(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<(AppId, App)> {
        unordered_map_pagination(&self.apps, from_index, limit)
    }

    pub fn internal_get_app_by_slug(&self, slug: &String) -> (AppId, VApp) {
        let app_id = self.app_id_by_slug.get(slug).expect("ERR_NO_SLUG");
        (app_id, self.apps.get(&app_id).expect("ERR_NO_APP"))
    }
}

pub(crate) fn unordered_map_pagination<K, VV, V>(
    m: &UnorderedMap<K, VV>,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> Vec<(K, V)>
    where
        K: BorshSerialize + BorshDeserialize,
        VV: BorshSerialize + BorshDeserialize,
        V: From<VV>,
{
    let keys = m.keys_as_vector();
    let values = m.values_as_vector();
    let from_index = from_index.unwrap_or(0);
    let limit = limit.unwrap_or(keys.len());
    (from_index..std::cmp::min(keys.len(), from_index + limit))
        .map(|index| (keys.get(index).unwrap(), values.get(index).unwrap().into()))
        .collect()
}

pub (crate) fn filter_slug(s: String) -> String {
    s.chars()
        .into_iter()
        .filter_map(|c| match c {
            | '_' |  '-' => Some(c),
            _ if c.is_alphanumeric() => Some(c),
            _ => None,
        })
        .collect()
}

pub (crate) fn filter_text(s: Option<String>) -> Option<String> {
    if let Some(s) = s {
        Some (
            s.chars()
            .into_iter()
            .filter_map(|c| match c {
                '\n' => Some(' '),
                ' ' | '_' | '.' | '-' | ',' | '!' | '(' | ')' | '/' | '=' | ':' | '+' | '?' | '#' | '%' | '|' => Some(c),
                _ if c.is_alphanumeric() => Some(c),
                _ => None,
            })
            .collect()
        )
    }
    else {
        None
    }
}

pub (crate) fn filter_html(s: Option<String>) -> Option<String> {
    if let Some(s) = s {
        Some (
    s.chars()
        .into_iter()
        .filter_map(|c| match c {
            '<' => Some("&lt;".to_string()),
            '>' => Some("&gt;".to_string()),
            '\n' => Some(('\n' as char).to_string()),
            ' ' | '_' | '.' | '-' | ',' | '!' | '(' | ')' | '/' | '=' | ':' | '+' | '?' | '#' | '%' | '|' | '\\' => Some((c as char).to_string()).clone(),
            _ if c.is_alphanumeric() => Some((c as char).to_string()).clone(),
            _ => None,
        })
        .collect()
        )
    }
    else {
        None
    }
}
