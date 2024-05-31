use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};

pub fn create_resolver() -> Result<TokioAsyncResolver, Box<dyn std::error::Error>> {
    let resolver = TokioAsyncResolver::tokio(
        ResolverConfig::default(),
        ResolverOpts::default(),
    )?;
    Ok(resolver)
}
