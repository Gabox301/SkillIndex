#[derive(Debug, Clone)]
pub struct DisplayTechnology {
    pub id: String,
    pub name: String,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DisplayCombo {
    pub id: String,
    pub name: String,
}
