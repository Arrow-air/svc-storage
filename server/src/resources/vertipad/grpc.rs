//! Vertipads

mod grpc_server {
    #![allow(unused_qualifications, missing_docs)]
    tonic::include_proto!("grpc.vertipad");
}

use super::VertipadPsql;
use crate::get_psql_pool;
use crate::grpc::*;
use crate::grpc_error;
use crate::memdb::MEMDB_LOG_TARGET;
use crate::memdb::VERTIPADS;
use crate::memdb_info;
use uuid::Uuid;

pub use grpc_server::vertipad_rpc_server::{VertipadRpc, VertipadRpcServer};
pub use grpc_server::{UpdateVertipad, Vertipad, VertipadData, Vertipads};

use postgres_types::ToSql;
use std::collections::HashMap;
use tonic::{Code, Request, Response, Status};

///Implementation of gRPC endpoints
#[derive(Debug, Default, Copy, Clone)]
pub struct VertipadImpl {}

#[tonic::async_trait]
impl VertipadRpc for VertipadImpl {
    /// Takes an UUID string (vertipad_id) to get the matching database record.
    ///
    /// # Arguments
    /// * self - The VertipadRpc struct
    /// * request - Request<Id> GRPC request
    ///
    /// # Returns
    /// * Result Vertipad object or Status `not_found` if there was no vertipad found for the given Id
    ///
    /// # Example
    /// ```
    /// use svc_storage_client_grpc::client::vertipad_rpc_client::VertipadRpcClient;
    /// use svc_storage_client_grpc::client::Id;
    ///
    /// let mut vertipad_client = VertipadRpcClient::connect(grpc_endpoint.clone()).await.unwrap();
    /// let result = match vertipad_client.vertipad_by_id(tonic::Request::new(Id {
    ///     id: "54acfe06-dd9b-42e8-8cb4-12a2fb2fa693"
    /// }))
    /// .await
    /// {
    ///    Ok(vertipad) => vertipad.into_inner(),
    ///    Err(e) => panic!("Something went wrong retrieving the vertipad: {}", e),
    /// };
    /// ```
    async fn vertipad_by_id(&self, request: Request<Id>) -> Result<Response<Vertipad>, Status> {
        let id = request.into_inner().id;
        if let Some(vertipad) = VERTIPADS.lock().await.get(&id) {
            memdb_info!("Found entry for Vertipad. uuid: {}", id);
            Ok(Response::new(vertipad.clone()))
        } else {
            let pool = get_psql_pool();
            let data = VertipadPsql::new(&pool, Uuid::try_parse(&id).unwrap()).await;
            match data {
                Ok(vertipad) => Ok(Response::new(Vertipad {
                    id,
                    data: Some(vertipad.into()),
                })),
                Err(_) => Err(Status::not_found("Not found")),
            }
        }
    }

