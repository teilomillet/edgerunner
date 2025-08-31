use yew::prelude::*;
use yew::TargetCast;
use web_sys::{HtmlInputElement, HtmlSelectElement};

#[derive(Clone, Copy, PartialEq)]
enum OddsFormat {
    Decimal,
    American,
    Fractional,
}

impl OddsFormat {
    fn all() -> &'static [(Self, &'static str)] {
        &[
            (Self::Decimal, "Decimal"),
            (Self::American, "American"),
            (Self::Fractional, "Fractional"),
        ]
    }
}

#[derive(Clone, Copy, PartialEq)]
enum BetSide { OnEvent, OnOpposite }

#[derive(Clone, PartialEq)]
struct OutcomeRow { name: String, mkt: f64, yours: f64 }

#[function_component(App)]
fn app() -> Html {
    // Single bet state
    let market_prob = use_state(|| 60.0_f64); // % market thinks event happens
    let your_prob = use_state(|| 55.0_f64);   // % you think event happens
    let bet_side = use_state(|| BetSide::OnEvent);
    let odds_format = use_state(|| OddsFormat::Decimal);
    let odds_input = use_state(|| String::from("2.00"));
    let bankroll = use_state(|| String::from("1000"));

    // Multi-outcome state
    let outcomes = use_state(|| vec![
        OutcomeRow { name: "A".into(), mkt: 50.0, yours: 60.0 },
        OutcomeRow { name: "B".into(), mkt: 50.0, yours: 40.0 },
    ]);

    // Helpers
    let bankroll_val = || bankroll.trim().replace(',', "").parse::<f64>().unwrap_or(0.0);

    // Market price as odds: prefer explicit odds, else derive from market %
    let decimal_odds = {
        let s_current = (*odds_input).clone();
        let s = s_current.trim();
        let parsed = match *odds_format {
            OddsFormat::Decimal => s.parse::<f64>().ok(),
            OddsFormat::American => parse_american(s).map(|d| d),
            OddsFormat::Fractional => parse_fractional(s).map(|d| d),
        };
        match parsed {
            Some(d) => Some(d),
            None => {
                let pm = (*market_prob / 100.0).clamp(1e-9, 1.0 - 1e-9);
                let priced = match *bet_side { BetSide::OnEvent => pm, BetSide::OnOpposite => 1.0 - pm };
                Some(1.0 / priced)
            }
        }
    };

    // Computations
    let (kelly_f, full_bet, half_bet, quarter_bet, ev_per_unit, implied_prob, edge_prob) =
        if let Some(d) = decimal_odds {
            if d <= 1.0 {
                (0.0, 0.0, 0.0, 0.0, f64::NAN, f64::NAN, f64::NAN)
            } else {
                let p_event = *your_prob as f64 / 100.0;
                let p = match *bet_side { BetSide::OnEvent => p_event, BetSide::OnOpposite => 1.0 - p_event };
                let b = d - 1.0; // net profit per 1 staked
                let q = 1.0 - p;
                let f = ((b * p) - q) / b; // Kelly fraction
                let f = f.clamp(0.0, 1.0);
                let bank = bankroll_val();
                let ev = (p * b) - q; // EV per 1 staked
                let imp = 1.0 / d; // implied prob of the side being backed
                let edgep = p - imp; // your edge on the backed side
                (f, bank * f, bank * (f * 0.5), bank * (f * 0.25), ev, imp, edgep)
            }
        } else {
            (0.0, 0.0, 0.0, 0.0, f64::NAN, f64::NAN, f64::NAN)
        };

    // Handlers
    let on_market_prob_input = {
        let market_prob = market_prob.clone();
        Callback::from(move |e: InputEvent| {
            let target: HtmlInputElement = e.target_unchecked_into();
            let v = target.value().parse::<f64>().unwrap_or(0.0).clamp(0.0, 100.0);
            market_prob.set(v);
        })
    };
    let on_market_prob_slider = {
        let market_prob = market_prob.clone();
        Callback::from(move |e: InputEvent| {
            let target: HtmlInputElement = e.target_unchecked_into();
            let v = target.value().parse::<f64>().unwrap_or(0.0).clamp(0.0, 100.0);
            market_prob.set(v);
        })
    };
    let on_your_prob_input = {
        let your_prob = your_prob.clone();
        Callback::from(move |e: InputEvent| {
            let target: HtmlInputElement = e.target_unchecked_into();
            let v = target.value().parse::<f64>().unwrap_or(0.0).clamp(0.0, 100.0);
            your_prob.set(v);
        })
    };
    let on_odds_format_change = {
        let odds_format = odds_format.clone();
        let odds_input = odds_input.clone();
        Callback::from(move |e: Event| {
            let target: HtmlSelectElement = e.target_unchecked_into();
            let sel = target.value();
            let new_format = match sel.as_str() {
                "Decimal" => OddsFormat::Decimal,
                "American" => OddsFormat::American,
                _ => OddsFormat::Fractional,
            };
            // Convert current input to new format sensibly when possible
            let current = (*odds_input).clone();
            let new_input = match (parse_any(&current), new_format) {
                (Some(d), OddsFormat::Decimal) => format_decimal(d),
                (Some(d), OddsFormat::American) => format_american(d),
                (Some(d), OddsFormat::Fractional) => format_fractional(d),
                _ => current,
            };
            odds_format.set(new_format);
            odds_input.set(new_input);
        })
    };
    let on_odds_input = {
        let odds_input = odds_input.clone();
        Callback::from(move |e: InputEvent| {
            let target: HtmlInputElement = e.target_unchecked_into();
            odds_input.set(target.value());
        })
    };
    let on_bankroll_input = {
        let bankroll = bankroll.clone();
        Callback::from(move |e: InputEvent| {
            let target: HtmlInputElement = e.target_unchecked_into();
            bankroll.set(target.value());
        })
    };

    let on_bet_side_change = {
        let bet_side = bet_side.clone();
        Callback::from(move |e: Event| {
            let target: HtmlSelectElement = e.target_unchecked_into();
            let sel = target.value();
            let side = if sel == "Opposite" { BetSide::OnOpposite } else { BetSide::OnEvent };
            bet_side.set(side);
        })
    };

    // Derived presentation strings
    let dec_str = decimal_odds.map(format_decimal).unwrap_or_else(|| "—".to_string());
    let am_str = decimal_odds.map(format_american).unwrap_or_else(|| "—".to_string());
    let fr_str = decimal_odds.map(format_fractional).unwrap_or_else(|| "—".to_string());

    // Multi-outcome calculations (approx): independent Kelly then scale if needed
    let multi_calc = {
        let list = (*outcomes).clone();
        let mut rows: Vec<(OutcomeRow, f64, f64)> = Vec::new(); // (row, dec_odds, kelly)
        let mut sum_k = 0.0;
        for r in list.iter() {
            let pm = (r.mkt/100.0).clamp(1e-9, 1.0 - 1e-9);
            let d = 1.0/pm; let b = d - 1.0;
            let p = (r.yours/100.0).clamp(0.0, 1.0);
            let q = 1.0 - p;
            let f = (((b*p) - q) / b).clamp(0.0, 1.0);
            sum_k += f;
            rows.push((r.clone(), d, f));
        }
        let scale = if sum_k > 1.0 { 1.0/sum_k } else { 1.0 };
        (rows, scale, sum_k)
    };
    let (multi_rows, multi_scale, _multi_sumk) = multi_calc;
    let total_mkt: f64 = multi_rows.iter().map(|(r, _, _)| r.mkt).sum();
    let warn_market_sum = (total_mkt - 100.0).abs() > 0.5;

    // Add-outcome handler
    let on_add_outcome = {
        let outcomes = outcomes.clone();
        Callback::from(move |_| {
            let mut v = (*outcomes).clone();
            v.push(OutcomeRow{ name: format!("O{}", v.len()+1), mkt: 0.0, yours: 0.0 });
            outcomes.set(v);
        })
    };

    html! {
        <div class="container">
            <header>
                <h1>{"EdgeRunner"}</h1>
                <span class="pill">{"Kelly Calculator"}</span>
            </header>

            <div class="grid">
                <div class="card">
                    <h2>{"Single Bet Inputs"}</h2>
                    <label>{"Market probability (event happens) %"}</label>
                    <div class="row" style="align-items:center;">
                        <input type="number" min="0" max="100" step="0.1" value={format!("{:.1}", *market_prob)} oninput={on_market_prob_input.clone()} />
                        <input type="range" min="0" max="100" step="0.1" value={format!("{:.1}", *market_prob)} oninput={on_market_prob_slider} />
                    </div>
                    <div class="row" style="margin-top:10px;">
                        <div>
                            <label>{"Market odds (optional)"}</label>
                            <div class="row">
                                <select onchange={on_odds_format_change}>
                                    { for OddsFormat::all().iter().map(|(f, name)| {
                                        let selected = *f == *odds_format;
                                        html!{ <option selected={selected}>{ *name }</option> }
                                    })}
                                </select>
                                <input placeholder={"e.g. 2.10, +110, 11/10"} value={(*odds_input).clone()} oninput={on_odds_input} />
                            </div>
                            <div class="hint" style="margin-top:6px;">{"If blank/invalid, odds derive from market % (no vig)."}</div>
                        </div>
                    </div>
                    <div class="row" style="margin-top:10px;">
                        <div>
                            <label>{"Your probability (event happens) %"}</label>
                            <input type="number" min="0" max="100" step="0.1" value={format!("{:.1}", *your_prob)} oninput={on_your_prob_input} />
                        </div>
                        <div>
                            <label>{"Bet side"}</label>
                            <select onchange={on_bet_side_change}>
                                <option selected={matches!(*bet_side, BetSide::OnEvent)}>{"Event"}</option>
                                <option selected={matches!(*bet_side, BetSide::OnOpposite)}>{"Opposite"}</option>
                            </select>
                            <div class="hint" style="margin-top:6px;">{"Choose 'Opposite' for views like '70% not happen'."}</div>
                        </div>
                    </div>
                    <div style="margin-top:10px;">
                        <label>{"Bankroll"}</label>
                        <input placeholder={"e.g. 1000"} value={(*bankroll).clone()} oninput={on_bankroll_input} />
                        <div class="hint" style="margin-top:6px;">{"Only numbers are used; commas are ignored."}</div>
                    </div>
                </div>

                <div class="card">
                    <h2>{"Recommendation"}</h2>
                    <div class="muted">{"Kelly Fraction"}</div>
                    <div class={classes!("result", if kelly_f == 0.0 {"danger"} else {""})}>{format!("{:.2}%", 100.0 * kelly_f)}</div>
                    <div style="margin-top:10px;" class="muted">{"Bet Sizes"}</div>
                    <div>{format!("Full: ${:.2}", full_bet)}</div>
                    <div>{format!("Half: ${:.2}", half_bet)}</div>
                    <div>{format!("Quarter: ${:.2}", quarter_bet)}</div>
                </div>

                <div class="card">
                    <h2>{"Edge & Odds"}</h2>
                    <div class="muted">{"Odds (converted)"}</div>
                    <div>{format!("Decimal: {}  |  American: {}  |  Fractional: {}", dec_str, am_str, fr_str)}</div>
                    <div style="margin-top:10px;" class="muted">{"Edge"}</div>
                    <div>{ if ev_per_unit.is_nan() { "—".into() } else { format!("EV per $1: {:+.4}", ev_per_unit) } }</div>
                    <div>{ if implied_prob.is_nan() { "".into() } else { format!("Implied P: {:.2}%", 100.0*implied_prob) } }</div>
                    <div>{ if edge_prob.is_nan() { "".into() } else { format!("Your Edge: {:+.2}%", 100.0*edge_prob) } }</div>
                    <div class="hint" style="margin-top:8px;">{"Kelly is clamped to [0, 1]. If negative, no bet."}</div>
                </div>

                <div class="card">
                    <h2>{"Multiple Outcomes (Mutually Exclusive)"}</h2>
                    <div class="hint" style="margin: 6px 0 10px;">{"Enter market vs your probabilities per outcome. Odds derive from market % (no vig)."}</div>
                    <div>
                        { for (*outcomes).iter().enumerate().map(|(i, r)| {
                            let outcomes_set = outcomes.clone();
                            let on_name = Callback::from(move |e: InputEvent| {
                                let mut v = (*outcomes_set).clone();
                                let t: HtmlInputElement = e.target_unchecked_into();
                                v[i].name = t.value();
                                outcomes_set.set(v);
                            });
                            let outcomes_set2 = outcomes.clone();
                            let on_mkt = Callback::from(move |e: InputEvent| {
                                let mut v = (*outcomes_set2).clone();
                                let t: HtmlInputElement = e.target_unchecked_into();
                                v[i].mkt = t.value().parse::<f64>().unwrap_or(0.0).clamp(0.0, 100.0);
                                outcomes_set2.set(v);
                            });
                            let outcomes_set3 = outcomes.clone();
                            let on_yours = Callback::from(move |e: InputEvent| {
                                let mut v = (*outcomes_set3).clone();
                                let t: HtmlInputElement = e.target_unchecked_into();
                                v[i].yours = t.value().parse::<f64>().unwrap_or(0.0).clamp(0.0, 100.0);
                                outcomes_set3.set(v);
                            });
                            let outcomes_set4 = outcomes.clone();
                            let on_remove = Callback::from(move |_| {
                                let mut v = (*outcomes_set4).clone();
                                if i < v.len() { v.remove(i); }
                                outcomes_set4.set(v);
                            });
                            html!{
                                <div class="row" style="gap:8px; margin-bottom:8px; align-items: end;">
                                    <div>
                                        <label>{"Name"}</label>
                                        <input value={r.name.clone()} oninput={on_name} />
                                    </div>
                                    <div>
                                        <label>{"Market %"}</label>
                                        <input type="number" min="0" max="100" step="0.1" value={format!("{:.1}", r.mkt)} oninput={on_mkt} />
                                    </div>
                                    <div>
                                        <label>{"Your %"}</label>
                                        <input type="number" min="0" max="100" step="0.1" value={format!("{:.1}", r.yours)} oninput={on_yours} />
                                    </div>
                                    <button onclick={on_remove} style="height:36px; background:#1a2233; color:#fff; border:1px solid rgba(255,255,255,0.1); border-radius:8px;">{"Remove"}</button>
                                </div>
                            }
                        }) }
                        <button onclick={on_add_outcome.clone()} style="margin-top:6px; background:#132033; color:#5cc8ff; border:1px solid rgba(92,200,255,0.3); padding:8px 12px; border-radius:8px;">{"Add Outcome"}</button>
                    </div>
                    <div style="margin-top:12px;">
                        <div class="muted">{format!("Market sum: {:.1}%{}", total_mkt, if warn_market_sum { " (check inputs)" } else { "" })}</div>
                        <div style="margin-top:8px;">
                            { for multi_rows.iter().map(|(r, d, f)| {
                                let rec = f * multi_scale;
                                html!{ <div>{format!("{}: Kelly {:.2}%  -> Recommend {:.2}% (odds {:.3})", r.name, 100.0*f, 100.0*rec, d)}</div> }
                            }) }
                        </div>
                        <div class="hint" style="margin-top:8px;">{"Allocation uses independent Kelly per leg, scaled so total ≤ 100%."}</div>
                    </div>
                </div>
            </div>

            <footer>
                {"Single bet + multi-outcome (approx). Next: portfolio mode and visualizations."}
            </footer>
        </div>
    }
}

