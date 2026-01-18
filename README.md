# Stylus-Trace

Developer Tooling

Details

Arbitrum Stylus brings a second virtual machine to the stack, letting teams write smart contracts in Rust and other Wasm languages that interoperate with Solidity on the EVM. That power comes with a painful gap. When a transaction behaves badly or costs spike, developers are left staring at raw trace JSON or dropping into low-level debuggers. It is slow, brittle, and hides the real story behind HostIO usage and the hot paths where Wasm and EVM meet. Teams guess, rerun tests, and still ship regressions to testnet or mainnet. What should be a quick diagnosis turns into hours of forensic work, and the cost of that uncertainty compounds across every release.


Stylus Trace Studio turns those opaque traces into something you can see, trust, and act on. The CLI ingests debug_traceTransaction with the Stylus tracer, aggregates HostIO events and call boundaries, and produces clear flamegraphs and concise JSON reports that highlight the true bottlenecks. A simple web viewer lets you zoom and compare runs side by side, while a ready-made GitHub Action spins up the Nitro dev node, replays a scripted transaction, and fails the pull request when thresholds are exceeded. No secrets, no keys, no reliance on paid RPCs by default. It complements cargo-stylus replay for step-through debugging and sits alongside unit test helpers, but fills the missing layer of aggregate, CI-friendly observability that catches problems before users ever feel them.


The result is a calmer, faster feedback loop for every Stylus team. Engineers get truthful pictures of performance and cost, reviewers get artifacts they can understand at a glance, and product owners get confidence that each merge moves the system forward instead of opening a hole. By making Stylus execution visible and comparable, the project gives Arbitrum builders a practical foundation for reliability at scale and a path to optimize without guesswork.
What innovation or value will your project bring to Arbitrum? What previously unaddressed problems is it solving? Is the project introducing genuinely new mechanisms.

Innovation and value:

Stylus Trace Studio gives Arbitrum developers a clear, Stylus aware observability layer. It turns debug_traceTransaction with stylusTracer into visual flamegraphs, simple before and after comparisons, and CI checks that stop performance and cost regressions before they ship. Teams get a truthful picture of WASM to EVM hot paths and HostIO usage, all generated locally with the Nitro dev node, with no keys, no paid RPC, and no backend.

Previously unaddressed problems:

Developers lack a transaction level performance view specific to Stylus. Current options focus on step by step replay or generic EVM views that miss WASM boundaries and HostIO behavior. This leaves guesswork around cost spikes and hidden bottlenecks. Stylus Trace Studio fills the gap by aggregating execution data into visuals that people can read quickly and by automating regression checks in CI so issues are caught early without expensive rounds on live networks.

Genuinely new mechanisms:

Stylus specific flamegraphs and side by side comparisons that highlight HostIO hotspots and cross VM execution.

a CI ready workflow that starts the Nitro dev node, runs a scripted transaction, produces reports, and fails pull requests when agreed thresholds are exceeded.

best effort source hints when debug info is present while still producing useful profiles when it is not.

Together these pieces create a practical standard for Stylus observability on Arbitrum. They lower iteration cost, improve reliability, and raise developer confidence across the ecosystem.

What is the current stage of your project.

Ideation/research

Do you have a target audience? If so, which one.

Developers building on Arbitrum with Stylus and Solidity who need clear performance and cost visibility during development and review

Protocol and dApp teams working on DeFi, gaming, and infra that run mixed WASM and EVM logic

Performance and reliability engineers who maintain CI pipelines and enforce budgets on HostIO and latency

Security and audit teams that want readable execution evidence to support findings

Orbit chain builders who need local, repeatable profiling for custom chains

DevRel and education groups that teach Stylus and need simple, visual examples for workshops and docs

Do you know about any comparable protocol, event, game, tool or project within the Arbitrum ecosystem.

Yes. There are adjacent tools and they serve different needs. None convert Stylus stylusTracer output into Stylus-aware flamegraphs with before and after diffs and CI regression gating.

cargo stylus replay
Official step by step debugging with GDB or LLDB. Logic inspection only. No aggregate performance view or CI thresholds.

Nitro dev node
Local Arbitrum node and trace endpoints. Provides the environment. Does not visualize traces or produce reports.

OpenZeppelin Motsu and Stylus test helpers
Rust unit testing and utilities. Contract correctness focus. Not transaction level performance profiling.

Remix plugin for Stylus and the Stylus playground
Editor and learning tools. Helpful for authoring and quick deploys. Not trace ingestion or analysis.

VS Code Stylus extensions
Editing ergonomics. No tracing or profiling.

Arbiscan debug and VM trace views
Explorer introspection for EVM. Limited or no Stylus HostIO visibility and no flamegraphs.

General EVM debuggers such as Tenderly
Great for Solidity and EVM traces. Not Stylus specific and no WASM to EVM hotpath profiling.

Recently funded local environment patches
Add native Arbitrum precompiles and deposit transaction support to Hardhat and Foundry. Improve local correctness. Do not turn Stylus traces into visuals or CI checks.

Overall, good building blocks exist. There is no public tool that ingests Stylus stylusTracer traces and outputs readable flamegraphs and diff reports while enforcing performance budgets in CI. That is the gap this project fills.

