//! Users

mod grpc_server {
    #![allow(unused_qualifications, missing_docs)]
    tonic::include_proto!("grpc.user");
}

use super::UserPsql;
use crate::get_psql_pool;
use crate::grpc::*;
use crate::grpc_error;
use crate::memdb::MEMDB_LOG_TARGET;
use crate::memdb::USERS;
use crate::memdb_info;
use uuid::Uuid;

pub use grpc_server::user_rpc_server::{UserRpc, UserRpcServer};
pub use grpc_server::{AuthMethod, UpdateUser, User, UserData, Users};

use postgres_types::ToSql;
use std::collections::HashMap;
use tonic::{Code, Request, Response, Status};

///Implementation of gRPC endpoints
#[derive(Debug, Default, Copy, Clone)]
pub struct UserImpl {}

#[tonic::async_trait]
impl UserRpc for UserImpl {
    /// Takes an UUID string (user_id) to get the matching database record.
    ///
    /// # Example
    /// ```
    /// use svc_storage_client_grpc::client::user_rpc_client::UserRpcClient;
    /// use svc_storage_client_grpc::client::Id;
    ///
    /// let mut user_client = UserRpcClient::connect(grpc_endpoint.clone()).await.unwrap();
    /// let result = match user_client.user_by_id(tonic::Request::new(Id {
    ///     id: "54acfe06-dd9b-42e8-8cb4-12a2fb2fa693"
    /// }))
    /// .await
    /// {
    ///    Ok(user) => user.into_inner(),
    ///    Err(e) => panic!("Something went wrong retrieving the user: {}", e),
    /// };
    /// ```
    async fn user_by_id(&self, request: Request<Id>) -> Result<Response<User>, Status> {
        let id = request.into_inner().id;
        if let Some(user) = USERS.lock().await.get(&id) {
            memdb_info!("Found entry for User. uuid: {}", id);
            Ok(Response::new(user.clone()))
        } else {
            let pool = get_psql_pool();
            let data = UserPsql::new(&pool, Uuid::try_parse(&id).unwrap()).await;
            match data {
                Ok(user) => Ok(Response::new(User {
                    id,
                    data: Some(user.into()),
                })),
                Err(_) => Err(Status::not_found("Not found")),
            }
        }
    }

