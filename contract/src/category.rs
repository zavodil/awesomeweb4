use crate::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VCategory {
    Current(Category),
}


#[derive(BorshSerialize, BorshDeserialize)]
pub struct Category {
    pub slug: String,
    pub title: String
}

impl From<VCategory> for Category {
    fn from(v_category: VCategory) -> Self {
        match v_category {
            VCategory::Current(category) => category,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CategoryJSON {
    pub slug: String,
    pub title: String
}

impl From<VCategory> for CategoryJSON {
    fn from(v_category: VCategory) -> Self {
        match v_category {
            VCategory::Current(category) => CategoryJSON {
                slug: category.slug,
                title: category.title
            }
        }
    }
}