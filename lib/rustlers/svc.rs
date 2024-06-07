use {
    super::{
        rustler::{Rustler, Ticker},
        rustlerjar::RustlerJar,
        MarketHourType,
    },
    crate::{
        bus::PublisherTrait,
        entities::{market, sea_orm::DatabaseConnection, ticker},
        rustlers::Quote,
    },
    eyre::Result,
    lool::{
        fail,
        logger::{info, warn},
        sched::{
            recur, ruleset, scheduler::tokio::Scheduler, utils::parse_time, RecurrenceRuleSet,
            SchedulingRule,
        },
    },
    std::sync::Arc,
    tokio::sync::{mpsc::Sender, Mutex},
};

/// #### 🐎 » Rustler Message
pub enum RustlerMsg {
    QuoteMsg(Quote),
}

/// #### 🐎 » create a quote message
#[inline]
pub fn quote(
    id: String,
    market: String,
    price: f64,
    change_percent: f64,
    time: i64,
    market_hours: MarketHourType,
) -> RustlerMsg {
    RustlerMsg::QuoteMsg(Quote {
        id,
        market,
        price,
        change_percent,
        time,
        market_hours,
    })
}

/// #### 🐎 » create a quote message from a `Quote`
#[inline]
pub fn to_msg(quote: Quote) -> RustlerMsg {
    RustlerMsg::QuoteMsg(quote)
}

/// #### 🐎 » Rustlers Service
///
/// `RustlersSvc` is a service that manages the rustlers and orchestrates their executions.
pub struct RustlersSvc<P>
where
    P: PublisherTrait<Quote> + Send + Sync + 'static + Clone,
{
    market_svc: market::Service,
    sched: Scheduler,
    rustlers: RustlerJar,
    publisher: P,
}

impl<Publisher> RustlersSvc<Publisher>
where
    Publisher: PublisherTrait<Quote> + Send + Sync + 'static + Clone,
{
    /// #### 🐎 » create service
    ///
    /// creates a new instance of the `RustlersSvc`
    ///
    /// **Arguments**
    /// - `conn` - the database connection that will be used to get market and tickers data
    /// - `rustlers` - the rustlers to be used by the service
    ///
    /// **Returns**
    /// the created `RustlersSvc` instance
    pub async fn new(conn: DatabaseConnection, rustlers: RustlerJar, publisher: Publisher) -> Self {
        let market_svc = market::Service::new(conn).await;
        let sched = Scheduler::new();

        Self {
            market_svc,
            rustlers,
            sched,
            publisher,
        }
    }

    /// #### 🐎 » start rustlers
    ///
    /// gets market data from the the database and starts
    /// the corresponding rustler for each market
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting rustlers");
        let markets = self.market_svc.get_all_with_tickers().await?;

        if !markets.is_empty() {
            let (sender, mut receiver) = tokio::sync::mpsc::channel(100);

            for (market, tickers) in markets {
                self.schedule_rustler_for((market, tickers), sender.clone()).await?;
            }

            // NOTE: if we wanted to stop all the rustlers for good for some reason, we should
            // use a select! instead and listen for a stop signal coming from somewhere
            let mut publisher = self.publisher.clone();
            while let Some(msg) = receiver.recv().await {
                match msg {
                    RustlerMsg::QuoteMsg(quote) => publisher.publish(quote).await?,
                }
            }

            fail!("Rustlers stopped")
        } else {
            fail!("No markets found")
        }
    }

    /// #### 🐎 » restart rustlers
    ///
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
        sender: Sender<RustlerMsg>,
    ) -> Result<()> {
        let (market, tickers) = market;
        let tickers: Vec<Ticker> = tickers.into_iter().map(|t| Ticker::from(&t, &market)).collect();

        let rules = self.get_schedule_rules_for(&market)?;
        let rustler = self.rustlers.get(&market);

        if let Some(rustler) = rustler {
            {
                let mut rustler = rustler.lock().await;
                info!("Setting message sender for rustler '{}'", rustler.name());
                rustler.set_msg_sender(Some(sender))
            }

            let start_name = format!("start-rustler-{}", market.short_name);
            let end_name = format!("end-rustler-{}", market.short_name);

            if let Some((start, stop)) = &rules {
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
            } else {
                info!("No schedule rules found for market '{}'", market.short_name);
            }

            if should_be_running_now(rules) {
                info!("Starting '{start_name}' right away");
                Self::start_rustler_for(rustler.clone(), tickers).await;
            }

            Ok(())
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

/// creates a rule for the given time and offset
///
/// TODO: handle timezones
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

/// the operation to be performed on the time (addition or subtraction)
enum Op {
    Add,
    Sub,
}

/// checks if the rustler should be running now
fn should_be_running_now(rules: Option<(SchedulingRule, SchedulingRule)>) -> bool {
    if let Some((start, stop)) = rules {
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
    } else {
        // if there are no rules, it means the rustler should be running all the time
        true
    }
}
