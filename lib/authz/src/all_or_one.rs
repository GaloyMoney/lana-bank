#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AllOrOne<T> {
    All,
    ById(T),
}

impl<T> std::str::FromStr for AllOrOne<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "*" => Ok(AllOrOne::All),
            _ => {
                let id = T::from_str(s).map_err(|e| format!("Invalid ID: {e}"))?;
                Ok(AllOrOne::ById(id))
            }
        }
    }
}

impl<T> std::fmt::Display for AllOrOne<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AllOrOne::All => write!(f, "*"),
            AllOrOne::ById(id) => write!(f, "{id}"),
        }
    }
}
