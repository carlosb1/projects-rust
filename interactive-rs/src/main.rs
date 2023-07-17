use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub enum DBTypes {
    Number(u32),
    Text(String),
}

impl Display for DBTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DBTypes::Number(number) => write!(f, "{}", number),
            DBTypes::Text(text) => write!(f, "'{}'", text),
        }
    }
}

struct FactoryTableQuery {
    name: String,
}

impl FactoryTableQuery {
    pub fn new(name: &str) -> Self {
        FactoryTableQuery {
            name: name.to_string(),
        }
    }

    pub fn create(&self, pair_values: Vec<(&str, &str)>) -> String {
        let query: Vec<String> = pair_values
            .iter()
            .map(|(name, typ)| format!("{:} {:}", name, typ))
            .collect();
        return format!("CREATE TABLE {:} ({:});", self.name, query.join(", "));
    }

    pub fn insert(&self, values: HashMap<&str, DBTypes>) -> String {
        let keys: String = values.keys().map(|s| *s).collect::<Vec<&str>>().join(",");
        let vals = values
            .values()
            .map(|s| format!("{}", s))
            .collect::<Vec<String>>()
            .join(",");

        return format!("INSERT INTO {:} ({:}) values ({:});", self.name, keys, vals);
    }

    pub fn select(&self, filter: Vec<&str>, condition: Vec<(&str, &str)>) -> String {
        let mut query_filter = String::from("*");
        if !filter.is_empty() {
            query_filter = filter.join(", ");
        }
        let mut query_condition = String::new();
        if !condition.is_empty() {
            query_condition = String::from("where ");
            query_condition.push_str(
                condition
                    .iter()
                    .map(|(name, typ)| format!("{:} = {:}", name, typ))
                    .collect::<Vec<String>>()
                    .join(" and ")
                    .as_str(),
            );
        }
        return format!(
            "SELECT {:} from {:} {:}",
            query_filter, self.name, query_condition
        );
    }

    pub fn new_inserted_id(&self) -> String {
        return format!("SELECT last_insert_rowid() as id");
    }

    pub fn delete(&self, values: Vec<(&str, &str)>) -> String {
        let query: String = values
            .iter()
            .map(|(name, typ)| format!("{:} = {:}", name, typ))
            .collect::<Vec<String>>()
            .join(" and ");
        return format!("DELETE from {:} where {:}", self.name, query);
    }

    pub fn drop(&self) -> String {
        return format!("DROP TABLE {:}", self.name);
    }
}

struct DataRepository {
    file_path: String,
    query_factory: FactoryTableQuery,
}
impl DataRepository {
    pub fn new(file_path: String, name_table: String) -> DataRepository {
        DataRepository {
            file_path,
            query_factory: FactoryTableQuery::new(name_table.as_str()),
        }
    }

    pub fn create(&self, pair_values: Vec<(&str, &str)>) {
        let connection = sqlite::open(self.file_path.clone())
            .map_err(|_| "It was not possible to open the connection")
            .unwrap();
        let _ = connection.execute(self.query_factory.create(pair_values));
    }

    pub fn add(&self, values: HashMap<&str, DBTypes>) -> Result<String, &'static str> {
        let connection = sqlite::open(self.file_path.clone())
            .map_err(|_| "It was not possible to open the connection")
            .unwrap();
        println!("{:}", self.query_factory.insert(values.clone()));
        let _ = connection
            .execute(self.query_factory.insert(values))
            .map_err(|_| "It was not possible insert the values")
            .unwrap();

