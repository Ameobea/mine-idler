use std::pin::Pin;

use futures::Stream;
use tonic::{Request, Response, Status};

use crate::protos::{mine_service_server::MineService, StartMiningRequest, StartMiningResponse};

struct MineServer {}

#[tonic::async_trait]
impl MineService for MineServer {
  type StartMiningStream =
    Pin<Box<dyn Stream<Item = Result<StartMiningResponse, Status>> + Send + Sync + 'static>>;

  async fn start_mining(
    &self,
    req: Request<StartMiningRequest>,
  ) -> Result<Response<Self::StartMiningStream>, Status> {
    Err(Status::unimplemented("Not yet implemented"))
  }
}
