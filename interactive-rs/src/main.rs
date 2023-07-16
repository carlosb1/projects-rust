use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

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

    pub fn select(&self, values: Vec<&str>) -> String {
        let mut filter = String::from("*");
        if !values.is_empty() {
            filter = values.join(", ");
        }
        return format!("SELECT {:} from {:}", filter, self.name);
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

    pub fn add(&self, values: HashMap<&str, DBTypes>) -> Result<String, &'static str> {
        let connection = sqlite::open(self.file_path.clone())
            .map_err(|_| "It was not possible to open the connection")
            .unwrap();
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

    pub fn list(&mut self) -> Result<HashMap<String, String>, &'static str> {
        let connection = sqlite::open(self.file_path.clone())
            .map_err(|_| "It was not possible to open the connection")
            .unwrap();
        let mut result: HashMap<String, String> = HashMap::new();
        connection
            .iterate(self.query_factory.select(Vec::new()), |pairs| {
                for &(name, value) in pairs.iter() {
                    result.insert(name.to_string(), value.unwrap().to_string());
                }
                true
            })
            .unwrap();
        Ok(result)
    }

    pub fn select_by(&mut self, filter: String) -> Result<HashMap<String, String>, &'static str> {
        let connection = sqlite::open(self.file_path.clone())
            .map_err(|_| "It was not possible to open the connection")
            .unwrap();
        let mut result: HashMap<String, String> = HashMap::new();
        connection
            .iterate(self.query_factory.select(vec![filter.as_str()]), |pairs| {
                for &(name, value) in pairs.iter() {
                    result.insert(name.to_string(), value.unwrap().to_string());
                }
                true
            })
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
        return HashMap::from([("link", dbtypes)]);
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
    /// does testing things
    Add {
        #[arg(short, long)]
        link: String,
        #[arg(short, long)]
        tags: Vec<String>,
    },
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
                println!("{:} -> {:}", key, value);
                if key == "id" {
                    let tags = tag_repo
                        .select_by("id_link".to_string())
                        .unwrap()
                        .values()
                        .map(|v| (*v).clone())
                        .collect::<Vec<String>>();
                    println!("{:?}", tags);
                }
            });
        }
    }
    Ok(())
}
