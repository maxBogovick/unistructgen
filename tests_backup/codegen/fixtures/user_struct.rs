#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: i64,
    #[validate(length(min = 1))]
    pub name: Option<String>,
}
