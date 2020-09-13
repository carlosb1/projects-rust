#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct News {
    id: String,
    link: String,
    title: String,
    descrip: String,
    data_ml: Array,
}
impl News {
    fn new(id: &str, link: &str, title: &str, descrip: &str, data_ml: Array) -> News {
        News {
            id: id.to_string(),
            link: link.to_string(),
            title: title.to_string(),
            descrip: descrip.to_string(),
            data_ml: data_ml,
        }
    }
}
