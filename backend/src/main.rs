use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, guard, web};
use async_graphql::{
    Context, EmptySubscription, Enum, ID, InputObject, Object, Schema, SimpleObject,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use snurr::Process;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// ================= Data Models =================

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct User {
    pub id: ID,
    pub username: String,
    pub balance: f64,
    pub total_pnl: f64,
    pub win_rate: f64,
    pub followers_count: i32,
    pub is_trader: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Enum)]
pub enum TradeDirection {
    Long,
    Short,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Enum)]
pub enum TradeStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Trade {
    pub id: ID,
    pub trader_id: ID,
    pub symbol: String,
    pub direction: TradeDirection,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub quantity: f64,
    pub pnl: Option<f64>,
    pub status: TradeStatus,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct CopyRelation {
    pub id: ID,
    pub follower_id: ID,
    pub trader_id: ID,
    pub copy_ratio: f64,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct CopiedTrade {
    pub id: ID,
    pub original_trade_id: ID,
    pub follower_id: ID,
    pub quantity: f64,
    pub pnl: Option<f64>,
    pub status: TradeStatus,
}

// ================= In-Memory Database =================

#[derive(Default)]
pub struct Database {
    pub users: HashMap<String, User>,
    pub trades: HashMap<String, Trade>,
    pub copy_relations: HashMap<String, CopyRelation>,
    pub copied_trades: HashMap<String, CopiedTrade>,
}

pub type DbPool = Arc<RwLock<Database>>;

// Initialize database with sample users and trades
fn init_sample_data(db: &mut Database) {
    // Traders
    let trader1 = User {
        id: ID("trader1".to_string()),
        username: "AlphaTrader".to_string(),
        balance: 100000.0,
        total_pnl: 15420.50,
        win_rate: 0.72,
        followers_count: 156,
        is_trader: true,
        created_at: Utc::now(),
    };
    let trader2 = User {
        id: ID("trader2".to_string()),
        username: "CryptoKing".to_string(),
        balance: 250000.0,
        total_pnl: 42350.0,
        win_rate: 0.68,
        followers_count: 312,
        is_trader: true,
        created_at: Utc::now(),
    };
    // Regular user
    let user1 = User {
        id: ID("user1".to_string()),
        username: "NewInvestor".to_string(),
        balance: 10000.0,
        total_pnl: 520.0,
        win_rate: 0.65,
        followers_count: 0,
        is_trader: false,
        created_at: Utc::now(),
    };

    db.users.insert(trader1.id.to_string(), trader1);
    db.users.insert(trader2.id.to_string(), trader2);
    db.users.insert(user1.id.to_string(), user1);

    // Trades
    let trade1 = Trade {
        id: ID("trade1".to_string()),
        trader_id: ID("trader1".to_string()),
        symbol: "BTC/USD".to_string(),
        direction: TradeDirection::Long,
        entry_price: 42500.0,
        exit_price: None,
        quantity: 0.5,
        pnl: None,
        status: TradeStatus::Open,
        created_at: Utc::now(),
        closed_at: None,
    };
    let trade2 = Trade {
        id: ID("trade2".to_string()),
        trader_id: ID("trader2".to_string()),
        symbol: "ETH/USD".to_string(),
        direction: TradeDirection::Long,
        entry_price: 2250.0,
        exit_price: Some(2380.0),
        quantity: 5.0,
        pnl: Some(650.0),
        status: TradeStatus::Closed,
        created_at: Utc::now(),
        closed_at: Some(Utc::now()),
    };

    db.trades.insert(trade1.id.to_string(), trade1);
    db.trades.insert(trade2.id.to_string(), trade2);
}

// ================= BPMN Workflow Contexts =================

#[derive(Default)]
pub struct TradeWorkflowCtx {
    pub trader_id: String,
    pub symbol: String,
    pub direction: String,
    pub entry_price: f64,
    pub quantity: f64,
    pub trade_id: String,
    pub is_valid: bool,
    pub error: Option<String>,
    pub db: Option<DbPool>,
}

#[derive(Default)]
pub struct CopyWorkflowCtx {
    pub follower_id: String,
    pub trader_id: String,
    pub copy_ratio: f64,
    pub relation_id: String,
    pub is_valid: bool,
    pub error: Option<String>,
    pub db: Option<DbPool>,
}

// ================= BPMN Workflow Execution =================

// Execute Create Trade workflow using BPMN
pub fn execute_create_trade(db: DbPool, input: &CreateTradeInput) -> Result<Trade, String> {
    println!("🔄 BPMN: Starting Create Trade workflow");

    let process = Process::<TradeWorkflowCtx>::new("bpmn/create_trade.bpmn")
        .map_err(|e| format!("BPMN parse error: {:?}", e))?
        // Validate trade input (quantity and entry price)
        .task("Validate Trade Input", |ctx| {
            println!("  📋 Task: Validate Trade Input");
            let mut guard = ctx.lock().unwrap();
            if guard.quantity <= 0.0 {
                guard.is_valid = false;
                guard.error = Some("Invalid quantity".to_string());
            } else if guard.entry_price <= 0.0 {
                guard.is_valid = false;
                guard.error = Some("Invalid price".to_string());
            } else {
                guard.is_valid = true;
            }
            println!(
                "    ✅ Validation: {}",
                if guard.is_valid { "PASSED" } else { "FAILED" }
            );
            None
        })
        // Conditional gateway: proceed only if input is valid
        .exclusive("Is Valid", |ctx| {
            let guard = ctx.lock().unwrap();
            if guard.is_valid { "Yes" } else { "No" }.into()
        })
        // Create trade record in DB
        .task("Create Trade Record", |ctx| {
            println!("  💾 Task: Create Trade Record");
            let mut guard = ctx.lock().unwrap();
            let trade_id = Uuid::new_v4().to_string();
            let direction = if guard.direction == "Long" {
                TradeDirection::Long
            } else {
                TradeDirection::Short
            };
            let trade = Trade {
                id: ID(trade_id.clone()),
                trader_id: ID(guard.trader_id.clone()),
                symbol: guard.symbol.clone(),
                direction,
                entry_price: guard.entry_price,
                exit_price: None,
                quantity: guard.quantity,
                pnl: None,
                status: TradeStatus::Open,
                created_at: Utc::now(),
                closed_at: None,
            };
            if let Some(ref db) = guard.db {
                db.write().trades.insert(trade_id.clone(), trade);
            }
            guard.trade_id = trade_id.clone();
            println!("    ✅ Trade created: {}", trade_id);
            None
        })
        // Copy trade to active followers
        .task("Copy Trade To Followers", |ctx| {
            println!("  👥 Task: Copy Trade To Followers");
            let guard = ctx.lock().unwrap();
            if let Some(ref db) = guard.db {
                let mut db_lock = db.write();
                let followers: Vec<CopyRelation> = db_lock
                    .copy_relations
                    .values()
                    .filter(|r| r.trader_id.to_string() == guard.trader_id && r.active)
                    .cloned()
                    .collect();
                let count = followers.len();
                for relation in followers {
                    let copied_trade_id = Uuid::new_v4().to_string();
                    let copied_trade = CopiedTrade {
                        id: ID(copied_trade_id.clone()),
                        original_trade_id: ID(guard.trade_id.clone()),
                        follower_id: relation.follower_id,
                        quantity: guard.quantity * relation.copy_ratio,
                        pnl: None,
                        status: TradeStatus::Open,
                    };
                    db_lock.copied_trades.insert(copied_trade_id, copied_trade);
                }
                println!("    ✅ Copied to {} followers", count);
            }
            None
        })
        .build()
        .map_err(|e| format!("BPMN build error: {:?}", e))?;

    // Prepare workflow context
    let ctx = TradeWorkflowCtx {
        trader_id: input.trader_id.to_string(),
        symbol: input.symbol.clone(),
        direction: format!("{:?}", input.direction),
        entry_price: input.entry_price,
        quantity: input.quantity,
        trade_id: String::new(),
        is_valid: false,
        error: None,
        db: Some(db.clone()),
    };

    // Execute workflow
    let result = process
        .run(ctx)
        .map_err(|e| format!("Workflow error: {:?}", e))?;
    println!("✅ BPMN: Workflow completed");

    // Retrieve created trade from DB
    if !result.trade_id.is_empty() {
        let db_lock = db.read();
        if let Some(trade) = db_lock.trades.get(&result.trade_id) {
            return Ok(trade.clone());
        }
    }

    Err(result
        .error
        .unwrap_or_else(|| "Trade creation failed".to_string()))
}

// Execute Copy Trader workflow using BPMN
pub fn execute_copy_trader(db: DbPool, input: &CopyTraderInput) -> Result<CopyRelation, String> {
    println!("🔄 BPMN: Starting Copy Trader workflow");
    let bpmn = "./bpmn/copy_trader.bpmn";
    let process = Process::<CopyWorkflowCtx>::new(bpmn)
        .map_err(|e| format!("BPMN parse error: {:?}", e))?
        // Validate copy request (ratio 0.01..1.0)
        .task("Validate Copy Request", |ctx| {
            println!("  📋 Task: Validate Copy Request");
            let mut guard = ctx.lock().unwrap();
            if guard.copy_ratio < 0.01 || guard.copy_ratio > 1.0 {
                guard.is_valid = false;
                guard.error = Some("Invalid copy ratio".to_string());
            } else {
                guard.is_valid = true;
            }
            println!(
                "    ✅ Validation: {}",
                if guard.is_valid { "PASSED" } else { "FAILED" }
            );
            None
        })
        // Conditional gateway: proceed only if valid
        .exclusive("Is Valid", |ctx| {
            let guard = ctx.lock().unwrap();
            if guard.is_valid { "Yes" } else { "No" }.into()
        })
        // Create copy relation in DB
        .task("Create Copy Relation", |ctx| {
            println!("  💾 Task: Create Copy Relation");
            let mut guard = ctx.lock().unwrap();
            let relation_id = Uuid::new_v4().to_string();
            let relation = CopyRelation {
                id: ID(relation_id.clone()),
                follower_id: ID(guard.follower_id.clone()),
                trader_id: ID(guard.trader_id.clone()),
                copy_ratio: guard.copy_ratio,
                active: true,
                created_at: Utc::now(),
            };
            if let Some(ref db) = guard.db {
                db.write()
                    .copy_relations
                    .insert(relation_id.clone(), relation);
            }
            guard.relation_id = relation_id.clone();
            println!("    ✅ Relation created: {}", relation_id);
            None
        })
        // Update trader's follower count
        .task("Update Follower Count", |ctx| {
            println!("  📊 Task: Update Follower Count");
            let guard = ctx.lock().unwrap();
            if let Some(ref db) = guard.db {
                let mut db_lock = db.write();
                if let Some(trader) = db_lock.users.get_mut(&guard.trader_id) {
                    trader.followers_count += 1;
                    println!("    ✅ Count: {}", trader.followers_count);
                }
            }
            None
        })
        .build()
        .map_err(|e| format!("BPMN build error: {:?}", e))?;

    // Prepare workflow context
    let ctx = CopyWorkflowCtx {
        follower_id: input.follower_id.to_string(),
        trader_id: input.trader_id.to_string(),
        copy_ratio: input.copy_ratio,
        relation_id: String::new(),
        is_valid: false,
        error: None,
        db: Some(db.clone()),
    };

    // Execute workflow
    let result = process
        .run(ctx)
        .map_err(|e| format!("Workflow error: {:?}", e))?;
    println!("✅ BPMN: Workflow completed");

    // Retrieve created relation from DB
    if !result.relation_id.is_empty() {
        let db_lock = db.read();
        if let Some(relation) = db_lock.copy_relations.get(&result.relation_id) {
            return Ok(relation.clone());
        }
    }

    Err(result.error.unwrap_or_else(|| "Copy failed".to_string()))
}

// ================= GraphQL Schema =================

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    // Fetch all traders
    async fn traders(&self, ctx: &Context<'_>) -> Vec<User> {
        ctx.data_unchecked::<DbPool>()
            .read()
            .users
            .values()
            .filter(|u| u.is_trader)
            .cloned()
            .collect()
    }

    // Fetch a specific user
    async fn user(&self, ctx: &Context<'_>, id: ID) -> Option<User> {
        ctx.data_unchecked::<DbPool>()
            .read()
            .users
            .get(&id.to_string())
            .cloned()
    }

    // Fetch all users
    async fn users(&self, ctx: &Context<'_>) -> Vec<User> {
        ctx.data_unchecked::<DbPool>()
            .read()
            .users
            .values()
            .cloned()
            .collect()
    }

    // Fetch trades, optionally filtered by trader
    async fn trades(&self, ctx: &Context<'_>, trader_id: Option<ID>) -> Vec<Trade> {
        let db = ctx.data_unchecked::<DbPool>().read();
        match trader_id {
            Some(id) => db
                .trades
                .values()
                .filter(|t| t.trader_id == id)
                .cloned()
                .collect(),
            None => db.trades.values().cloned().collect(),
        }
    }

    // Fetch all open trades
    async fn open_trades(&self, ctx: &Context<'_>) -> Vec<Trade> {
        ctx.data_unchecked::<DbPool>()
            .read()
            .trades
            .values()
            .filter(|t| t.status == TradeStatus::Open)
            .cloned()
            .collect()
    }

    // Fetch active copy relations for a follower
    async fn my_copy_relations(&self, ctx: &Context<'_>, follower_id: ID) -> Vec<CopyRelation> {
        ctx.data_unchecked::<DbPool>()
            .read()
            .copy_relations
            .values()
            .filter(|r| r.follower_id == follower_id && r.active)
            .cloned()
            .collect()
    }

    // Fetch copied trades for a follower
    async fn my_copied_trades(&self, ctx: &Context<'_>, follower_id: ID) -> Vec<CopiedTrade> {
        ctx.data_unchecked::<DbPool>()
            .read()
            .copied_trades
            .values()
            .filter(|t| t.follower_id == follower_id)
            .cloned()
            .collect()
    }
}

#[derive(InputObject)]
pub struct CreateTradeInput {
    pub trader_id: ID,
    pub symbol: String,
    pub direction: TradeDirection,
    pub entry_price: f64,
    pub quantity: f64,
}

#[derive(InputObject)]
pub struct CopyTraderInput {
    pub follower_id: ID,
    pub trader_id: ID,
    pub copy_ratio: f64,
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    // Create a new trade
    async fn create_trade(
        &self,
        ctx: &Context<'_>,
        input: CreateTradeInput,
    ) -> async_graphql::Result<Trade> {
        let db = ctx.data_unchecked::<DbPool>().clone();
        execute_create_trade(db, &input).map_err(async_graphql::Error::new)
    }

