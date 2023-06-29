//! gRPC server implementation
use super::GrpcSimpleService;
use crate::config::Config;
use crate::resources::base::ResourceObject;
use crate::shutdown_signal;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic::{Request, Status};

// include gRPC generic structs
include!("../../../out/grpc/grpc.rs");

// include gRPC services for all resources
grpc_server!(adsb, "adsb");
grpc_server!(flight_plan, "flight_plan");
grpc_server!(group, "group");
grpc_server!(itinerary, "itinerary");
grpc_server!(parcel, "parcel");
grpc_server!(pilot, "pilot");
grpc_server!(parcel_scan, "parcel_scan");
grpc_server!(scanner, "scanner");
grpc_server!(user, "user");
grpc_server!(vehicle, "vehicle");
grpc_server!(vertipad, "vertipad");
grpc_server!(vertiport, "vertiport");

/// Module to expose linked resource implementations for itinerary_flight_plan
pub mod itinerary_flight_plan {
    use super::flight_plan;
    use super::itinerary;
    pub use super::itinerary::rpc_flight_plan_link_server::*;
    use super::itinerary::ItineraryFlightPlans;
    pub use super::user::rpc_group_link_server::*;
    use super::{Id, IdList};
    use crate::grpc::GrpcLinkService;
    use crate::resources::base::linked_resource::LinkOtherResource;
    use crate::resources::base::ResourceObject;
    use prost::Message;
    use tonic::{Request, Status};

    /// Dummy struct for ItineraryFlightPlan Data
    /// Allows us to implement the required traits
    #[derive(Clone, Message, Copy)]
    pub struct Data {}

    build_grpc_server_link_service_impl!(
        itinerary,
        flight_plan,
        RpcFlightPlanLink,
        ItineraryFlightPlans
    );
}

/// Module to expose linked resource implementations for user_group
pub mod user_group {
    use super::group;
    use super::user;
    pub use super::user::rpc_group_link_server::*;
    use super::user::UserGroups;
    use super::{Id, IdList};
    use crate::grpc::GrpcLinkService;
    use crate::resources::base::linked_resource::LinkOtherResource;
    use crate::resources::base::ResourceObject;
    use prost::Message;
    use tonic::{Request, Status};

    /// Dummy struct for UserGroup Data
    /// Allows us to implement the required traits
    #[derive(Clone, Message, Copy)]
    pub struct Data {}

    build_grpc_server_link_service_impl!(user, group, RpcGroupLink, UserGroups);
}

/// Module to expose linked resource implementations for user_group
/// Uses the user_group Data object implementation for database schema definitions
pub mod group_user {
    use super::group;
    pub use super::group::rpc_user_link_server::*;
    use super::group::GroupUsers;
    use super::user;
    pub use super::user_group::Data;
    use super::{Id, IdList};
    use crate::grpc::GrpcLinkService;
    use crate::resources::base::linked_resource::LinkOtherResource;
    use crate::resources::base::ResourceObject;
    use tonic::{Request, Status};

    build_grpc_server_link_service_impl!(group, user, RpcUserLink, GroupUsers);
}

/// Provide search helpers
pub mod search {
    include!("../../../includes/search.rs");
}

/// Provide geo types and conversions
pub mod grpc_geo_types {
    pub use geo_types::{Coord, LineString, Point, Polygon};
    use serde::{Deserialize, Serialize};

    /// Geo Location Point representation
    /// <https://mapscaping.com/latitude-x-or-y/>
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, Copy, PartialEq, ::prost::Message, Serialize, Deserialize)]
    pub struct GeoPoint {
        /// longitude (x / horizontal / east-west)
        /// range: -180 - 180
        #[prost(double, tag = "1")]
        pub longitude: f64,
        /// latitude (y / vertical / north-south)
        /// range: -90 - 90
        #[prost(double, tag = "2")]
        pub latitude: f64,
    }
    /// Geo Location Line representation
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, Copy, PartialEq, ::prost::Message, Serialize, Deserialize)]
    pub struct GeoLine {
        /// line start point as long/lat
        #[prost(message, optional, tag = "1")]
        pub start: ::core::option::Option<GeoPoint>,
        /// line end point as long/lat
        #[prost(message, optional, tag = "2")]
        pub end: ::core::option::Option<GeoPoint>,
    }
    /// Geo Location Shape representation
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
    pub struct GeoLineString {
        /// list of points
        #[prost(message, repeated, tag = "1")]
        pub points: ::prost::alloc::vec::Vec<GeoPoint>,
    }
    /// Geo Location Polygon representation
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
    pub struct GeoPolygon {
        /// exterior
        #[prost(message, optional, tag = "1")]
        pub exterior: ::core::option::Option<GeoLineString>,
        /// interiors
        #[prost(message, repeated, tag = "2")]
        pub interiors: ::prost::alloc::vec::Vec<GeoLineString>,
    }

    include!("../../../includes/geo_types.rs");
}