    /// Takes a `SearchFilter` object to search the database with the provided values.
    ///
    /// This method supports paged results. When the `search_field` and `search_value` are empty, no filters will be applied.
    ///
    /// # Example
    /// ```
    /// use svc_storage_client_grpc::client::user_rpc_client::UserRpcClient;
    /// use svc_storage_client_grpc::client::SearchFilter;
    ///
    /// let mut user_client = UserRpcClient::connect(grpc_endpoint.clone()).await.unwrap();
    /// let result = match user_client.users(tonic::Request::new(SearchFilter {
    ///     search_field: "auth_method",
    ///     search_value: "1",
    ///     page_number: 1,
    ///     results_per_page: 20
    /// }))
    /// .await
    /// {
    ///    Ok(users) => user.into_inner(),
    ///    Err(e) => panic!("Something went wrong creating retrieving users: {}", e),
    /// };
    /// ```
    async fn users(&self, request: Request<SearchFilter>) -> Result<Response<Users>, Status> {
        let filter = request.into_inner();
        let mut filter_hash = HashMap::<String, String>::new();
        filter_hash.insert("column".to_string(), filter.search_field);
        filter_hash.insert("value".to_string(), filter.search_value);

        let pool = get_psql_pool();
        match super::psql::search(&pool, &filter_hash).await {
            Ok(users) => Ok(Response::new(users.into())),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    /// Takes an `UserData` object to create a new user with the provided data.
    ///
    /// A new UUID will be generated by the database and returned as `id` as part of the returned `User` object.
    ///
    /// # Example
    /// ```
    /// use svc_storage_client_grpc::client::user_rpc_client::{UserRpcClient, UserData};
    ///
    /// let mut user_client = UserRpcClient::connect(grpc_endpoint.clone()).await.unwrap();
    /// let result = match user_client.insert_user(tonic::Request::new(UserData {
    ///     first_name: "John",
    ///     last_name: "Doe",
    ///     auth_method: 0
    /// }))
    /// .await
    /// {
    ///    Ok(user) => user.into_inner(),
    ///    Err(e) => panic!("Something went wrong creating the user: {}", e),
    /// };
    /// ```
    async fn insert_user(&self, request: Request<UserData>) -> Result<Response<User>, Status> {
        let mut users = USERS.lock().await;
        let data = request.into_inner();
        let pool = get_psql_pool();

        let mut user_data = HashMap::<&str, &(dyn ToSql + Sync)>::new();

        user_data.insert("first_name", &data.first_name);
        user_data.insert("last_name", &data.last_name);
        let auth_method = match AuthMethod::from_i32(data.auth_method) {
            Some(auth) => auth.as_str_name(),
            None => todo!(),
        };
        user_data.insert("auth_method", &auth_method);

        match super::psql::create(&pool, user_data.clone()).await {
            Ok(user) => {
                let id = user.id.to_string();
                let user = User {
                    id: id.clone(),
                    data: Some(data.clone()),
                };
                users.insert(id, user.clone());
                Ok(Response::new(user))
            }
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    /// Takes an `UpdateUser` object to store new data to the database
    ///
    /// A field mask can be provided to restrict updates to specific fields.
    /// Returns the updated `User` on success.
    ///
    /// # Example
    /// ```
    /// use svc_storage_client_grpc::client::user_rpc_client::{UserRpcClient, UpdateUser};
    ///
    /// let mut user_client = UserRpcClient::connect(grpc_endpoint.clone()).await.unwrap();
    /// let updated_user = match user_client.update_user(tonic::Request::new(UpdateUser {
    ///     id: "54acfe06-dd9b-42e8-8cb4-12a2fb2fa693"
    ///     data: Some(UserData {
    ///         first_name: "John",
    ///         last_name: "Doe",
    ///     }),
    ///     mask: Some(FieldMask {
    ///         paths: vec!["first_name".to_string()]
    ///     })
    /// }))
    /// .await
    /// {
    ///    Ok(user) => user.into_inner(),
    ///    Err(e) => panic!("Something went wrong updating the user info: {}", e),
    /// };
    /// ```
    async fn update_user(&self, request: Request<UpdateUser>) -> Result<Response<User>, Status> {
        let user_req = request.into_inner();
        let id = match Uuid::try_parse(&user_req.id) {
            Ok(id) => id,
            Err(_e) => Uuid::new_v4(),
        };

        let data = match user_req.data {
            Some(data) => data,
            None => {
                grpc_error!("No data provided for update user with id: {}", id);
                return Err(Status::cancelled("No data found for update user"));
            }
        };

        let pool = get_psql_pool();
        let user = match UserPsql::new(&pool, id).await {
            Ok(user) => user,
            Err(e) => {
                grpc_error!("Could not find user with id: {}. {}", id, e);
                return Err(Status::not_found("Unknown user id"));
            }
        };

        let mut user_data = HashMap::<&str, &(dyn ToSql + Sync)>::new();
        user_data.insert("first_name", &data.first_name);
        user_data.insert("last_name", &data.last_name);
        let auth_method = match AuthMethod::from_i32(data.auth_method) {
            Some(auth) => auth.as_str_name(),
            None => todo!(),
        };
        user_data.insert("auth_method", &auth_method);

        match user.update(user_data.clone()).await {
            Ok(user_data) => {
                let result = User {
                    id: id.to_string(),
                    data: Some(user_data.into()),
                };
                // Update cache
                let mut users = USERS.lock().await;
                users.insert(id.to_string(), result.clone());

                Ok(Response::new(result.clone()))
            }
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    /// Takes an UUID string (user_id) to set the matching user record a deleted in the database.
    ///
    /// The `deleted_at` column will be set to the current timestamp, indicating that the User is not active anymore.
    ///
    /// # Example
    /// ```
    /// use svc_storage_client_grpc::client::user_rpc_client::UserRpcClient;
    /// use svc_storage_client_grpc::client::Id;
    ///
    /// let mut user_client = UserRpcClient::connect(grpc_endpoint.clone()).await.unwrap();
    /// let result = match user_client.delete_user(tonic::Request::new(Id {
    ///     id: "54acfe06-dd9b-42e8-8cb4-12a2fb2fa693"
    /// }))
    /// .await
    /// {
    ///    Ok(user) => user.into_inner(),
    ///    Err(e) => panic!("Something went wrong deleting the user: {}", e),
    /// };
    /// ```
    async fn delete_user(&self, request: Request<Id>) -> Result<Response<()>, Status> {
        let id = request.into_inner().id;
        let pool = get_psql_pool();
        let user = UserPsql::new(&pool, Uuid::try_parse(&id).unwrap()).await?;

        match user.delete().await {
            Ok(_) => {
                // Update cache
                let mut users = USERS.lock().await;
                users.remove(&id);
                Ok(Response::new(()))
            }
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }
}