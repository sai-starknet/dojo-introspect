use dojo_introspect_types::ty::DojoMember;
use dojo_introspect_utils::selector::compute_selector_from_dojo_tag;
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub struct DojoManager<Store> {
    pub tables: HashMap<Felt, DojoTable>,
    pub store: Store,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DojoTable {
    pub name: String,
    pub fields: HashMap<Felt, DojoMember>,
    pub record_order: Vec<Felt>,
}

impl DojoTable {
    fn get_schema(&self) -> Vec<DojoMember> {
        self.record_order
            .iter()
            .filter_map(|selector| self.fields.get(selector).cloned())
            .collect()
    }
}

pub struct JsonStore {
    pub path: PathBuf,
}

impl JsonStore {
    pub fn new(path: &Path) -> Self {
        if !path.exists() {
            std::fs::create_dir_all(path).expect("Unable to create directory");
        }
        Self {
            path: path.to_path_buf(),
        }
    }
}

pub trait StoreTrait {
    type Table;
    fn dump(&self, table_id: Felt, data: &Self::Table) -> bool;
    fn load(&self, table_id: Felt) -> Option<Self::Table>;
}

fn felt_to_fixed_hex_string(felt: &Felt) -> String {
    format!("0x{:0>32x}", felt)
}
fn felt_to_json_file_name(felt: &Felt) -> String {
    format!("{}.json", felt_to_fixed_hex_string(felt))
}

impl StoreTrait for JsonStore {
    type Table = DojoTable;

    fn dump(&self, table_id: Felt, data: &Self::Table) -> bool {
        let file_path = self.path.join(felt_to_json_file_name(&table_id));
        std::fs::write(file_path, serde_json::to_string(data).unwrap())
            .expect("Unable to write file");
        true
    }

    fn load(&self, table_id: Felt) -> Option<Self::Table> {
        let file_path = self.path.join(felt_to_json_file_name(&table_id));
        let data = std::fs::read_to_string(file_path).expect("Unable to read file");
        serde_json::from_str(&data).ok()
    }
}

pub trait IntrospectManager {
    type Field;
    type Table;
    fn register_table(&mut self, id: Felt, name: &str, fields: Vec<Self::Field>) -> bool;
    fn update_table(&mut self, id: Felt, name: &str, fields: Vec<Self::Field>) -> bool;
    fn get_table(&self, id: Felt) -> Option<Self::Table>;
    fn get_table_schema(&self, id: Felt) -> Option<Vec<Self::Field>>;
    fn get_table_name(&self, table_id: Felt) -> Option<String>;
    fn get_table_field(&self, table_id: Felt, field_selector: Felt) -> Option<Self::Field>;
    fn get_table_fields(&self, table_id: Felt) -> Option<Vec<Self::Field>>;
}

impl<Store> IntrospectManager for DojoManager<Store>
where
    Store: StoreTrait<Table = DojoTable>,
{
    type Field = DojoMember;
    type Table = DojoTable;
    fn register_table(&mut self, id: Felt, name: &str, fields: Vec<Self::Field>) -> bool {
        if self.tables.contains_key(&id) {
            return false;
        }
        let mut field_map = HashMap::new();
        let mut record_order = Vec::new();
        for field in fields {
            let field_selector = compute_selector_from_dojo_tag(&field.name);
            record_order.push(field_selector);
            field_map.insert(field_selector, field);
        }
        let table = DojoTable {
            name: name.to_string(),
            fields: field_map,
            record_order,
        };
        self.store.dump(id, &table);
        self.tables.insert(id, table);

        true
    }

    fn update_table(&mut self, id: Felt, name: &str, fields: Vec<Self::Field>) -> bool {
        if !self.tables.contains_key(&id) {
            return false;
        }
        let table = match self.tables.get_mut(&id) {
            Some(t) => t,
            None => return false,
        };
        table.name = name.to_string();
        let mut record_order = Vec::new();
        for field in fields {
            let field_selector = compute_selector_from_dojo_tag(&field.name);
            record_order.push(field_selector);
            table.fields.insert(field_selector, field);
        }
        self.store.dump(id, &table);
        true
    }

    fn get_table(&self, id: Felt) -> Option<Self::Table> {
        self.tables.get(&id).map(|table| table.clone())
    }

    fn get_table_name(&self, table_id: Felt) -> Option<String> {
        self.tables.get(&table_id).map(|table| table.name.clone())
    }

    fn get_table_schema(&self, id: Felt) -> Option<Vec<Self::Field>> {
        self.tables.get(&id).map(|table| table.get_schema())
    }

    fn get_table_field(&self, table_id: Felt, field_selector: Felt) -> Option<Self::Field> {
        self.tables
            .get(&table_id)
            .and_then(|table| table.fields.get(&field_selector).cloned())
    }
    fn get_table_fields(&self, table_id: Felt) -> Option<Vec<Self::Field>> {
        self.tables
            .get(&table_id)
            .map(|table| table.fields.values().cloned().collect::<Vec<DojoMember>>())
    }
}
