#[macro_use]
/// log macro's for gRPC logging
pub mod macros;

pub mod grpc_server {
    #![allow(unused_qualifications, missing_docs)]
    tonic::include_proto!("grpc");
}

pub use grpc_server::storage_rpc_server::{StorageRpc, StorageRpcServer};
pub use grpc_server::{
    Id, ReadyRequest, ReadyResponse, SearchFilter, ValidationError, ValidationResult,
};
use tokio_postgres::Row;

pub use crate::common::{ArrErr, GRPC_LOG_TARGET};
use crate::postgres::{PsqlObjectType, PsqlResourceType};
use crate::resources::base::{GenericObjectType, GenericResourceResult};

use anyhow::Error;
use prost_types::Timestamp;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::SystemTime;
use tokio::runtime::{Handle, Runtime};
use tonic::transport::Server;
use tonic::{Code, Request, Response, Status};

use crate::common::Config;
use crate::resources::flight_plan;
use crate::resources::pilot::{PilotImpl, PilotRpcServer};
use crate::resources::vehicle::{VehicleImpl, VehicleRpcServer};
use crate::resources::vertipad;
use crate::resources::vertiport;

#[derive(Debug, Clone)]
/// gRPC field types
pub enum GrpcField {
    /// String
    String(String),
    /// Vec\<i64\>
    I64List(Vec<i64>),
    /// i64
    I64(i64),
    /// f64
    F64(f64),
    /// i32
    I32(i32),
    /// f32
    F32(f32),
    /// bool
    Bool(bool),
    /// i16
    I16(i16),
    /// Timestamp
    Timestamp(Timestamp),
    /// Option GrpcFieldOption
    Option(GrpcFieldOption),
}
#[derive(Debug, Clone)]
/// gRPC field types as Option
pub enum GrpcFieldOption {
    /// Option\<String\>
    String(Option<String>),
    /// Option\<Vec\<i64\>\>
    I64List(Option<Vec<i64>>),
    /// Option\<i64\>
    I64(Option<i64>),
    /// Option\<f64\>
    F64(Option<f64>),
    /// Option\<i32\>
    I32(Option<i32>),
    /// Option\<f32\>
    F32(Option<f32>),
    /// Option\<bool\>
    Bool(Option<bool>),
    /// Option\<i16\>
    I16(Option<i16>),
    /// Option\<Timestamp\>
    Timestamp(Option<Timestamp>),
    /// [None]
    None,
}