        let mut id = String::new();
        let _ = connection.iterate(self.query_factory.new_inserted_id(), |pairs| {
            id = pairs.iter().next().unwrap().1.unwrap().to_string();
            true
        });
        Ok(id)
    }

    pub fn list(&mut self) -> Result<HashMap<String, HashMap<String, String>>, &'static str> {
        let connection = sqlite::open(self.file_path.clone())
            .map_err(|_| "It was not possible to open the connection")
            .unwrap();
        let mut result: HashMap<String, HashMap<String, String>> = HashMap::new();
        connection
            .iterate(self.query_factory.select(Vec::new(), Vec::new()), |pairs| {
                let mut new_value: HashMap<String, String> = HashMap::new();
                let mut id = String::new();
                for &(name, value) in pairs.iter() {
                    if name == "id" {
                        id = value.unwrap().to_string();
                    }
                    new_value.insert(name.to_string(), value.unwrap().to_string());
                }
                result.insert(id, new_value);
                true
            })
            .unwrap();
        Ok(result)
    }

    pub fn select_by(
        &mut self,
        filter: Vec<&str>,
        condition: Vec<(&str, &str)>,
    ) -> Result<HashMap<String, HashMap<String, String>>, &'static str> {
        let connection = sqlite::open(self.file_path.clone())
            .map_err(|_| "It was not possible to open the connection")
            .unwrap();

        let mut filter_with_id = filter.clone();
        filter_with_id.insert(0, "id");
        let mut result: HashMap<String, HashMap<String, String>> = HashMap::new();
        connection
            .iterate(
                self.query_factory.select(filter_with_id, condition),
                |pairs| {
                    let mut new_value: HashMap<String, String> = HashMap::new();
                    let mut id = String::new();
                    for &(name, value) in pairs.iter() {
                        if name == "id" {
                            id = value.unwrap().to_string();
                        }
                        new_value.insert(name.to_string(), value.unwrap().to_string());
                    }
                    result.insert(id, new_value);
                    true
                },
            )
            .unwrap();
        Ok(result)
    }

    pub fn remove(&mut self, values: Vec<(&str, &str)>) {
        let connection = sqlite::open(self.file_path.clone())
            .map_err(|e| "It was not possible to open the connection")
            .unwrap();
        let _ = connection.execute(self.query_factory.delete(values));
    }
}

struct Link {
    link: String,
}

impl Link {
    pub fn new(link: String) -> Self {
        Link { link }
    }

    pub fn hashmap<'a>(&self) -> HashMap<&'a str, DBTypes> {
        let dbtypes = DBTypes::Text(self.link.clone());
        return HashMap::from([("url", dbtypes)]);
    }
}

struct Tag {
    id_link: String,
    tag: String,
}

impl Tag {
    pub fn new(id_link: String, tag: String) -> Self {
        Tag { id_link, tag }
    }
    pub fn hashmap<'a>(&self) -> HashMap<&'a str, DBTypes> {
        return HashMap::from([
            ("id_link", DBTypes::Text(self.id_link.clone())),
            ("tag", DBTypes::Text(self.tag.clone())),
        ]);
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add new link with its tags
    Add {
        #[arg(short, long)]
        link: String,
        #[arg(short, long)]
        tags: Vec<String>,
    },
    /// Delete new link
    Delete {
        #[arg(short, long)]
        id_link: String,
    },
    List,
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let mut links_repo = DataRepository::new("data.db".to_string(), "links".to_string());
    let mut tag_repo = DataRepository::new("data.db".to_string(), "tags".to_string());

    let query_create_table_links = vec![("id", "INTEGER PRIMARY KEY"), ("url", "TEXT")];
    let query_create_table_tags = vec![
        ("id", "INTEGER PRIMARY KEY"),
        ("id_link", "INTEGER"),
        ("tag", "TEXT"),
    ];
    links_repo.create(query_create_table_links);
    tag_repo.create(query_create_table_tags);

    match args.command {
        Commands::Add { link, tags } => {
            let id: String = links_repo
                .add(Link::new(link).hashmap())
                .map_err(|_| "It was not possible insert a link")
                .unwrap();

            tags.iter().for_each(|tag| {
                let _ = tag_repo.add(Tag::new(id.clone(), (*tag).clone()).hashmap());
            });
        }
        Commands::Delete { id_link } => {
            let query = vec![("id", id_link.as_str())];
            links_repo.remove(query);
            let query_tag = vec![("id_link", id_link.as_str())];
            tag_repo.remove(query_tag);
        }
        Commands::List => {
            let values = links_repo.list()?;
            values.iter().for_each(|(key, value)| {
                println!("{:} -> {:?}", key, value);
                let tags = tag_repo
                    .select_by(Vec::from(["tag"]), Vec::from([("id_link", key.as_str())]))
                    .unwrap()
                    .values()
                    .map(|v| {
                        println!("!!!{:?}", v);
                        (*(*v).get("tag").unwrap()).clone()
                    })
                    .collect::<Vec<String>>();
                println!("{:?}", tags);
            });
        }
    }
    Ok(())
}
