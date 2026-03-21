use anyhow::Result;

use crate::cli::TimeAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(client: &mut GraphQLClient, action: TimeAction, json: bool) -> Result<()> {
    match action {
        TimeAction::Get => {
            let vars = time_get::Variables {};
            let data = client.execute::<TimeGet>(vars).await?;
            render_time(&data.time, json)?;
        }
        TimeAction::AdvanceToNextEod => {
            let vars = time_advance_to_next_end_of_day::Variables {};
            let data = client.execute::<TimeAdvanceToNextEndOfDay>(vars).await?;
            render_time(&data.time_advance_to_next_end_of_day.time, json)?;
        }
    }

    Ok(())
}

fn render_time<T>(time: &T, json: bool) -> Result<()>
where
    T: serde::Serialize + TimeView,
{
    if json {
        output::print_json(time)?;
    } else {
        output::print_kv(&[
            ("Current Date", time.current_date()),
            ("Current Time", time.current_time()),
            ("Next EOD At", time.next_end_of_day_at()),
            ("Timezone", time.timezone()),
            ("End Of Day Time", time.end_of_day_time()),
            (
                "Can Advance To Next EOD",
                if time.can_advance_to_next_end_of_day() {
                    "true"
                } else {
                    "false"
                },
            ),
        ]);
    }

    Ok(())
}

trait TimeView {
    fn current_date(&self) -> &str;
    fn current_time(&self) -> &str;
    fn next_end_of_day_at(&self) -> &str;
    fn timezone(&self) -> &str;
    fn end_of_day_time(&self) -> &str;
    fn can_advance_to_next_end_of_day(&self) -> bool;
}

impl TimeView for time_get::TimeGetTime {
    fn current_date(&self) -> &str {
        &self.current_date
    }

    fn current_time(&self) -> &str {
        &self.current_time
    }

    fn next_end_of_day_at(&self) -> &str {
        &self.next_end_of_day_at
    }

    fn timezone(&self) -> &str {
        &self.timezone
    }

    fn end_of_day_time(&self) -> &str {
        &self.end_of_day_time
    }

    fn can_advance_to_next_end_of_day(&self) -> bool {
        self.can_advance_to_next_end_of_day
    }
}

impl TimeView
    for time_advance_to_next_end_of_day::TimeAdvanceToNextEndOfDayTimeAdvanceToNextEndOfDayTime
{
    fn current_date(&self) -> &str {
        &self.current_date
    }

    fn current_time(&self) -> &str {
        &self.current_time
    }

    fn next_end_of_day_at(&self) -> &str {
        &self.next_end_of_day_at
    }

    fn timezone(&self) -> &str {
        &self.timezone
    }

    fn end_of_day_time(&self) -> &str {
        &self.end_of_day_time
    }

    fn can_advance_to_next_end_of_day(&self) -> bool {
        self.can_advance_to_next_end_of_day
    }
}