    /// Takes a `SearchFilter` object to search the database with the provided values.
    ///
    /// This method supports paged results. When the `search_field` and `search_value` are empty, no filters will be applied.
    ///
    /// # Arguments
    /// * self  - The VertipadRpc struct
    /// * request - Request<SearchFilter> GRPC request
    ///
    /// # Returns
    /// * Result Vertipads object or Status `Code::Internal` if there was an error in the search query
    ///
    /// # Example
    /// ```
    /// use svc_storage_client_grpc::client::vertipad_rpc_client::VertipadRpcClient;
    /// use svc_storage_client_grpc::client::SearchFilter;
    ///
    /// let mut vertipad_client = VertipadRpcClient::connect(grpc_endpoint.clone()).await.unwrap();
    /// let result = match vertipad_client.vertipads(tonic::Request::new(SearchFilter {
    ///     search_field: "auth_method",
    ///     search_value: "1",
    ///     page_number: 1,
    ///     results_per_page: 20
    /// }))
    /// .await
    /// {
    ///    Ok(vertipads) => vertipad.into_inner(),
    ///    Err(e) => panic!("Something went wrong creating retrieving vertipads: {}", e),
    /// };
    /// ```
    async fn vertipads(
        &self,
        request: Request<SearchFilter>,
    ) -> Result<Response<Vertipads>, Status> {
        let filter = request.into_inner();
        let mut filter_hash = HashMap::<String, String>::new();
        filter_hash.insert("column".to_string(), filter.search_field);
        filter_hash.insert("value".to_string(), filter.search_value);

        let pool = get_psql_pool();
        match super::psql::search(&pool, &filter_hash).await {
            Ok(vertipads) => Ok(Response::new(vertipads.into())),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    /// Takes an `VertipadData` object to create a new vertipad with the provided data.
    ///
    /// A new UUID will be generated by the database and returned as `id` as part of the returned `Vertipad` object.
    ///
    /// # Arguments
    /// * self - The VertipadRpc struct
    /// * request - Request<VertipadData> GRPC request
    ///
    /// # Returns
    /// Result Vertipad object or Status `Code::Internal` if the vertipad could not be inserted
    ///
    /// # Example
    /// ```
    /// use svc_storage_client_grpc::client::vertipad_rpc_client::{VertipadRpcClient, VertipadData};
    /// const CAL_WORKDAYS_8AM_6PM: &str = "\
    /// DTSTART:20221020T180000Z;DURATION:PT14H
    /// RRULE:FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR
    /// DTSTART:20221022T000000Z;DURATION:PT24H
    /// RRULE:FREQ=WEEKLY;BYDAY=SA,SU";
    ///
    /// let mut vertipad_client = VertipadRpcClient::connect(grpc_endpoint.clone()).await.unwrap();
    /// let result = match vertipad_client.insert_vertipad(tonic::Request::new(VertipadData {
    ///     vertiport_id: 0f27d530-b748-4cbf-be4d-2d2b1ac5bf4e,
    ///     description: format!("First vertipad for San Francisco"),
    ///     latitude: 37.7749,
    ///     longitude: -122.4194,
    ///     enabled: true,
    ///     occupied: false,
    ///     schedule: Some(CAL_WORKDAYS_8AM_6PM.to_string()),
    /// }))
    /// .await
    /// {
    ///    Ok(vertipad) => vertipad.into_inner(),
    ///    Err(e) => panic!("Something went wrong creating the vertipad: {}", e),
    /// };
    /// ```
    async fn insert_vertipad(
        &self,
        request: Request<VertipadData>,
    ) -> Result<Response<Vertipad>, Status> {
        let mut vertipads = VERTIPADS.lock().await;
        let data = request.into_inner();
        let pool = get_psql_pool();

        let mut vertipad_data = HashMap::<&str, &(dyn ToSql + Sync)>::new();

        let vertiport_id = match Uuid::try_parse(&data.vertiport_id) {
            Ok(id) => id,
            Err(e) => {
                grpc_error!("Could not convert [vertiport_id] to UUID: {}", e);
                return Err(Status::new(Code::Internal, e.to_string()));
            }
        };
        vertipad_data.insert("vertiport_id", &vertiport_id);
        vertipad_data.insert("description", &data.description);
        vertipad_data.insert("longitude", &data.longitude);
        vertipad_data.insert("latitude", &data.latitude);
        vertipad_data.insert("enabled", &data.enabled);
        vertipad_data.insert("occupied", &data.occupied);

        match super::psql::create(&pool, vertipad_data.clone()).await {
            Ok(vertipad) => {
                let id = vertipad.id.to_string();
                let vertipad = Vertipad {
                    id: id.clone(),
                    data: Some(data.clone()),
                };
                vertipads.insert(id, vertipad.clone());
                Ok(Response::new(vertipad))
            }
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    /// Takes an `UpdateVertipad` object to store new data to the database
    ///
    /// A field mask can be provided to restrict updates to specific fields.
    /// Returns the updated `Vertipad` on success.
    ///
    /// # Arguments
    /// * self  - The VertipadRpc struct
    /// * request - Request<UpdateVertipad> GRPC request
    ///
    /// # Returns
    /// * Result Vertipad object or:
    ///   - Status `not_found` if there was no vertipad found for the given Id
    ///   - Status `cancelled` if no data was provided in the UpdateVertipad object
    ///   - Status `Code::Internal` if the vertipad could not be updated
    ///
    /// # Example
    /// ```
    /// use svc_storage_client_grpc::client::vertipad_rpc_client::{VertipadRpcClient, UpdateVertipad};
    ///
    /// let mut vertipad_client = VertipadRpcClient::connect(grpc_endpoint.clone()).await.unwrap();
    /// let updated_vertipad = match vertipad_client.update_vertipad(tonic::Request::new(UpdateVertipad {
    ///     id: "54acfe06-dd9b-42e8-8cb4-12a2fb2fa693"
    ///     data: Some(VertipadData {
    ///         first_name: "John",
    ///         last_name: "Doe",
    ///     }),
    ///     mask: Some(FieldMask {
    ///         paths: vec!["first_name".to_string()]
    ///     })
    /// }))
    /// .await
    /// {
    ///    Ok(vertipad) => vertipad.into_inner(),
    ///    Err(e) => panic!("Something went wrong updating the vertipad info: {}", e),
    /// };
    /// ```
    async fn update_vertipad(
        &self,
        request: Request<UpdateVertipad>,
    ) -> Result<Response<Vertipad>, Status> {
        let vertipad_req = request.into_inner();
        let id = match Uuid::try_parse(&vertipad_req.id) {
            Ok(id) => id,
            Err(_e) => Uuid::new_v4(),
        };

        let data = match vertipad_req.data {
            Some(data) => data,
            None => {
                grpc_error!("No data provided for update vertipad with id: {}", id);
                return Err(Status::cancelled("No data found for update vertipad"));
            }
        };

        let pool = get_psql_pool();
        let vertipad = match VertipadPsql::new(&pool, id).await {
            Ok(vertipad) => vertipad,
            Err(e) => {
                grpc_error!("Could not find vertipad with id: {}. {}", id, e);
                return Err(Status::not_found("Unknown vertipad id"));
            }
        };

        let mut vertipad_data = HashMap::<&str, &(dyn ToSql + Sync)>::new();

        let vertiport_id = match Uuid::try_parse(&data.vertiport_id) {
            Ok(id) => id,
            Err(e) => {
                grpc_error!("Could not convert [vertiport_id] to UUID: {}", e);
                return Err(Status::new(Code::Internal, e.to_string()));
            }
        };
        vertipad_data.insert("vertiport_id", &vertiport_id);
        vertipad_data.insert("description", &data.description);
        vertipad_data.insert("longitude", &data.longitude);
        vertipad_data.insert("latitude", &data.latitude);
        vertipad_data.insert("enabled", &data.enabled);
        vertipad_data.insert("occupied", &data.occupied);

        match vertipad.update(vertipad_data.clone()).await {
            Ok(vertipad_data) => {
                let result = Vertipad {
                    id: id.to_string(),
                    data: Some(vertipad_data.into()),
                };
                // Update cache
                let mut vertipads = VERTIPADS.lock().await;
                vertipads.insert(id.to_string(), result.clone());

                Ok(Response::new(result.clone()))
            }
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    /// Takes an UUID string (vertipad_id) to set the matching vertipad record a deleted in the database.
    ///
    /// The `deleted_at` column will be set to the current timestamp, indicating that the Vertipad is not active anymore.
    ///
    /// # Arguments
    /// * self  - The VertipadRpc struct
    /// * request - Request<Id> GRPC request
    ///
    /// # Returns
    /// * Result empty or Status `Code::Internal` if the vertipad could not be deleted
    ///
    /// # Example
    /// ```
    /// use svc_storage_client_grpc::client::vertipad_rpc_client::VertipadRpcClient;
    /// use svc_storage_client_grpc::client::Id;
    ///
    /// let mut vertipad_client = VertipadRpcClient::connect(grpc_endpoint.clone()).await.unwrap();
    /// let result = match vertipad_client.delete_vertipad(tonic::Request::new(Id {
    ///     id: "54acfe06-dd9b-42e8-8cb4-12a2fb2fa693"
    /// }))
    /// .await
    /// {
    ///    Ok(vertipad) => vertipad.into_inner(),
    ///    Err(e) => panic!("Something went wrong deleting the vertipad: {}", e),
    /// };
    /// ```
    async fn delete_vertipad(&self, request: Request<Id>) -> Result<Response<()>, Status> {
        let id = request.into_inner().id;
        let pool = get_psql_pool();
        let vertipad = VertipadPsql::new(&pool, Uuid::try_parse(&id).unwrap()).await?;

        match vertipad.delete().await {
            Ok(_) => {
                // Update cache
                let mut vertipads = VERTIPADS.lock().await;
                vertipads.remove(&id);
                Ok(Response::new(()))
            }
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }
}
