use std::{pin::Pin, time::Duration};

use foundations::BootstrapResult;
use futures::{Stream, StreamExt};
use tonic::{codec::CompressionEncoding, transport::Body, Request, Response, Status};
use tonic_middleware::{InterceptorFor, RequestInterceptor};
use tonic_web::GrpcWebLayer;
use tower_http::{
  cors::{AllowMethods, AllowOrigin, CorsLayer},
  trace::TraceLayer,
};

use crate::{
  auth::verify_password,
  conf::Settings,
  db::{insert_session_token, validate_session_token},
  game::{
    items::mine_locations,
    mine::{start_mining, stop_mining},
  },
  protos::{
    mine_private_service_server::{MinePrivateService, MinePrivateServiceServer},
    mine_public_service_server::{MinePublicService, MinePublicServiceServer},
    GetAccountRequest, GetAccountResponse, GetInventoryRequest, GetInventoryResponse,
    GetItemDescriptorsRequest, GetMineLocationsRequest, GetMineLocationsResponse, LoginRequest,
    LoginResponse, MineLocationRes, RegisterRequest, RegisterResponse, SortBy, SortDirection,
    StartMiningRequest, StartMiningResponse, StopMiningRequest, StopMiningResponse,
  },
};

struct MinePrivateServer {}

struct MinePublicServer {}

struct UserCredentials {
  user_id: i32,
}

type BoxResultStream<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send + Sync + 'static>>;

trait AuthenticatedRequestExt {
  fn user_id(&self) -> i32;
}

impl<T> AuthenticatedRequestExt for Request<T> {
  fn user_id(&self) -> i32 { self.extensions().get::<UserCredentials>().unwrap().user_id }
}

#[tonic::async_trait]
impl MinePrivateService for MinePrivateServer {
  type StartMiningStream = BoxResultStream<StartMiningResponse>;

  // General

  async fn get_item_descriptors(
    &self,
    _req: Request<GetItemDescriptorsRequest>,
  ) -> Result<Response<crate::protos::GetItemDescriptorsResponse>, Status> {
    let item_descriptors = crate::game::items::item_descriptors().clone();
    Ok(Response::new(crate::protos::GetItemDescriptorsResponse {
      item_descriptors,
    }))
  }

  async fn get_mine_locations(
    &self,
    _req: Request<GetMineLocationsRequest>,
  ) -> Result<Response<GetMineLocationsResponse>, Status> {
    Ok(Response::new(GetMineLocationsResponse {
      mine_locations: mine_locations()
        .iter()
        .map(|loc| MineLocationRes {
          descriptor: Some(loc.descriptor.clone()),
          is_available: true, // TODO
        })
        .collect(),
    }))
  }

  // Account

  async fn get_account(
    &self,
    req: Request<GetAccountRequest>,
  ) -> Result<Response<GetAccountResponse>, Status> {
    let user_id = req.user_id();
    match crate::db::get_user_account(user_id).await {
      Ok(Some(account_info)) => Ok(Response::new(GetAccountResponse {
        user_account_info: Some(account_info),
      })),
      Ok(None) => Err(Status::not_found("User account not found")),
      Err(err) => {
        error!("Error reading user account from database: {err}");
        Err(Status::internal("Internal DB error fetching account info"))
      },
    }
  }

  async fn get_inventory(
    &self,
    req: Request<GetInventoryRequest>,
  ) -> Result<Response<GetInventoryResponse>, Status> {
    let user_id = req.user_id();
    let GetInventoryRequest {
      page_size,
      page_number,
      sort_by,
      sort_direction,
    } = req.into_inner();

    let sort_by = SortBy::try_from(sort_by).unwrap_or(SortBy::DateAcquired);
    let sort_direction =
      SortDirection::try_from(sort_direction).unwrap_or(SortDirection::Descending);
    let items =
      crate::db::get_user_inventory(user_id, page_size, page_number, sort_by, sort_direction)
        .await
        .map_err(|err| {
          error!("Error reading user inventory from database: {err}");
          Status::internal("Internal DB error fetching inventory")
        })?;

    Ok(Response::new(GetInventoryResponse { items }))
  }

  // Gameplay

