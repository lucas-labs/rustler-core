use {
    entities::{market, sea_orm::DatabaseConnection, ticker},
    eyre::Result,
    lool::{
        logger::{info, warn},
        sched::{
            recur, ruleset, scheduler::tokio::Scheduler, utils::parse_time, RecurrenceRuleSet,
            SchedulingRule,
        },
    },
    std::sync::Arc,
    tokio::sync::Mutex,
};

use crate::{
    rustlerjar::RustlerJar,
    rustlers::{Rustler, Ticker},
};

// interface MarketExecData {
//     entity: MarketModel;
//     startJob?: Job;
//     stopJob?: Job;
// }

// interface ScrapperData {
//     markets?: MarketExecData[];
//     // subscription?: Subscription
// }]]

// struct MarketExecData {
// entity: MarketModel,
// startJob?: Job,
// stopJob?: Job,
// }

pub struct RustlersSvc {
    market_svc: market::Service,
    sched: Scheduler,
    rustlers: RustlerJar,
}

impl RustlersSvc {
    /// creates a new instance of the `RustlersSvc`
    pub async fn new(conn: DatabaseConnection, rustlers: RustlerJar) -> Self {
        let market_svc = market::Service::new(conn).await;
        let sched = Scheduler::new();

        Self {
            market_svc,
            rustlers,
            sched,
        }
    }

    /// gets market data from the the database and starts
    /// the corresponding rustler for each market
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting rustlers");
        let markets = self.market_svc.get_all_with_tickers().await?;

        for (market, tickers) in markets {
            self.schedule_rustler_for((market, tickers)).await?;
        }