/// Starts the grpc servers for this microservice using the provided configuration
///
/// # Examples
/// ```
/// use svc_storage::common::ArrErr;
/// use svc_storage::grpc::server::grpc_server;
/// use svc_storage::Config;
/// async fn example() -> Result<(), tokio::task::JoinError> {
///     let config = Config::default();
///     tokio::spawn(grpc_server(config, None)).await
/// }
/// ```
#[cfg(not(tarpaulin_include))]
// no_coverage: Can not be tested in unittest, should be part of integration
// tests
pub async fn grpc_server(config: Config, shutdown_rx: Option<tokio::sync::oneshot::Receiver<()>>) {
    grpc_debug!("(grpc_server) entry.");

    // GRPC Server
    let grpc_port = config.docker_port_grpc;
    let full_grpc_addr: SocketAddr = match format!("[::]:{}", grpc_port).parse() {
        Ok(addr) => addr,
        Err(e) => {
            grpc_error!("Failed to parse gRPC address: {}", e);
            return;
        }
    };

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<adsb::RpcServiceServer<adsb::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<flight_plan::RpcServiceServer<flight_plan::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<group::RpcServiceServer<group::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<group_user::RpcUserLinkServer<group_user::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<itinerary::RpcServiceServer<itinerary::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<itinerary_flight_plan::RpcFlightPlanLinkServer<itinerary_flight_plan::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<parcel::RpcServiceServer<parcel::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<parcel_scan::RpcServiceServer<parcel_scan::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<pilot::RpcServiceServer<pilot::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<scanner::RpcServiceServer<scanner::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<user::RpcServiceServer<user::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<user_group::RpcGroupLinkServer<user_group::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<vehicle::RpcServiceServer<vehicle::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<vertipad::RpcServiceServer<vertipad::GrpcServer>>()
        .await;
    health_reporter
        .set_serving::<vertiport::RpcServiceServer<vertiport::GrpcServer>>()
        .await;

    //start server
    grpc_info!(
        "(grpc_server) Starting gRPC services on: {}.",
        full_grpc_addr
    );
    match Server::builder()
        .add_service(health_service)
        .add_service(adsb::RpcServiceServer::new(adsb::GrpcServer::default()))
        .add_service(flight_plan::RpcServiceServer::new(
            flight_plan::GrpcServer::default(),
        ))
        .add_service(group::RpcServiceServer::new(group::GrpcServer::default()))
        .add_service(group_user::RpcUserLinkServer::new(
            group_user::GrpcServer::default(),
        ))
        .add_service(itinerary::RpcServiceServer::new(
            itinerary::GrpcServer::default(),
        ))
        .add_service(itinerary_flight_plan::RpcFlightPlanLinkServer::new(
            itinerary_flight_plan::GrpcServer::default(),
        ))
        .add_service(parcel::RpcServiceServer::new(parcel::GrpcServer::default()))
        .add_service(parcel_scan::RpcServiceServer::new(
            parcel_scan::GrpcServer::default(),
        ))
        .add_service(pilot::RpcServiceServer::new(pilot::GrpcServer::default()))
        .add_service(scanner::RpcServiceServer::new(
            scanner::GrpcServer::default(),
        ))
        .add_service(user::RpcServiceServer::new(user::GrpcServer::default()))
        .add_service(user_group::RpcGroupLinkServer::new(
            user_group::GrpcServer::default(),
        ))
        .add_service(vehicle::RpcServiceServer::new(
            vehicle::GrpcServer::default(),
        ))
        .add_service(vertipad::RpcServiceServer::new(
            vertipad::GrpcServer::default(),
        ))
        .add_service(vertiport::RpcServiceServer::new(
            vertiport::GrpcServer::default(),
        ))
        .serve_with_shutdown(full_grpc_addr, shutdown_signal("grpc", shutdown_rx))
        .await
    {
        Ok(_) => grpc_info!("(grpc_server) gRPC server running at: {}.", full_grpc_addr),
        Err(e) => {
            grpc_error!("(grpc_server) Could not start gRPC server: {}", e);
        }
    };
}
