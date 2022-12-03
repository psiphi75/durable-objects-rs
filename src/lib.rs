use serde::{Deserialize, Serialize};

use worker::*;

mod utils;

//
// Durable Object component
//

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    name: String,
    email: String,
}

impl User {
    pub fn new(name: String, email: String) -> User {
        User { name, email }
    }
}

#[durable_object]
pub struct Counter {
    // Create a ephemeral counter, this will be lost after around 30 seconds
    // of non-use, or each time project is published.
    calls: usize,

    // These two properties come with Durable Objects
    state: State,
    // env: Env,
}

#[durable_object]
impl DurableObject for Counter {
    fn new(state: State, _env: Env) -> Self {
        Self {
            calls: 0,
            state,
            // env,
        }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        self.calls += 1;

        // Retrieve from storage, otherwise create one if it doesn't exist.
        let mut count = if let Ok(cnt) = self.state.storage().get::<i16>("counter").await {
            cnt
        } else {
            0
        };

        let (loaded, user) = if let Ok(usr) = self.state.storage().get::<User>("user").await {
            (true, usr)
        } else {
            (
                false,
                User::new(String::from("Simon"), String::from("Simon@somewhere.com")),
            )
        };

        match req.path().as_str() {
            "/x/increment" => count += 1,
            "/x/decrement" => count -= 1,
            "/x/" => (),
            _ => return Response::error("Not found", 404),
        }

        // Save back into persistent storage.
        self.state.storage().put("counter", count).await?;
        self.state.storage().put("user", &user).await?;

        Response::ok(&format!(
            "counter: {}, path: {}, calls: {}, user: {:?} ({})",
            count,
            req.path(),
            self.calls,
            user,
            loaded
        ))
    }
}

//
// Worker component
//

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();
    console_log!(
        "{} {}, located at: {:?}, within: {}",
        req.method().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );

    let router = Router::new();

    router
        .get_async("/user", |req, ctx| async move {
            // Retrieve the durable object namesapce
            let namespace = ctx.durable_object("USER")?;
            // Look up a specific stun instance of the durable object
            let stub = namespace.id_from_name("B")?.get_stub()?;
            // Forward the request to the Durable Object and return it's result
            stub.fetch_with_str(req.url()?.as_str()).await
        })
        .get_async("/x/*path", |req, ctx| async move {
            // Retrieve the durable object namesapce
            let namespace = ctx.durable_object("COUNTER")?;

            // Look up a specific stun instance of the durable object
            let stub = namespace.id_from_name("A")?.get_stub()?;

            // Forward the request to the Durable Object and return it's result
            stub.fetch_with_str(req.url()?.as_str()).await
        })
        .run(req, env)
        .await
}
