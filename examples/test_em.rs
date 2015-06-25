extern crate rustorm;
extern crate uuid;
extern crate chrono;
extern crate rustc_serialize;


use rustorm::db::postgres::Postgres;
use rustorm::codegen;
use uuid::Uuid;
use chrono::datetime::DateTime;
use chrono::offset::utc::UTC;
use rustc_serialize::json;

use gen::bazaar::Product; 
use rustorm::em::EntityManager;
use rustorm::table::IsTable;
use rustorm::dao::IsDao;

mod gen;


fn main(){
    let pg:Result<Postgres,&str> = Postgres::new("postgres://postgres:p0stgr3s@localhost/bazaar_v6");
       match pg{
        Ok(pg) => {
            let em = EntityManager::new(&pg);
            let daos = em.get_all(&Product::table());
            for d in daos{
                let prod = Product::from_dao(&d);
                let pid:Uuid = d.get("product_id");
                let name:String = d.get("name");
                println!("\n{} {}", pid, name);
                println!("{}-{}", prod.product_id, prod.name.unwrap());
            }
        }
        Err(error) =>{
            println!("{}",error);
        }
    }
}