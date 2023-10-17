//! Asset Groups

pub use crate::grpc::server::asset_group::*;

use log::debug;
use std::collections::HashMap;
use tokio_postgres::row::Row;
use tokio_postgres::types::Type as PsqlFieldType;
use uuid::Uuid;

use super::base::simple_resource::*;
use super::base::{FieldDefinition, ResourceDefinition};
use crate::common::ArrErr;
use crate::grpc::{GrpcDataObjectType, GrpcField, GrpcFieldOption};

// Generate `From` trait implementations for GenericResource into and from Grpc defined Resource
crate::build_generic_resource_impl_from!();

// Generate grpc server implementations
crate::build_grpc_simple_resource_impl!(asset_group);

impl Resource for ResourceObject<Data> {
    fn get_definition() -> ResourceDefinition {
        ResourceDefinition {
            psql_table: String::from("asset_group"),
            psql_id_cols: vec![String::from("asset_group_id")],
            fields: HashMap::from([
                (
                    "name".to_string(),
                    FieldDefinition::new(PsqlFieldType::TEXT, true),
                ),
                (
                    "description".to_string(),
                    FieldDefinition::new(PsqlFieldType::TEXT, true),
                ),
                (
                    "default_vertiport_schedule".to_string(),
                    FieldDefinition::new(PsqlFieldType::TEXT, false),
                ),
                (
                    "default_aircraft_schedule".to_string(),
                    FieldDefinition::new(PsqlFieldType::TEXT, false),
                ),
                (
                    "created_at".to_string(),
                    FieldDefinition::new_internal(PsqlFieldType::TIMESTAMPTZ, true)
                        .set_default(String::from("CURRENT_TIMESTAMP")),
                ),
                (
                    "updated_at".to_string(),
                    FieldDefinition::new_internal(PsqlFieldType::TIMESTAMPTZ, true)
                        .set_default(String::from("CURRENT_TIMESTAMP")),
                ),
                (
                    "deleted_at".to_string(),
                    FieldDefinition::new_internal(PsqlFieldType::TIMESTAMPTZ, false),
                ),
            ]),
        }
    }
}

impl GrpcDataObjectType for Data {
    fn get_field_value(&self, key: &str) -> Result<GrpcField, ArrErr> {
        match key {
            "name" => Ok(GrpcField::String(self.name.clone())),
            "description" => Ok(GrpcField::String(self.description.clone())),
            "default_vertiport_schedule" => Ok(GrpcField::Option(GrpcFieldOption::String(
                self.default_vertiport_schedule.clone(),
            ))), // ::core::option::Option<::prost::alloc::string::String>,
            "default_aircraft_schedule" => Ok(GrpcField::Option(GrpcFieldOption::String(
                self.default_aircraft_schedule.clone(),
            ))), // ::core::option::Option<::prost::alloc::string::String>,
            _ => Err(ArrErr::Error(format!(
                "Invalid key specified [{}], no such field found",
                key
            ))),
        }
    }
}

#[cfg(not(tarpaulin_include))]
// no_coverage: Can not be tested in unittest until https://github.com/sfackler/rust-postgres/pull/979 has been merged
impl TryFrom<Row> for Data {
    type Error = ArrErr;

    fn try_from(row: Row) -> Result<Self, ArrErr> {
        debug!("(try_from) Converting Row to asset_group::Data: {:?}", row);
        Ok(Data {
            name: row.get("name"),
            description: row.get("description"),
            default_vertiport_schedule: row.get("default_vertiport_schedule"),
            default_aircraft_schedule: row.get("default_aircraft_schedule"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, init_logger, test_util::*};

    #[test]
    fn test_asset_group_schema() {
        init_logger(&Config::try_from_env().unwrap_or_default());
        unit_test_info!("(test_asset_group_schema) start");

        let id = Uuid::new_v4().to_string();
        let data = mock::get_data_obj();
        let object: ResourceObject<Data> = Object {
            id,
            data: Some(data.clone()),
        }
        .into();
        test_schema::<ResourceObject<Data>, Data>(object);

        let result = validate::<ResourceObject<Data>>(&data);
        assert!(result.is_ok());
        if let Ok((sql_fields, validation_result)) = result {
            unit_test_info!("{:?}", sql_fields);
            unit_test_info!("{:?}", validation_result);
            assert_eq!(validation_result.success, true);
        }
        unit_test_info!("(test_asset_group_schema) success");
    }

    #[test]
    fn test_asset_group_invalid_data() {
        init_logger(&Config::try_from_env().unwrap_or_default());
        unit_test_info!("(test_asset group_invalid_data) start");

        let data = Data {
            name: String::from(""),
            description: String::from(""),
            default_vertiport_schedule: Some(String::from("")),
            default_aircraft_schedule: Some(String::from("")),
        };

        let result = validate::<ResourceObject<Data>>(&data);
        assert!(result.is_ok());
        if let Ok((_, validation_result)) = result {
            unit_test_info!("{:?}", validation_result);
            assert_eq!(validation_result.success, false);

            let expected_errors = vec!["parent_group_id"];
            assert_eq!(expected_errors.len(), validation_result.errors.len());
            assert!(contains_field_errors(&validation_result, &expected_errors));
        }
        unit_test_info!("(test_group_invalid_data) success");
    }
}