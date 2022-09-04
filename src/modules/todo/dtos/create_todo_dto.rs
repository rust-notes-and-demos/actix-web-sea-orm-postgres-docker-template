use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct CreateTodoDto {
    #[validate(length(min = 1, max = 30))]
    pub title: String,
    #[validate(length(min = 1, max = 1000))]
    pub description: String,
    pub done: bool,
}