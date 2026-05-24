mod database;

use anyhow;

use crate::database::storage::init_storage;


fn main() -> anyhow::Result<()> {
    let db_path = "db.json";
    init_storage(db_path)?;
    
    loop {

    }
}
