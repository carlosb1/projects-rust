use rusoto_core::Region;
use futures::{Stream, Future};
use tokio::runtime::Runtime;
use std::collections::HashMap;

use dynomite::{
    dynamodb::{
        AttributeValue, DynamoDb, DynamoDbClient, GetItemInput, PutItemInput, ScanInput,
    },
    retry::{Policy,RetryingDynamoDb},
    DynamoDbExt, FromAttributes, Item, Retries,
};


#[derive(Deserialize, Serialize)]
#[derive(Item, Debug, Clone)]
pub struct Test {
    id: String,
    param1: String,
}

impl Test  {
    pub fn new(id: String, param1: String) -> Test {
        Test{id: id, param1: param1}
    }
}

#[derive(Clone)]
pub struct DummyClientDB {
    name_id: String,
    name_table: String,
    client: RetryingDynamoDb<DynamoDbClient>,
}

 impl DummyClientDB {
    pub fn new(name_id: String, name_table: String) -> DummyClientDB {
        let client = DynamoDbClient::new(Region::UsEast1).with_retries(Policy::default());
        DummyClientDB {name_id: name_id, name_table: name_table, client: client}
    }
    
    pub fn list(mut self) -> Vec<Test> {
        let mut rt = Runtime::new().expect("failed to initialize futures runtime"); 
        let scanInput = ScanInput {limit: Some(1), table_name: self.name_table, ..ScanInput::default()};
        let values = rt.block_on(self.client.clone().scan_pages(scanInput).map(|item| { Test::from_attrs(item) }).collect());
    
        let result_values = match values {
            Ok(found_values) => { 
                found_values.into_iter().filter_map(Result::ok).collect()
        },
            Err(e) => {
                println!("{:#?}",e);
                Vec::new()
            },
        };
        result_values
    }
    pub fn put(mut self, test: &Test) -> bool {
        let mut rt = Runtime::new().expect("failed to initialize futures runtime"); 
        let putItemInput = PutItemInput{table_name: self.name_table, item: test.clone().into(), ..PutItemInput::default()};
        let values = rt.block_on(self.client.put_item(putItemInput));
        let result = match values {
            Ok(val) => {
               true
            }
            Err(e) => { 
                println!("{:#?}",e);
              false }
        };
         result
    }
    pub fn get(mut self, id: &str) -> Option<Test>{
        let mut rt = Runtime::new().expect("failed to initialize futures runtime"); 
        let mut query_key: HashMap<String, AttributeValue> = HashMap::new();
        query_key.insert(String::from(self.name_id), AttributeValue{s: Some(id.to_string()), ..Default::default()});

    
        let getItemInput = GetItemInput{table_name: self.name_table, key: query_key.clone(), ..GetItemInput::default()};
        let value = rt.block_on(self.client.get_item(getItemInput).map(|result| result.item.map(Test::from_attrs)));
    
        let result = match value {
            Ok(val) => {
                let tmp_val = val.unwrap();
                tmp_val.ok()
            },
            Err(e) => { 
                println!("{:#?}",e);
                None }
        };
        result
    }
}
