use query::Query;
use dao::Dao;

use dao::Value;
use database::Database;
use writer::SqlFrag;
use database::SqlOption;

use mysql::value::Value as MyValue;
use mysql::error::MyResult;
use mysql::conn::Stmt;
use mysql::conn::pool::MyPool;

use table::Table;
use database::DatabaseDDL;
use database::DbError;

pub struct Mysql {
    pool: Option<MyPool>,
}

impl Mysql{
    pub fn new() -> Self {
        Mysql { pool: None }
    }

    pub fn with_pooled_connection(pool: MyPool) -> Self {
        Mysql { pool: Some(pool) }
    }

    fn from_rust_type_tosql(types: &[Value]) -> Vec<MyValue> {
        let mut params: Vec<MyValue> = vec![];
        for t in types {
            match t {
                &Value::String(ref x) => {
                    params.push(MyValue::Bytes(x.as_bytes().to_owned()));
                }
                _ => panic!("not yet here {:?}", t),
            }
        }
        params
    }

    /// convert a record of a row into rust type
    fn from_sql_to_rust_type(row: &[MyValue], index: usize) -> Value {
        let value = row.get(index);
        match value {
            Some(value) => Value::String(value.into_str()),
            None => Value::Null,
        }
    }

    ///
    /// convert rust data type names to database data type names
    /// will be used in generating SQL for table creation
    /// FIXME, need to restore the exact data type as before
    fn rust_type_to_dbtype(&self, rust_type: &str) -> String {
        match rust_type {
            "bool" => {
                "boolean".to_owned()
            }
            "i8" => {
                "integer".to_owned()
            }
            "i16" => {
                "integer".to_owned()
            }
            "i32" => {
                "integer".to_owned()
            }
            "u32" => {
                "integer".to_owned()
            }
            "i64" => {
                "integer".to_owned()
            }
            "f32" => {
                "real".to_owned()
            }
            "f64" => {
                "real".to_owned()
            }
            "String" => {
                "text".to_owned()
            }
            "Vec<u8>" => {
                "blob".to_owned()
            }
            "Json" => {
                "text".to_owned()
            }
            "Uuid" => {
                "varchar(36)".to_owned()
            }
            "NaiveDateTime" => {
                "numeric".to_owned()
            }
            "DateTime<UTC>" => {
                "numeric".to_owned()
            }
            "NaiveDate" => {
                "numeric".to_owned()
            }
            "NaiveTime" => {
                "numeric".to_owned()
            }
            "HashMap<String, Option<String>>" => {
                "text".to_owned()
            }
            _ => panic!("Unable to get the equivalent database data type for {}",
                        rust_type),
        }
    }

    fn get_prepared_statement<'a>(&'a self, sql: &'a str) -> MyResult<Stmt> {
        self.pool.as_ref().unwrap().prepare(sql)
    }
}

impl Database for Mysql {
    fn version(&self) -> Result<String, DbError> {
        let sql = "SELECT version()";
        let dao = try!(self.execute_sql_with_one_return(sql, &vec![]));
        match dao {
            Some(dao) => Ok(dao.get("version")),
            None => Err(DbError::new("Unable to get database version")),
        }
    }

    fn begin(&self) {
        unimplemented!()
    }
    fn commit(&self) {
        unimplemented!()
    }
    fn rollback(&self) {
        unimplemented!()
    }
    fn is_transacted(&self) -> bool {
        false
    }
    fn is_closed(&self) -> bool {
        false
    }
    fn is_connected(&self) -> bool {
        false
    }
    fn close(&self) {
    }
    fn is_valid(&self) -> bool {
        false
    }
    fn reset(&self) {
        unimplemented!()
    }

    /// return this list of options, supported features in the database
    fn sql_options(&self) -> Vec<SqlOption> {
        vec![
            SqlOption::UsesQuestionMark,//mysql uses question mark instead of the numbered params
        ]
    }

    fn update(&self, _query: &Query) -> Dao {
        unimplemented!()
    }
    fn delete(&self, _query: &Query) -> Result<usize, String> {
        unimplemented!()
    }

    fn execute_sql_with_return(&self, sql: &str, params: &[Value]) -> Result<Vec<Dao>, DbError> {
        assert!(self.pool.is_some());
        let mut stmt = try!(self.get_prepared_statement(sql));
        let mut columns = vec![];
        for col in stmt.columns_ref().unwrap() {
            let column_name = String::from_utf8(col.name.clone()).unwrap();
            columns.push(column_name);
        }
        let mut daos = vec![];
        let param = Mysql::from_rust_type_tosql(params);
        let rows = try!(stmt.execute(&param));
        for row in rows {
            let row = try!(row);
            let mut index = 0;
            let mut dao = Dao::new();
            for col in &columns {
                let rtype = Mysql::from_sql_to_rust_type(&row, index);
                dao.set_value(col, rtype);
                index += 1;
            }
            daos.push(dao);
        }
        Ok(daos)
    }

    fn execute_sql_with_one_return(&self,
                                   sql: &str,
                                   params: &[Value])
                                   -> Result<Option<Dao>, DbError> {
        let dao = try!(self.execute_sql_with_return(sql, params));
        if dao.len() >= 1 {
            Ok(Some(dao[0].clone()))
        } else {
            Ok(None)
        }
    }

    /// generic execute sql which returns not much information,
    /// returns only the number of affected records or errors
    /// can be used with DDL operations (CREATE, DELETE, ALTER, DROP)
    fn execute_sql(&self, sql: &str, params: &[Value]) -> Result<usize, DbError> {
        let to_sql_types = Mysql::from_rust_type_tosql(params);
        assert!(self.pool.is_some());
        let result = try!(self.pool.as_ref().unwrap().prep_exec(sql, &to_sql_types));
        Ok(result.affected_rows() as usize)
    }

}

impl DatabaseDDL for Mysql{
    fn create_schema(&self, _schema: &str) {
        unimplemented!()
    }

    fn drop_schema(&self, _schema: &str) {
        unimplemented!()
    }

    fn build_create_table(&self, table: &Table) -> SqlFrag {
        let mut w = SqlFrag::new(self.sql_options());
        w.append("CREATE TABLE ");
        w.append(&table.name);
        w.append("(");
        w.ln_tab();
        let mut do_comma = false;
        for c in &table.columns {
            if do_comma {
                w.commasp();
            } else {
                do_comma = true;
            }
            w.append(&c.name);
            w.append(" ");
            let dt = self.rust_type_to_dbtype(&c.data_type);
            w.append(&dt);
            if c.is_primary {
                w.append(" PRIMARY KEY ");
            }
        }
        w.append(")");
        w
    }
    fn create_table(&self, table: &Table) -> Result<(), DbError> {
        let frag = self.build_create_table(table);
        let _ = try!(self.execute_sql(&frag.sql, &vec![]));
        Ok(())
    }

    fn rename_table(&self, _table: &Table, _new_tablename: String) {
        unimplemented!()
    }

    fn drop_table(&self, _table: &Table) {
        unimplemented!()
    }

    fn set_foreign_constraint(&self, _model: &Table) {
        unimplemented!()
    }

    fn set_primary_constraint(&self, _model: &Table) {
        unimplemented!()
    }
}


// TODO: need to implement trait DatabaseDev for Mysql
// Mysql can be used as development database
