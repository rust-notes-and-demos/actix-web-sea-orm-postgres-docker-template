use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct UpdateTodoDto {
    #[validate(length(min = 1, max = 30))]
    pub title: Option<String>,
    #[validate(length(min = 1, max = 1000))]
    pub description: Option<String>,
    pub done: Option<bool>,
}