    // Close an existing trade
    async fn close_trade(&self, ctx: &Context<'_>, trade_id: ID, exit_price: f64) -> Option<Trade> {
        let mut db = ctx.data_unchecked::<DbPool>().write();
        if let Some(trade) = db.trades.get_mut(&trade_id.to_string()) {
            let pnl = match trade.direction {
                TradeDirection::Long => (exit_price - trade.entry_price) * trade.quantity,
                TradeDirection::Short => (trade.entry_price - exit_price) * trade.quantity,
            };
            trade.exit_price = Some(exit_price);
            trade.pnl = Some(pnl);
            trade.status = TradeStatus::Closed;
            trade.closed_at = Some(Utc::now());
            let closed = trade.clone();

            // Update copied trades for followers
            for ct in db
                .copied_trades
                .values_mut()
                .filter(|ct| ct.original_trade_id == trade_id)
            {
                ct.pnl = Some(match closed.direction {
                    TradeDirection::Long => (exit_price - closed.entry_price) * ct.quantity,
                    TradeDirection::Short => (closed.entry_price - exit_price) * ct.quantity,
                });
                ct.status = TradeStatus::Closed;
            }
            return Some(closed);
        }
        None
    }

    // Copy a trader
    async fn copy_trader(
        &self,
        ctx: &Context<'_>,
        input: CopyTraderInput,
    ) -> async_graphql::Result<CopyRelation> {
        let db = ctx.data_unchecked::<DbPool>().clone();
        execute_copy_trader(db, &input).map_err(async_graphql::Error::new)
    }