  async fn start_mining(
    &self,
    req: Request<StartMiningRequest>,
  ) -> Result<Response<Self::StartMiningStream>, Status> {
    let user_id = req.user_id();
    let StartMiningRequest { location_name } = req.into_inner();

    let loot_stream = start_mining(user_id, &location_name).await?;
    Ok(Response::new(Box::pin(loot_stream.map(Ok))))
  }

  async fn stop_mining(
    &self,
    req: Request<StopMiningRequest>,
  ) -> Result<Response<StopMiningResponse>, Status> {
    let user_id = req.user_id();
    stop_mining(user_id).await;
    Ok(Response::new(StopMiningResponse {}))
  }
}

#[tonic::async_trait]
impl MinePublicService for MinePublicServer {
  async fn login(&self, req: Request<LoginRequest>) -> Result<Response<LoginResponse>, Status> {
    let LoginRequest { username, password } = req.into_inner();
    let user_id = verify_password(&username, &password).await?;

    let session_token = crate::auth::generate_session_token();
    insert_session_token(user_id, &session_token)
      .await
      .map_err(|err| {
        error!("Error inserting session token: {err}");
        Status::internal("Internal DB error")
      })?;

    info!("User {username} successfully logged in");
    Ok(Response::new(LoginResponse { session_token }))
  }

  async fn register(
    &self,
    req: Request<RegisterRequest>,
  ) -> Result<Response<RegisterResponse>, Status> {
    let RegisterRequest { username, password } = req.into_inner();

    let user_id = crate::db::insert_new_user(&username, &password).await?;

    let session_token = crate::auth::generate_session_token();
    insert_session_token(user_id, &session_token)
      .await
      .map_err(|err| {
        error!("Error inserting session token: {err}");
        Status::internal("Internal DB error")
      })?;

    info!("User {username} successfully registered");
    Ok(Response::new(RegisterResponse { session_token }))
  }
}

#[derive(Clone)]
struct AuthInterceptor {
  session_token_lifetime: Duration,
}

impl AuthInterceptor {
  fn new(settings: &Settings) -> Self {
    Self {
      session_token_lifetime: Duration::from_secs(settings.auth.session_token_lifetime_seconds),
    }
  }
}

#[async_trait::async_trait]
impl RequestInterceptor for AuthInterceptor {
  async fn intercept(
    &self,
    mut req: tonic::codegen::http::Request<Body>,
  ) -> Result<tonic::codegen::http::Request<Body>, Status> {
    let token = match req.headers().get("authorization") {
      Some(token) => token
        .to_str()
        .map_err(|_| Status::unauthenticated("Invalid token"))?,
      None => return Err(Status::unauthenticated("Missing `authorization` header")),
    };

    let user_id = match validate_session_token(token, self.session_token_lifetime).await? {
      Some(user_id) => user_id,
      None => return Err(Status::unauthenticated("Invalid session token")),
    };

    req.extensions_mut().insert(UserCredentials { user_id });

    Ok(req)
  }
}

pub async fn start_server(settings: &Settings) -> BootstrapResult<()> {
  let svc = MinePrivateServer {};
  let private_service = MinePrivateServiceServer::new(svc)
    .accept_compressed(CompressionEncoding::Gzip)
    .send_compressed(CompressionEncoding::Gzip)
    .max_decoding_message_size(256 * 1024 * 1024);
  let public_service = MinePublicServiceServer::new(MinePublicServer {});

  let addr = format!("0.0.0.0:{}", settings.server.port)
    .parse()
    .expect("Failed to parse address");
  info!("Starting gRPC server on {addr}");

  let trace_layer = TraceLayer::new_for_grpc();

  let cors_layer = CorsLayer::new()
    .allow_origin(AllowOrigin::mirror_request())
    .allow_methods(AllowMethods::mirror_request())
    .allow_headers(tower_http::cors::Any)
    .expose_headers(tower_http::cors::Any);

  tonic::transport::Server::builder()
    .accept_http1(true)
    .timeout(Duration::from_secs(600))
    .layer(cors_layer)
    .layer(trace_layer)
    .layer(GrpcWebLayer::new())
    .add_service(InterceptorFor::new(
      private_service,
      AuthInterceptor::new(settings),
    ))
    .add_service(public_service)
    .serve(addr)
    .await?;
  Ok(())
}
