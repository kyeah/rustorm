extern crate rustorm;

use rustorm::query::Query;
use rustorm::query::{Filter,Equality};
use rustorm::pool::ManagedPool;

fn main(){
    let url = "sqlite:///file.db";
    let mut pool = ManagedPool::init(&url, 1).unwrap();
    let db = pool.connect().unwrap();
        
        Query::insert()
            .set("name", &"Test Product")
        .into_table(&"product")
            .execute(db.as_ref());
}