Have you received a grant from the DAO, Foundation, or any Arbitrum ecosystem related program or conducted any IRL like a hackathon or workshop.

Yes. We received a grant in the last season which birthed the "Arbitrum Creative Hub". We surpassed our KPIs, collaborated closely with key stakeholders in Arbitrum during our hackathon, we also produced 20+ robust testnet ready Stylus powered applications.

Kindly check full report here:
https://forum.arbitrum.foundation/t/arbitrum-creative-hub-final-report/29082

Have you received a grant from any other entity in other blockchains that are not Arbitrum.

Yes. We have received grant from NEAR and Solana.

What is the idea/project for which you are applying for a grant.

Stylus Trace Studio gives Arbitrum developers a practical way to see how Stylus transactions actually execute. Stylus adds a second virtual machine (Wasm) alongside the EVM so teams can write high-performance contracts in Rust or C while staying interoperable with Solidity; this multi-VM power also creates blind spots when a transaction is slow or costly because raw traces are hard to interpret at a glance. Our tool consumes the Nitro node’s native stylusTracer (via debug_traceTransaction) and converts that JSON into readable flamegraphs, concise summaries, and “before vs after” diffs that highlight HostIO hot paths and cross-VM bottlenecks. It complements the official cargo-stylus replay flow for step-through debugging by adding aggregate, CI-friendly observability. Teams can run everything locally on the Nitro dev node to avoid paid tracing plans.

Concretely, the project has three parts. A CLI pulls a Stylus trace from a local Nitro dev node or a tracing-enabled RPC and aggregates HostIO events and VM boundaries into SVG flamegraphs plus a JSON report with the top hot paths and HostIO event counts (including storage-related HostIO when aArbitrum Docsvailable). A static web viewer opens those files for zoom, search, and side-by-side diffs. A GitHub Action starts the Nitro dev node inside CI, replays a scripted transaction, produces the reports, and fails the pull request when thresholds are exceeded so performance and cost regressions are caught before testnet or mainnet. This replaces manual JSON inspection with artifacts reviewers can act on, and it aligns with how Arbitrum recommends working with traces today.

Implementation plan:

Trace capture and schema
Default capture path is local Nitro dev node. The CLI connects to the node and calls debug_traceTransaction with {"tracer":"stylusTracer"} to obtain the raw JSON; for users with a tracing provider, the CLI can point at that RPC as well. We will version the parser against the documented stylusTracer fields and smoke-test on the dev node plus at least one public provider each release. (Note: many providers gate debug_* behind paid tiers, which is why we default to the dev node.)

Aggregation and profiling
The CLI converts HostIO records and WASM↔EVM boundaries into collapsed stacks with weights, then renders SVG flamegraphs and emits a JSON summary of top hot paths and HostIO categories. Where builds include debug symbols, we surface best-effort source hints for file or function; when symbols are absent, profiles still label modules and HostIO categories so they remain useful. This creates an aggregate view that step-through debuggers and generic EVM explorers do not provide for Stylus execution.

Diff and regression checks
We normalize two profiles from the same scripted run, compute deltas, and render a diff that highlights increases and decreases. The GitHub Action runs against the local Nitro dev node, executes the scripted transaction or integration test, generates profiles, uploads them as CI artifacts, and enforces thresholds on totals like HostIO so teams block regressions in review without needing keys or paid endpoints.

Developer experience and documentation
We will publish a starter repo with a small Stylus contract and a Solidity caller that demonstrates the full flow end-to-end, plus documentation that includes Nitro dev-node quick start, a CI cookbook, and guidance on when to switch to cargo-stylus replay for step-through debugging. We will also point to OpenZeppelin Motsu for Rust unit tests, clarifying how unit-level correctness complements transaction-level profiling.

Outline the major deliverables you will obtain with this grant.

Milestone 1 Trace ingest and flamegraph:
CLI that fetches stylusTracer output via debug_traceTransaction from the Nitro dev node or a tracing RPC

Parser and versioned JSON schema for the profile summary output

SVG flamegraph generation

Cross platform builds and release packaging

Unit tests plus an end to end smoke test using a sample transaction

Milestone 2 Diff engine and CI Action
Before and after diff generation plus a machine readable diff report schema

Threshold configuration for regression gating

GitHub Action that starts a Nitro dev node in CI, runs a scripted transaction or test, generates profile artifacts, uploads them, and fails PRs on regression

Example workflow files and caching guidance

Milestone 3 Static viewer and source hints
Static web viewer that opens profiles locally with zoom and search

Side by side diff view

Best effort source hints when debug info is present and graceful fallback labels when not

Hosted static demo build and exportable bundle

Milestone 4 Docs, starter repo, templates, demo
Starter repository with a Stylus contract, a Solidity caller, and a green CI pipeline producing flamegraph artifacts

Quickstart docs, CI cookbook, troubleshooting, and provider notes

At least one education template or lesson plan using the tool

At least one Orbit oriented template example using a custom chain ID and RPC

One short demo video and one tutorial post showing the full workflow

Milestone 5 Post release support and ecosystem enablement
Six months of post release support as defined below

Monthly smoke test logs

At least three workshops or posts demonstrating the tool during the support window