fn parse_any(s: &str) -> Option<f64> { // decimal odds
    let s = s.trim();
    // try decimal
    if let Ok(v) = s.parse::<f64>() { if v > 1.0 { return Some(v); } }
    // american
    if let Some(d) = parse_american(s) { return Some(d); }
    // fractional
    if let Some(d) = parse_fractional(s) { return Some(d); }
    None
}

fn parse_american(s: &str) -> Option<f64> { // returns decimal odds
    let s = s.trim().replace(',', "");
    let s = s.as_str();
    if s.is_empty() { return None; }
    let n = s.parse::<i64>().ok().or_else(|| {
        if s.starts_with('+') || s.starts_with('-') { s[1..].parse::<i64>().ok().map(|v| if s.starts_with('-') { -v } else { v }) } else { None }
    })?;
    if n > 0 { Some(1.0 + (n as f64)/100.0) } else { Some(1.0 + 100.0/(-(n as f64))) }
}

fn parse_fractional(s: &str) -> Option<f64> { // returns decimal odds
    let s = s.trim();
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 2 { return None; }
    let num = parts[0].trim().parse::<f64>().ok()?;
    let den = parts[1].trim().parse::<f64>().ok()?;
    if den <= 0.0 { return None; }
    Some(1.0 + num/den)
}