#[tonic::async_trait]
/// Generic gRPC object traits to provide wrappers for common `Resource` functions
pub trait GrpcObjectType<T, U>
where
    T: GenericObjectType<U> + PsqlResourceType + From<Id> + Clone + Sync + Send,
    U: GrpcDataObjectType + TryFrom<Row>,
    Status: From<<U as TryFrom<Row>>::Error>,
{
    /// Returns a [tonic] gRCP [`Response`] containing an object of provided type `V`.
    /// `V` will contain the record data found for the provided [`Id`].
    ///
    /// # Errors
    ///
    /// Returns [`Status`] with [`Code::NotFound`] if no record is returned from the database.  
    /// Returns [`Status`] with [`Code::Internal`] if the provided Id can not be converted to a [`uuid::Uuid`].  
    /// Returns [`Status`] with [`Code::Internal`] if the resulting [`Row`] data could not be converted into `U`.  
    ///
    async fn generic_get_by_id<V>(&self, request: Request<Id>) -> Result<Response<V>, Status>
    where
        V: From<T>,
    {
        let id: Id = request.into_inner();
        let mut resource: T = id.clone().into();
        let obj: Result<Row, ArrErr> = T::get_by_id(&resource.try_get_uuid()?).await;
        if let Ok(obj) = obj {
            resource.set_data(obj.try_into()?);
            Ok(Response::new(resource.into()))
        } else {
            let error = format!("No resource found for specified uuid: {}", id.id);
            grpc_error!("{}", error);
            Err(Status::new(Code::NotFound, error))
        }
    }

    /// Returns a [tonic] gRCP [`Response`] containing an object of provided type `V`.
    /// `V`(TryFrom\<Vec\<Row\>\>) will contain all records found in the database using the the provided [`SearchFilter`].
    ///
    /// This method supports paged results.
    /// When the `search_field` and `search_value` are empty, no filters will be applied.
    ///
    /// # Errors
    ///
    /// Returns [`Status`] with [`Code::Internal`] if any error is returned from the db search result.  
    /// Returns [`Status`] with [`Code::Internal`] if the resulting [`Vec<Row>`] data could not be converted into `V`.  
    ///
    async fn generic_get_all_with_filter<V>(
        &self,
        request: Request<SearchFilter>,
    ) -> Result<Response<V>, Status>
    where
        V: TryFrom<Vec<Row>>,
        Status: From<<V as TryFrom<Vec<Row>>>::Error>,
    {
        let filter: SearchFilter = request.into_inner();
        let mut filter_hash = HashMap::<String, String>::new();
        filter_hash.insert("column".to_string(), filter.search_field);
        filter_hash.insert("value".to_string(), filter.search_value);

        match T::search(&filter_hash).await {
            Ok(rows) => Ok(Response::new(rows.try_into()?)),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    /// Returns a [tonic] gRCP [`Response`] containing an object of provided type `V`.
    /// `V`(From<GenericResourceResult<T, U>>) will contain the inserted record after saving the provided data `U`.
    ///
    /// The given data will be validated before insert.  
    /// A new UUID will be generated by the database and returned as `id` as part of the returned `U` object.  
    /// Any errors found during validation will be added to the [`ValidationResult`].  
    ///
    /// # Errors
    ///
    /// Returns [`Status`] with [`Code::Internal`] if the [`Request`] doesn't contain any data.  
    /// Returns [`Status`] with [`Code::Internal`] if any error is returned from a db call.
    ///
    async fn generic_insert<V>(&self, request: Request<U>) -> Result<Response<V>, Status>
    where
        T: From<U>,
        U: 'async_trait,
        V: From<GenericResourceResult<T, U>>,
    {
        let data = request.into_inner();
        let mut resource: T = data.into();
        grpc_debug!("Inserting with data {:?}", resource.try_get_data()?);
        let (id, validation_result) = T::create(&resource.try_get_data()?).await?;
        if let Some(id) = id {
            resource.set_id(id.to_string());
            let obj: T = resource;
            let result = GenericResourceResult::<T, U> {
                phantom: PhantomData,
                validation_result,
                resource: Some(obj),
            };
            Ok(Response::new(result.into()))
        } else {
            let error = "Error calling insert function";
            grpc_error!("{}", error);
            grpc_debug!("{:?}", resource.try_get_data()?);
            grpc_debug!("{:?}", validation_result);
            let result = GenericResourceResult::<T, U> {
                phantom: PhantomData,
                validation_result,
                resource: None,
            };
            Ok(Response::new(result.into()))
        }
    }

    /// Returns a [tonic] gRCP [`Response`] containing an object of provided type `V`.
    /// `V`(GenericResourceResult<T, U>) will contain the updated record after saving the provided data `U`.
    ///
    /// The given data will be validated before insert.
    /// Any errors found during validation will be added to the [`ValidationResult`].
    /// A field [`prost_types::FieldMask`] can be provided to restrict updates to specific fields.
    ///
    /// # Errors
    ///
    /// Returns [`Status`] with [`Code::Cancelled`] if the [`Request`] doesn't contain any data.  
    /// Returns [`Status`] with [`Code::Internal`] if any error is returned from a db call.  
    /// Returns [`Status`] with [`Code::Internal`] if the provided Id can not be converted to a [`uuid::Uuid`].  
    /// Returns [`Status`] with [`Code::Internal`] if the resulting [`Row`] data could not be converted into `U`.  
    ///
    async fn generic_update<V, W>(&self, request: Request<W>) -> Result<Response<V>, Status>
    where
        T: From<W> + PsqlObjectType<U>,
        V: From<GenericResourceResult<T, U>>,
        W: Send,
    {
        let req: T = request.into_inner().into();
        let id: Id = Id {
            id: req.try_get_id()?,
        };
        let mut resource: T = id.into();

        let data = match req.get_data() {
            Some(data) => data,
            None => {
                let err = format!("No data provided for update with id: {}", req.try_get_id()?);
                grpc_error!("{}", err);
                return Err(Status::cancelled(err));
            }
        };

        let (data, validation_result) = resource.update(&data).await?;
        if let Some(data) = data {
            resource.set_data(data.try_into()?);
            let result = GenericResourceResult {
                phantom: PhantomData,
                validation_result,
                resource: Some(resource),
            };
            Ok(Response::new(result.into()))
        } else {
            let error = "Error calling update function";
            grpc_error!("{}", error);
            grpc_debug!("{:?}", data);
            grpc_debug!("{:?}", validation_result);
            let result = GenericResourceResult {
                phantom: PhantomData,
                validation_result,
                resource: None,
            };
            Ok(Response::new(result.into()))
        }
    }

    /// Takes an [`Id`] to set the matching database record as deleted in the database.
    ///
    /// # Errors
    ///
    /// Returns [`Status`] with [`Code::NotFound`] if no record is returned from the database.  
    /// Returns [`Status`] with [`Code::Internal`] if any error is returned from a db call.  
    async fn generic_delete(&self, request: Request<Id>) -> Result<Response<()>, Status>
    where
        T: PsqlObjectType<U>,
    {
        let id: Id = request.into_inner();
        let resource: T = id.into();
        match resource.delete().await {
            Ok(_) => Ok(Response::new(())),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }
}

/// Provides function to get field values of gRPC `Data` objects
pub trait GrpcDataObjectType: prost::Message + Clone {
    /// get the value of a field using the field name
    fn get_field_value(&self, key: &str) -> Result<GrpcField, ArrErr>;
}

#[derive(Debug, Default, Copy, Clone)]
/// struct to implement the gRPC server functions
pub struct StorageImpl {}

#[tonic::async_trait]
impl StorageRpc for StorageImpl {
    /// Returns ready:true when service is available
    async fn is_ready(
        &self,
        _request: Request<ReadyRequest>,
    ) -> Result<Response<ReadyResponse>, Status> {
        let response = ReadyResponse { ready: true };
        Ok(Response::new(response))
    }
}

impl From<ArrErr> for Status {
    fn from(err: ArrErr) -> Self {
        // These errors come from modules like Postgres, where you
        // probably wouldn't want to include error details in the
        // response, log them here instead which will include
        // tracing information from the request handler
        //
        // <https://cheatsheetseries.owasp.org/cheatsheets/REST_Security_Cheat_Sheet.html#error-handling>
        // <https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html#which-events-to-log>
        let err: Error = err.into();
        grpc_warn!("{:#}", err);

        tonic::Status::internal("error".to_string())
    }
}

impl From<GrpcField> for String {
    fn from(field: GrpcField) -> Self {
        match field {
            GrpcField::String(field) => field,
            _ => format!("{:?}", field),
        }
    }
}
impl From<GrpcField> for Vec<i64> {
    fn from(field: GrpcField) -> Self {
        match field {
            GrpcField::I64List(field) => field,
            GrpcField::I64(field) => vec![field],
            _ => vec![],
        }
    }
}
impl From<GrpcField> for i64 {
    fn from(field: GrpcField) -> Self {
        match field {
            GrpcField::I64(field) => field,
            _ => 0,
        }
    }
}
impl From<GrpcField> for f64 {
    fn from(field: GrpcField) -> Self {
        match field {
            GrpcField::F64(field) => field,
            _ => 0.0,
        }
    }
}
impl From<GrpcField> for i32 {
    fn from(field: GrpcField) -> Self {
        match field {
            GrpcField::I32(field) => field,
            _ => 0,
        }
    }
}
impl From<GrpcField> for i16 {
    fn from(field: GrpcField) -> Self {
        match field {
            GrpcField::I16(field) => field,
            _ => 0,
        }
    }
}
impl From<GrpcField> for Timestamp {
    fn from(field: GrpcField) -> Self {
        match field {
            GrpcField::Timestamp(field) => field,
            _ => Timestamp::from(SystemTime::now()),
        }
    }
}

impl From<GrpcFieldOption> for Option<GrpcField> {
    fn from(field: GrpcFieldOption) -> Self {
        match field {
            GrpcFieldOption::String(field) => field.map(GrpcField::String),
            GrpcFieldOption::I64List(field) => field.map(GrpcField::I64List),
            GrpcFieldOption::I64(field) => field.map(GrpcField::I64),
            GrpcFieldOption::F64(field) => field.map(GrpcField::F64),
            GrpcFieldOption::I32(field) => field.map(GrpcField::I32),
            GrpcFieldOption::I16(field) => field.map(GrpcField::I16),
            GrpcFieldOption::F32(field) => field.map(GrpcField::F32),
            GrpcFieldOption::Bool(field) => field.map(GrpcField::Bool),
            GrpcFieldOption::Timestamp(field) => field.map(GrpcField::Timestamp),
            GrpcFieldOption::None => None,
        }
    }
}

/// Starts the grpc servers for this microservice using the configuration settings found in the environment
///
/// ```
/// use svc_storage::common::ArrErr;
/// use svc_storage::grpc::grpc_server;
/// async fn example() -> Result<(), ArrErr> {
///     tokio::spawn(grpc_server()).await?;
///     Ok(())
/// }
/// ```
pub async fn grpc_server() {
    let settings = match Config::from_env() {
        Ok(settings) => settings,
        Err(e) => {
            let _ = ArrErr::from(e);
            Config::new()
        }
    };

    let grpc_port = settings.docker_port_grpc.unwrap_or(50051);
    let full_grpc_addr = format!("[::]:{}", grpc_port).parse().unwrap();

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<StorageRpcServer<StorageImpl>>()
        .await;
    health_reporter
        .set_serving::<VehicleRpcServer<VehicleImpl>>()
        .await;
    health_reporter
        .set_serving::<flight_plan::RpcServiceServer<flight_plan::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<vertipad::RpcServiceServer<vertipad::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<vertiport::RpcServiceServer<vertiport::GrpcServer>>()
        .await;

    grpc_info!("Starting GRPC servers on {}.", full_grpc_addr);
    Server::builder()
        .add_service(health_service)
        .add_service(StorageRpcServer::new(StorageImpl::default()))
        .add_service(VehicleRpcServer::new(VehicleImpl::default()))
        .add_service(PilotRpcServer::new(PilotImpl::default()))
        .add_service(flight_plan::RpcServiceServer::new(
            flight_plan::GrpcServer::default(),
        ))
        .add_service(vertipad::RpcServiceServer::new(
            vertipad::GrpcServer::default(),
        ))
        .add_service(vertiport::RpcServiceServer::new(
            vertiport::GrpcServer::default(),
        ))
        .serve_with_shutdown(full_grpc_addr, shutdown_signal())
        .await
        .unwrap();
    grpc_info!("gRPC server running at: {}", full_grpc_addr);
}

/// Tokio signal handler that will wait for a user to press CTRL+C.
/// We use this in our tonic `Server` method `serve_with_shutdown`.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Tokio signal ctrl-c received!");
    grpc_info!("signal shutdown!");
}

/// Get the tokio handle of the current runtime.
/// Makes sure a Handle is returned, even if there is no current handle found.
/// The handle can be used to spawn a separate task, or run an async function
/// from a non-async function.
///
/// ```
/// use svc_storage::grpc::get_runtime_handle;
/// use svc_storage::postgres::PsqlResourceType;
/// use svc_storage::resources::base::GenericResource;
/// use svc_storage::resources::vertipad;
/// async fn example() {
///     let id = uuid::Uuid::new_v4();
///     let handle = get_runtime_handle();
///     // start a blocking task so we can make sure
///     // our function is ready before we continue our code
///     let data = tokio::task::block_in_place(move || {
///         // use the tokio handle to block on our async function
///         handle.block_on(async move {
///             // run async function
///             <GenericResource<vertipad::Data> as
///             PsqlResourceType>::get_by_id(&id).await
///         })
///     });
/// }
/// ```
pub fn get_runtime_handle() -> Handle {
    match Handle::try_current() {
        Ok(h) => h,
        Err(_) => {
            let rt = Runtime::new().unwrap();
            rt.handle().clone()
        }
    }
}
