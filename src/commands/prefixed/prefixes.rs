use std::future::Future;
use std::pin::Pin;

use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::state::StateBound;
use crate::utils::pinbox;

/// The context passed to the bot's prefix getter.
///
/// It contains the bot's state and the message event that triggered the execution of this prefix
/// getter.
pub struct PrefixesContext<State = ()>
where
    State: StateBound,
{
    /// The bot's state.
    pub state: State,

    /// The message event that triggered the execution of this prefix getter.
    pub event: MessageCreate,
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
    /// [`Vec<String>`] - A vector of prefixes for the bot to listen for in the given context.
    fn get(
        &self,
        context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>>;
}

impl<State> Prefixes<State> for &str
where
    State: StateBound,
{
    fn get(
        &self,
        _context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        pinbox(vec![self.to_string()])
    }
}

impl<State> Prefixes<State> for String
where
    State: StateBound,
{
    fn get(
        &self,
        _context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        pinbox(vec![self.clone()])
    }
}

impl<State> Prefixes<State> for Vec<&str>
where
    State: StateBound,
{
    fn get(
        &self,
        _context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        pinbox(self.iter().map(|s| s.to_string()).collect())
    }
}

impl<State> Prefixes<State> for Vec<String>
where
    State: StateBound,
{
    fn get(
        &self,
        _context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        pinbox(self.clone())
    }
}

impl<State> Prefixes<State> for &[String]
where
    State: StateBound,
{
    fn get(
        &self,
        _context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        pinbox(self.to_vec())
    }
}

impl<State> Prefixes<State> for &[&str]
where
    State: StateBound,
{
    fn get(
        &self,
        _context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        pinbox(self.iter().map(|s| s.to_string()).collect())
    }
}

impl<State, const N: usize> Prefixes<State> for [&str; N]
where
    State: StateBound,
{
    fn get(
        &self,
        _context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        pinbox(self.iter().map(|s| s.to_string()).collect())
    }
}

impl<F, Fut, State> Prefixes<State> for F
where
    F: Fn(PrefixesContext<State>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Vec<String>> + Send + 'static,
    State: StateBound,
{
    fn get(
        &self,
        context: PrefixesContext<State>,
    ) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        Box::pin(async move { (self)(context).await })
    }
}