fn format_decimal(d: f64) -> String { format!("{:.3}", d) }
fn format_american(d: f64) -> String {
    if d <= 1.0 { return "—".into(); }
    let b = d - 1.0;
    if d >= 2.0 { // positive American odds
        let n = (b * 100.0).round() as i64;
        format!("+{}", n)
    } else {
        let n = (100.0 / b).round() as i64;
        format!("-{}", n)
    }
}
fn format_fractional(d: f64) -> String {
    if d <= 1.0 { return "—".into(); }
    let b = d - 1.0;
    // represent b as a simple fraction with small denominator
    let (num, den) = approx_fraction(b, 1_000, 100);
    format!("{}/{}", num, den)
}

fn approx_fraction(x: f64, max_den: i64, max_iter: i32) -> (i64, i64) {
    // continued fraction approximation
    let mut x = x;
    let mut a0 = x.floor();
    let mut h0: i64 = 1; let mut k0: i64 = 0;
    let mut h1: i64 = a0 as i64; let mut k1: i64 = 1;
    let mut iter = 0;
    while iter < max_iter {
        let frac = x - a0;
        if frac.abs() < 1e-9 { break; }
        x = 1.0/frac;
        a0 = x.floor();
        let h2 = h0 + (a0 as i64)*h1;
        let k2 = k0 + (a0 as i64)*k1;
        if k2 > max_den { break; }
        h0 = h1; k0 = k1; h1 = h2; k1 = k2;
        iter += 1;
    }
    (h1, k1)
}

fn main() {
    yew::Renderer::<App>::new().render();
}
