use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::state::StateBound;
use crate::utils::pinbox;

/// The context passed to the bot's prefix getter.
///
/// It contains the bot's state and the message event that triggered the execution of this prefix
/// getter.
#[derive(Clone)]
pub struct PrefixesContext<State = ()>
where
    State: StateBound,
{
    /// The bot's state.
    pub state: State,

    /// The message event that triggered the execution of this prefix getter.
    pub event: MessageCreate,
}

/// The result type all prefix getter function results get normalized to.
pub type PrefixesResult = Result<Vec<String>, Arc<dyn Error + Send + Sync>>;

/// Normalizes a prefix getter function result into [`PrefixesResult`].
pub trait IntoPrefixesResult {
    /// Normalizes a prefix getter function result into [`PrefixesResult`].
    ///
    /// Returns:
    /// [`PrefixesResult`] - The normalized result.
    fn into_prefixes_result(self) -> PrefixesResult;
}

impl IntoPrefixesResult for Vec<String> {
    fn into_prefixes_result(self) -> PrefixesResult {
        Ok(self)
    }
}

impl<T, E> IntoPrefixesResult for Result<T, E>
where
    T: Into<Vec<String>>,
    E: Error + Send + Sync + 'static,
{
    fn into_prefixes_result(self) -> PrefixesResult {
        match self {
            Ok(v) => Ok(v.into()),
            Err(e) => Err(Arc::new(e)),
        }
    }
}

pub trait Prefixes<State = ()>: Send + Sync
where
    State: StateBound,
{
    /// Gets the prefixes for the bot to listen for in a given context.
    ///
    /// Arguments:
    /// * `context` - The context passed to the prefix getter.
    ///
    /// Returns:
    /// * `Ok(Vec<String>)` - A vector of prefixes for the bot to listen for in the given context.
    /// * `Err(Arc<Err>)` - An error, if the prefixes fail to be gotten.
    fn get_prefixes(
        &self,
        context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = PrefixesResult> + Send + '_>>;
}

impl<State> Prefixes<State> for &str
where
    State: StateBound,
{
    fn get_prefixes(
        &self,
        _context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = PrefixesResult> + Send + '_>> {
        pinbox(Ok(vec![self.to_string()]))
    }
}

impl<State> Prefixes<State> for String
where
    State: StateBound,
{
    fn get_prefixes(
        &self,
        _context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = PrefixesResult> + Send + '_>> {
        pinbox(Ok(vec![self.clone()]))
    }
}

impl<State> Prefixes<State> for Vec<&str>
where
    State: StateBound,
{
    fn get_prefixes(
        &self,
        _context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = PrefixesResult> + Send + '_>> {
        pinbox(Ok(self.iter().map(|s| s.to_string()).collect()))
    }
}

impl<State> Prefixes<State> for Vec<String>
where
    State: StateBound,
{
    fn get_prefixes(
        &self,
        _context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = PrefixesResult> + Send + '_>> {
        pinbox(Ok(self.clone()))
    }
}

impl<State> Prefixes<State> for &[String]
where
    State: StateBound,
{
    fn get_prefixes(
        &self,
        _context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = PrefixesResult> + Send + '_>> {
        pinbox(Ok(self.to_vec()))
    }
}

impl<State> Prefixes<State> for &[&str]
where
    State: StateBound,
{
    fn get_prefixes(
        &self,
        _context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = PrefixesResult> + Send + '_>> {
        pinbox(Ok(self.iter().map(|s| s.to_string()).collect()))
    }
}

impl<State, const N: usize> Prefixes<State> for [&str; N]
where
    State: StateBound,
{
    fn get_prefixes(
        &self,
        _context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = PrefixesResult> + Send + '_>> {
        pinbox(Ok(self.iter().map(|s| s.to_string()).collect()))
    }
}

impl<State, Func, Fut, Res> Prefixes<State> for Func
where
    State: StateBound,
    Func: Fn(PrefixesContext<State>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoPrefixesResult,
{
    fn get_prefixes(
        &self,
        context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = PrefixesResult> + Send + '_>> {
        Box::pin(async move { (self)(context).await.into_prefixes_result() })
    }
}
