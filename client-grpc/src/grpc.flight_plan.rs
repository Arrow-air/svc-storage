/// FlightPlan
#[derive(Eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlightPlan {
    /// id UUID v4
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    /// data
    #[prost(message, optional, tag="2")]
    pub data: ::core::option::Option<FlightPlanData>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateFlightPlan {
    /// id UUID v4
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub data: ::core::option::Option<FlightPlanData>,
    #[prost(message, optional, tag="3")]
    pub mask: ::core::option::Option<::prost_types::FieldMask>,
}
/// FlightPlanData
#[derive(Eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlightPlanData {
    /// pilot_id UUID v4
    #[prost(string, tag="1")]
    pub pilot_id: ::prost::alloc::string::String,
    /// vehicle_id UUID v4
    #[prost(string, tag="2")]
    pub vehicle_id: ::prost::alloc::string::String,
    /// cargo weight per package
    #[prost(uint32, repeated, tag="3")]
    pub cargo_weight: ::prost::alloc::vec::Vec<u32>,
    /// flight_distance in meters
    #[prost(uint32, tag="4")]
    pub flight_distance: u32,
    /// weather_conditions
    #[prost(string, tag="5")]
    pub weather_conditions: ::prost::alloc::string::String,
    /// departure_vertiport_id UUID v4
    #[prost(string, tag="6")]
    pub departure_vertiport_id: ::prost::alloc::string::String,
    /// departure_pad_id UUID v4
    #[prost(string, tag="7")]
    pub departure_pad_id: ::prost::alloc::string::String,
    /// destination_vertiport_id UUID v4
    #[prost(string, tag="8")]
    pub destination_vertiport_id: ::prost::alloc::string::String,
    /// destination_pad_id UUID v4
    #[prost(string, tag="9")]
    pub destination_pad_id: ::prost::alloc::string::String,
    /// scheduled_departure
    #[prost(message, optional, tag="10")]
    pub scheduled_departure: ::core::option::Option<::prost_types::Timestamp>,
    /// scheduled_arrival
    #[prost(message, optional, tag="11")]
    pub scheduled_arrival: ::core::option::Option<::prost_types::Timestamp>,
    /// actual_departure
    #[prost(message, optional, tag="12")]
    pub actual_departure: ::core::option::Option<::prost_types::Timestamp>,
    /// actual_arrival
    #[prost(message, optional, tag="13")]
    pub actual_arrival: ::core::option::Option<::prost_types::Timestamp>,
    /// flight_release_approval date and time
    #[prost(message, optional, tag="14")]
    pub flight_release_approval: ::core::option::Option<::prost_types::Timestamp>,
    /// flight_plan_submitted date and time
    #[prost(message, optional, tag="15")]
    pub flight_plan_submitted: ::core::option::Option<::prost_types::Timestamp>,
    /// approved_by UUID v4
    #[prost(string, optional, tag="16")]
    pub approved_by: ::core::option::Option<::prost::alloc::string::String>,
    /// flight_status
    #[prost(enumeration="FlightStatus", tag="17")]
    pub flight_status: i32,
    /// flightPriority
    #[prost(enumeration="FlightPriority", tag="18")]
    pub flight_priority: i32,
}
/// FlightPlans
#[derive(Eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlightPlans {
    /// array/vector of flight items
    #[prost(message, repeated, tag="1")]
    pub flight_plans: ::prost::alloc::vec::Vec<FlightPlan>,
}
/// Flight Status Enum
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FlightStatus {
    /// READY
    Ready = 0,
    /// BOARDING
    Boarding = 1,
    /// IN_FLIGHT
    InFlight = 3,
    /// FINISHED
    Finished = 4,
    /// CANCELLED
    Cancelled = 5,
    /// DRAFT
    Draft = 6,
}
impl FlightStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            FlightStatus::Ready => "READY",
            FlightStatus::Boarding => "BOARDING",
            FlightStatus::InFlight => "IN_FLIGHT",
            FlightStatus::Finished => "FINISHED",
            FlightStatus::Cancelled => "CANCELLED",
            FlightStatus::Draft => "DRAFT",
        }
    }
}
/// Flight Priority Enum
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FlightPriority {
    /// LOW
    Low = 0,
    /// HIGH
    High = 1,
    /// EMERGENCY
    Emergency = 2,
}
impl FlightPriority {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            FlightPriority::Low => "LOW",
            FlightPriority::High => "HIGH",
            FlightPriority::Emergency => "EMERGENCY",
        }
    }
}
/// Generated client implementations.
pub mod flight_plan_rpc_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    ///FlightPlanRpc service
    #[derive(Debug, Clone)]
    pub struct FlightPlanRpcClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl FlightPlanRpcClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> FlightPlanRpcClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> FlightPlanRpcClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            FlightPlanRpcClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        pub async fn flight_plans(
            &mut self,
            request: impl tonic::IntoRequest<super::super::SearchFilter>,
        ) -> Result<tonic::Response<super::FlightPlans>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc.flight_plan.FlightPlanRpc/flight_plans",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn flight_plan_by_id(
            &mut self,
            request: impl tonic::IntoRequest<super::super::Id>,
        ) -> Result<tonic::Response<super::FlightPlan>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc.flight_plan.FlightPlanRpc/flight_plan_by_id",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn insert_flight_plan(
            &mut self,
            request: impl tonic::IntoRequest<super::FlightPlanData>,
        ) -> Result<tonic::Response<super::FlightPlan>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc.flight_plan.FlightPlanRpc/insert_flight_plan",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_flight_plan(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateFlightPlan>,
        ) -> Result<tonic::Response<super::FlightPlan>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc.flight_plan.FlightPlanRpc/update_flight_plan",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_flight_plan(
            &mut self,
            request: impl tonic::IntoRequest<super::super::Id>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc.flight_plan.FlightPlanRpc/delete_flight_plan",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}