        Ok(())
    }

    /// stops all rustlers and then starts them again
    pub async fn restart(&self) -> Result<()> {
        // TODO: restart all rustlers; this method should clear everything we set up about the
        //       rustlers and then call `start` again. here we will need access to the job handlers
        //       which we are not storing right now, so we're going to need to store them in the
        //       `RustlersSvc` struct

        todo!()
    }

    /// gets the right rustler for the given market and starts it
    ///
    /// depending on the market configuraation, the rustler might be started
    /// immediately or its start might be scheduled for a later time
    ///
    /// this function also schedules the stop of the rustler at the end of the market
    /// trading hours if the market is configured to stop at a specific time
    async fn schedule_rustler_for(
        &mut self,
        market: (market::Model, Vec<ticker::Model>),
    ) -> Result<()> {
        let (market, tickers) = market;
        let tickers: Vec<Ticker> = tickers.into_iter().map(|t| Ticker::from(&t, &market)).collect();

        let rules = self.get_schedule_rules_for(&market)?;
        let rustler = self.rustlers.get(&market);

        if let Some(rustler) = rustler {
            if let Some((start, stop)) = rules {
                let start_name = format!("start-rustler-{}", market.short_name);
                let end_name = format!("end-rustler-{}", market.short_name);

                // TODO: we will need to store the job handlers in the `RustlersSvc` struct
                //       so that we can stop them when we need to restart the rustlers

                let start_job = self
                    .sched
                    .schedule_fut(
                        start_name.to_owned(),
                        Self::start_rustler_for(rustler.clone(), tickers.clone()),
                        start.clone(),
                    )
                    .await;

                let end_job = self
                    .sched
                    .schedule_fut(
                        end_name.to_owned(),
                        Self::stop_rustler_for(rustler.clone(), tickers.clone()),
                        stop.clone(),
                    )
                    .await;

                info!(
                    "Scheduled next execution for start job {start_name} for market '{}' at {:?}",
                    market.short_name,
                    start_job.get_next_run()
                );
                info!(
                    "Scheduled next execution for stop job {end_name} for market '{}' at {:?}",
                    market.short_name,
                    end_job.get_next_run()
                );

                if should_be_running_now(start, stop) {
                    info!("Starting '{start_name}' right away");
                    Self::start_rustler_for(rustler.clone(), tickers).await;
                }

                Ok(())
            } else {
                warn!("No schedule rules found for market '{}'", market.short_name);
                Ok(())
            }
        } else {
            warn!("No rustler found for market '{}'", market.short_name);
            Ok(())
        }
    }

    /// creates schedule rules for the given market
    fn get_schedule_rules_for(
        &self,
        mkt: &market::Model,
    ) -> Result<Option<(SchedulingRule, SchedulingRule)>> {
        if mkt.open_time.is_none() || mkt.close_time.is_none() {
            return Ok(None);
        }

        let open_time = parse_time(&mkt.open_time.clone().unwrap())?;
        let close_time = parse_time(&mkt.close_time.clone().unwrap())?;
        let pre_offset = mkt.pre_market_offset.unwrap_or(0);
        let post_offset = mkt.post_market_offset.unwrap_or(0);

        let mut rule = ruleset();
        rule.from_to_dow(mkt.opens_from.unwrap_or(0), mkt.opens_till.unwrap_or(0));

        let start_rule = make_rule(&rule, open_time, pre_offset, Op::Sub);
        let stop_rule = make_rule(&rule, close_time, post_offset, Op::Add);

        Ok(Some((recur(&start_rule), recur(&stop_rule))))
    }

    /// starts a rustler by adding the tickers to it
    async fn start_rustler_for(rustler: Arc<Mutex<Box<dyn Rustler>>>, tickers: Vec<Ticker>) {
        let mut rustler = rustler.lock().await;
        match rustler.start().await {
            Ok(()) => {
                if !tickers.is_empty() {
                    info!("Rustler {} started for market", rustler.name());

                    match rustler.add(&tickers).await {
                        Ok(()) => info!(
                            "Tickers {:?} added to rustler '{}'",
                            tickers,
                            rustler.name()
                        ),
                        Err(e) => warn!(
                            "Failed to add tickers to rustler '{}': {}",
                            rustler.name(),
                            e
                        ),
                    }
                }
            }
            Err(e) => warn!("Failed to start rustler '{}': {}", rustler.name(), e),
        };
    }

    /// stops a rustler for the given market/tickers
    ///
    /// if the rustler is being used by other markets, or the ticker list does not contain
    /// all the tickers that the rustler is using for the given market, the rustler will not
    /// be stopped, but will stop gathering data for the given tickers.
    async fn stop_rustler_for(rustler: Arc<Mutex<Box<dyn Rustler>>>, tickers: Vec<Ticker>) {
        let mut rustler = rustler.lock().await;

        if !tickers.is_empty() {
            // we delete the tickers from the rustler, but it will still be running if
            // there are other markets using the same rustler.
            match rustler.delete(&tickers).await {
                Ok(()) => info!(
                    "Tickers {:?} removed from rustler '{}'",
                    tickers,
                    rustler.name()
                ),
                Err(e) => warn!(
                    "Failed to remove tickers from rustler '{}': {}",
                    rustler.name(),
                    e
                ),
            }
        }
    }
}

fn make_rule(
    base: &RecurrenceRuleSet,
    time: (u32, u32, u32),
    offset: u32,
    op: Op,
) -> RecurrenceRuleSet {
    let (h, m, s) = time;
    let h = match op {
        Op::Add => h.saturating_add(offset),
        Op::Sub => h.saturating_sub(offset),
    };

    let mut rule = base.clone();
    rule.at_time(h, m, s);
    rule
}

enum Op {
    Add,
    Sub,
}

/// checks if the rustler should be running now
fn should_be_running_now(start: SchedulingRule, stop: SchedulingRule) -> bool {
    let now = chrono::Local::now();

    let start_date = start.next_from(now);
    let stop_date = stop.next_from(now);

    // if start date is Some in the past and stop_date is None, we should be running

    if start_date.is_some() && stop_date.is_none() {
        return true;
    }

    match (start_date, stop_date) {
        (Some(start), Some(stop)) => stop < start && now < stop,
        _ => true,
    }
}
