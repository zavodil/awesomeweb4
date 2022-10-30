use crate::*;

#[ext_contract(ext_web4)]
trait ExtWeb4Contract {
    fn web4_get(&self, request: Web4Request) -> Web4Response;
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn after_web4_get(&self, app: AppJSON, deposit: WrappedBalance, added_by_account_id: AccountId, ignore_promise_success: bool);
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Web4Request {
    #[serde(rename = "accountId")]
    pub(crate) account_id: Option<AccountId>,
    pub(crate) path: String,
    pub(crate) params: Option<HashMap<String, String>>,
    pub(crate) query: Option<HashMap<String, Vec<String>>>,
    pub(crate) preloads: Option<HashMap<String, Web4Response>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde", untagged)]
pub enum Web4Response {
    Body {
        #[serde(rename = "contentType")]
        content_type: String,
        body: near_sdk::json_types::Base64VecU8,
    },
    BodyUrl {
        #[serde(rename = "bodyUrl")]
        body_url: String,
    },
    PreloadUrls {
        #[serde(rename = "preloadUrls")]
        preload_urls: Vec<String>,
    },
    Status {
        status: u32,
    }
}

impl Web4Response {
    pub fn html_response(html: String) -> Self {
        Self::Body {
            content_type: String::from("text/html; charset=UTF-8"),
            body: html.as_bytes().to_owned().into(),
        }
    }

    pub fn plain_response(text: String) -> Self {
        Self::Body {
            content_type: String::from("text/plain; charset=UTF-8"),
            body: text.as_bytes().to_owned().into()
        }
    }

    pub fn svg_response(text: String) -> Self {
        Self::Body {
            content_type: String::from("image/svg+xml"),
            body: text.as_bytes().to_owned().into()
        }
    }

    pub fn preload_urls(urls: Vec<String>) -> Self {
        Self::PreloadUrls {
            preload_urls: urls
        }
    }

    pub fn body_url(url: String) -> Self {
        Self::BodyUrl {
            body_url: url
        }
    }

    pub fn status(status: u32) -> Self {
        Self::Status {
            status
        }
    }
}


#[near_bindgen]
impl Contract {
    #[allow(unused_variables)]
    pub fn web4_get(&self, request: Web4Request) -> Web4Response {
        let path = request.path;

        if path == "/robots.txt" {
            return Web4Response::plain_response("User-agent: *\nDisallow:".to_string());
        }

        if path == "/style.css" {
            return Web4Response::html_response(
                include_str!("../res/style.css").to_string()
            );
        }


        if path == "/no-image.svg" {
            return Web4Response::svg_response(
                include_str!("../res/no-image.svg").to_string()
            );
        }

        if path == "/submit"  && !request.query.unwrap_or_default().contains_key("transactionHashes") {
            let (show_login, form_visibility, user_account_id, categories_html) =
                if let Some(user_account_id) = request.account_id {
                    let mut categories_html = "".to_string();

                    for category_id in 0..self.next_category_id {
                        let category: Category = self.categories.get(&category_id).expect("ERR_NO_CATEGORY").into();
                        categories_html = format!(r#"{}<div><label class="form-checkbox"><input type="checkbox" value="{}" name="app.categories[]" /><i class="form-icon"></i> {}</label></div>"#, categories_html, category_id, category.title);
                    }

                (
                    "".to_string(),
                    "block".to_string(),
                    user_account_id.to_string(),
                    categories_html,
                )
            }
            else {
                (
                    format!(r#"<h2 class="hero-subtitle">Sign in with NEAR account to submit new app.</h2><div><a href="/web4/login?web4_contract_id={}" class="btn btn-primary">Sing in</a></div>"#, env::current_account_id()),
                    "none".to_string(),
                    "".to_string(),
                    "".to_string()
                )
            };

            return Web4Response::html_response(
                include_str!("../res/submit.html")
                    .replace("%CONTRACT_NAME%", &env::current_account_id().to_string())
                    .replace("%SHOW_LOGIN%",  &show_login)
                    .replace("%SHOW_FORM%", &form_visibility)
                    .replace("%USER_ACCOUNT_ID%", &user_account_id)
                    .replace("%CATEGORIES_CHECKBOXES%", &categories_html)

            );
        }

        if path.starts_with("/app/") {
            let slug = &path[5..]; // 5 = "/app/".len()
            let (app_id, v_app) = self.internal_get_app_by_slug(&slug.to_string());
            let app: App = v_app.into();

            let mut tags_html: String = "".to_string();
            for category_id in app.categories.to_vec() {
                let category_data: Category = self.categories.get(&category_id).expect("ERR_WRONG_CATEGORY").into();
                tags_html = format!("{}<a class=\"tag-item awesome-tag\" href=\"/category/{}\">{}</a>", tags_html, category_data.slug, category_data.title);
            }
            let category_html = format!("<div class=\"hero-tags\">{}</div>", &tags_html);

            let social_links = format!("{}{}{}{}{}{}",
                                       format_icon(app.twitter, "twitter", false),
                                       format_icon(app.facebook, "facebook", false),
                                       format_icon(app.medium, "medium", false),
                                       format_icon(app.telegram, "telegram", false),
                                       format_icon(app.github, "github", false),
                                       format_icon(app.discord, "discord", false));

            let mut image_url = app.logo_url.unwrap_or_default();
            if image_url.is_empty() {
                image_url = "/no-image.svg".to_string();
            }

            return Web4Response::html_response(
                include_str!("../res/app.html")
                    .replace("%APP_PAGE_TITLE%", &app.title)
                    .replace("%APP_PAGE_IMAGE%", &image_url)
                    .replace("%APP_PAGE_DAPP_CONTRACT%", &app.dapp_account_id.to_string())
                    .replace("%APP_PAGE_ONELINER%", &app.oneliner.unwrap_or_default())
                    .replace("%APP_PAGE_DESCRIPTION%", &app.description.unwrap_or_default().replace('\n', "</p><p>"))
                    .replace("%APP_PAGE_CATEGORIES%", &category_html)
                    .replace("%APP_PAGE_SOCIAL_LINKS%", &social_links)
                    .replace("%APP_PAGE_SLUG%", &app.slug)
                    .replace("%APP_PAGE_ADDED_BY%", &app.added_by_account_id.to_string())
                    .replace("%APP_PAGE_ID%", &app_id.to_string())

            );
        }

        // MAIN PAGE
        let mut active_category_id: Option<CategoryId> = None;
        let mut app_html: String = "".to_string();
        // APPS for a specific category
        if path.starts_with("/category/") {
            let slug = &path[10..]; // 10 = "/category/".len()
            for category_id in 0..self.next_category_id {
                let category: Category = self.categories.get(&category_id).expect("ERR_NO_CATEGORY").into();
                if category.slug == slug {
                    let app_ids = self.apps_ids_by_category_id.get(&category_id).expect("ERR_NO_CATEGORY");
                    for app_id in app_ids.as_vector().iter() {
                        let app: App = self.apps.get(&app_id).expect("ERR_NO_APP").into();
                        if app.active.unwrap_or(true) {
                            app_html = format!("{}{}", app_html, self.format_app(app));
                        }
                    }
                    active_category_id = Some(category_id);
                    break;
                }
            }
        } else { // ALL APPS
            for (app_id, app) in self.internal_get_apps(None, None) {
                if app.active.unwrap_or(true) {
                    app_html = format!("{}{}", app_html, self.format_app(app));
                }
            }
        }


        Web4Response::html_response(
            include_str!("../res/catalog.html")
                .replace("%APPLICATIONS%", &app_html)
                .replace("%CATEGORIES%", &self.format_categories_menu(active_category_id))
        )
    }
}

impl Contract {
    fn format_categories_menu(&self, active_category_id: Option<CategoryId>) -> String {
        let set_active_category = active_category_id.is_some();
        let active_category_id: CategoryId = active_category_id.unwrap_or_default();
        let mut categories_html = "".to_string();
        for (category_id, category) in self.get_categories(None, None) {
            let active_class = if set_active_category && category_id == active_category_id { " active" } else { "" };
            let apps_in_category = self.apps_ids_by_category_id.get(&category_id).expect("ERR_NO_CATEGORY").len();
            categories_html = format!(r#"{}<div><a class="menu-parent{}" href="/category/{}">{}<span class="menu-badge">{}</span></a></div>"#, categories_html, active_class, category.slug, category.title, apps_in_category);
        }
        categories_html = format!(r#"<div><a class="menu-parent" href="/">All<span class="menu-badge">{}</span></a></div>{}"#, (self.next_app_id - self.disabled_apps), categories_html);
        categories_html
    }

    fn format_app(&self, app: App) -> String {
        let mut tags_html: String = "".to_string();
        for category_id in app.categories.to_vec() {
            let category_data: Category = self.categories.get(&category_id).expect("ERR_WRONG_CATEGORY").into();
            tags_html = format!("{}<span>{}</span>", tags_html, category_data.title);
        }

        let mut image_url = app.logo_url.unwrap_or_default();
        if image_url.is_empty() {
            image_url = "/no-image.svg".to_string();
        }

        format!(r##"
<div class="column col-4 col-lg-6 col-sm-12">
    <div style="padding: 1rem" class="near-item mainnet">
        <a href="/app/{}">
            <div class="near-item-header">
                <div class="tile">
                    <div class="tile-icon"><img src="{}"></div>
                    <div class="tile-content">
                        <h2 class="tile-title">{}</h2>
                        <div class="tile-tags">{}</div>
                    </div>
                </div>
                <div class="tile"><h3 class="tile-subtitle">{}</h3></div>
            </div>
        </a>
        <div class="near-item-footer">
           <div class="tile-social">
                <a href="https://{}.page" target="_blank"><svg class="icon" height="20" width="20"><use xlink:href="#icon-website"></use></svg></a>
                {}
                {}
                {}
                {}
                {}
                {}
           </div>
           <div class="tile-series">
               <div class="label-series near">
                    <svg class="icon icon-series" height="20" width="20"><use xlink:href="#icon-near"></use></svg>
               </div>
           </div>
        </div>
    </div>
</div>"##,
                app.slug,
                image_url,
                app.title,
                tags_html,
                app.oneliner.unwrap_or_default(),
                app.dapp_account_id,
                format_icon(app.twitter, "twitter", true),
                format_icon(app.facebook, "facebook", true),
                format_icon(app.medium, "medium", true),
                format_icon(app.telegram, "telegram", true),
                format_icon(app.github, "github", true),
                format_icon(app.discord, "discord", true)
        )
    }
}

fn format_icon(value: Option<String>, icon: &str, homepage: bool) -> String {
    if let Some(value) = value {
        if !value.is_empty() {
            let link = match icon {
                "twitter" => format!("https://twitter.com/{}", value),
                "facebook" => format!("https://facebook.com/{}", value),
                "medium" => format!("https://medium.com/{}", value),
                "telegram" => format!("https://t.me/{}", value),
                "discord" => value,
                "github" => format!("https://github.com/{}", value),
                _ => "".to_string()
            };

            return if homepage {
                format!(r##"<a href="{}" target="_blank"><svg class="icon" height="20" width="20"><use xlink:href="#icon-{}"></use></svg></a>"##, link, icon)
            } else {
                format!(r##"<a href="{}" target="_blank" rel="noopener noreferrer" class="link-item btn btn-lg btn-link" title="{}"><svg class="icon icon-{}" height="20" width="20"><use xlink:href="#icon-{}"></use></svg></a>"##, link, icon, icon, icon)
            }
        }
    }

    "".to_string()
}