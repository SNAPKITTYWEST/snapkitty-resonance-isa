/// kitty-swarm — Swarm Rule Governance Kernel (SRGK)
///
/// Pure Rust + async-nats. No TypeScript.
///
/// Connects to the existing SnapKitty NATS bus:
///   NATS_URL          env var (default: nats://localhost:4222)
///   NATS_AUTH_TOKEN   env var (matches existing .env)
///
/// Publishes:
///   kitty.swarm.rule.commit   — committed RuleTable as JSON
///   kitty.swarm.state.tick    — UMO waveform per tick
///
/// Subscribes:
///   kitty.swarm.proposal.*    — external agent proposals from other nodes

use std::time::Duration;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use tokio::time::sleep;
use futures_util::StreamExt;

// ── RULE TABLE ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleTable {
    pub trust:            f64,
    pub entropy_limit:    f64,
    pub resonance_weight: f64,
    pub mutation_rate:    f64,
    pub age:              u64,
    pub seal:             Option<String>,
}

impl Default for RuleTable {
    fn default() -> Self {
        Self { trust: 1.0, entropy_limit: 0.05, resonance_weight: 1.0,
               mutation_rate: 0.01, age: 0, seal: None }
    }
}

impl RuleTable {
    pub fn worm_seal(&self) -> String {
        let payload = format!("kitty.swarm.rule|τ={:.6}|ε={:.6}|ρ={:.6}|age={}",
            self.trust, self.entropy_limit, self.resonance_weight, self.age);
        hex::encode(Sha256::digest(payload.as_bytes()))
    }

    pub fn apl_execute(&self, state: &[f64]) -> f64 {
        let gate = if self.entropy_limit < 0.21 { 1.0 } else { 0.0 };
        state.iter().sum::<f64>() * self.trust * gate * self.resonance_weight
    }
}

// ── PROPOSAL ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub agent_id:        String,
    pub trust_delta:     f64,
    pub entropy_delta:   f64,
    pub resonance_delta: f64,
    pub weight:          f64,
}

// ── FIVE SWARM AGENTS ────────────────────────────────────────────────────────

fn proposals(rule: &RuleTable) -> Vec<Proposal> {
    let entropy_hot = rule.entropy_limit > 0.18;
    vec![
        Proposal { agent_id: "STABILITY".into(),
            trust_delta: 0.01, entropy_delta: -0.005, resonance_delta: 0.005,
            weight: if rule.trust < 0.8 { 2.0 } else { 1.0 } },
        Proposal { agent_id: "EXPLORATION".into(),
            trust_delta: -0.005, entropy_delta: 0.01, resonance_delta: 0.02, weight: 0.6 },
        Proposal { agent_id: "EFFICIENCY".into(),
            trust_delta: 0.005, entropy_delta: -0.01, resonance_delta: 0.01,
            weight: if rule.entropy_limit > 0.15 { 1.5 } else { 1.0 } },
        Proposal { agent_id: "GEOMETRY".into(),
            trust_delta: 0.002, entropy_delta: -0.002, resonance_delta: 0.03, weight: 1.0 },
        Proposal { agent_id: "RISK".into(),
            trust_delta:     if entropy_hot { 0.02 } else { 0.0 },
            entropy_delta:   if entropy_hot { -0.02 } else { 0.0 },
            resonance_delta: 0.0,
            weight:          if entropy_hot { 3.0 } else { 0.5 } },
    ]
}

// ── VOTE (weighted fold) ─────────────────────────────────────────────────────

fn vote(ps: &[Proposal]) -> Proposal {
    let w: f64 = ps.iter().map(|p| p.weight).sum();
    if w == 0.0 { return ps[0].clone(); }
    let (td, ed, rd) = ps.iter().fold((0.0, 0.0, 0.0), |a, p|
        (a.0 + p.trust_delta * p.weight,
         a.1 + p.entropy_delta * p.weight,
         a.2 + p.resonance_delta * p.weight));
    Proposal { agent_id: "SWARM".into(),
        trust_delta: td/w, entropy_delta: ed/w, resonance_delta: rd/w, weight: w }
}

// ── CONSTRAINT GATE ───────────────────────────────────────────────────────────

