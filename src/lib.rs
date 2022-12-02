use worker::*;

mod utils;

//
// Durable Object component
//

#[durable_object]
pub struct Counter {
    // Create a ephemeral counter, this will be lost after around 30 seconds
    // of non-use, or each time project is published.
    calls: usize,

    // These two properties come with Durable Objects
    state: State,
    env: Env,
}

#[durable_object]
impl DurableObject for Counter {
    fn new(state: State, env: Env) -> Self {
        Self {
            calls: 0,
            state,
            env,
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

        match req.path().as_str() {
            "/increment" => count += 1,
            "/decrement" => count -= 1,
            "/" => (),
            _ => return Response::error("Not found", 404),
        }

        // Save back into persistent storage.
        self.state.storage().put("counter", count).await?;

        Response::ok(&format!(
            "counter: {}, path: {}, calls: {}",
            count,
            req.path(),
            self.calls
        ))
    }
}

//
// Worker component
//

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();

    let router = Router::new();

    router
        .on_async("/*path", |req, ctx| async move {
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