    // Stop copying a trader
    async fn stop_copying(&self, ctx: &Context<'_>, relation_id: ID) -> Option<CopyRelation> {
        let mut db = ctx.data_unchecked::<DbPool>().write();
        if let Some(rel) = db.copy_relations.get_mut(&relation_id.to_string()) {
            rel.active = false;
            let trader_id = rel.trader_id.to_string();
            let result = rel.clone();
            if let Some(trader) = db.users.get_mut(&trader_id) {
                trader.followers_count = (trader.followers_count - 1).max(0);
            }
            return Some(result);
        }
        None
    }

    // Register a new user
    async fn register_user(&self, ctx: &Context<'_>, username: String, is_trader: bool) -> User {
        let mut db = ctx.data_unchecked::<DbPool>().write();
        let user = User {
            id: ID(Uuid::new_v4().to_string()),
            username,
            balance: 10000.0,
            total_pnl: 0.0,
            win_rate: 0.0,
            followers_count: 0,
            is_trader,
            created_at: Utc::now(),
        };
        db.users.insert(user.id.to_string(), user.clone());
        user
    }
}

// ================= HTTP Handlers =================

// GraphQL endpoint
async fn graphql_handler(
    schema: web::Data<Schema<QueryRoot, MutationRoot, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

// GraphQL Playground
async fn graphql_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(async_graphql::http::playground_source(
            async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
        ))
}

// ================= Main Server =================

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 CopyTrade Backend + Snurr BPMN");
    println!("📋 Workflows: bpmn/create_trade.bpmn, bpmn/copy_trader.bpmn");
    println!("📊 Playground: http://localhost:8080/playground\n");

    // Initialize in-memory DB with sample data
    let mut db = Database::default();
    init_sample_data(&mut db);
    let db_pool: DbPool = Arc::new(RwLock::new(db));

    // Build GraphQL schema
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(db_pool)
        .finish();

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(web::Data::new(schema.clone()))
            .route("/graphql", web::post().to(graphql_handler))
            .route(
                "/graphql",
                web::get()
                    .guard(guard::Header("upgrade", "websocket"))
                    .to(graphql_handler),
            )
            .route("/playground", web::get().to(graphql_playground))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
