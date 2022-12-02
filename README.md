# Cloudflare Durable Object example in Rust

A very simple example of Cloudflare Durable Objects in Rust.  This uses the
worker-rs crate provided by Cloudflare and is very much like the equivalent
JavaScript example.

## Usage

Edit the `wrangler.toml` file and set the `account_id` value to the one for your
Cloudflare Workers account.

Then run the following:

```sh
wrangler publish
```

Then navigate to the URL that is published.  You can try the following
URLs:

- https://{your-host}/decrement
- https://{your-host}/increment
- https://{your-host}/
