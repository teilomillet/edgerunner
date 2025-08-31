use yew::prelude::*;
use yew::TargetCast;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use std::collections::HashMap;

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

#[derive(Clone, PartialEq)]
struct CompareRow { name: String, group: String, odds: String, your: f64 }

#[derive(Clone, PartialEq)]
struct ThreeRow { name: String, mkt: f64, yours: f64 }

#[function_component(App)]
fn app() -> Html {
    // Single bet state
    let market_prob = use_state(|| 60.0_f64); // % market thinks event happens
    let your_prob = use_state(|| 55.0_f64);   // % you think event happens
    let bet_side = use_state(|| BetSide::OnEvent);
    let odds_format = use_state(|| OddsFormat::Decimal);
    // Default to blank odds so market % drives implied odds by default.
    let odds_input = use_state(|| String::from(""));
    let bankroll = use_state(|| String::from("1000"));

    // Multi-outcome state
    let outcomes = use_state(|| vec![
        OutcomeRow { name: "A".into(), mkt: 50.0, yours: 60.0 },
        OutcomeRow { name: "B".into(), mkt: 50.0, yours: 40.0 },
    ]);

    // Compare bets state (live comparisons across different selections/markets)
    let compares = use_state(|| vec![
        CompareRow { name: "Selection 1".into(), group: "Market 1".into(), odds: "".into(), your: 55.0 },
    ]);

    // Three-way market (exact Kelly for a single event with 3 outcomes)
    let three = use_state(|| vec![
        ThreeRow { name: "Home".into(), mkt: 40.0, yours: 45.0 },
        ThreeRow { name: "Draw".into(), mkt: 30.0, yours: 25.0 },
        ThreeRow { name: "Away".into(), mkt: 30.0, yours: 30.0 },
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
                // Interpret your input as the probability of the SELECTED side (Yes/No).
                let p = *your_prob as f64 / 100.0;
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

    // Per-$1 and fair odds metrics for the selected side
    let p_selected = *your_prob as f64 / 100.0;
    let d_selected = decimal_odds.unwrap_or(f64::NAN);
    let b_selected = if d_selected > 1.0 { d_selected - 1.0 } else { f64::NAN };
    let win_per_1 = b_selected; // profit if win per $1 staked
    let loss_per_1 = if d_selected.is_nan() { f64::NAN } else { 1.0 };
    let fair_decimal = if p_selected > 0.0 { 1.0 / p_selected } else { f64::INFINITY };
    let fair_dec_str = if fair_decimal.is_finite() { format_decimal(fair_decimal) } else { "—".into() };
    let fair_am_str = if fair_decimal.is_finite() { format_american(fair_decimal) } else { "—".into() };
    let fair_fr_str = if fair_decimal.is_finite() { format_fractional(fair_decimal) } else { "—".into() };
    let g_full = if kelly_f > 0.0 && b_selected.is_finite() {
        let p = p_selected; let f = kelly_f; let b = b_selected;
        p * (1.0 + f*b).ln() + (1.0 - p) * (1.0 - f).ln()
    } else { 0.0 };

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
            let idx = target.selected_index();
            let new_format = match idx { 0 => OddsFormat::Decimal, 1 => OddsFormat::American, _ => OddsFormat::Fractional };
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
        let odds_input = odds_input.clone();
        let odds_format = odds_format.clone();
        let your_prob_state = your_prob.clone();
        Callback::from(move |e: Event| {
            let target: HtmlSelectElement = e.target_unchecked_into();
            let idx = target.selected_index();
            let side = if idx > 0 { BetSide::OnOpposite } else { BetSide::OnEvent };
            // Flip odds input to the complementary side if present
            let current = (*odds_input).clone();
            if let Some(d) = parse_any(&current) {
                if d > 1.0 + 1e-9 {
                    let d2 = complement_decimal(d);
                    let formatted = match *odds_format {
                        OddsFormat::Decimal => format_decimal(d2),
                        OddsFormat::American => format_american(d2),
                        OddsFormat::Fractional => format_fractional(d2),
                    };
                    odds_input.set(formatted);
                }
            }
            // Since "Your %" refers to the SELECTED side, mirror it when toggling
            let cur = *your_prob_state;
            your_prob_state.set(100.0 - cur);
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

    // Add-compare handler
    let on_add_compare = {
        let compares = compares.clone();
        Callback::from(move |_| {
            let mut v = (*compares).clone();
            v.push(CompareRow{ name: format!("Selection {}", v.len()+1), group: "Market".into(), odds: "".into(), your: 50.0 });
            compares.set(v);
        })
    };

    // Validation helpers
    let bankroll_valid = bankroll_val() > 0.0;
    let odds_valid = decimal_odds.is_some();
    let market_sum_valid = !warn_market_sum;
    
    // Status indicators
    let kelly_status = if kelly_f == 0.0 { "danger" } else if kelly_f > 0.25 { "warning" } else { "success" };
    let edge_status = if edge_prob.is_nan() { "muted" } else if edge_prob <= 0.0 { "danger" } else { "success" };

    // Side labels and complementary odds for clarity in UI
    let selected_side_label = match *bet_side { BetSide::OnEvent => "Yes", BetSide::OnOpposite => "No" };
    let other_side_label = match *bet_side { BetSide::OnEvent => "No", BetSide::OnOpposite => "Yes" };
    let comp_dec_odds = decimal_odds.map(|d| complement_decimal(d));

    // Compare panel computations: build grouped view data outside html!
    let cmp_rows = (*compares).clone();
    let mut cmp_by_group: HashMap<String, Vec<(usize, f64, f64, f64, f64)>> = HashMap::new();
    for (idx, r) in cmp_rows.iter().enumerate() {
        if let Some(d) = parse_any(&r.odds) {
            if d > 1.0 {
                let p = (r.your/100.0).clamp(0.0, 1.0);
                let imp = 1.0/d;
                let b = d - 1.0; let q = 1.0 - p;
                let f = (((b*p) - q) / b).clamp(0.0, 1.0);
                let ev = (p*b) - q;
                cmp_by_group.entry(r.group.clone()).or_default().push((idx, d, f, imp, ev));
            }
        }
    }
    let mut compare_view: Vec<(String, f64, f64, Vec<(usize, f64, f64, f64, f64)>)> = Vec::new();
    for (g, items) in cmp_by_group.into_iter() {
        let sum_f: f64 = items.iter().map(|(_,_,f,_,_)| *f).sum();
        let scale = if sum_f > 1.0 { 1.0/sum_f } else { 1.0 };
        compare_view.push((g, sum_f, scale, items));
    }
    let bank_for_cmp = bankroll_val();

    // Three-way exact Kelly compute
    let three_rows = (*three).clone();
    let mut p_vec: Vec<f64> = Vec::new();
    let mut d_vec: Vec<f64> = Vec::new();
    for r in three_rows.iter() {
        let pm = (r.mkt/100.0).clamp(1e-9, 1.0 - 1e-9);
        let d = 1.0/pm; // use market-implied odds for each outcome
        let p = (r.yours/100.0).clamp(1e-9, 1.0);
        p_vec.push(p);
        d_vec.push(d);
    }
    let three_alloc = kelly_multi_exact(&p_vec, &d_vec, 1.0);
    let three_sum: f64 = three_alloc.iter().sum();

    html! {
        <div class="container">
            <header>
                <h1>{"EdgeRunner"}</h1>
                <div class="tooltip pill" data-tooltip="Professional Kelly Criterion calculator for optimal bet sizing">
                    {"Kelly Calculator"}
                </div>
            </header>

            <div class="grid">
                <div class="card">
                    <h2>
                        <span>{"Single Bet Inputs"}</span>
                        { if !bankroll_valid || !odds_valid {
                            html!{ <span class="status-indicator warning">{"Check inputs"}</span> }
                        } else {
                            html!{ <span class="status-indicator success">{"Valid"}</span> }
                        }}
                    </h2>
                    
                    <div class="input-group">
                        <label class="tooltip" data-tooltip="Market's implied probability that the event will occur">
                            {"Market Probability (%)"}
                        </label>
                        <div class="row" style="align-items:center;">
                            <input 
                                type="number" 
                                min="0" 
                                max="100" 
                                step="0.1" 
                                value={format!("{:.1}", *market_prob)} 
                                oninput={on_market_prob_input.clone()}
                                aria-label="Market probability percentage" />
                            <input 
                                type="range" 
                                min="0" 
                                max="100" 
                                step="0.1" 
                                value={format!("{:.1}", *market_prob)} 
                                oninput={on_market_prob_slider}
                                aria-label="Market probability slider" />
                        </div>
                        <div class="hint">{"Current: "}{format!("{:.1}%", *market_prob)}</div>
                    </div>

                    <div class="input-group">
                        <label class="tooltip" data-tooltip="Enter specific odds or leave blank to derive from market probability">
                            {"Market Odds (Optional)"}
                        </label>
                        <div class="row">
                            <select onchange={on_odds_format_change} aria-label="Odds format selection">
                                { for OddsFormat::all().iter().map(|(f, name)| {
                                    let selected = *f == *odds_format;
                                    html!{ <option selected={selected}>{ *name }</option> }
                                })}
                            </select>
                            <input 
                                placeholder={"e.g. 2.10, +110, 11/10"} 
                                value={(*odds_input).clone()} 
                                oninput={on_odds_input}
                                class={if odds_valid { "" } else { "error" }}
                                aria-label="Odds input" />
                        </div>
                        <div class="hint">
                            { if odds_valid {
                                "Valid odds format"
                            } else {
                                "Using market probability (no vig)"
                            }}
                        </div>
                    </div>

                    <div class="section-divider"></div>

                    <div class="row" style="align-items:end;">
                        <div class="input-group">
                            <label class="tooltip" data-tooltip="Your assessment of the probability this event will occur">
                                {"Your Probability (%)"}
                            </label>
                            <input 
                                type="number" 
                                min="0" 
                                max="100" 
                                step="0.1" 
                                value={format!("{:.1}", *your_prob)} 
                                oninput={on_your_prob_input}
                                aria-label="Your probability assessment" />
                        </div>
                        <div class="input-group">
                            <label class="tooltip" data-tooltip="Choose which side of the bet you're considering">
                                {"Bet Side"}
                            </label>
                            <select onchange={on_bet_side_change} aria-label="Bet side selection">
                                <option selected={matches!(*bet_side, BetSide::OnEvent)}>{"Yes"}</option>
                                <option selected={matches!(*bet_side, BetSide::OnOpposite)}>{"No"}</option>
                            </select>
                        </div>
                    </div>

                    <div class="input-group">
                        <label class="tooltip" data-tooltip="Your total available betting capital">
                            {"Total Bankroll ($)"}
                        </label>
                        <input 
                            type="text"
                            placeholder={"e.g. 1000"} 
                            value={(*bankroll).clone()} 
                            oninput={on_bankroll_input}
                            class={if bankroll_valid { "" } else { "error" }}
                            aria-label="Bankroll amount" />
                        <div class="hint">
                            { if bankroll_valid {
                                format!("Available: ${:.2}", bankroll_val())
                            } else {
                                "Enter a valid amount".to_string()
                            }}
                        </div>
                    </div>
                </div>

                <div class="card">
                    <h2>
                        <span>{"Recommendation"}</span>
                        <span class={format!("status-indicator {}", kelly_status)}>
                            { match kelly_status {
                                "success" => "Optimal",
                                "warning" => "High risk", 
                                _ => "No bet"
                            }}
                        </span>
                    </h2>
                    
                    <div class="muted">{"Kelly Fraction"}</div>
                    <div class={format!("result large {}", if kelly_f == 0.0 { "danger" } else if kelly_f > 0.25 { "warning" } else { "success" })}>
                        {format!("{:.2}%", 100.0 * kelly_f)}
                    </div>
                    
                    { if kelly_f > 0.0 {
                        html!{
                            <>
                                <div class="section-divider"></div>
                                <div class="metric-grid">
                                    <div class="metric-item">
                                        <div class="metric-value">{format!("${:.0}", full_bet)}</div>
                                        <div class="metric-label">{"Full Kelly"}</div>
                                    </div>
                                    <div class="metric-item">
                                        <div class="metric-value">{format!("${:.0}", half_bet)}</div>
                                        <div class="metric-label">{"Half Kelly"}</div>
                                    </div>
                                    <div class="metric-item">
                                        <div class="metric-value">{format!("${:.0}", quarter_bet)}</div>
                                        <div class="metric-label">{"Quarter Kelly"}</div>
                                    </div>
                                    <div class="metric-item">
                                        <div class="metric-value">{format!("{:.1}%", (full_bet/bankroll_val()*100.0))}</div>
                                        <div class="metric-label">{"% of Bankroll"}</div>
                                    </div>
                                </div>
                                <div class="hint" style="margin-top:12px;">
                                    {"Consider fractional Kelly sizing (Half/Quarter) to reduce volatility"}
                                </div>
                            </>
                        }
                    } else {
                        html!{
                            <div class="hint" style="margin-top:12px;">
                                {"No betting edge detected. Kelly suggests no bet."}
                            </div>
                        }
                    }}
                </div>

                <div class="card">
                    <h2>
                        <span>{"Edge Analysis"}</span>
                        <span class={format!("status-indicator {}", edge_status)}>
                            { if edge_prob.is_nan() { 
                                "—" 
                            } else if edge_prob > 0.0 { 
                                "Positive edge" 
                            } else { 
                                "No edge" 
                            }}
                        </span>
                    </h2>
                    
                    <div class="muted">{format!("Odds — {}", selected_side_label)}</div>
                    <div style="margin-bottom:16px;">
                        <div>{"Decimal: "}<strong>{dec_str}</strong></div>
                        <div>{"American: "}<strong>{am_str}</strong></div>
                        <div>{"Fractional: "}<strong>{fr_str}</strong></div>
                    </div>
                    { if let Some(cd) = comp_dec_odds {
                        html!{ <div class="hint" style="margin-top:-8px; margin-bottom:12px;">{format!("Odds — {}: decimal {:.3}", other_side_label, cd)}</div> }
                    } else { html!{} }}
                    
                    <div class="metric-grid">
                        <div class="metric-item">
                            <div class={format!("metric-value {}", if ev_per_unit.is_nan() { "muted" } else if ev_per_unit > 0.0 { "success" } else { "danger" })}>
                                { if ev_per_unit.is_nan() { "—".into() } else { format!("{:+.3}", ev_per_unit) }}
                            </div>
                            <div class="metric-label">{"EV per $1"}</div>
                        </div>
                        <div class="metric-item">
                            <div class="metric-value">{ if win_per_1.is_nan() { "—".into() } else { format!("{:.3}", win_per_1) } }</div>
                            <div class="metric-label">{"Win Profit per $1"}</div>
                        </div>
                        <div class="metric-item">
                            <div class="metric-value">{ if loss_per_1.is_nan() { "—".into() } else { format!("{:.0}", loss_per_1) } }</div>
                            <div class="metric-label">{"Loss per $1"}</div>
                        </div>
                        <div class="metric-item">
                            <div class="metric-value">
                                { if implied_prob.is_nan() { "—".into() } else { format!("{:.1}%", 100.0*implied_prob) }}
                            </div>
                            <div class="metric-label">{format!("Implied Prob — {}", selected_side_label)}</div>
                        </div>
                        <div class="metric-item">
                            <div class={format!("metric-value {}", if edge_prob.is_nan() { "muted" } else if edge_prob > 0.0 { "success" } else { "danger" })}>
                                { if edge_prob.is_nan() { "—".into() } else { format!("{:+.1}%", 100.0*edge_prob) }}
                            </div>
                            <div class="metric-label">{"Your Edge"}</div>
                        </div>
                        <div class="metric-item">
                            <div class="metric-value">
                                {format!("{:.1}%", *your_prob)}
                            </div>
                            <div class="metric-label">{format!("Your Prob — {}", selected_side_label)}</div>
                        </div>
                        <div class="metric-item">
                            <div class="metric-value">{format!("{} | {} | {}", fair_dec_str, fair_am_str, fair_fr_str)}</div>
                            <div class="metric-label">{format!("Your Fair Odds — {}", selected_side_label)}</div>
                        </div>
                        <div class="metric-item">
                            <div class="metric-value">{format!("{:+.3} bp", g_full * 10_000.0)}</div>
                            <div class="metric-label">{"Log Growth @ Full Kelly"}</div>
                        </div>
                    </div>
                </div>

                <div class="card">
                    <h2>
                        <span>{"Multiple Outcomes"}</span>
                        { if market_sum_valid {
                            html!{ <span class="status-indicator success">{"Valid"}</span> }
                        } else {
                            html!{ <span class="status-indicator warning">{"Check sum"}</span> }
                        }}
                    </h2>
                    <div class="hint" style="margin-bottom:16px;">
                        {"Add mutually exclusive outcomes. Market probabilities should sum to ~100%."}
                    </div>
                    
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
                                <div class="row three" style="gap:8px; margin-bottom:12px; align-items: end;">
                                    <div>
                                        <label>{"Outcome Name"}</label>
                                        <input value={r.name.clone()} oninput={on_name} aria-label="Outcome name" />
                                    </div>
                                    <div>
                                        <label>{"Market %"}</label>
                                        <input type="number" min="0" max="100" step="0.1" value={format!("{:.1}", r.mkt)} oninput={on_mkt} aria-label="Market probability" />
                                    </div>
                                    <div>
                                        <label>{"Your %"}</label>
                                        <input type="number" min="0" max="100" step="0.1" value={format!("{:.1}", r.yours)} oninput={on_yours} aria-label="Your probability assessment" />
                                    </div>
                                    <button onclick={on_remove} class="danger" style="height:40px;" aria-label="Remove outcome">
                                        {"Remove"}
                                    </button>
                                </div>
                            }
                        }) }
                        <button onclick={on_add_outcome.clone()} style="margin-top:8px; width: 100%;" aria-label="Add new outcome">
                            {"Add Outcome"}
                        </button>
                    </div>
                    
                    <div class="section-divider"></div>
                    
                    <div class="muted" style="margin-bottom:12px;">
                        {"Market Sum: "}
                        <span class={if warn_market_sum { "warning" } else { "success" }}>
                            {format!("{:.1}%", total_mkt)}
                        </span>
                        { if warn_market_sum {
                            html!{ <span class="warning">{" (should be ~100%)"}</span> }
                        } else { html!{} }}
                    </div>
                    
                    { if multi_rows.len() > 0 {
                        html!{
                            <div>
                                { for multi_rows.iter().map(|(r, d, f)| {
                                    let rec = f * multi_scale;
                                    let kelly_pct = 100.0 * f;
                                    let rec_pct = 100.0 * rec;
                                    html!{ 
                                        <div style="padding:8px; background:rgba(255,255,255,0.02); border-radius:6px; margin-bottom:6px;">
                                            <strong>{&r.name}</strong>
                                            <div style="font-size:12px; color: var(--muted); margin-top:2px;">
                                                {format!("Kelly: {:.1}% → Recommend: {:.1}% (odds {:.2})", kelly_pct, rec_pct, d)}
                                            </div>
                                        </div>
                                    }
                                }) }
                                <div class="hint" style="margin-top:12px;">
                                    {"Independent Kelly per outcome, scaled so total ≤ 100% of bankroll"}
                                </div>
                            </div>
                        }
                    } else { html!{} }}
                </div>
            </div>

            <div class="card">
                <h2>
                    <span>{"Compare Bets (Live)"}</span>
                </h2>
                <div class="hint" style="margin-bottom:12px;">{"Add selections across one or more markets (groups). Odds can be decimal, American, or fractional."}</div>

                <div>
                    { for (*compares).iter().enumerate().map(|(i, r)| {
                        let compares_set = compares.clone();
                        let on_name = Callback::from(move |e: InputEvent| {
                            let mut v = (*compares_set).clone();
                            let t: HtmlInputElement = e.target_unchecked_into();
                            v[i].name = t.value();
                            compares_set.set(v);
                        });
                        let compares_set2 = compares.clone();
                        let on_group = Callback::from(move |e: InputEvent| {
                            let mut v = (*compares_set2).clone();
                            let t: HtmlInputElement = e.target_unchecked_into();
                            v[i].group = t.value();
                            compares_set2.set(v);
                        });
                        let compares_set3 = compares.clone();
                        let on_odds = Callback::from(move |e: InputEvent| {
                            let mut v = (*compares_set3).clone();
                            let t: HtmlInputElement = e.target_unchecked_into();
                            v[i].odds = t.value();
                            compares_set3.set(v);
                        });
                        let compares_set4 = compares.clone();
                        let on_your = Callback::from(move |e: InputEvent| {
                            let mut v = (*compares_set4).clone();
                            let t: HtmlInputElement = e.target_unchecked_into();
                            v[i].your = t.value().parse::<f64>().unwrap_or(0.0).clamp(0.0, 100.0);
                            compares_set4.set(v);
                        });
                        let compares_set5 = compares.clone();
                        let on_remove = Callback::from(move |_| {
                            let mut v = (*compares_set5).clone();
                            if i < v.len() { v.remove(i); }
                            compares_set5.set(v);
                        });
                        html!{
                            <div class="row three" style="gap:8px; margin-bottom:12px; align-items: end;">
                                <div>
                                    <label>{"Name"}</label>
                                    <input value={r.name.clone()} oninput={on_name} aria-label="Compare name" />
                                </div>
                                <div>
                                    <label>{"Group (market)"}</label>
                                    <input value={r.group.clone()} oninput={on_group} aria-label="Compare group" />
                                </div>
                                <div>
                                    <label>{"Odds"}</label>
                                    <input placeholder={"e.g. 2.10, +110, 11/10"} value={r.odds.clone()} oninput={on_odds} aria-label="Compare odds" />
                                </div>
                                <div>
                                    <label>{"Your %"}</label>
                                    <input type="number" min="0" max="100" step="0.1" value={format!("{:.1}", r.your)} oninput={on_your} aria-label="Your probability" />
                                </div>
                                <button onclick={on_remove} class="danger" style="height:40px;">{"Remove"}</button>
                            </div>
                        }
                    }) }
                    <button onclick={on_add_compare.clone()} style="margin-top:8px; width:100%;">{"Add Selection"}</button>
                </div>

                <div style="margin-top:12px;">
                    { for compare_view.iter().map(|(g, sum_f, scale, items)| {
                        html!{
                            <div style="margin-bottom:12px;">
                                <div class="muted">{format!("Group: {} — total Kelly {:.1}% (scaled: {}x)", g, 100.0*sum_f, format!("{:.2}", scale))}</div>
                                { for items.iter().map(|(idx, d, f, _imp, ev)| {
                                    let r = &cmp_rows[*idx];
                                    let rec = f * *scale;
                                    html!{
                                        <div style="padding:8px; background:rgba(255,255,255,0.02); border-radius:6px; margin-top:6px;">
                                            <strong>{&r.name}</strong>{" — "}{format!("{}", &r.group)}
                                            <div style="font-size:12px; color: var(--muted); margin-top:2px;">
                                                {format!("Odds {:.3} | Kelly {:.1}% → Recommend {:.1}% | EV/1 {:+.3} | Stake ${:.0}", d, 100.0*(*f), 100.0*rec, ev, bank_for_cmp*rec)}
                                            </div>
                                        </div>
                                    }
                                }) }
                            </div>
                        }
                    }) }
                </div>
            </div>

            <div class="card">
                <h2>
                    <span>{"Three-Way Market (Exact Kelly)"}</span>
                    <span class={if (three_sum - 1.0).abs() < 1e-6 { "status-indicator success" } else { "status-indicator" }}>
                        {format!("Total stake: {:.1}%", 100.0*three_sum)}
                    </span>
                </h2>
                <div class="hint" style="margin-bottom:12px;">{"Enter market vs your probabilities for 3 mutually exclusive outcomes (e.g., Team A / Draw / Team B). This computes the exact Kelly allocation across outcomes."}</div>

                { for (*three).iter().enumerate().map(|(i, r)| {
                    let three_set = three.clone();
                    let on_name = Callback::from(move |e: InputEvent| {
                        let mut v = (*three_set).clone();
                        let t: HtmlInputElement = e.target_unchecked_into();
                        v[i].name = t.value();
                        three_set.set(v);
                    });
                    let three_set2 = three.clone();
                    let on_mkt = Callback::from(move |e: InputEvent| {
                        let mut v = (*three_set2).clone();
                        let t: HtmlInputElement = e.target_unchecked_into();
                        v[i].mkt = t.value().parse::<f64>().unwrap_or(0.0).clamp(0.0, 100.0);
                        three_set2.set(v);
                    });
                    let three_set3 = three.clone();
                    let on_yours = Callback::from(move |e: InputEvent| {
                        let mut v = (*three_set3).clone();
                        let t: HtmlInputElement = e.target_unchecked_into();
                        v[i].yours = t.value().parse::<f64>().unwrap_or(0.0).clamp(0.0, 100.0);
                        three_set3.set(v);
                    });
                    html!{
                        <div class="row three" style="gap:8px; margin-bottom:12px; align-items: end;">
                            <div>
                                <label>{"Outcome"}</label>
                                <input value={r.name.clone()} oninput={on_name} aria-label="Three-way outcome name" />
                            </div>
                            <div>
                                <label>{"Market %"}</label>
                                <input type="number" min="0" max="100" step="0.1" value={format!("{:.1}", r.mkt)} oninput={on_mkt} aria-label="Three-way market probability" />
                            </div>
                            <div>
                                <label>{"Your %"}</label>
                                <input type="number" min="0" max="100" step="0.1" value={format!("{:.1}", r.yours)} oninput={on_yours} aria-label="Three-way your probability" />
                            </div>
                        </div>
                    }
                })}
                <div class="section-divider"></div>
                <div>
                    { for three_rows.iter().enumerate().map(|(i, r)| {
                        let frac = three_alloc.get(i).cloned().unwrap_or(0.0);
                        html!{
                            <div style="padding:8px; background:rgba(255,255,255,0.02); border-radius:6px; margin-top:6px;">
                                <strong>{&r.name}</strong>
                                <div style="font-size:12px; color: var(--muted); margin-top:2px;">
                                    {format!("Recommend: {:.1}% of bankroll → ${:.0}", 100.0*frac, bankroll_val()*frac)}
                                </div>
                            </div>
                        }
                    }) }
                </div>
                <div class="hint" style="margin-top:12px;">{"Optimization: maximize expected log growth under sum of stakes ≤ 100%."}</div>
            </div>

            <footer>
                {"EdgeRunner v0.1 - Professional Kelly Criterion calculator for optimal bet sizing"}
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

fn complement_decimal(d: f64) -> f64 {
    // Convert decimal odds for an event to the opposite side under no-vig assumption.
    // d_opposite = d / (d - 1)
    if d <= 1.0 { return f64::NAN; }
    d / (d - 1.0)
}

// ---- Exact Kelly for mutually exclusive outcomes (N-outcome market) ----
fn kelly_multi_exact(p: &[f64], d: &[f64], cap: f64) -> Vec<f64> {
    let n = p.len();
    if n == 0 || d.len() != n { return vec![]; }
    // Initialize with independent Kelly scaled
    let mut f: Vec<f64> = Vec::with_capacity(n);
    let mut sumk = 0.0;
    for i in 0..n {
        let b = d[i] - 1.0;
        let q = 1.0 - p[i];
        let fi = if b > 0.0 { (((b*p[i]) - q) / b).clamp(0.0, 1.0) } else { 0.0 };
        sumk += fi; f.push(fi);
    }
    if sumk > cap && sumk > 0.0 {
        let scale = cap / sumk;
        for i in 0..n { f[i] *= scale; }
    }

    // Helper closures
    let obj = |f: &Vec<f64>| -> f64 {
        let fsum: f64 = f.iter().sum();
        let mut val = 0.0;
        for i in 0..n {
            let wi = 1.0 - fsum + d[i]*f[i];
            if wi <= 1e-12 { return f64::NEG_INFINITY; }
            val += p[i] * wi.ln();
        }
        val
    };
    let grad = |f: &Vec<f64>| -> Vec<f64> {
        let fsum: f64 = f.iter().sum();
        let mut g = vec![0.0; n];
        // precompute S = sum_i p_i / W_i
        let mut s_over = 0.0;
        let mut inv_w: Vec<f64> = vec![0.0; n];
        for i in 0..n {
            let wi = 1.0 - fsum + d[i]*f[i];
            let inv = if wi <= 1e-12 { 1e12 } else { 1.0/wi };
            inv_w[i] = inv;
            s_over += p[i] * inv;
        }
        for k in 0..n {
            g[k] = -s_over + p[k] * d[k] * inv_w[k];
        }
        g
    };

    let proj = |v: &mut Vec<f64>| {
        // Project onto simplex {x >= 0, sum x <= cap}
        for x in v.iter_mut() { if *x < 0.0 { *x = 0.0; } }
        let sum: f64 = v.iter().sum();
        if sum <= cap { return; }
        // Euclidean projection onto capped simplex via sorting
        let mut u: Vec<f64> = v.clone();
        u.sort_by(|a,b| b.partial_cmp(a).unwrap());
        let mut cssv = 0.0;
        let mut rho = -1;
        for (j, &u_j) in u.iter().enumerate() {
            cssv += u_j;
            let t = (cssv - cap) / ((j as f64)+1.0);
            if u_j - t > 0.0 { rho = j as i32; }
        }
        let rho = if rho < 0 { 0 } else { rho as usize };
        let theta = (u.iter().take(rho+1).sum::<f64>() - cap) / ((rho+1) as f64);
        for x in v.iter_mut() { *x = (*x - theta).max(0.0); }
    };

    // Projected gradient ascent with backtracking
    let mut step = 0.25;
    let mut best_f = f.clone();
    let mut best_obj = obj(&f);
    for _iter in 0..300 {
        let g = grad(&f);
        // tentative
        let mut cand = f.iter().zip(g.iter()).map(|(a, b)| a + step * b).collect::<Vec<_>>();
        proj(&mut cand);
        let o_new = obj(&cand);
        if o_new.is_finite() && o_new > best_obj + 1e-9 {
            f = cand;
            best_obj = o_new;
            best_f = f.clone();
            // try slightly larger step
            step = (step * 1.05).min(1.0);
        } else {
            // reduce step
            step *= 0.5;
            if step < 1e-6 { break; }
        }
    }
    best_f
}

fn main() {
    yew::Renderer::<App>::new().render();
}
