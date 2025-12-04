use salvo::prelude::*;
use std::sync::Arc;
use sea_orm::DatabaseConnection;
use super::JwtService;

pub struct DepsMiddleware {
    db: Arc<DatabaseConnection>,
    jwt_service: Arc<JwtService>,
}

#[async_trait]
impl Handler for DepsMiddleware {
    async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
        depot.insert("db", self.db.clone());
        depot.insert("jwt_service", self.jwt_service.clone());
        ctrl.call_next(req, depot, res).await;
    }
}

pub fn create_deps_middleware(db: Arc<DatabaseConnection>, jwt_service: Arc<JwtService>) -> DepsMiddleware {
    DepsMiddleware {
        db,
        jwt_service,
    }
}
