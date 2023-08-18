// TODO
// - Actx 
// - Frontend
// - CQRS

pub struct Order;
pub struct NewAddress;
pub struct PaymentMethod;

trait OrderingWriteService {
    fn ship(order_id: u64);
    fn change_order_shipment_address(order_id: u64, new_address: NewAddress);
    fn create_order(order: Order);
    fn change_order_payment_method(order_id: u64, payment_method: PaymentMethod);
}

trait OrderingReadService {
    fn get_order(order_id: u64) -> Order;
}

pub mod schema;
use schema::posts;


#[derive(Serialize)]
#[derive(Debug)]
#[derive(Queryable)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub publihed: bool,
}

#[derive(Serialize, Deserialize)]
#[derive(Insertable)]
#[derive(Clone)]
#[derive(Debug)]
#[table_name="posts"]
pub struct NewPost{
    pub title: String,
    pub body: String,
}

impl FromDataSimple for NewPost {
    type Error = String;
    
    #[allow(unused_variables)]
    fn from_data(req: &Request, data: Data) -> data::Outcome<Self, String> {
        let reader = data.open();
        match serde_json::from_reader(reader).map(|val| val) {
            Ok(value) => Success(value),
            Err(e) => Failure((Status::BadRequest, e.to_string())),
        }
    }
}

// DB classes
#[derive(Clone, Copy)]
pub struct DBPost {}

pub trait DBAdapter {
    fn create(& self, post: NewPost) -> Post;
    fn read(& self) -> Vec<Post>;
}


impl DBPost {
    // DATABASE classes
        pub fn establish_connection(self) -> PgConnection {
            dotenv().ok();
            let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
            PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
        }


      pub fn clear(self)  {
            let conn = self.establish_connection();
            let _ = diesel::delete(posts::table).execute(&conn);
      }
}

impl DBAdapter for DBPost {
        fn create(& self, post: NewPost) -> Post {
            let conn = self.establish_connection();
            diesel::insert_into(posts::table).values(&post).get_result(&conn).expect("Error saving!")
        }
        fn read(& self) -> Vec<Post> {
            let conn = self.establish_connection();
            posts::table.load::<Post>(&conn).unwrap()
      }
}

fn main() {
    println!("Hello, world!");
}