fn validate(rule: &RuleTable, p: &Proposal) -> bool {
    rule.entropy_limit + p.entropy_delta < 0.21 &&
    rule.trust         + p.trust_delta   > 0.7
}

// ── APPLY (immutable swap) ───────────────────────────────────────────────────

fn apply(rule: &RuleTable, p: &Proposal) -> RuleTable {
    let next = RuleTable {
        trust:            (rule.trust            + p.trust_delta).clamp(0.0, 1.0),
        entropy_limit:    (rule.entropy_limit    + p.entropy_delta).clamp(0.0, 1.0),
        resonance_weight: (rule.resonance_weight + p.resonance_delta).clamp(0.0, 2.0),
        mutation_rate:    rule.mutation_rate,
        age:              rule.age + 1,
        seal:             None,
    };
    RuleTable { seal: Some(next.worm_seal()), ..next }
}

// ── UMO GLYPH ────────────────────────────────────────────────────────────────

fn glyph(v: f64) -> char {
    match v.abs() {
        x if x > 0.9 => '☉', x if x > 0.7 => '◉',
        x if x > 0.5 => '◇', x if x > 0.3 => '▣',
        x if x > 0.1 => '▒', _             => '⛔',
    }
}

fn umo(rule: &RuleTable, t: f64) -> String {
    (0..12).map(|i| {
        let ti = t + i as f64 * 0.4;
        glyph((rule.trust * ti).sin() * (rule.resonance_weight * ti).cos()
              * (1.0 - rule.entropy_limit))
    }).collect()
}

// ── TICK ─────────────────────────────────────────────────────────────────────

fn tick(rule: &RuleTable, t: u64) -> (RuleTable, bool) {
    let ps   = proposals(rule);
    let v    = vote(&ps);
    let ok   = validate(rule, &v);
    let next = if ok { apply(rule, &v) } else { rule.clone() };
    let apl  = next.apl_execute(&[next.trust, next.resonance_weight]);
    let wave = umo(&next, t as f64 * 0.5);
    let seal = next.seal.as_deref().map(|s| &s[..10]).unwrap_or("none");
    println!("t={:<4} age={} τ={:.4} ε={:.4} ρ={:.4} apl={:.4} {} seal:{}… {}",
        t, next.age, next.trust, next.entropy_limit, next.resonance_weight,
        apl, if ok { "COMMIT" } else { "FROZEN" }, seal, wave);
    (next, ok)
}

// ── MAIN ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let url   = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".into());
    let token = std::env::var("NATS_AUTH_TOKEN").ok();

    println!("⟦ Ω ⟧ KITTY SWARM  url={url}");
    println!("{}", "─".repeat(80));

    // Connect to NATS bus (existing SnapKitty infra)
    let nc = {
        let opts = if let Some(t) = &token {
            async_nats::ConnectOptions::with_token(t.clone())
        } else {
            async_nats::ConnectOptions::new()
        };
        match opts.connect(&url).await {
            Ok(c)  => { println!("✓ NATS connected"); Some(c) }
            Err(e) => { println!("⚠ NATS unavailable ({e}) — standalone mode"); None }
        }
    };

    let mut rule = RuleTable::default();
    let mut t    = 0u64;

    // Spawn NATS proposal subscriber
    if let Some(nc) = &nc {
        let mut sub = nc.subscribe("kitty.swarm.proposal.*").await.unwrap();
        let nc2 = nc.clone();
        tokio::spawn(async move {
            while let Some(msg) = sub.next().await {
                if let Ok(p) = serde_json::from_slice::<Proposal>(&msg.payload) {
                    println!("  📡 external proposal: {}", p.agent_id);
                    // Forward to main tick via publish so it joins the vote
                    let _ = nc2.publish("kitty.swarm.proposal.external",
                        serde_json::to_string(&p).unwrap().into()).await;
                }
            }
        });
    }

    loop {
        let (next, committed) = tick(&rule, t);

        if let Some(nc) = &nc {
            if committed {
                if let Ok(json) = serde_json::to_string(&next) {
                    let _ = nc.publish("kitty.swarm.rule.commit", json.into()).await;
                }
            }
            let wave = umo(&next, t as f64 * 0.5);
            let _ = nc.publish("kitty.swarm.state.tick", wave.into()).await;
        }

        rule = next;
        t   += 1;
        sleep(Duration::from_millis(500)).await;
    }
}
