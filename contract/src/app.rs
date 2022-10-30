use crate::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VApp {
    Current(App),
}


#[derive(BorshSerialize, BorshDeserialize)]
pub struct App {
    pub added_by_account_id: AccountId,
    pub dapp_account_id: AccountId,

    pub slug: Slug,
    pub title: String,
    pub categories: UnorderedSet<CategoryId>,
    pub oneliner: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub twitter: Option<String>,
    pub facebook: Option<String>,
    pub medium: Option<String>,
    pub telegram: Option<String>,
    pub github: Option<String>,
    pub discord: Option<String>,
    pub symbol: Option<String>,
    pub contracts: UnorderedSet<AccountId>,
    pub token_address: Option<AccountId>,
    pub active: Option<bool>,
}

impl From<VApp> for App {
    fn from(v_app: VApp) -> Self {
        match v_app {
            VApp::Current(app) => app,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AppJSON {
    pub added_by_account_id: Option<AccountId>,
    pub dapp_account_id: AccountId,

    pub slug: Slug,
    pub title: String,
    pub categories: Vec<String>,
    pub oneliner: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub twitter: Option<String>,
    pub facebook: Option<String>,
    pub medium: Option<String>,
    pub telegram: Option<String>,
    pub github: Option<String>,
    pub discord: Option<String>,
    pub symbol: Option<String>,
    pub contracts: Option<Vec<AccountId>>,
    pub token_address: Option<AccountId>,
    pub active: Option<bool>,
}

impl From<VApp> for AppJSON {
    fn from(v_app: VApp) -> Self {
        match v_app {
            VApp::Current(app) => {
                let mut categories = vec![];
                for category in app.categories.to_vec() {
                    categories.push(category.to_string())
                }

                AppJSON {
                    added_by_account_id: Some(app.added_by_account_id),
                    dapp_account_id: app.dapp_account_id,
                    slug: app.slug,
                    title: app.title,
                    categories,
                    oneliner: app.oneliner,
                    description: app.description,
                    logo_url: app.logo_url,
                    twitter: app.twitter,
                    facebook: app.facebook,
                    medium: app.medium,
                    telegram: app.telegram,
                    github: app.github,
                    discord: app.discord,
                    symbol: app.symbol,
                    contracts: Some(app.contracts.to_vec()),
                    token_address: app.token_address,
                    active: app.active,
                }
            }
        }
    }
}


impl Contract {
    pub(crate) fn internal_insert_app(&mut self, app_id: AppId, app: AppJSON, added_by_account_id: Option<AccountId>,
                                      categories: UnorderedSet<CategoryId>, contracts: UnorderedSet<AccountId>) {
        self.app_id_by_slug.insert(&app.slug, &app_id);
        self.app_id_by_dapp_account_id.insert(&app.dapp_account_id, &app_id);

        let app = App {
            added_by_account_id: added_by_account_id.expect("ERR_MISSING_ADDED_BY_ACCOUNT_ID"),
            dapp_account_id: app.dapp_account_id,
            slug: app.slug.to_lowercase(),
            title: app.title,
            categories,
            oneliner: app.oneliner,
            description: app.description,
            logo_url: app.logo_url,
            twitter: app.twitter,
            facebook: app.facebook,
            medium: app.medium,
            telegram: app.telegram,
            github: app.github,
            discord: app.discord,
            symbol: app.symbol,
            contracts,
            token_address: app.token_address,
            active: app.active,
        };

        self.apps.insert(&app_id, &VApp::Current(app));
    }
}