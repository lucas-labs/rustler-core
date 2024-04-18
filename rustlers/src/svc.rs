use entities::{market, sea_orm::DatabaseConnection, ticker};
use eyre::{Ok, Result};
use lool::{
    logger::{info, warn},
    s,
    sched::{
        recur, ruleset, scheduler::tokio::Scheduler, utils::parse_time, RecurrenceRuleSet,
        SchedulingRule,
    },
};
use std::collections::HashMap;

use crate::rustlers::{Rustler, Ticker};

// interface MarketExecData {
//     entity: MarketModel;
//     startJob?: Job;
//     stopJob?: Job;
// }

// interface ScrapperData {
//     markets?: MarketExecData[];
//     // subscription?: Subscription
// }]]

struct MarketExecData {
    // entity: MarketModel,
    // startJob?: Job,
    // stopJob?: Job,
}

type RustlerFactory = Box<dyn Send + Sync + FnMut(&market::Model) -> Option<Box<dyn Rustler>>>;

pub struct RustlersSvc {
    market_svc: market::Service,
    factory: RustlerFactory,
}

impl RustlersSvc {
    pub async fn new(conn: DatabaseConnection) -> Self {
        let market_svc = market::Service::new(conn).await;
        let factory = |_mkt: &market::Model| None;

        Self {
            market_svc,
            factory: Box::new(factory),
        }
    }

    /// gets market data from the the database and starts
    /// the corresponding rustler for each market
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting rustlers");
        let markets = self.market_svc.get_all_with_tickers().await?;

        for (market, tickers) in markets {
            self.start_rustler_for((market, tickers)).await?;
        }

        Ok(())
    }

    /// stops all rustlers and then starts them again
    pub async fn restart(&self) -> Result<()> {
        todo!()
    }

    /// gets the corresponding rustler for the given market and starts it
    ///
    /// depending on the market configuraation, the rustler might be started
    /// immediately or its start might be scheduled for a later time
    ///
    /// this function also schedules the stop of the rustler at the end of the market
    /// trading hours if the market is configured to stop at a specific time
    async fn start_rustler_for(
        &mut self,
        market: (market::Model, Vec<ticker::Model>),
    ) -> Result<()> {
        let (market, tickers) = market;
        let rules = self.get_schedule_rules_for(&market)?;
        let rustler = (self.factory)(&market);

        if let Some(rustler) = rustler {
            if let Some((start, end)) = rules {
                let mut sched = Scheduler::new();
                let start_name = format!("start-{}", market.short_name);
                let end_name = format!("end-{}", market.short_name);

                let start_job = sched.schedule(start_name, || async move {}, start).await;
                let end_job = sched.schedule(end_name, || async move {}, end).await;

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

    /// get the rustler according to the market
    fn get_rustler_for(&mut self, market: &market::Model) -> Option<Box<dyn Rustler>> {
        let scrapper = (self.factory)(market);


        scrapper
    }

    /// starts a rustler by adding the tickers to it
    async fn start_rustler(
        rustler: &mut Box<dyn Rustler>,
        market: market::Model,
        tickers: Vec<ticker::Model>,
    ) -> Result<()> {
        if tickers.len() > 0 {
            let tickers: Vec<Ticker> =
                tickers.into_iter().map(|t| Ticker::from(&t, &market)).collect();

            rustler.add(tickers).await?;
        }

        Ok(())
    }

    /// stops a rustler by deleting all its tickers
    async fn stop_rustler(
        rustler: &mut Box<dyn Rustler>,
        market: market::Model,
        tickers: Vec<ticker::Model>,
    ) -> Result<()> {
        if tickers.len() > 0 {
            let tickers: Vec<Ticker> =
                tickers.into_iter().map(|t| Ticker::from(&t, &market)).collect();

            rustler.delete(tickers).await?;
        }

        Ok(())
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
