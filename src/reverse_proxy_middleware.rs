use async_trait::async_trait;
use tide::{Middleware, Next, Request};

pub struct ReverseProxyMiddleware {}

#[async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for ReverseProxyMiddleware {
    async fn handle(&self, mut req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let ll_req: &mut tide::http::Request = req.as_mut();
        let forwarded = tide::http::proxies::Forwarded::from_headers(ll_req)?.clone();

        if let Some(forwarded) = forwarded {
            // here we clone stuff, because we need to mutable borrow the ll_req (forwarded object keeps Cow refs)
            let host = forwarded.host().map(|h| h.to_string());
            let proto = forwarded.proto().map(|p| p.to_string());

            let url = ll_req.url_mut();
            if let Some(host) = host {
                url.set_host(Some(&host))?;
            }
            if let Some(proto) = proto {
                url.set_scheme(&proto).unwrap(); // kinda wonky, but dunno what error should I return
            }
        }
        Ok(next.run(req).await)
    }
}
